use super::domain::{AccountAccess, IdentityContext};
use crate::shared_kernel::id::{AccountId, DeviceId, OperationContext, UserId};
use crate::shared_kernel::{NightingaleError, Result};
use rusqlite::{Connection, OptionalExtension, Transaction, params};

pub trait AccessStore {
    fn save_access(&self, tx: &Transaction, access: &AccountAccess) -> Result<()>;
    fn access(
        &self,
        connection: &Connection,
        account: &AccountId,
        user: &UserId,
        device: &DeviceId,
    ) -> Result<Option<AccountAccess>>;
    fn save_context(&self, tx: &Transaction, context: &IdentityContext) -> Result<()>;
    fn context(&self, connection: &Connection) -> Result<Option<IdentityContext>>;
    fn clear_context(&self, tx: &Transaction) -> Result<()>;
    fn deny_cached_access(
        &self,
        tx: &Transaction,
        user: &UserId,
        device: &DeviceId,
        account: Option<&AccountId>,
    ) -> Result<()>;
}

pub struct SqliteAccessStore;

impl SqliteAccessStore {
    /// Last valid installation selected by this local profile, without HTTP.
    pub fn selected_access(&self, c: &Connection) -> Result<Option<AccountAccess>> {
        let Some(context) = self.context(c)? else {
            return Ok(None);
        };
        let Some(account) = context.selected_account_id else {
            return Ok(None);
        };
        let device=c.query_row("SELECT device_id FROM identity_account_access WHERE account_id=?1 AND user_id=?2 ORDER BY checked_at DESC,rowid DESC LIMIT 1",params![account.as_str(),context.user_id.as_str()],|r|r.get::<_,String>(0)).optional().map_err(storage_error)?;

        match device {
            Some(device) => {
                self.access(c, &account, &context.user_id, &DeviceId::try_from(device)?)
            }
            None => Ok(None),
        }
    }
}

fn storage_error(error: impl std::fmt::Display) -> NightingaleError {
    NightingaleError::internal(error.to_string())
}

impl AccessStore for SqliteAccessStore {
    fn clear_context(&self, tx: &Transaction) -> Result<()> {
        tx.execute("DELETE FROM identity_context WHERE singleton=1", [])
            .map_err(storage_error)?;

        Ok(())
    }

    fn deny_cached_access(
        &self,
        tx: &Transaction,
        user: &UserId,
        device: &DeviceId,
        account: Option<&AccountId>,
    ) -> Result<()> {
        // Online denial disables cached grants; it never deletes local intentions.
        tx.execute(
            "UPDATE identity_account_access SET status='inactive',allowed_actions='[]'
            WHERE user_id=?1 AND device_id=?2 AND (?3 IS NULL OR account_id=?3)",
            params![
                user.as_str(),
                device.as_str(),
                account.map(AccountId::as_str)
            ],
        )
        .map_err(storage_error)?;

        Ok(())
    }

    fn save_access(&self, tx: &Transaction, access: &AccountAccess) -> Result<()> {
        let context = &access.context;
        let role = serde_json::to_value(&access.role).map_err(storage_error)?;
        let status = serde_json::to_value(&access.status).map_err(storage_error)?;

        // Keep the last response, including revoked/inactive, to prevent stale offline access.
        // https://www.sqlite.org/lang_upsert.html
        tx.execute("INSERT INTO identity_account_access (account_id,user_id,device_id,role,status,permission_epoch,allowed_actions,checked_at)
            VALUES (?1,?2,?3,?4,?5,?6,?7,?8)
            ON CONFLICT (account_id,user_id,device_id) DO UPDATE SET role=excluded.role,status=excluded.status,
                permission_epoch=excluded.permission_epoch,allowed_actions=excluded.allowed_actions,checked_at=excluded.checked_at",
            params![context.account_id.as_str(),context.user_id.as_str(),context.device_id.as_str(),role.as_str(),status.as_str(),context.permission_epoch,
                serde_json::to_string(&access.allowed_actions).map_err(storage_error)?,access.checked_at_millis]).map_err(storage_error)?;

        Ok(())
    }

    fn access(
        &self,
        connection: &Connection,
        account: &AccountId,
        user: &UserId,
        device: &DeviceId,
    ) -> Result<Option<AccountAccess>> {
        let record = connection.query_row("SELECT role,status,permission_epoch,allowed_actions,checked_at FROM identity_account_access
            WHERE account_id=?1 AND user_id=?2 AND device_id=?3",
            params![account.as_str(),user.as_str(),device.as_str()], |row| Ok((row.get::<_,String>(0)?,row.get::<_,String>(1)?,row.get::<_,i64>(2)?,row.get::<_,String>(3)?,row.get::<_,i64>(4)?)))
            .optional().map_err(storage_error)?;

        record
            .map(|(role, status, epoch, actions, checked_at_millis)| {
                Ok(AccountAccess {
                    context: OperationContext {
                        account_id: account.clone(),
                        user_id: user.clone(),
                        device_id: device.clone(),
                        permission_epoch: epoch,
                    },
                    role: serde_json::from_value(serde_json::Value::String(role))
                        .map_err(storage_error)?,
                    status: serde_json::from_value(serde_json::Value::String(status))
                        .map_err(storage_error)?,
                    allowed_actions: serde_json::from_str(&actions).map_err(storage_error)?,
                    checked_at_millis,
                })
            })
            .transpose()
    }

    fn save_context(&self, tx: &Transaction, context: &IdentityContext) -> Result<()> {
        tx.execute("INSERT INTO identity_context (singleton,issuer,subject,user_id,selected_account_id) VALUES (1,?1,?2,?3,?4)
            ON CONFLICT (singleton) DO UPDATE SET issuer=excluded.issuer,subject=excluded.subject,user_id=excluded.user_id,selected_account_id=excluded.selected_account_id",
            params![context.issuer,context.subject,context.user_id.as_str(),context.selected_account_id.as_ref().map(AccountId::as_str)]).map_err(storage_error)?;

        Ok(())
    }

    fn context(&self, connection: &Connection) -> Result<Option<IdentityContext>> {
        let record = connection.query_row("SELECT issuer,subject,user_id,selected_account_id FROM identity_context WHERE singleton=1", [],
            |row| Ok((row.get::<_,String>(0)?,row.get::<_,String>(1)?,row.get::<_,String>(2)?,row.get::<_,Option<String>>(3)?)))
            .optional().map_err(storage_error)?;

        record
            .map(|(issuer, subject, user, account)| {
                Ok(IdentityContext {
                    issuer,
                    subject,
                    user_id: UserId::try_from(user)?,
                    selected_account_id: account.map(AccountId::try_from).transpose()?,
                })
            })
            .transpose()
    }
}
