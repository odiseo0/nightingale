//! Provisional occupancy: pending local appointments block locally, a
//! provisional cancellation frees the slot, and rejected/conflicting
//! intentions never anchor new reservations.
use super::adapters::outbound::{LocalChange, SqliteScheduleOccupancyQuery, insert_local_change};
use super::application::ScheduleOccupancyQuery;
use crate::{
    components::sync::{
        adapters::outbound::SqliteOperationStore, application::record_operation,
        domain::NewOperation,
    },
    platform::sqlite,
    shared_kernel::{Clock, IdGenerator, id::OperationContext},
};
use jiff::Timestamp;
use rusqlite::Connection;
use serde_json::json;
use std::str::FromStr;

const PROFESSIONAL: &str = "019a1234-0000-7000-8000-000000000004";
const OTHER_PROFESSIONAL: &str = "019a1234-0000-7000-8000-000000000005";
const APPOINTMENT: &str = "019a1234-0000-7000-8000-000000000010";
const OTHER_APPOINTMENT: &str = "019a1234-0000-7000-8000-000000000011";

fn scope() -> OperationContext {
    serde_json::from_value(json!({
        "accountId":"019a1234-0000-7000-8000-000000000001",
        "userId":"019a1234-0000-7000-8000-000000000002",
        "deviceId":"019a1234-0000-7000-8000-000000000003", "permissionEpoch":1
    }))
    .unwrap()
}

struct Ids(u64);

impl IdGenerator for Ids {
    fn new_id(&self) -> uuid::Uuid {
        format!("019a1234-0000-7000-8000-{:012}", self.0 + 1)
            .parse()
            .unwrap()
    }
}

struct FixedClock;

impl Clock for FixedClock {
    fn now(&self) -> std::time::SystemTime {
        std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_millis(1_793_829_600_000)
    }
}

struct Database(std::path::PathBuf);

impl Database {
    fn new() -> Self {
        Self(std::env::temp_dir().join(format!(
            "nightingale-appointments-occupancy-{}.sqlite",
            uuid::Uuid::now_v7()
        )))
    }
}

impl Drop for Database {
    fn drop(&mut self) {
        std::fs::remove_file(&self.0).unwrap();
    }
}

fn ts(value: &str) -> Timestamp {
    Timestamp::from_str(value).unwrap()
}

/// Intent + Outbox in one transaction, as every real appointment write will do.
fn persist_intent(
    connection: &mut Connection,
    seq: u64,
    appointment: &str,
    action: &str,
    starts: &str,
    ends: &str,
    status: &str,
) -> String {
    let scope = scope();
    let mut operation_id = String::new();
    sqlite::in_transaction(connection, |tx| {
        let id = record_operation(
            tx,
            &SqliteOperationStore,
            &Ids(seq),
            &FixedClock,
            NewOperation {
                context: scope.clone(),
                depends_on: vec![],
                action: action.into(),
                entity_type: "appointment".into(),
                entity_key: json!(appointment),
                expected_version: 0,
                payload: json!({}),
                occurred_at_local: None,
            },
        )
        .map_err(|_| rusqlite::Error::InvalidQuery)?;
        operation_id = id.to_string();
        insert_local_change(
            tx,
            &scope,
            &LocalChange {
                operation_id: operation_id.clone(),
                appointment_id: appointment.into(),
                action: action.into(),
                expected_version: 0,
                base_operation_id: None,
                depends_on: "[]".into(),
                payload: "{}".into(),
                draft: json!({
                    "professionalId": PROFESSIONAL,
                    "patientId": "019a1234-0000-7000-8000-000000000020",
                    "serviceId": "019a1234-0000-7000-8000-000000000021",
                    "startsAt": starts,
                    "endsAt": ends,
                    "status": status,
                })
                .to_string(),
                created_at: seq as i64,
            },
        )
    })
    .unwrap();
    operation_id
}

fn occupancy(
    connection: &Connection,
    professional: &str,
    from: &str,
    until: &str,
) -> Vec<(String, String, String)> {
    SqliteScheduleOccupancyQuery {
        connection,
        scope: &scope(),
    }
    .list_occupancy(professional, ts(from), ts(until))
    .unwrap()
    .into_iter()
    .map(|interval| {
        (
            interval.appointment_id,
            interval.starts_at.to_string(),
            interval.ends_at.to_string(),
        )
    })
    .collect()
}

