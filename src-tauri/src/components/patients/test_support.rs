use super::{adapters::outbound::SqlitePatientRepository, application::PatientUseCases};
use crate::{
    components::identity::{
        domain::{AccessRole, AccessStatus, AccountAccess},
        repository::{AccessStore, SqliteAccessStore},
    },
    shared_kernel::{Clock, IdGenerator, id::OperationContext},
};
use rusqlite::Connection;
use std::{
    cell::Cell,
    path::PathBuf,
    time::{Duration, SystemTime},
};

pub struct Ids(Cell<u64>);

impl Default for Ids {
    fn default() -> Self {
        Self(Cell::new(100))
    }
}

impl IdGenerator for Ids {
    fn new_id(&self) -> uuid::Uuid {
        let n = self.0.get();
        self.0.set(n + 1);
        format!("019a1234-0000-7000-8000-{n:012}").parse().unwrap()
    }
}

pub struct FixedClock;

impl Clock for FixedClock {
    fn now(&self) -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_millis(1793829600000)
    }
}

pub fn access() -> AccountAccess {
    let context: OperationContext = serde_json::from_value(serde_json::json!({"accountId":"019a1234-0000-7000-8000-000000000001","userId":"019a1234-0000-7000-8000-000000000002","deviceId":"019a1234-0000-7000-8000-000000000003","permissionEpoch":1})).unwrap();
    AccountAccess {
        context,
        role: AccessRole::Assistant,
        status: AccessStatus::Active,
        checked_at_millis: 1,
        allowed_actions: [
            "RegisterPatient",
            "UpdatePatientProfile",
            "SetPatientProfileStatus",
            "AddPatientIdentifier",
            "SetPatientCompletion",
            "CreateContactPerson",
            "UpdateContactPerson",
            "SetContactPersonStatus",
            "CreatePatientContactRelationship",
            "UpdatePatientContactRelationship",
            "SetPatientContactRelationshipStatus",
        ]
        .map(str::to_owned)
        .to_vec(),
    }
}

pub fn seed_access(c: &mut Connection, a: &AccountAccess) {
    let tx = c.transaction().unwrap();
    SqliteAccessStore.save_access(&tx, a).unwrap();
    tx.commit().unwrap();
}

pub fn database() -> Connection {
    let mut c = crate::platform::sqlite::open(std::path::Path::new(":memory:")).unwrap();
    seed_access(&mut c, &access());
    c
}

pub fn run<T>(
    c: &mut Connection,
    a: &AccountAccess,
    ids: &Ids,
    f: impl FnOnce(&mut PatientUseCases<'_, SqlitePatientRepository<'_>, Ids, FixedClock>) -> T,
) -> T {
    let mut repository = SqlitePatientRepository { connection: c };
    f(&mut PatientUseCases {
        repository: &mut repository,
        access: a,
        ids,
        clock: &FixedClock,
    })
}

pub struct Database(pub PathBuf);

impl Database {
    pub fn new() -> Self {
        Self(std::env::temp_dir().join(format!(
            "nightingale-patients-cases-{}.sqlite",
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
