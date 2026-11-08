use super::{
    adapters::control::SqliteSyncControlStore,
    application::CursorStore,
    coordinator::{CycleOutcome, CycleTrigger, SyncEngine},
    dispatch::OutboxDispatcher,
};
use crate::{
    components::devices::activation::DeviceCredential,
    platform::http::{APICode, APIFailure},
    shared_kernel::{
        NightingaleError, Result, SystemClock, UuidV7Generator,
        id::{AccountId, OperationContext},
    },
};
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamOutcome {
    Ended,
    Busy,
    Revoked,
    RecoveryRequired,
    Retry,
}

#[derive(Default)]
pub(crate) struct Parser {
    line: Vec<u8>,
    event: String,
    data: String,
    cr: bool,
    size: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Notice {
    Changes,
    Heartbeat,
    Revoked,
}

impl Parser {
    pub fn feed(&mut self, bytes: &[u8], account: &AccountId) -> Result<Vec<Notice>> {
        let mut notices = Vec::new();

        for byte in bytes {
            if *byte == b'\n' && self.cr {
                self.cr = false;

                continue;
            }

            self.cr = *byte == b'\r';

            if matches!(*byte, b'\r' | b'\n') {
                let line = String::from_utf8(std::mem::take(&mut self.line))
                    .map_err(|_| NightingaleError::validation("SSE UTF-8 inválido"))?;

                if line.is_empty() {
                    match self.event.as_str() {
                        "changes_available" => {
                            #[derive(Deserialize)]
                            #[serde(rename_all = "camelCase")]
                            struct Data {
                                account_id: AccountId,
                                cursor: String,
                            }

                            let data: Data = serde_json::from_str(&self.data)
                                .map_err(|_| NightingaleError::validation("aviso SSE inválido"))?;

                            if data.account_id != *account || data.cursor.is_empty() {
                                return Err(NightingaleError::validation("alcance SSE inválido"));
                            }

                            notices.push(Notice::Changes);
                        }
                        "heartbeat" => {
                            #[derive(Deserialize)]
                            #[serde(rename_all = "camelCase")]
                            struct Data {
                                server_time: String,
                            }

                            let data: Data = serde_json::from_str(&self.data)
                                .map_err(|_| NightingaleError::validation("heartbeat inválido"))?;

                            if data.server_time.is_empty() {
                                return Err(NightingaleError::validation("heartbeat vacío"));
                            }

                            notices.push(Notice::Heartbeat);
                        }
                        "access_revoked" => notices.push(Notice::Revoked),
                        _ => {}
                    }
                    self.event.clear();
                    self.data.clear();
                    self.size = 0;
                } else if !line.starts_with(':') {
                    let (field, value) = line.split_once(':').unwrap_or((&line, ""));
                    let value = value.strip_prefix(' ').unwrap_or(value);

                    match field {
                        "event" => self.event = value.to_owned(),
                        "data" => {
                            if !self.data.is_empty() {
                                self.data.push('\n');
                            }
                            self.data.push_str(value);
                        }
                        _ => {}
                    }
                }
            } else {
                self.line.push(*byte);
                self.size += 1;

                if self.size > 16 * 1024 {
                    return Err(NightingaleError::validation("aviso SSE demasiado grande"));
                }
            }
        }

        Ok(notices)
    }
}

pub fn reconnect_delay(attempt: i64, jitter: u64) -> u64 {
    super::dispatch::retry_delay(attempt, jitter).max(3_000) as u64
}

impl SyncEngine<'_> {
    /// One connection per installation/account. The caller owns cancellation and
    /// schedules reconnects; no event ID or advertised cursor is ever applied.
    pub async fn listen_once(
        &self,
        credential: &DeviceCredential,
        scope: &OperationContext,
        projector: &impl super::remote::RemoteProjector,
    ) -> Result<StreamOutcome> {
        self.coordinator
            .listen_once(self, credential, scope, projector)
            .await
    }
}

