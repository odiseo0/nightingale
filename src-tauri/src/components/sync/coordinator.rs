use super::{
    adapters::{
        control::SqliteSyncControlStore,
        outbound::{OutboxSummary, SqliteOperationStore},
    },
    application::CursorStore,
    dispatch::{DispatchFailure, DispatchOutcome, OutboxDispatcher, millis},
    download::{ChangeDownloader, DownloadOutcome},
    remote::RemoteProjector,
    snapshot::{SnapshotMode, SnapshotOutcome, SnapshotRecovery},
};
use crate::{
    components::{
        devices::activation::DeviceCredential,
        identity::{
            domain::AccessStatus,
            repository::{AccessStore, SqliteAccessStore},
        },
    },
    platform::http::{APICode, APITransport},
    shared_kernel::{
        Clock, ErrorCode, IdGenerator, NightingaleError, Result, id::OperationContext,
    },
};
use rusqlite::Connection;
use serde::Serialize;
use std::{
    collections::HashMap,
    future::Future,
    sync::{Arc, Mutex},
};
use tauri::async_runtime::Mutex as AsyncMutex;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CycleTrigger {
    Startup,
    Reconnect,
    Manual,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncPhase {
    #[default]
    Idle,
    Access,
    Recovering,
    Sending,
    Downloading,
    Rebuilding,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncStatus {
    pub context: OperationContext,
    pub phase: SyncPhase,
    pub outbox: OutboxSummary,
    pub cursor_ready: bool,
    pub snapshot_pending: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CycleOutcome {
    Complete,
    Offline,
    RetryWaiting,
    AccessDenied,
    RecoveryRequired,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CycleReport {
    pub outcome: CycleOutcome,
    pub status: SyncStatus,
    pub remote_code: Option<APICode>,
    pub local_code: Option<ErrorCode>,
}

#[derive(Default)]
struct FlightState {
    generation: u64,
    phase: SyncPhase,
    manual: bool,
    last: Option<CycleReport>,
}

#[derive(Default)]
pub(crate) struct Flight {
    gate: AsyncMutex<()>,
    events: AsyncMutex<()>,
    state: Mutex<FlightState>,
}

fn unavailable() -> NightingaleError {
    NightingaleError::internal("coordinador no disponible")
}

impl Flight {
    pub(crate) fn phase(&self, phase: SyncPhase) -> Result<()> {
        self.state.lock().map_err(|_| unavailable())?.phase = phase;

        Ok(())
    }
}

// Cancellation releases the async gate and clears the transient phase. Durable
// leases and intentions remain in SQLite; a later cycle recovers their results.
struct Running<'a>(&'a Flight);

impl Drop for Running<'_> {
    fn drop(&mut self) {
        if let Ok(mut state) = self.0.state.lock() {
            state.phase = SyncPhase::Idle;
        }
    }
}

type ScopeKey = (String, String, String);

/// Keep one coordinator per local profile. Each account/actor/device has one
/// cycle; changing permission epoch never changes ownership of saved work.
#[derive(Default)]
pub struct SyncCoordinator {
    flights: Mutex<HashMap<ScopeKey, Arc<Flight>>>,
}

impl SyncCoordinator {
    pub(super) async fn listen_once(
        &self,
        engine: &SyncEngine<'_>,
        credential: &DeviceCredential,
        scope: &OperationContext,
        projector: &impl RemoteProjector,
    ) -> Result<super::events::StreamOutcome> {
        let flight = self.flight(scope)?;
        let Ok(_guard) = flight.events.try_lock() else {
            return Ok(super::events::StreamOutcome::Busy);
        };

        super::events::listen(engine, credential, scope, projector).await
    }

    pub(crate) fn flight(&self, scope: &OperationContext) -> Result<Arc<Flight>> {
        let key = (
            scope.account_id.as_str().to_owned(),
            scope.user_id.as_str().to_owned(),
            scope.device_id.as_str().to_owned(),
        );

        Ok(self
            .flights
            .lock()
            .map_err(|_| unavailable())?
            .entry(key)
            .or_default()
            .clone())
    }

    pub(crate) async fn run_with<F, Fut>(
        &self,
        scope: &OperationContext,
        trigger: CycleTrigger,
        mut work: F,
    ) -> Result<CycleReport>
    where
        F: FnMut(bool) -> Fut,
        Fut: Future<Output = Result<CycleReport>>,
    {
        let flight = self.flight(scope)?;
        let generation = {
            let mut state = flight.state.lock().map_err(|_| unavailable())?;
            state.manual |= trigger == CycleTrigger::Manual;
            state.generation
        };

        // Tauri re-exports the installed Tokio 1.54.0 mutex. The guard protects
        // only a cycle, never a SQLite connection, and waits without blocking.
        // https://docs.rs/tauri/2.12.4/tauri/async_runtime/index.html
        // https://docs.rs/tokio/1.54.0/tokio/sync/struct.Mutex.html#method.lock
        let _gate = flight.gate.lock().await;

        {
            let state = flight.state.lock().map_err(|_| unavailable())?;

            if state.generation != generation {
                return state.last.clone().ok_or_else(unavailable);
            }
        }

        let _running = Running(&flight);

        loop {
            let manual = {
                let mut state = flight.state.lock().map_err(|_| unavailable())?;
                std::mem::take(&mut state.manual)
            };
            let mut report = work(manual).await?;
            let mut state = flight.state.lock().map_err(|_| unavailable())?;

            // A manual request that joined during HTTP gets one more pass in
            // this cycle. It cannot run beside the active pass or lose its retry.
            if state.manual {
                continue;
            }

            report.status.phase = SyncPhase::Idle;
            state.phase = SyncPhase::Idle;
            state.generation = state.generation.wrapping_add(1);
            state.last = Some(report.clone());

            return Ok(report);
        }
    }

    /// Reads only local durable state. Connectivity and in-progress phases are
    /// transient; a new process derives pending work, results and waits again.
    pub fn status(
        &self,
        database: &Mutex<Connection>,
        scope: &OperationContext,
    ) -> Result<SyncStatus> {
        let phase = self
            .flight(scope)?
            .state
            .lock()
            .map_err(|_| unavailable())?
            .phase;
        let c = database.lock().map_err(|_| unavailable())?;

        Ok(SyncStatus {
            context: scope.clone(),
            phase,
            outbox: SqliteOperationStore.summary(&c, scope)?,
            cursor_ready: SqliteSyncControlStore.cursor(&c, scope)?.is_some(),
            snapshot_pending: SqliteSyncControlStore.snapshot_pending(&c, scope)?,
        })
    }
}

/// Native entry point shared by startup, reconnect and manual requests. Session
/// composition supplies the credential; this case never issues or logs tokens.
pub struct SyncEngine<'a> {
    pub coordinator: &'a SyncCoordinator,
    pub http: &'a APITransport,
    pub database: &'a Mutex<Connection>,
}

impl SyncEngine<'_> {
    pub async fn run(
        &self,
        credential: &DeviceCredential,
        scope: &OperationContext,
        trigger: CycleTrigger,
        clock: &impl Clock,
        ids: &impl IdGenerator,
        projector: &impl RemoteProjector,
    ) -> Result<CycleReport> {
        self.coordinator
            .run_with(scope, trigger, |manual| {
                self.pass(credential, scope, manual, clock, ids, projector)
            })
            .await
    }

    async fn pass(
        &self,
        credential: &DeviceCredential,
        requested: &OperationContext,
        manual: bool,
        clock: &impl Clock,
        ids: &impl IdGenerator,
        projector: &impl RemoteProjector,
    ) -> Result<CycleReport> {
        let started = std::time::Instant::now();
        let mut scope = requested.clone();
        let result = self
            .exchange(credential, &mut scope, manual, clock, ids, projector)
            .await;
        let (outcome, remote_code, local_code) = match result {
            Ok(outcome) => (outcome, None, None),
            Err(DispatchFailure::AccessDenied) => (CycleOutcome::AccessDenied, None, None),
            Err(DispatchFailure::AccessChanged) => (CycleOutcome::RecoveryRequired, None, None),
            Err(DispatchFailure::Local(error)) => (CycleOutcome::Failed, None, Some(error.code)),
            Err(DispatchFailure::Remote(error)) => {
                let outcome = if matches!(error.status, Some(401 | 403)) {
                    CycleOutcome::AccessDenied
                } else if error.status.is_none() && error.code == APICode::TemporaryFailure {
                    CycleOutcome::Offline
                } else if matches!(error.status, Some(429 | 503))
                    || matches!(
                        error.code,
                        APICode::TemporaryFailure | APICode::ServiceUnavailable
                    )
                {
                    CycleOutcome::RetryWaiting
                } else {
                    CycleOutcome::Failed
                };
                (outcome, Some(error.code), None)
            }
        };
        let report = CycleReport {
            outcome,
            status: self.coordinator.status(self.database, &scope)?,
            remote_code,
            local_code,
        };

        use crate::platform::observability::{Event, Kind, Outcome, emit};

        let mut event = Event::new(
            Kind::Cycle,
            match outcome {
                CycleOutcome::Complete => Outcome::Success,
                CycleOutcome::Offline => Outcome::Offline,
                CycleOutcome::AccessDenied => Outcome::Denied,
                CycleOutcome::RecoveryRequired => Outcome::Review,
                CycleOutcome::RetryWaiting => Outcome::Retry,
                CycleOutcome::Failed => Outcome::Failed,
            },
            started.elapsed(),
        );
        let q = &report.status.outbox;
        event.pending = Some(q.pending + q.unknown + q.in_flight + q.retryable);
        event.conflicts = Some(q.conflicts);
        event.code = report.remote_code;

        emit(&event);

        Ok(report)
    }

    async fn exchange(
        &self,
        credential: &DeviceCredential,
        scope: &mut OperationContext,
        manual: bool,
        clock: &impl Clock,
        ids: &impl IdGenerator,
        projector: &impl RemoteProjector,
    ) -> std::result::Result<CycleOutcome, DispatchFailure> {
        let flight = self
            .coordinator
            .flight(scope)
            .map_err(DispatchFailure::Local)?;
        let phase = |p| flight.phase(p).map_err(DispatchFailure::Local);
        let dispatcher = OutboxDispatcher {
            http: self.http,
            database: self.database,
        };
        let downloader = ChangeDownloader {
            http: self.http,
            database: self.database,
        };

        phase(SyncPhase::Access)?;

        let access = match dispatcher.authorize(credential, scope, clock).await {
            Ok(access) => access,
            Err(DispatchFailure::AccessChanged) => {
                let access = downloader
                    .local(|c| {
                        SqliteAccessStore
                            .access(c, &scope.account_id, &scope.user_id, &scope.device_id)
                            .map_err(DispatchFailure::Local)
                    })?
                    .filter(|a| a.status == AccessStatus::Active)
                    .ok_or(DispatchFailure::AccessDenied)?;
                *scope = access.context.clone();
                access
            }
            Err(error) => return Err(error),
        };

        phase(SyncPhase::Recovering)?;

        let candidates = downloader.local(|c| {
            SqliteOperationStore
                .prepare_cycle(
                    c,
                    &access,
                    millis(clock).map_err(DispatchFailure::Local)?,
                    manual,
                )
                .map_err(DispatchFailure::Local)
        })?;

        for id in candidates {
            dispatcher
                .recover(credential, scope, &id, clock, ids)
                .await?;
        }

        phase(SyncPhase::Sending)?;

        while let DispatchOutcome::Received { .. } =
            dispatcher.dispatch(credential, scope, clock, ids).await?
        {}

        phase(SyncPhase::Downloading)?;

        let download = downloader
            .download(credential, scope, clock, projector)
            .await?;
        let snapshot_pending = downloader.local(|c| {
            SqliteSyncControlStore
                .snapshot_pending(c, scope)
                .map_err(DispatchFailure::Local)
        })?;

        if snapshot_pending || matches!(download, DownloadOutcome::RecoveryRequired) {
            phase(SyncPhase::Rebuilding)?;
            let recovery = SnapshotRecovery {
                http: self.http,
                database: self.database,
            };
            // Resume a download first. Restart only after an explicit recovery
            // result (expired scope/capture). Recovery has a fixed retry count.
            let mut rebuilt = recovery
                .rebuild(credential, scope, clock, SnapshotMode::Resume, projector)
                .await?;

            if matches!(rebuilt, SnapshotOutcome::RecoveryRequired) {
                rebuilt = recovery
                    .rebuild(credential, scope, clock, SnapshotMode::Restart, projector)
                    .await?;
            }

            if matches!(rebuilt, SnapshotOutcome::RecoveryRequired) {
                return Ok(CycleOutcome::RecoveryRequired);
            }

            phase(SyncPhase::Downloading)?;

            if matches!(
                downloader
                    .download(credential, scope, clock, projector)
                    .await?,
                DownloadOutcome::RecoveryRequired
            ) {
                // Resume may have only released an already installed capture.
                // If its cursor expired, one new capture is still required.
                phase(SyncPhase::Rebuilding)?;

                if matches!(
                    recovery
                        .rebuild(credential, scope, clock, SnapshotMode::Restart, projector)
                        .await?,
                    SnapshotOutcome::RecoveryRequired
                ) {
                    return Ok(CycleOutcome::RecoveryRequired);
                }

                phase(SyncPhase::Downloading)?;

                if matches!(
                    downloader
                        .download(credential, scope, clock, projector)
                        .await?,
                    DownloadOutcome::RecoveryRequired
                ) {
                    return Ok(CycleOutcome::RecoveryRequired);
                }
            }
        }

        Ok(CycleOutcome::Complete)
    }
}
