//! Shared fixtures for appointment tests (synthetic data only).
use crate::components::appointments::adapters::outbound::SqliteAppointmentRepository;
use crate::components::identity::{
    domain::{AccessRole, AccessStatus, AccountAccess},
    repository::{AccessStore, SqliteAccessStore},
};
use crate::shared_kernel::id::OperationContext;
use rusqlite::Connection;
use serde_json::json;
use std::{
    cell::Cell,
    time::{Duration, SystemTime},
};

pub const PROFESSIONAL: &str = "019a1234-0000-7000-8000-000000000004";
pub const PATIENT: &str = "019a1234-0000-7000-8000-000000000010";
pub const SERVICE: &str = "019a1234-0000-7000-8000-000000000021";

pub struct Ids(pub Cell<u64>);

impl Default for Ids {
    fn default() -> Self {
        Self(Cell::new(100))
    }
}

impl crate::shared_kernel::IdGenerator for Ids {
    fn new_id(&self) -> uuid::Uuid {
        let n = self.0.get();
        self.0.set(n + 1);
        format!("019a1234-0000-7000-8000-{n:012}").parse().unwrap()
    }
}

pub struct FixedClock;

impl crate::shared_kernel::Clock for FixedClock {
    fn now(&self) -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_millis(1_793_829_600_000)
    }
}

pub fn access() -> AccountAccess {
    let context: OperationContext = serde_json::from_value(json!({
        "accountId":"019a1234-0000-7000-8000-000000000001",
        "userId":"019a1234-0000-7000-8000-000000000002",
        "deviceId":"019a1234-0000-7000-8000-000000000003", "permissionEpoch":1
    }))
    .unwrap();
    AccountAccess {
        context,
        role: AccessRole::Assistant,
        status: AccessStatus::Active,
        checked_at_millis: 1,
        allowed_actions: [
            "CreateAppointment",
            "ConfirmAppointment",
            "RescheduleAppointment",
            "CancelAppointment",
            "CompleteAppointment",
            "MarkAppointmentNoShow",
        ]
        .map(str::to_owned)
        .to_vec(),
    }
}

pub fn database() -> Connection {
    let mut connection = crate::platform::sqlite::open(std::path::Path::new(":memory:")).unwrap();
    let access = access();
    let tx = connection.transaction().unwrap();
    SqliteAccessStore.save_access(&tx, &access).unwrap();
    tx.commit().unwrap();
    connection
}

pub fn repository(connection: &mut Connection) -> SqliteAppointmentRepository<'_> {
    SqliteAppointmentRepository { connection }
}

pub fn snapshots() -> crate::components::appointments::domain::Snapshots {
    crate::components::appointments::domain::Snapshots::new(
        "Consulta general".into(),
        30,
        "Dra. Prueba".into(),
        "Consultorio Central".into(),
        "Av. Principal 1".into(),
        "VE".into(),
        "Distrito Capital".into(),
        "Caracas".into(),
        "America/Caracas".into(),
    )
    .unwrap()
}

pub fn interval(
    starts: &str,
    ends: &str,
) -> crate::components::appointments::domain::AppointmentInterval {
    crate::components::appointments::domain::AppointmentInterval::new(
        crate::components::appointments::domain::MinuteInstant::parse(starts).unwrap(),
        crate::components::appointments::domain::MinuteInstant::parse(ends).unwrap(),
    )
    .unwrap()
}

pub fn new_appointment() -> crate::components::appointments::application::NewAppointment {
    crate::components::appointments::application::NewAppointment {
        patient_id: PATIENT.into(),
        professional_id: PROFESSIONAL.into(),
        service_id: SERVICE.into(),
        responsible_contact_id: None,
        interval: interval("2026-05-04T13:00:00Z", "2026-05-04T13:30:00Z"),
        snapshots: snapshots(),
    }
}

pub struct Database(pub std::path::PathBuf);

impl Database {
    pub fn new() -> Self {
        Self(std::env::temp_dir().join(format!(
            "nightingale-appointments-cases-{}.sqlite",
            uuid::Uuid::now_v7()
        )))
    }
}

impl Drop for Database {
    fn drop(&mut self) {
        if self.0.exists() {
            std::fs::remove_file(&self.0).unwrap();
        }
    }
}
