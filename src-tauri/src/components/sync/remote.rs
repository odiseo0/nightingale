use super::{
    adapters::{control::SqliteSyncControlStore, remote_models::RemoteState},
    application::{InboxStore, RemoteProjectionStore},
    domain::RemoteItem,
};
use crate::shared_kernel::{
    NightingaleError, Result,
    id::{OperationContext, OperationId, canonical_uuid_v7},
};
use rusqlite::{Connection, Transaction, TransactionBehavior};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityType {
    Account,
    AccountDevice,
    Membership,
    Invitation,
    Professional,
    PatientProfile,
    PatientIdentifier,
    ContactPerson,
    PatientContact,
    Service,
    ServiceOffer,
    ScheduleRule,
    ScheduleException,
    ScheduleBlock,
    Appointment,
    AppointmentHistory,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeKind {
    Upsert,
    Remove,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncChange {
    pub entity_type: EntityType,
    pub entity_key: Value,
    pub entity_version: i64,
    pub change_kind: ChangeKind,
    pub state: Value,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeGroup {
    pub group_id: OperationId,
    pub operation_id: OperationId,
    pub accepted_at: String,
    pub changes: Vec<SyncChange>,
}

/// The state type follows entityType, never a guess from the state's fields.
pub enum RemoteChange {
    Upsert {
        item: RemoteItem,
        state: Box<RemoteState>,
    },
    Remove {
        entity_type: EntityType,
        key: Value,
    },
}

/// A component writes only its accepted view, using this transaction.
/// Do not write local intentions or perform network work in this port.
pub trait RemoteProjector {
    /// Clears only this installation's accepted view for snapshot replacement.
    fn reset(&self, tx: &Transaction, scope: &OperationContext) -> Result<()>;

    fn project(
        &self,
        tx: &Transaction,
        scope: &OperationContext,
        change: &RemoteChange,
    ) -> Result<()>;
}

impl RemoteProjector for () {
    fn reset(&self, _: &Transaction, _: &OperationContext) -> Result<()> {
        Ok(())
    }

    fn project(&self, _: &Transaction, _: &OperationContext, _: &RemoteChange) -> Result<()> {
        Ok(())
    }
}

pub(crate) fn storage_error(_: rusqlite::Error) -> NightingaleError {
    NightingaleError::internal("almacén de entrada no disponible")
}

fn invalid() -> NightingaleError {
    NightingaleError::validation("proyección remota inválida")
}

impl EntityType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Account => "account",
            Self::AccountDevice => "account_device",
            Self::Membership => "membership",
            Self::Invitation => "invitation",
            Self::Professional => "professional",
            Self::PatientProfile => "patient_profile",
            Self::PatientIdentifier => "patient_identifier",
            Self::ContactPerson => "contact_person",
            Self::PatientContact => "patient_contact",
            Self::Service => "service",
            Self::ServiceOffer => "service_offer",
            Self::ScheduleRule => "schedule_rule",
            Self::ScheduleException => "schedule_exception",
            Self::ScheduleBlock => "schedule_block",
            Self::Appointment => "appointment",
            Self::AppointmentHistory => "appointment_history",
        }
    }

    fn validate_key(self, key: &Value) -> Result<()> {
        let fields: &[&str] = match self {
            Self::PatientContact => &["patientId", "contactPersonId"],
            Self::ServiceOffer => &["professionalId", "serviceId"],
            _ => return canonical_uuid_v7(key.as_str().ok_or_else(invalid)?).map(|_| ()),
        };
        let object = key.as_object().ok_or_else(invalid)?;

        if object.len() != fields.len() {
            return Err(invalid());
        }

        for field in fields {
            canonical_uuid_v7(
                object
                    .get(*field)
                    .and_then(Value::as_str)
                    .ok_or_else(invalid)?,
            )?;
        }

        Ok(())
    }
}

impl SyncChange {
    pub fn translate(&self, scope: &OperationContext) -> Result<RemoteChange> {
        self.entity_type.validate_key(&self.entity_key)?;

        if self.entity_version < 1 {
            return Err(invalid());
        }

        if let ChangeKind::Remove = self.change_kind {
            if !self.state.is_null() {
                return Err(invalid());
            }

            return Ok(RemoteChange::Remove {
                entity_type: self.entity_type,
                key: self.entity_key.clone(),
            });
        }

        let state: RemoteState = serde_json::from_value(serde_json::json!({
            "entityType": self.entity_type, "state": self.state
        }))
        .map_err(|_| invalid())?;

        if state.version() != self.entity_version
            || state.key() != self.entity_key
            || self
                .state
                .get("accountId")
                .is_some_and(|id| !id.is_null() && id.as_str() != Some(scope.account_id.as_str()))
            || (self.entity_type == EntityType::Account
                && self.entity_key.as_str() != Some(scope.account_id.as_str()))
            || matches!(&state, RemoteState::ServiceOffer(offer) if offer.service_version < 1)
        {
            return Err(invalid());
        }

        Ok(RemoteChange::Upsert {
            item: RemoteItem {
                entity_type: self.entity_type.as_str().to_owned(),
                entity_key: self.entity_key.clone(),
                entity_version: self.entity_version,
                state: self.state.clone(),
            },
            state: Box::new(state),
        })
    }
}

/// Applies a complete group list, or leaves all tables unchanged on failure.
pub fn apply_groups(
    connection: &mut Connection,
    scope: &OperationContext,
    groups: &[ChangeGroup],
    projector: &impl RemoteProjector,
) -> Result<()> {
    // Dropping the transaction on error also rolls back a component projector.
    // https://docs.rs/rusqlite/0.40.2/rusqlite/struct.Transaction.html
    let tx = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(storage_error)?;

    for group in groups {
        apply_group(&tx, scope, group, projector)?;
    }

    tx.commit().map_err(storage_error)
}

pub(crate) fn apply_group(
    tx: &Transaction,
    scope: &OperationContext,
    group: &ChangeGroup,
    projector: &impl RemoteProjector,
) -> Result<()> {
    let store = SqliteSyncControlStore;

    if !store.record_group(tx, scope, &group.group_id, &group.operation_id)? {
        return Ok(());
    }

    for change in &group.changes {
        let change = change.translate(scope)?;

        match &change {
            RemoteChange::Upsert { item, .. } => {
                // Equal offer versions can carry a newer catalog version.
                if store
                    .item(tx, scope, &item.entity_type, &item.entity_key)?
                    .is_some_and(|stored| stored.entity_version > item.entity_version)
                {
                    continue;
                }

                store.upsert(tx, scope, item)?;
            }
            RemoteChange::Remove { entity_type, key } => {
                store.remove(tx, scope, entity_type.as_str(), key)?
            }
        }

        projector.project(tx, scope, &change)?;
    }

    Ok(())
}

#[cfg(test)]
#[path = "remote_tests.rs"]
mod tests;
