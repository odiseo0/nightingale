use super::{
    adapters::{
        api::{OperationBatchResult, SyncAPI},
        outbound::SqliteOperationStore,
    },
    domain::OperationEnvelope,
};
use crate::{
    components::{
        devices::activation::DeviceCredential,
        identity::{
            api::IdentityAPI,
            domain::{AccessStatus, AccountAccess},
            repository::{AccessStore, SqliteAccessStore},
        },
    },
    platform::http::{APIFailure, APITransport},
    shared_kernel::{
        Clock, IdGenerator, NightingaleError, Result,
        id::{OperationContext, OperationId},
    },
};
use rusqlite::Connection;
use std::{sync::Mutex, time::UNIX_EPOCH};

pub(crate) const LEASE_MILLIS: i64 = 60_000;

pub(crate) fn receipt_event(
    record: &super::adapters::api::OperationRecord,
    correlation: Option<OperationId>,
    duration: std::time::Duration,
) -> crate::platform::observability::Event {
    use super::adapters::api::OriginalStatus;
    use crate::platform::observability::{Event, Kind, Outcome};

    let outcome = match record.status {
        OriginalStatus::Accepted => Outcome::Accepted,
        OriginalStatus::Conflict => Outcome::Conflict,
        OriginalStatus::Rejected => Outcome::Rejected,
        OriginalStatus::RetryableFailure => Outcome::Retry,
    };
    let mut event = Event::new(Kind::Operation, outcome, duration);
    event.operation_id = Some(record.operation_id.clone());
    event.causation_id = Some(record.operation_id.clone());
    event.correlation_id =
        correlation.or_else(|| record.error.as_ref().map(|e| e.correlation_id.clone()));
    event.code = record.error.as_ref().map(|e| e.code);
    event
}

fn log_receipt(
    record: &super::adapters::api::OperationRecord,
    correlation: Option<OperationId>,
    duration: std::time::Duration,
) {
    crate::platform::observability::emit(&receipt_event(record, correlation, duration));
}

pub(crate) fn storage_error(_: rusqlite::Error) -> NightingaleError {
    NightingaleError::internal("almacén de envío no disponible")
}

fn log_failure(lease: &DispatchLease, error: &APIFailure, duration: std::time::Duration) {
    use crate::platform::{
        http::APICode,
        observability::{Event, Kind, Outcome, emit},
    };

    for entry in &lease.entries {
        let outcome = if matches!(error.status, Some(401 | 403)) {
            Outcome::Denied
        } else if error.status == Some(404) || error.code == APICode::ResultExpired {
            Outcome::Review
        } else {
            Outcome::Retry
        };

        let mut event = Event::new(Kind::Operation, outcome, duration);

        event.operation_id = Some(entry.envelope.operation_id.clone());
        event.causation_id = event.operation_id.clone();
        event.correlation_id = error.correlation_id.clone();
        event.code = Some(error.code);

        emit(&event);
    }
}

/// A durable attempt. Its response must match these attempts before reconciliation.
/// If the process stops first, the same envelopes remain available for recovery.
pub struct DispatchLease {
    pub(crate) context: OperationContext,
    pub(crate) until: i64,
    pub(crate) entries: Vec<LeasedOperation>,
}

pub enum RecoveryOutcome {
    Idle,
    Resolved(super::domain::OperationStatus),
    ReviewRequired(crate::platform::http::APICode),
}

pub(crate) struct LeasedOperation {
    pub envelope: OperationEnvelope,
    pub attempt: i64,
    pub delay: i64,
}

impl DispatchLease {
    pub fn operations(&self) -> Vec<OperationEnvelope> {
        self.entries
            .iter()
            .map(|entry| entry.envelope.clone())
            .collect()
    }
}

pub enum DispatchOutcome {
    Idle,
    Received {
        lease: DispatchLease,
        response: OperationBatchResult,
    },
}

