use crate::shared_kernel::Result;
use crate::shared_kernel::clock::Clock;
use crate::shared_kernel::id::IdGenerator;
use rusqlite::{Connection, Transaction};
use serde_json::Value;
use uuid::Uuid;

use super::domain::NewOperation;

/// Operation ready to persist, with its generated identity.
#[derive(Debug, Clone)]
pub struct PendingOperation {
    pub operation_id: Uuid,
    pub action: String,
    pub entity_type: String,
    pub entity_key: Value,
    pub expected_version: i64,
    pub payload: Value,
    pub occurred_at_local: Option<String>,
    pub created_at_millis: i64,
}

/// Outbound port implemented by storage adapters.
pub trait OperationStore {
    fn append(&self, transaction: &Transaction, operation: &PendingOperation) -> Result<()>;

    fn list_pending(&self, connection: &Connection, limit: i64) -> Result<Vec<PendingOperation>>;
}

/// Records a user operation in the outbox. Must run inside the same SQLite
/// transaction as the business change it announces (Master Plan §9.11); the
/// caller passes the open transaction, so a failed business change cannot
/// leave a dangling operation and vice versa.
pub fn record_operation(
    transaction: &Transaction,
    store: &impl OperationStore,
    id_generator: &impl IdGenerator,
    clock: &impl Clock,
    input: NewOperation,
) -> Result<Uuid> {
    input.validate()?;

    let created_at_millis = clock
        .now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_| {
            crate::shared_kernel::NightingaleError::internal("reloj del sistema inválido")
        })?
        .as_millis() as i64;
    let operation = PendingOperation {
        operation_id: id_generator.new_id(),
        action: input.action,
        entity_type: input.entity_type,
        entity_key: input.entity_key,
        expected_version: input.expected_version,
        payload: input.payload,
        occurred_at_local: input.occurred_at_local,
        created_at_millis,
    };

    store.append(transaction, &operation)?;

    Ok(operation.operation_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::sync::adapters::outbound::SqliteOperationStore;
    use crate::platform::sqlite::open;
    use crate::shared_kernel::clock::SystemClock;
    use crate::shared_kernel::id::UuidV7Generator;
    use serde_json::json;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT: AtomicU64 = AtomicU64::new(0);

    struct Fixture(std::path::PathBuf);

    impl Fixture {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "nightingale-sync-{}-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }

        fn database(&self) -> std::path::PathBuf {
            self.0.join("test.sqlite")
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }

    fn operation() -> NewOperation {
        NewOperation {
            action: "RegisterPatient".to_owned(),
            entity_type: "patient_profile".to_owned(),
            entity_key: json!("019f0000-0000-7000-8000-000000000101"),
            expected_version: 0,
            payload: json!({ "fullName": "Paciente de prueba" }),
            occurred_at_local: None,
        }
    }

    #[test]
    fn records_operation_inside_transaction_and_lists_it() {
        let fixture = Fixture::new();
        let mut connection = open(&fixture.database()).unwrap();
        let store = SqliteOperationStore;
        let ids_generator = UuidV7Generator;
        let clock = SystemClock;

        let transaction = connection
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
            .unwrap();
        let id =
            record_operation(&transaction, &store, &ids_generator, &clock, operation()).unwrap();
        transaction.commit().unwrap();

        let pending = store.list_pending(&connection, 10).unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].operation_id, id);
        assert_eq!(pending[0].action, "RegisterPatient");
        assert_eq!(
            pending[0].entity_key,
            json!("019f0000-0000-7000-8000-000000000101")
        );
        assert_eq!(
            pending[0].payload,
            json!({ "fullName": "Paciente de prueba" })
        );
        assert!(pending[0].created_at_millis > 0);
    }

    #[test]
    fn failed_business_change_rolls_back_operation() {
        let fixture = Fixture::new();
        let mut connection = open(&fixture.database()).unwrap();
        let store = SqliteOperationStore;
        let ids_generator = UuidV7Generator;
        let clock = SystemClock;

        {
            let transaction = connection
                .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
                .unwrap();
            let result =
                record_operation(&transaction, &store, &ids_generator, &clock, operation());
            assert!(result.is_ok());
            // The business change fails: dropping the transaction rolls the
            // outbox record back with it.
        }

        let pending = store.list_pending(&connection, 10).unwrap();
        assert!(pending.is_empty());
    }
}
