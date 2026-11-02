use super::*;
use crate::{
    components::sync::remote::{ChangeKind, SyncChange},
    shared_kernel::SystemClock,
};
use serde_json::json;

fn scope(account: u8) -> OperationContext {
    serde_json::from_value(json!({
        "accountId": format!("019a1234-0000-7000-8000-00000000000{account}"),
        "userId": "019a1234-0000-7000-8000-000000000008",
        "deviceId": "019a1234-0000-7000-8000-000000000009", "permissionEpoch": 1
    }))
    .unwrap()
}

fn database() -> Connection {
    crate::platform::sqlite::open(std::path::Path::new(":memory:")).unwrap()
}

fn change(scope: &OperationContext, version: i64) -> RemoteChange {
    SyncChange {
        entity_type: EntityType::Account, entity_key: json!(scope.account_id),
        entity_version: version, change_kind: ChangeKind::Upsert,
        state: json!({"id":scope.account_id,"kind":"independent","ownerUserId":scope.user_id,
            "displayName":format!("Synthetic {version}"),"timeZone":"America/Caracas","locationName":"Synthetic",
            "locationAddress":"Synthetic","locationCity":"Synthetic","locationState":"Synthetic","locationCountry":"VE",
            "noShowGraceMinutes":15,"status":"active","version":version,"createdAt":null,"updatedAt":null}),
    }.translate(scope).unwrap()
}

#[test]
fn projection_is_scoped_replay_safe_and_remove_keeps_preferences() {
    let mut c = database();
    let a = scope(1);
    let b = scope(2);
    let tx = c.transaction().unwrap();
    for s in [&a, &b] {
        AccountProjector.project(&tx, s, &change(s, 2)).unwrap();
        AccountProjector.project(&tx, s, &change(s, 2)).unwrap();
        AccountProjector.project(&tx, s, &change(s, 1)).unwrap();
    }
    tx.commit().unwrap();
    {
        let mut repository = SqliteAccountRepository {
            connection: &mut c,
            clock: &SystemClock,
        };
        assert_eq!(repository.account_view(&a).unwrap().unwrap().version, 2);
        assert_eq!(
            repository
                .account_policy(&b)
                .unwrap()
                .unwrap()
                .no_show_grace_minutes(),
            15
        );
        repository
            .set_preference(&a, Preference::Theme(Theme::Dark))
            .unwrap();
        assert_eq!(repository.preferences(&b).unwrap(), Preferences::default());
    }
    let tx = c.transaction().unwrap();
    AccountProjector
        .project(
            &tx,
            &a,
            &RemoteChange::Remove {
                entity_type: EntityType::Account,
                key: json!(a.account_id),
            },
        )
        .unwrap();
    tx.commit().unwrap();
    let repository = SqliteAccountRepository {
        connection: &mut c,
        clock: &SystemClock,
    };
    assert!(repository.account_view(&a).unwrap().is_none());
    assert!(repository.account_view(&b).unwrap().is_some());
    assert_eq!(repository.preferences(&a).unwrap().theme, Theme::Dark);
    assert_eq!(
        c.query_row("SELECT count(*) FROM sync_outbox", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        0
    );
}

#[test]
fn snapshot_reset_rebuilds_own_scope_and_ignores_other_types() {
    let mut c = database();
    let a = scope(1);
    let b = scope(2);
    let tx = c.transaction().unwrap();
    AccountProjector.project(&tx, &a, &change(&a, 3)).unwrap();
    AccountProjector.project(&tx, &b, &change(&b, 2)).unwrap();
    AccountProjector.reset(&tx, &a).unwrap();
    AccountProjector.project(&tx, &a, &change(&a, 1)).unwrap();
    AccountProjector
        .project(
            &tx,
            &a,
            &RemoteChange::Remove {
                entity_type: EntityType::Service,
                key: json!(a.account_id),
            },
        )
        .unwrap();
    assert!(AccountProjector.project(&tx, &a, &change(&b, 5)).is_err());
    tx.commit().unwrap();
    let repository = SqliteAccountRepository {
        connection: &mut c,
        clock: &SystemClock,
    };
    assert_eq!(repository.account_view(&a).unwrap().unwrap().version, 1);
    assert_eq!(repository.account_view(&b).unwrap().unwrap().version, 2);
}

#[test]
fn failed_preference_transaction_rolls_back_and_never_queues_work() {
    let mut c = database();
    let a = scope(1);
    c.execute_batch("CREATE TRIGGER reject_preference BEFORE INSERT ON account_preferences BEGIN SELECT RAISE(ABORT, 'synthetic'); END;").unwrap();
    let mut repository = SqliteAccountRepository {
        connection: &mut c,
        clock: &SystemClock,
    };
    assert!(
        repository
            .set_preference(&a, Preference::Theme(Theme::Dark))
            .is_err()
    );
    assert_eq!(repository.preferences(&a).unwrap(), Preferences::default());
    assert_eq!(
        c.query_row("SELECT count(*) FROM sync_outbox", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        0
    );
}
