use crate::{
    components::identity::{
        domain::{AccessRole, AccessStatus, AccountAccess},
        repository::{AccessStore, SqliteAccessStore},
    },
    shared_kernel::id::OperationContext,
};
use rusqlite::{Connection, params};
use serde_json::json;

pub const PROFESSIONAL: &str = "019a1234-0000-7000-8000-000000000004";
pub const SERVICE: &str = "019a1234-0000-7000-8000-000000000005";
pub fn access() -> AccountAccess {
    let context:OperationContext=serde_json::from_value(json!({"accountId":"019a1234-0000-7000-8000-000000000001","userId":"019a1234-0000-7000-8000-000000000002","deviceId":"019a1234-0000-7000-8000-000000000003","permissionEpoch":1})).unwrap();
    AccountAccess {
        context,
        role: AccessRole::Owner,
        status: AccessStatus::Active,
        checked_at_millis: 1,
        allowed_actions: [
            "CreateService",
            "UpdateService",
            "SetServiceStatus",
            "OfferService",
            "UpdateServiceOffer",
            "SetServiceOfferStatus",
            "UpdateProfessionalProfile",
        ]
        .map(str::to_owned)
        .to_vec(),
    }
}
pub fn database() -> Connection {
    let mut c = crate::platform::sqlite::open(std::path::Path::new(":memory:")).unwrap();
    seed(&mut c, &access());
    c
}
pub fn seed(c: &mut Connection, a: &AccountAccess) {
    let s = &a.context;
    let tx = c.transaction().unwrap();
    SqliteAccessStore.save_access(&tx, a).unwrap();
    tx.execute(
        "INSERT INTO professional_view VALUES (?1,?2,?3,?4,?2,'Synthetic','active',1)",
        params![
            s.account_id.as_str(),
            s.user_id.as_str(),
            s.device_id.as_str(),
            PROFESSIONAL
        ],
    )
    .unwrap();
    tx.commit().unwrap();
}
pub fn seed_service(c: &Connection, s: &OperationContext, id: &str, version: i64) {
    c.execute(
        "INSERT INTO service_catalog_view VALUES (?1,?2,?3,?4,?5,'Synthetic',30,'active',?6)",
        params![
            s.account_id.as_str(),
            s.user_id.as_str(),
            s.device_id.as_str(),
            id,
            PROFESSIONAL,
            version
        ],
    )
    .unwrap();
}
pub fn seed_offer(c: &Connection, s: &OperationContext) {
    c.execute("INSERT INTO service_offer_view VALUES (?1,?2,?3,?4,?5,'Synthetic',30,NULL,30,'active','active',1,1)",params![s.account_id.as_str(),s.user_id.as_str(),s.device_id.as_str(),PROFESSIONAL,SERVICE]).unwrap();
}
pub fn receipt(c: &Connection, id: &str, status: &str, version: Option<i64>, code: Option<&str>) {
    let result = json!({"operationId":id,"status":status,"entityVersion":version,"state":null,"error":code.map(|code|json!({"code":code,"correlationId":"019a1234-0000-7000-8000-000000000009"}))});
    c.execute(
        "UPDATE sync_outbox SET status=?2,result=?3 WHERE operation_id=?1",
        params![id, status, result.to_string()],
    )
    .unwrap();
}