#[derive(Debug)]
pub enum DispatchFailure {
    Local(NightingaleError),
    Remote(APIFailure),
    AccessChanged,
    AccessDenied,
}

/// One native dispatch attempt. No timer, background loop or SQLite lock spans HTTP.
pub struct OutboxDispatcher<'a> {
    pub http: &'a APITransport,
    pub database: &'a Mutex<Connection>,
}

pub(super) fn millis(clock: &impl Clock) -> Result<i64> {
    clock
        .now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|duration| i64::try_from(duration.as_millis()).ok())
        .ok_or_else(|| NightingaleError::internal("reloj inválido para envío"))
}

/// Exponential delay with bounded jitter; the caller supplies fresh random bits.
/// The maximum wait is five minutes, even after many failed attempts.
pub(crate) fn retry_delay(attempt: i64, jitter: u64) -> i64 {
    let ceiling = (1_000_i64 << (attempt.saturating_sub(1).clamp(0, 9) as u32)).min(300_000);

    ceiling / 2 + (jitter % (ceiling as u64 / 2 + 1)) as i64
}

impl OutboxDispatcher<'_> {
    fn local<T>(
        &self,
        work: impl FnOnce(&mut Connection) -> Result<T>,
    ) -> std::result::Result<T, DispatchFailure> {
        let mut connection = self.database.lock().map_err(|_| {
            DispatchFailure::Local(NightingaleError::internal("almacén de envío no disponible"))
        })?;

        work(&mut connection).map_err(DispatchFailure::Local)
    }

    pub(super) fn deny(
        &self,
        scope: &OperationContext,
        error: &APIFailure,
    ) -> std::result::Result<(), DispatchFailure> {
        if matches!(error.status, Some(401 | 403)) {
            self.local(|connection| {
                let tx = connection.transaction().map_err(storage_error)?;
                SqliteAccessStore.deny_cached_access(
                    &tx,
                    &scope.user_id,
                    &scope.device_id,
                    if error.status == Some(401) {
                        None
                    } else {
                        Some(&scope.account_id)
                    },
                )?;

                tx.commit().map_err(storage_error)?;

                Ok(())
            })?;
        }

        Ok(())
    }

    pub async fn dispatch(
        &self,
        credential: &DeviceCredential,
        scope: &OperationContext,
        clock: &impl Clock,
        ids: &impl IdGenerator,
    ) -> std::result::Result<DispatchOutcome, DispatchFailure> {
        let started = std::time::Instant::now();
        let access = self.authorize(credential, scope, clock).await?;
        let now = millis(clock).map_err(DispatchFailure::Local)?;

        // The existing UUID generator supplies random low bits, not a new operation ID.
        let jitter = ids.new_id().as_u128() as u64;
        let Some(lease) = self.local(|connection| {
            SqliteOperationStore.lease_batch(connection, &access, now, jitter)
        })?
        else {
            return Ok(DispatchOutcome::Idle);
        };

        match SyncAPI(self.http)
            .send_operations(&credential.token, &scope.account_id, &lease.operations())
            .await
        {
            Ok(response) => {
                let records = response
                    .value
                    .results
                    .iter()
                    .map(|result| result.original())
                    .collect::<std::result::Result<Vec<_>, _>>()
                    .map_err(DispatchFailure::Remote)?;
                let now = millis(clock).map_err(DispatchFailure::Local)?;

                self.local(|connection| {
                    SqliteOperationStore.reconcile(connection, &lease, &records, now)
                })?;

                for record in &records {
                    log_receipt(
                        record,
                        Some(response.value.correlation_id.clone()),
                        started.elapsed(),
                    );
                }

                Ok(DispatchOutcome::Received {
                    lease,
                    response: response.value,
                })
            }
            Err(mut error) => {
                if error.status == Some(404) {
                    error.code = crate::platform::http::APICode::NotFound;
                }
                let now = millis(clock).map_err(DispatchFailure::Local)?;
                self.local(|connection| {
                    SqliteOperationStore.release_unknown(connection, &lease, now, error.code)
                })?;

                self.deny(scope, &error)?;
                log_failure(&lease, &error, started.elapsed());

                Err(DispatchFailure::Remote(error))
            }
        }
    }

    /// Query one uncertain result after its scheduled wait. This never issues a POST.
    pub async fn recover(
        &self,
        credential: &DeviceCredential,
        scope: &OperationContext,
        operation: &OperationId,
        clock: &impl Clock,
        ids: &impl IdGenerator,
    ) -> std::result::Result<RecoveryOutcome, DispatchFailure> {
        let started = std::time::Instant::now();
        let access = self.authorize(credential, scope, clock).await?;
        let now = millis(clock).map_err(DispatchFailure::Local)?;
        let jitter = ids.new_id().as_u128() as u64;
        let Some(lease) = self.local(|connection| {
            SqliteOperationStore.lease_recovery(connection, &access, operation, now, jitter)
        })?
        else {
            return Ok(RecoveryOutcome::Idle);
        };

        match SyncAPI(self.http)
            .operation_result(&credential.token, &scope.account_id, operation)
            .await
        {
            Ok(response) => {
                let status = response.value.local_status();
                let now = millis(clock).map_err(DispatchFailure::Local)?;
                self.local(|connection| {
                    SqliteOperationStore.reconcile(
                        connection,
                        &lease,
                        std::slice::from_ref(&response.value),
                        now,
                    )
                })?;
                log_receipt(
                    &response.value,
                    response.correlation_id.clone(),
                    started.elapsed(),
                );

                Ok(RecoveryOutcome::Resolved(status))
            }
            Err(mut error) => {
                if error.status == Some(404) {
                    error.code = crate::platform::http::APICode::NotFound;
                }
                let now = millis(clock).map_err(DispatchFailure::Local)?;
                self.local(|connection| {
                    SqliteOperationStore.release_unknown(connection, &lease, now, error.code)
                })?;
                self.deny(scope, &error)?;
                log_failure(&lease, &error, started.elapsed());

                if error.status == Some(404)
                    || error.code == crate::platform::http::APICode::ResultExpired
                {
                    return Ok(RecoveryOutcome::ReviewRequired(error.code));
                }

                Err(DispatchFailure::Remote(error))
            }
        }
    }

    pub(super) async fn authorize(
        &self,
        credential: &DeviceCredential,
        scope: &OperationContext,
        clock: &impl Clock,
    ) -> std::result::Result<AccountAccess, DispatchFailure> {
        if credential.user_id != scope.user_id || credential.device_id != scope.device_id {
            return Err(DispatchFailure::AccessDenied);
        }

        // Refresh authorization before every batch, including retries.
        let response = match IdentityAPI(self.http)
            .access(&credential.token, &scope.account_id)
            .await
        {
            Ok(response) => response.value,
            Err(error) => {
                self.deny(scope, &error)?;
                return Err(DispatchFailure::Remote(error));
            }
        };

        if response.account_id != scope.account_id
            || response.user_id != scope.user_id
            || response.device_id != scope.device_id
        {
            return Err(DispatchFailure::AccessDenied);
        }

        let access = AccountAccess {
            context: OperationContext {
                permission_epoch: response.permission_epoch,
                ..scope.clone()
            },
            role: response.role,
            status: response.status,
            allowed_actions: response.allowed_actions,
            checked_at_millis: millis(clock).map_err(DispatchFailure::Local)?,
        };
        self.local(|connection| {
            let tx = connection.transaction().map_err(storage_error)?;
            SqliteAccessStore.save_access(&tx, &access)?;
            tx.commit().map_err(storage_error)?;

            Ok(())
        })?;

        if access.status != AccessStatus::Active {
            return Err(DispatchFailure::AccessDenied);
        }

        if access.context.permission_epoch != scope.permission_epoch {
            return Err(DispatchFailure::AccessChanged);
        }

        Ok(access)
    }
}
