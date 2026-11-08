use super::adapters::outbound::SqliteSchedulingRepository;
use crate::components::appointments::application::{OccupancyInterval, ScheduleOccupancyQuery};
use crate::components::identity::{
    domain::{AccessRole, AccessStatus, AccountAccess},
    repository::{AccessStore, SqliteAccessStore},
};
use crate::platform::sqlite;
use crate::shared_kernel::{Clock, IdGenerator, Result, id::OperationContext};
use jiff::Timestamp;
use rusqlite::Connection;
use serde_json::json;
use std::{
    cell::Cell,
    path::Path,
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

/// No known occupancy: most local writes have nothing to prevalidate against.
pub struct NoOccupancy;

impl ScheduleOccupancyQuery for NoOccupancy {
    fn list_occupancy(
        &self,
        _professional_id: &str,
        _from: Timestamp,
        _until: Timestamp,
    ) -> Result<Vec<OccupancyInterval>> {
        Ok(vec![])
    }
}

/// In-memory occupancy keyed by professional, for SLOT_UNAVAILABLE checks.
pub struct FakeOccupancy(pub Vec<(String, OccupancyInterval)>);

impl ScheduleOccupancyQuery for FakeOccupancy {
    fn list_occupancy(
        &self,
        professional_id: &str,
        _from: Timestamp,
        _until: Timestamp,
    ) -> Result<Vec<OccupancyInterval>> {
        Ok(self
            .0
            .iter()
            .filter(|(professional, _)| professional == professional_id)
            .map(|(_, interval)| interval.clone())
            .collect())
    }
}

pub const PROFESSIONAL: &str = "019a1234-0000-7000-8000-000000000008";

pub fn access() -> AccountAccess {
    access_with_role(AccessRole::Assistant)
}

pub fn owner_access() -> AccountAccess {
    access_with_role(AccessRole::Owner)
}

fn access_with_role(role: AccessRole) -> AccountAccess {
    let context: OperationContext = serde_json::from_value(json!({
        "accountId":"019a1234-0000-7000-8000-000000000001",
        "userId":"019a1234-0000-7000-8000-000000000002",
        "deviceId":"019a1234-0000-7000-8000-000000000003",
        "permissionEpoch":1
    }))
    .unwrap();
    AccountAccess {
        context,
        role,
        status: AccessStatus::Active,
        checked_at_millis: 1,
        allowed_actions: [
            "CreateScheduleRule",
            "UpdateScheduleRule",
            "SetScheduleRuleStatus",
            "CreateScheduleException",
            "UpdateScheduleException",
            "SetScheduleExceptionStatus",
            "CreateScheduleBlock",
            "UpdateScheduleBlock",
            "SetScheduleBlockStatus",
        ]
        .map(str::to_owned)
        .to_vec(),
    }
}

pub fn database() -> Connection {
    let mut c = sqlite::open(Path::new(":memory:")).unwrap();
    seed_account(&mut c);
    c
}

/// Base con la vista de profesional cuyo `professional_user_id` es el usuario
/// de la sesión (permite bloqueos globales del propietario).
pub fn database_with_professional() -> Connection {
    let mut c = database();
    let tx = c.transaction().unwrap();
    tx.execute(
        "INSERT INTO professional_view (account_id,user_id,device_id,professional_id,professional_user_id,public_name,status,version) VALUES ('019a1234-0000-7000-8000-000000000001','019a1234-0000-7000-8000-000000000002','019a1234-0000-7000-8000-000000000003',?1,?2,'Profesional ficticio','active',1)",
        rusqlite::params![PROFESSIONAL, "019a1234-0000-7000-8000-000000000002"],
    )
    .unwrap();
    tx.commit().unwrap();
    c
}

pub fn seed_account(c: &mut Connection) {
    let tx = c.transaction().unwrap();
    SqliteAccessStore.save_access(&tx, &access()).unwrap();
    SqliteAccessStore
        .save_context(
            &tx,
            &crate::components::identity::domain::IdentityContext {
                issuer: "https://synthetic.example".into(),
                subject: "synthetic".into(),
                user_id: access().context.user_id,
                selected_account_id: Some(access().context.account_id),
            },
        )
        .unwrap();
    tx.execute(
        "INSERT INTO account_view (account_id,user_id,device_id,display_name,time_zone,location_name,location_address,location_city,location_state,location_country,no_show_grace_minutes,status,version) VALUES ('019a1234-0000-7000-8000-000000000001','019a1234-0000-7000-8000-000000000002','019a1234-0000-7000-8000-000000000003','Cuenta ficticia','America/Caracas','Consultorio','Calle 0','Caracas','DC','VE',15,'active',1)",
        [],
    )
    .unwrap();
    tx.commit().unwrap();
}

pub fn repository(c: &mut Connection) -> SqliteSchedulingRepository<'_> {
    SqliteSchedulingRepository { connection: c }
}
