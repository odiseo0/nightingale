//! SQLite persistence and translation of accepted API projections.
use super::super::{
    application::{AccountError, AccountReadRepository, AccountViewQuery, Result},
    domain::{AccountPolicy, AccountStatus, AccountView, Preference, Preferences, Theme},
};
use crate::{
    components::sync::{
        adapters::remote_models::{AccountResponse, AccountResponseStatus, RemoteState},
        remote::{EntityType, RemoteChange, RemoteProjector},
    },
    shared_kernel::{Clock, NightingaleError, id::OperationContext},
};
use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};

fn storage_error(_: rusqlite::Error) -> NightingaleError {
    NightingaleError::internal("datos locales de cuenta no disponibles")
}

pub struct SqliteAccountRepository<'a> {
    pub connection: &'a mut Connection,
    pub clock: &'a dyn Clock,
}

impl AccountViewQuery for SqliteAccountRepository<'_> {
    fn account_view(&self, scope: &OperationContext) -> Result<Option<AccountView>> {
        let row = self.connection.query_row(
            "SELECT display_name,time_zone,location_name,location_address,location_city,location_state,location_country,no_show_grace_minutes,status,version,updated_at FROM account_view WHERE account_id=?1 AND user_id=?2 AND device_id=?3",
            params![scope.account_id.as_str(),scope.user_id.as_str(),scope.device_id.as_str()],
            |r| Ok(
                (
                    r.get(0)?,
                    r.get(1)?,
                    r.get(2)?,
                    r.get(3)?,
                    r.get(4)?,
                    r.get(5)?,
                    r.get(6)?,
                    r.get::<_,i64>(7)?,
                    r.get::<_,String>(8)?,
                    r.get(9)?,
                    r.get(10)?
                )
            ),
        ).optional().map_err(storage_error)?;

        row.map(
            |(
                display_name,
                time_zone,
                location_name,
                location_address,
                location_city,
                location_state,
                location_country,
                minutes,
                status,
                version,
                updated_at,
            )| {
                Ok(AccountView {
                    account_id: scope.account_id.clone(),
                    display_name,
                    time_zone,
                    location_name,
                    location_address,
                    location_city,
                    location_state,
                    location_country,
                    policy: AccountPolicy::new(minutes)?,
                    status: AccountStatus::try_from(status.as_str())?,
                    version,
                    updated_at,
                })
            },
        )
        .transpose()
    }

    fn account_policy(&self, scope: &OperationContext) -> Result<Option<AccountPolicy>> {
        Ok(self.account_view(scope)?.map(|view| view.policy))
    }
}

fn read_preferences(connection: &Connection, scope: &OperationContext) -> Result<Preferences> {
    let mut query = connection.prepare("SELECT key,value FROM account_preferences WHERE account_id=?1 AND user_id=?2 AND device_id=?3 ORDER BY key").map_err(storage_error)?;
    let rows = query
        .query_map(
            params![
                scope.account_id.as_str(),
                scope.user_id.as_str(),
                scope.device_id.as_str()
            ],
            |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
        )
        .map_err(storage_error)?;
    let mut preferences = Preferences::default();

    for row in rows {
        let (key, value) = row.map_err(storage_error)?;
        let value: serde_json::Value = serde_json::from_str(&value)
            .map_err(|_| AccountError::new("INTERNAL_ERROR", "preferencia local inválida"))?;
        let preference = match key.as_str() {
            "theme" => {
                Preference::Theme(Theme::try_from(value.as_str().ok_or_else(|| {
                    AccountError::new("INTERNAL_ERROR", "tema local inválido")
                })?)?)
            }
            "servicesShowInactive" => {
                Preference::ServicesShowInactive(value.as_bool().ok_or_else(|| {
                    AccountError::new("INTERNAL_ERROR", "preferencia local inválida")
                })?)
            }
            _ => {
                return Err(AccountError::new(
                    "INTERNAL_ERROR",
                    "preferencia local inválida",
                ));
            }
        };
        preferences.apply(preference);
    }

    Ok(preferences)
}

