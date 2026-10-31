use super::{
    api::{AccountSummary, ActivityStatus, DeviceRegistration, IdentityAPI},
    application::AccountActivation,
    domain::AccessStatus,
    oidc::{LoginAttempt, LoginCancellation, OidcProvider, OidcSession, SystemBrowser},
    repository::{AccessStore, SqliteAccessStore},
};
use crate::{
    components::{
        devices::activation::{DeviceCredential, DeviceCredentials},
        sync::{
            coordinator::{
                CycleOutcome, CycleReport, CycleTrigger, SyncCoordinator, SyncEngine, SyncStatus,
            },
            events::{StreamOutcome, reconnect_delay},
        },
    },
    platform::{config::APIConfig, http::APITransport, secrets::NativeSecretStore},
    shared_kernel::{NightingaleError, Result, SystemClock, UuidV7Generator, id::AccountId},
};
use rusqlite::Connection;
use serde::Serialize;
use std::sync::{Arc, Condvar, Mutex};
use tauri::{AppHandle, Manager, State};

const PROFILE: &str = "desktop";

type Pending = (OidcProvider, LoginAttempt);
type Stop = Arc<(Mutex<bool>, Condvar)>;

struct Watch {
    stop: Stop,
    task: Option<tauri::async_runtime::JoinHandle<()>>,
}

impl Drop for Watch {
    fn drop(&mut self) {
        if let Ok(mut stopped) = self.stop.0.lock() {
            *stopped = true;
            self.stop.1.notify_all();
        }

        if let Some(task) = &self.task {
            task.abort();
        }
    }
}

#[derive(Default)]
pub struct DesktopSession {
    gate: tauri::async_runtime::Mutex<()>,
    pending: Mutex<Option<Pending>>,
    pending_identity: Mutex<Option<(String, String)>>,
    cancellation: Mutex<LoginCancellation>,
    watch: Mutex<Option<Watch>>,
    last: Mutex<Option<CycleReport>>,
    last_success: Mutex<Option<(crate::shared_kernel::id::OperationContext, u64)>>,
}