pub(super) async fn listen(
    engine: &SyncEngine<'_>,
    credential: &DeviceCredential,
    scope: &OperationContext,
    projector: &impl super::remote::RemoteProjector,
) -> Result<StreamOutcome> {
    let dispatcher = OutboxDispatcher {
        http: engine.http,
        database: engine.database,
    };
    let cursor = {
        let c = engine
            .database
            .lock()
            .map_err(|_| NightingaleError::internal("almacén no disponible"))?;
        SqliteSyncControlStore.cursor(&c, scope)?
    };
    let Some(cursor) = cursor else {
        return Ok(StreamOutcome::RecoveryRequired);
    };
    let mut response = match engine
        .http
        .events(&credential.token, &scope.account_id, &cursor.cursor)
        .await
    {
        Ok(response) => response,
        Err(error) => {
            dispatcher
                .deny(scope, &error)
                .map_err(|_| NightingaleError::temporary("acceso SSE no disponible"))?;
            return Ok(if matches!(error.status, Some(401 | 403)) {
                StreamOutcome::Revoked
            } else if error.code == APICode::CursorExpired {
                StreamOutcome::RecoveryRequired
            } else {
                StreamOutcome::Retry
            });
        }
    };

    let mut parser = Parser::default();

    // https://docs.rs/reqwest/0.12.28/reqwest/struct.Response.html#method.chunk
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| NightingaleError::temporary("conexión SSE interrumpida"))?
    {
        for notice in parser.feed(&chunk, &scope.account_id)? {
            match notice {
                Notice::Heartbeat => {}
                Notice::Revoked => {
                    dispatcher
                        .deny(
                            scope,
                            &APIFailure {
                                status: Some(403),
                                code: APICode::PermissionDenied,
                                correlation_id: None,
                            },
                        )
                        .map_err(|_| NightingaleError::temporary("denegación SSE no disponible"))?;

                    return Ok(StreamOutcome::Revoked);
                }
                Notice::Changes => {
                    let result = engine
                        .run(
                            credential,
                            scope,
                            CycleTrigger::Reconnect,
                            &SystemClock,
                            &UuidV7Generator,
                            projector,
                        )
                        .await?;

                    if result.outcome == CycleOutcome::AccessDenied {
                        return Ok(StreamOutcome::Revoked);
                    }

                    if result.outcome == CycleOutcome::RecoveryRequired {
                        return Ok(StreamOutcome::RecoveryRequired);
                    }
                }
            }
        }
    }

    Ok(StreamOutcome::Ended)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::components::sync::download_tests::scope;

    #[test]
    fn http_stream_repeats_use_applied_cursor_and_revoke_or_429_stop_safely() {
        use crate::tests::components::sync::download_tests::{Database, access, credential};
        use crate::{
            components::sync::coordinator::SyncCoordinator,
            platform::{config::APIConfig, http::APITransport},
        };
        use std::{
            io::{Read, Write},
            net::TcpListener,
        };

        tauri::async_runtime::block_on(async {
            for status in [200, 429] {
                let db = Database::new();
                let listener = TcpListener::bind("127.0.0.1:0").unwrap();
                let url = format!("http://{}/", listener.local_addr().unwrap());
                let data = db.data.clone();
                let handle = std::thread::spawn(move || {
                    let mut change_reads = 0;

                    for n in 0..if status == 200 { 9 } else { 1 } {
                        let (mut stream, _) = listener.accept().unwrap();
                        stream
                            .set_read_timeout(Some(std::time::Duration::from_secs(5)))
                            .unwrap();
                        let mut request = Vec::new();

                        while !request.windows(4).any(|w| w == b"\r\n\r\n") {
                            let mut b = [0; 4096];
                            let size = stream.read(&mut b).unwrap();
                            assert!(size > 0);
                            request.extend_from_slice(&b[..size]);
                        }
                        let request = String::from_utf8(request).unwrap();
                        assert!(data.try_lock().is_ok());

                        if n == 0 {
                            assert!(request.contains("cursor=c%2B%2F%3F%26%3D+x"));
                            assert!(!request.to_lowercase().contains("last-event-id"));
                            let notice = format!(
                                "id: forged\nevent: changes_available\ndata: {{\"accountId\":\"{}\",\"cursor\":\"advertised\"}}\n\n",
                                scope().account_id.as_str()
                            );
                            let body = format!(
                                "{notice}{notice}event: heartbeat\ndata: {{\"serverTime\":\"2026-10-30T12:00:00Z\"}}\n\nevent: access_revoked\n\n"
                            );
                            write!(stream,"HTTP/1.1 {status} Test\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",body.len(),body).unwrap();
                        } else {
                            let body=if request.contains("/access "){access(1,"active")} else {change_reads+=1;assert!(request.contains(if change_reads==1 {"cursor=c%2B%2F%3F%26%3D+x"} else {"cursor=applied"}));serde_json::json!({"groups":[],"nextCursor":"applied","hasMore":false})}.to_string();
                            write!(stream,"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",body.len(),body).unwrap();
                        }
                    }
                });
                let config = APIConfig::parse(&url).unwrap();
                let http = APITransport::new(APIConfig::parse(&url).unwrap()).unwrap();
                let coordinator = SyncCoordinator::default();
                let result = SyncEngine {
                    http: &http,
                    database: &db.data,
                    coordinator: &coordinator,
                }
                .listen_once(&credential(&config), &scope(), &())
                .await
                .unwrap();
                assert_eq!(
                    result,
                    if status == 200 {
                        StreamOutcome::Revoked
                    } else {
                        StreamOutcome::Retry
                    }
                );
                let c = db.data.lock().unwrap();
                let cursor = SqliteSyncControlStore
                    .cursor(&c, &scope())
                    .unwrap()
                    .unwrap();
                assert_eq!(
                    cursor.cursor,
                    if status == 200 { "applied" } else { "c+/?&= x" }
                );
                handle.join().unwrap();
            }
        });
    }

    #[test]
    fn chunks_crlf_multiline_repeats_and_event_ids_are_only_notices() {
        let mut parser = Parser::default();
        let text = format!(
            "id: forged\r\nevent: changes_available\r\ndata: {{\"accountId\":\"{}\",\r\ndata: \"cursor\":\"untrusted\"}}\r\n\r\nevent: heartbeat\ndata: {{\"serverTime\":\"2026-10-30T00:00:00Z\"}}\n\nevent: access_revoked\n\n",
            scope().account_id.as_str()
        );
        let mut events = Vec::new();
        for byte in text.as_bytes() {
            events.extend(parser.feed(&[*byte], &scope().account_id).unwrap());
        }
        assert_eq!(
            events,
            vec![Notice::Changes, Notice::Heartbeat, Notice::Revoked]
        );
        assert!(
            parser
                .feed(b"event: heartbeat\ndata: {}\n\n", &scope().account_id)
                .is_err()
        );
        assert!(
            Parser::default()
                .feed(&vec![b'x'; 16385], &scope().account_id)
                .is_err()
        );
        assert_eq!(reconnect_delay(1, 0), 3000);
        assert!(reconnect_delay(100, 0) <= 300000);
    }
}