impl AccountReadRepository for SqliteAccountRepository<'_> {
    fn preferences(&self, scope: &OperationContext) -> Result<Preferences> {
        read_preferences(self.connection, scope)
    }

    fn set_preference(
        &mut self,
        scope: &OperationContext,
        preference: Preference,
    ) -> Result<Preferences> {
        let (key, value) = match preference {
            Preference::Theme(theme) => ("theme", serde_json::json!(theme.as_str())),
            Preference::ServicesShowInactive(value) => {
                ("servicesShowInactive", serde_json::json!(value))
            }
        };
        let now = self
            .clock
            .now()
            .duration_since(std::time::UNIX_EPOCH)
            .ok()
            .and_then(|duration| i64::try_from(duration.as_millis()).ok())
            .ok_or_else(|| AccountError::new("INTERNAL_ERROR", "reloj local inválido"))?;

        // Drop rolls back on any read/write failure. No Outbox entry is involved.
        // https://docs.rs/rusqlite/0.40.2/rusqlite/struct.Transaction.html
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        tx.execute("INSERT INTO account_preferences (account_id,user_id,device_id,key,value,updated_at) VALUES (?1,?2,?3,?4,?5,?6) ON CONFLICT(account_id,user_id,device_id,key) DO UPDATE SET value=excluded.value,updated_at=excluded.updated_at",
            params![
                scope.account_id.as_str(),
                scope.user_id.as_str(),
                scope.device_id.as_str(),
                key,
                value.to_string(),
                now
            ]).map_err(storage_error)?;
        let preferences = read_preferences(&tx, scope)?;

        tx.commit().map_err(storage_error)?;

        Ok(preferences)
    }
}

pub struct AccountProjectionRepository;

impl AccountProjectionRepository {
    fn upsert(
        tx: &Transaction,
        scope: &OperationContext,
        state: &AccountResponse,
    ) -> crate::shared_kernel::Result<()> {
        let id: String = state.id.clone().into();

        if id != scope.account_id.as_str() {
            return Err(NightingaleError::validation(
                "cuenta remota fuera de ámbito",
            ));
        }

        AccountPolicy::new(state.no_show_grace_minutes)?;

        let status = match state.status {
            AccountResponseStatus::Active => "active",
            AccountResponseStatus::Inactive => "inactive",
        };

        // The WHERE guard keeps a replay from replacing a newer accepted view.
        // https://www.sqlite.org/lang_upsert.html
        tx.execute("INSERT INTO account_view (account_id,user_id,device_id,display_name,time_zone,location_name,location_address,location_city,location_state,location_country,no_show_grace_minutes,status,version,updated_at)
            VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)
            ON CONFLICT(account_id,user_id,device_id) DO UPDATE SET display_name=excluded.display_name,time_zone=excluded.time_zone,location_name=excluded.location_name,location_address=excluded.location_address,location_city=excluded.location_city,location_state=excluded.location_state,location_country=excluded.location_country,no_show_grace_minutes=excluded.no_show_grace_minutes,status=excluded.status,version=excluded.version,updated_at=excluded.updated_at WHERE excluded.version>=account_view.version",
            params![scope.account_id.as_str(),scope.user_id.as_str(),scope.device_id.as_str(),state.display_name,state.time_zone,state.location_name,state.location_address,state.location_city,state.location_state,state.location_country,state.no_show_grace_minutes,status,state.version,state.updated_at]).map_err(storage_error)?;

        Ok(())
    }
}

pub struct AccountProjector;

impl RemoteProjector for AccountProjector {
    fn reset(
        &self,
        tx: &Transaction,
        scope: &OperationContext,
    ) -> crate::shared_kernel::Result<()> {
        tx.execute(
            "DELETE FROM account_view WHERE account_id=?1 AND user_id=?2 AND device_id=?3",
            params![
                scope.account_id.as_str(),
                scope.user_id.as_str(),
                scope.device_id.as_str()
            ],
        )
        .map_err(storage_error)?;

        Ok(())
    }

    fn project(
        &self,
        tx: &Transaction,
        scope: &OperationContext,
        change: &RemoteChange,
    ) -> crate::shared_kernel::Result<()> {
        match change {
            RemoteChange::Upsert { state, .. } => {
                if let RemoteState::Account(state) = state.as_ref() {
                    AccountProjectionRepository::upsert(tx, scope, state)?;
                }
            }
            RemoteChange::Remove {
                entity_type: EntityType::Account,
                key,
            } => {
                if key.as_str() != Some(scope.account_id.as_str()) {
                    return Err(NightingaleError::validation(
                        "cuenta remota fuera de ámbito",
                    ));
                }
                self.reset(tx, scope)?;
            }
            _ => {}
        }

        Ok(())
    }
}