#[test]
fn pending_appointment_blocks_and_provisional_cancel_frees() {
    let database = Database::new();
    let mut connection = sqlite::open(&database.0).unwrap();

    // Provisional appointment 13:00-13:30Z blocks locally.
    persist_intent(
        &mut connection,
        1,
        APPOINTMENT,
        "CreateAppointment",
        "2026-05-04T13:00:00Z",
        "2026-05-04T13:30:00Z",
        "scheduled",
    );
    assert_eq!(
        occupancy(
            &connection,
            PROFESSIONAL,
            "2026-05-04T00:00:00Z",
            "2026-05-05T00:00:00Z"
        ),
        [(
            APPOINTMENT.to_string(),
            "2026-05-04T13:00:00Z".to_string(),
            "2026-05-04T13:30:00Z".to_string()
        )]
    );

    // Provisional cancellation: the latest draft frees the slot.
    persist_intent(
        &mut connection,
        2,
        APPOINTMENT,
        "CancelAppointment",
        "2026-05-04T13:00:00Z",
        "2026-05-04T13:30:00Z",
        "cancelled",
    );
    assert!(
        occupancy(
            &connection,
            PROFESSIONAL,
            "2026-05-04T00:00:00Z",
            "2026-05-05T00:00:00Z"
        )
        .is_empty()
    );

    // Another professional does not see the provisional appointment.
    persist_intent(
        &mut connection,
        3,
        APPOINTMENT,
        "RescheduleAppointment",
        "2026-05-04T13:00:00Z",
        "2026-05-04T13:30:00Z",
        "scheduled",
    );
    assert!(
        occupancy(
            &connection,
            OTHER_PROFESSIONAL,
            "2026-05-04T00:00:00Z",
            "2026-05-05T00:00:00Z"
        )
        .is_empty()
    );
}

#[test]
fn conflicted_intention_is_excluded_from_occupancy() {
    let database = Database::new();
    let mut connection = sqlite::open(&database.0).unwrap();

    let operation_id = persist_intent(
        &mut connection,
        1,
        APPOINTMENT,
        "CreateAppointment",
        "2026-05-04T13:00:00Z",
        "2026-05-04T13:30:00Z",
        "scheduled",
    );
    connection
        .execute(
            "UPDATE sync_outbox SET status='conflict', last_error='\"SLOT_UNAVAILABLE\"' WHERE operation_id=?1",
            rusqlite::params![operation_id],
        )
        .unwrap();

    assert!(
        occupancy(
            &connection,
            PROFESSIONAL,
            "2026-05-04T00:00:00Z",
            "2026-05-05T00:00:00Z"
        )
        .is_empty()
    );
}

#[test]
fn accepted_and_provisional_appointments_combine_without_duplicates() {
    let database = Database::new();
    let mut connection = sqlite::open(&database.0).unwrap();
    let scope = scope();

    // Accepted occupancy contiguous with the provisional one; both occupy.
    connection
        .execute(
            "INSERT INTO appointment_occupancy_view (account_id,user_id,device_id,appointment_id,professional_id,starts_at,ends_at,status,version) VALUES (?1,?2,?3,?4,?5,'2026-05-04T12:00:00Z','2026-05-04T13:00:00Z','confirmed',1)",
            rusqlite::params![
                scope.account_id.as_str(),
                scope.user_id.as_str(),
                scope.device_id.as_str(),
                OTHER_APPOINTMENT,
                PROFESSIONAL
            ],
        )
        .unwrap();
    persist_intent(
        &mut connection,
        1,
        APPOINTMENT,
        "CreateAppointment",
        "2026-05-04T13:00:00Z",
        "2026-05-04T14:00:00Z",
        "scheduled",
    );

    let rows = occupancy(
        &connection,
        PROFESSIONAL,
        "2026-05-04T00:00:00Z",
        "2026-05-05T00:00:00Z",
    );
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].0, OTHER_APPOINTMENT);
    assert_eq!(rows[1].0, APPOINTMENT);

    // Reopening keeps the provisional occupancy.
    drop(connection);
    let connection = sqlite::open(&database.0).unwrap();
    assert_eq!(
        occupancy(
            &connection,
            PROFESSIONAL,
            "2026-05-04T00:00:00Z",
            "2026-05-05T00:00:00Z"
        )
        .len(),
        2
    );
}