fn failed(_: impl std::fmt::Debug) -> NightingaleError {
    NightingaleError::temporary("acción no disponible; revise configuración, acceso o conexión")
}
fn locked() -> NightingaleError {
    NightingaleError::internal("estado local no disponible")
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopView {
    pub last_sync_at: Option<u64>,
    pub reviews: Vec<crate::components::sync::adapters::outbound::ReviewSummary>,
    pub configured: bool,
    pub signed_in: bool,
    pub account_id: Option<AccountId>,
    pub access: Option<AccessStatus>,
    pub sync: Option<SyncStatus>,
    pub outcome: Option<CycleOutcome>,
}

fn credential(config: &APIConfig, c: &Mutex<Connection>) -> Result<DeviceCredential> {
    let context = SqliteAccessStore
        .context(&*c.lock().map_err(|_| locked())?)?
        .ok_or_else(|| NightingaleError::validation("inicie sesión"))?;
    let value = DeviceCredentials {
        config,
        profile: PROFILE,
        identity: (&context.issuer, &context.subject),
        secrets: &NativeSecretStore,
    }
    .load()
    .map_err(failed)?
    .ok_or_else(|| NightingaleError::validation("credencial local no disponible; inicie sesión"))?;

    if value.user_id != context.user_id {
        return Err(NightingaleError::validation("identidad local no coincide"));
    }

    Ok(value)
}

#[tauri::command]
pub fn identity_get_status(app: AppHandle) -> Result<DesktopView> {
    let c = app.state::<Mutex<Connection>>();
    let session = app.state::<DesktopSession>();
    let (context, access) = {
        let c = c.lock().map_err(|_| locked())?;
        (
            SqliteAccessStore.context(&c)?,
            SqliteAccessStore.selected_access(&c)?,
        )
    };
    let last = session.last.lock().map_err(|_| locked())?;

    Ok(DesktopView {
        last_sync_at: session
            .last_success
            .lock()
            .map_err(|_| locked())?
            .as_ref()
            .filter(|(context, _)| access.as_ref().is_some_and(|a| a.context == *context))
            .map(|(_, time)| *time),
        reviews: match &access {
            Some(access) => crate::components::sync::adapters::outbound::SqliteOperationStore
                .reviews(&*c.lock().map_err(|_| locked())?, &access.context)?,
            None => vec![],
        },
        configured: APIConfig::from_environment().is_ok(),
        signed_in: context.is_some(),
        account_id: context.and_then(|c| c.selected_account_id),
        access: access.as_ref().map(|a| a.status.clone()),
        sync: access
            .as_ref()
            .map(|a| app.state::<SyncCoordinator>().status(&c, &a.context))
            .transpose()?,
        outcome: last
            .as_ref()
            .filter(|r| {
                access
                    .as_ref()
                    .is_some_and(|a| a.context == r.status.context)
            })
            .map(|r| r.outcome),
    })
}

#[tauri::command]
pub async fn identity_begin_login(app: AppHandle) -> Result<()> {
    let state = app.state::<DesktopSession>();
    let _gate = state.gate.lock().await;
    let cancellation = LoginCancellation::default();
    *state.cancellation.lock().map_err(|_| locked())? = cancellation.clone();

    if state.pending.lock().map_err(|_| locked())?.is_some() {
        return Err(NightingaleError::validation("ya existe un acceso en curso"));
    }

    let c = app.state::<Mutex<Connection>>();

    if SqliteAccessStore
        .context(&*c.lock().map_err(|_| locked())?)?
        .is_some()
    {
        return Err(NightingaleError::validation("cierre la sesión anterior"));
    }

    let config = APIConfig::from_environment()?;
    let http = APITransport::new(config).map_err(failed)?;
    let auth = IdentityAPI(&http)
        .auth_config()
        .await
        .map_err(failed)?
        .value;
    let provider = OidcProvider::discover(&auth, &http).await.map_err(failed)?;

    if cancellation.is_cancelled() {
        return Err(NightingaleError::temporary("acceso cancelado"));
    }

    let attempt = provider.begin(&SystemBrowser).map_err(failed)?;
    *state.pending.lock().map_err(|_| locked())? = Some((provider, attempt));

    Ok(())
}

#[tauri::command]
pub fn identity_cancel_login(state: State<DesktopSession>) -> Result<()> {
    state.cancellation.lock().map_err(|_| locked())?.cancel();
    state.pending.lock().map_err(|_| locked())?.take();

    Ok(())
}

async fn accounts(
    http: &APITransport,
    credential: &DeviceCredential,
) -> Result<Vec<AccountSummary>> {
    let mut result = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let mut page = 1;

    loop {
        let response = IdentityAPI(http)
            .accounts(&credential.token, page, 100)
            .await
            .map_err(failed)?
            .value;
        let count = response.items.len();

        for item in response.items {
            if !seen.insert(item.id.as_str().to_owned()) {
                return Err(NightingaleError::validation("página de cuentas repetida"));
            }

            result.push(item);
        }

        if result.len() as u64 >= response.total {
            return Ok(result);
        }

        if count == 0 {
            return Err(NightingaleError::validation("página de cuentas incompleta"));
        }

        page += 1;
    }
}

#[tauri::command]
pub async fn identity_complete_login(app: AppHandle) -> Result<Vec<AccountSummary>> {
    let state = app.state::<DesktopSession>();
    let _gate = state.gate.lock().await;
    let (provider, attempt) = state
        .pending
        .lock()
        .map_err(|_| locked())?
        .take()
        .ok_or_else(|| NightingaleError::validation("no hay acceso pendiente"))?;
    let cancel = state.cancellation.lock().map_err(|_| locked())?.clone();
    let config = APIConfig::from_environment()?;
    let http = APITransport::new(APIConfig::parse(config.base_url().as_str())?).map_err(failed)?;
    let session = provider
        .complete(attempt, cancel, &http)
        .await
        .map_err(failed)?;
    *state.pending_identity.lock().map_err(|_| locked())? =
        Some((session.issuer().into(), session.subject().into()));
    let c = app.state::<Mutex<Connection>>();
    let registration = DeviceRegistration {
        device_name: format!("Nightingale ({})", config.platform()),
        client_version: config.client_version().into(),
        local_schema_version: c
            .lock()
            .map_err(|_| locked())?
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .map_err(failed)?,
    };
    let credential = DeviceCredentials {
        config: &config,
        profile: PROFILE,
        identity: (session.issuer(), session.subject()),
        secrets: &NativeSecretStore,
    }
    .ensure(&http, session.access_token(), &registration)
    .await
    .map_err(failed)?;
    let me = IdentityAPI(&http)
        .me(&credential.token)
        .await
        .map_err(failed)?
        .value;

    if me.user_id != credential.user_id
        || me.oidc_issuer != session.issuer()
        || me.oidc_subject != session.subject()
        || !matches!(me.status, ActivityStatus::Active)
    {
        return Err(NightingaleError::validation("identidad no autorizada"));
    }

    session
        .persist(&config, PROFILE, &NativeSecretStore)
        .map_err(failed)?;

    {
        let mut c = c.lock().map_err(|_| locked())?;
        let tx = c.transaction().map_err(failed)?;
        SqliteAccessStore.save_context(
            &tx,
            &super::domain::IdentityContext {
                issuer: session.issuer().into(),
                subject: session.subject().into(),
                user_id: credential.user_id.clone(),
                selected_account_id: None,
            },
        )?;
        tx.commit().map_err(failed)?;
    }

    accounts(&http, &credential).await
}

#[tauri::command]
pub async fn identity_list_accounts(app: AppHandle) -> Result<Vec<AccountSummary>> {
    let state = app.state::<DesktopSession>();
    let _gate = state.gate.lock().await;
    let config = APIConfig::from_environment()?;
    let c = app.state::<Mutex<Connection>>();
    let credential = credential(&config, &c)?;

    accounts(&APITransport::new(config).map_err(failed)?, &credential).await
}

#[tauri::command]
pub async fn identity_activate_account(app: AppHandle, account_id: AccountId) -> Result<()> {
    let state = app.state::<DesktopSession>();
    let _gate = state.gate.lock().await;

    stop_watch_join(&app).await?;

    let config = APIConfig::from_environment()?;
    let http = APITransport::new(APIConfig::parse(config.base_url().as_str())?).map_err(failed)?;
    let c = app.state::<Mutex<Connection>>();
    let context = SqliteAccessStore
        .context(&*c.lock().map_err(|_| locked())?)?
        .ok_or_else(|| NightingaleError::validation("inicie sesión"))?;
    let session = OidcSession::load(
        &config,
        PROFILE,
        (&context.issuer, &context.subject),
        &NativeSecretStore,
    )
    .map_err(failed)?
    .ok_or_else(|| NightingaleError::validation("inicie sesión"))?;
    let access = AccountActivation {
        config: &config,
        profile: PROFILE,
        http: &http,
        secrets: &NativeSecretStore,
        database: &c,
    }
    .activate(&session, &account_id, &UuidV7Generator, &SystemClock)
    .await
    .map_err(failed)?;

    if access.status != AccessStatus::Active {
        return Err(NightingaleError::validation(
            "cuenta o instalación sin acceso",
        ));
    }

    start_watch(app.clone())?;

    Ok(())
}

#[tauri::command]
pub async fn identity_logout(app: AppHandle) -> Result<()> {
    let state = app.state::<DesktopSession>();
    state.cancellation.lock().map_err(|_| locked())?.cancel();
    let _gate = state.gate.lock().await;
    stop_watch_join(&app).await?;
    state.pending.lock().map_err(|_| locked())?.take();
    let c = app.state::<Mutex<Connection>>();
    let context = SqliteAccessStore.context(&*c.lock().map_err(|_| locked())?)?;
    let identity = context
        .map(|c| (c.issuer, c.subject))
        .or_else(|| state.pending_identity.lock().ok().and_then(|c| c.clone()));

    if let Some((issuer, subject)) = identity {
        let config = APIConfig::from_environment()?;
        let http =
            APITransport::new(APIConfig::parse(config.base_url().as_str())?).map_err(failed)?;

        AccountActivation {
            config: &config,
            profile: PROFILE,
            http: &http,
            secrets: &NativeSecretStore,
            database: &c,
        }
        .logout((&issuer, &subject))
        .map_err(failed)?;
    }

    *state.last.lock().map_err(|_| locked())? = None;
    state.last_success.lock().map_err(|_| locked())?.take();
    state.pending_identity.lock().map_err(|_| locked())?.take();

    Ok(())
}

#[tauri::command]
pub async fn sync_now(app: AppHandle) -> Result<CycleReport> {
    let state = app.state::<DesktopSession>();
    let _gate = state.gate.lock().await;
    let result = run_cycle(&app, CycleTrigger::Manual).await?;

    if result.outcome == CycleOutcome::Complete {
        start_watch(app.clone())?;
    }

    Ok(result)
}

async fn run_cycle(app: &AppHandle, trigger: CycleTrigger) -> Result<CycleReport> {
    let config = APIConfig::from_environment()?;
    let c = app.state::<Mutex<Connection>>();
    let credential = credential(&config, &c)?;
    let context = {
        let c = c.lock().map_err(|_| locked())?;
        let account = SqliteAccessStore
            .context(&c)?
            .and_then(|c| c.selected_account_id)
            .ok_or_else(|| NightingaleError::validation("elija una cuenta"))?;
        SqliteAccessStore
            .access(&c, &account, &credential.user_id, &credential.device_id)?
            .ok_or_else(|| NightingaleError::validation("acceso local no disponible"))?
            .context
    };
    let http = APITransport::new(config).map_err(failed)?;
    let coordinator = app.state::<SyncCoordinator>();
    let report = SyncEngine {
        coordinator: &coordinator,
        http: &http,
        database: &c,
    }
    .run(
        &credential,
        &context,
        trigger,
        &SystemClock,
        &UuidV7Generator,
        &(),
    )
    .await?;
    *app.state::<DesktopSession>()
        .last
        .lock()
        .map_err(|_| locked())? = Some(report.clone());

    if report.outcome == CycleOutcome::Complete
        && let Some(time) = crate::platform::observability::timestamp()
    {
        *app.state::<DesktopSession>()
            .last_success
            .lock()
            .map_err(|_| locked())? = Some((report.status.context.clone(), time));
    }

    Ok(report)
}

pub fn stop_watch(app: &AppHandle) {
    if let Ok(mut watch) = app.state::<DesktopSession>().watch.lock() {
        watch.take();
    }
}

async fn stop_watch_join(app: &AppHandle) -> Result<()> {
    let mut watch = app
        .state::<DesktopSession>()
        .watch
        .lock()
        .map_err(|_| locked())?
        .take();

    if let Some(mut watch) = watch.take() {
        let task = watch.task.take();
        drop(watch);

        if let Some(task) = task {
            task.abort();
            let _ = task.await;
        }
    }

    Ok(())
}

pub fn start_watch(app: AppHandle) -> Result<()> {
    let state = app.state::<DesktopSession>();
    let mut current = state.watch.lock().map_err(|_| locked())?;

    if current.as_ref().is_some_and(|watch| {
        watch
            .task
            .as_ref()
            .is_some_and(|task| !task.inner().is_finished())
    }) {
        return Ok(());
    }

    current.take();

    let stop: Stop = Arc::new((Mutex::new(false), Condvar::new()));
    let signal = stop.clone();
    let app = app.clone();
    let task = tauri::async_runtime::spawn(async move {
        let mut attempt = 0;

        loop {
            if *signal.0.lock().unwrap_or_else(|e| e.into_inner()) {
                break;
            }

            let result = run_cycle(
                &app,
            if attempt == 0 {
                    CycleTrigger::Startup
                } else {
                    CycleTrigger::Reconnect
                },
            )
            .await;

            if matches!(
                result.as_ref().map(|r| r.outcome),
                Ok(CycleOutcome::AccessDenied)
            ) {
                break;
            }

            let connection = async {
                let config = APIConfig::from_environment()?;
                let c = app.state::<Mutex<Connection>>();
                let cred = credential(&config, &c)?;
                let access = SqliteAccessStore
                    .selected_access(&*c.lock().map_err(|_| locked())?)?
                    .ok_or_else(|| NightingaleError::validation("sin cuenta activa"))?;
                let http = APITransport::new(config).map_err(failed)?;
                let coord = app.state::<SyncCoordinator>();
                SyncEngine {
                    coordinator: &coord,
                    http: &http,
                    database: &c,
                }
                .listen_once(&cred, &access.context, &())
                .await
            }
            .await;

            if matches!(connection, Ok(StreamOutcome::Revoked)) {
                break;
            }

            attempt += 1;
            let delay = reconnect_delay(attempt, uuid::Uuid::now_v7().as_u128() as u64);
            let signal = signal.clone();
            let stopped = tauri::async_runtime::spawn_blocking(move || {
                let lock = signal.0.lock().unwrap_or_else(|e| e.into_inner());
                let (lock, _) = signal
                    .1
                    .wait_timeout_while(
                        lock,
                        std::time::Duration::from_millis(delay),
                        |cancelled| !*cancelled,
                    )
                    .unwrap_or_else(|e| e.into_inner());
                *lock
            })
            .await
            .unwrap_or(true);

            if stopped {
                break;
            }
        }
    });

    *current = Some(Watch {
        stop,
        task: Some(task),
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        components::{
            identity::domain::{AccessRole, AccountAccess, IdentityContext},
            sync::{
                adapters::outbound::SqliteOperationStore, application::record_operation,
                domain::NewOperation,
            },
        },
        platform::sqlite::open,
        shared_kernel::id::OperationContext,
    };
    use serde_json::json;

    #[test]
    fn cached_selection_and_review_dto_preserve_scope_without_payload_or_remote_state() {
        let path = std::env::temp_dir().join(format!(
            "nightingale-desktop-{}.sqlite",
            uuid::Uuid::now_v7()
        ));
        let mut c = open(&path).unwrap();
        let scope:OperationContext=serde_json::from_value(json!({"accountId":"019a1234-0000-7000-8000-000000000001","userId":"019a1234-0000-7000-8000-000000000002","deviceId":"019a1234-0000-7000-8000-000000000003","permissionEpoch":1})).unwrap();
        let tx = c.transaction().unwrap();
        SqliteAccessStore
            .save_context(
                &tx,
                &IdentityContext {
                    issuer: "https://issuer.example".into(),
                    subject: "subject".into(),
                    user_id: scope.user_id.clone(),
                    selected_account_id: Some(scope.account_id.clone()),
                },
            )
            .unwrap();
        SqliteAccessStore
            .save_access(
                &tx,
                &AccountAccess {
                    context: scope.clone(),
                    role: AccessRole::Owner,
                    status: AccessStatus::Active,
                    allowed_actions: vec!["RegisterPatient".into()],
                    checked_at_millis: 1,
                },
            )
            .unwrap();
        record_operation(
            &tx,
            &SqliteOperationStore,
            &UuidV7Generator,
            &SystemClock,
            NewOperation {
                context: scope.clone(),
                depends_on: vec![],
                action: "RegisterPatient".into(),
                entity_type: "patient_profile".into(),
                entity_key: json!("019a1234-0000-7000-8000-000000000010"),
                expected_version: 0,
                payload: json!({"fullName":"DO_NOT_EXPOSE"}),
                occurred_at_local: None,
            },
        )
        .unwrap();
        tx.commit().unwrap();
        c.execute("UPDATE sync_outbox SET status='conflict',last_error='\"VERSION_CONFLICT\"',result='DO_NOT_EXPOSE'",[]).unwrap();
        assert_eq!(
            SqliteAccessStore
                .selected_access(&c)
                .unwrap()
                .unwrap()
                .context,
            scope
        );
        let reviews = SqliteOperationStore.reviews(&c, &scope).unwrap();
        assert_eq!(reviews.len(), 1);
        let view = DesktopView {
            last_sync_at: None,
            reviews,
            configured: false,
            signed_in: true,
            account_id: Some(scope.account_id.clone()),
            access: Some(AccessStatus::Active),
            sync: None,
            outcome: None,
        };
        let value = serde_json::to_value(view).unwrap();
        assert!(value.get("signedIn").unwrap().as_bool().unwrap());
        let output = value.to_string();
        for word in [
            "DO_NOT_EXPOSE",
            "fullName",
            "payload",
            "entityKey",
            "result",
        ] {
            assert!(!output.contains(word));
        }
        assert!(
            !serde_json::to_string(&failed("secret-token SQL private path"))
                .unwrap()
                .contains("secret-token")
        );
        let mut other = scope;
        other.permission_epoch = 2;
        other.account_id =
            serde_json::from_value(json!("019a1234-0000-7000-8000-000000000099")).unwrap();
        assert!(SqliteOperationStore.reviews(&c, &other).unwrap().is_empty());
        drop(c);
        std::fs::remove_file(path).unwrap();
    }
}
