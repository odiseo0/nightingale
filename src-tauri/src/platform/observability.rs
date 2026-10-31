use super::http::APICode;
use crate::shared_kernel::id::OperationId;
use serde::Serialize;
use std::{
    io::Write,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Http,
    Operation,
    Cycle,
    Sse,
}
#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Success,
    Accepted,
    Conflict,
    Rejected,
    Retry,
    Review,
    Denied,
    Offline,
    Failed,
}

/// Only typed metadata can reach logs. There is no message, URL, token, cursor,
/// payload, entity key, SQL text or remote state field in this interface.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub kind: Kind,
    pub outcome: Outcome,
    pub timestamp_millis: Option<u64>,
    pub duration_millis: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<OperationId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub causation_id: Option<OperationId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<OperationId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<APICode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pending: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conflicts: Option<i64>,
}

pub fn timestamp() -> Option<u64> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|d| u64::try_from(d.as_millis()).ok())
}

impl Event {
    pub fn new(kind: Kind, outcome: Outcome, elapsed: Duration) -> Self {
        Self {
            kind,
            outcome,
            timestamp_millis: timestamp(),
            duration_millis: elapsed.as_millis().min(u64::MAX as u128) as u64,
            operation_id: None,
            causation_id: None,
            correlation_id: None,
            code: None,
            pending: None,
            conflicts: None,
        }
    }
}

/// Telemetry errors cannot change the use case's result.
pub fn write(writer: &mut impl Write, event: &Event) {
    if let Ok(mut bytes) = serde_json::to_vec(event) {
        bytes.push(b'\n');
        let _ = writer.write_all(&bytes);
    }
}
pub fn emit(event: &Event) {
    write(&mut std::io::stderr().lock(), event);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn whitelist_serialization_keeps_trace_ids_but_no_external_text() {
        let id = OperationId::try_from("019a1234-0000-7000-8000-000000000001".to_owned()).unwrap();
        let mut event = Event::new(Kind::Operation, Outcome::Conflict, Duration::from_millis(7));
        event.operation_id = Some(id.clone());
        event.causation_id = Some(id.clone());
        event.correlation_id = Some(id);
        event.code = Some(APICode::VersionConflict);
        let mut bytes = Vec::new();
        write(&mut bytes, &event);
        let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(value["durationMillis"], 7);
        assert_eq!(value["code"], "VERSION_CONFLICT");
        assert!(value["operationId"].is_string());
        for name in [
            "payload",
            "message",
            "details",
            "token",
            "state",
            "entityKey",
            "url",
            "cursor",
        ] {
            assert!(value.get(name).is_none());
        }
    }
    #[test]
    fn broken_telemetry_does_not_fail_or_panic() {
        struct Broken;
        impl Write for Broken {
            fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
                Err(std::io::Error::other("secret destination"))
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        write(
            &mut Broken,
            &Event::new(Kind::Http, Outcome::Failed, Duration::ZERO),
        );
    }
}
