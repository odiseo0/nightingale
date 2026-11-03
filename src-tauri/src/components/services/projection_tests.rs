use super::*;
use crate::{
    components::{
        services::{adapters::outbound::SqliteCatalogRepository, application::*, test_support::*},
        sync::remote::{ChangeGroup, ChangeKind, SyncChange, apply_groups},
    },
    shared_kernel::{SystemClock, UuidV7Generator},
};
use serde_json::{Value, json};

fn service(version: i64) -> SyncChange {
    SyncChange {
        entity_type: EntityType::Service,
        entity_key: json!(SERVICE),
        entity_version: version,
        change_kind: ChangeKind::Upsert,
        state: json!({"id":SERVICE,"professionalId":PROFESSIONAL,"name":format!("Service {version}"),"defaultDurationMinutes":30,"status":"active","version":version,"createdAt":null,"updatedAt":null}),
    }
}
fn offer(s: &OperationContext, version: i64, service_version: i64, duration: i64) -> SyncChange {
    SyncChange {
        entity_type: EntityType::ServiceOffer,
        entity_key: json!({"professionalId":PROFESSIONAL,"serviceId":SERVICE}),
        entity_version: version,
        change_kind: ChangeKind::Upsert,
        state: json!({"accountId":s.account_id,"professionalId":PROFESSIONAL,"serviceId":SERVICE,"name":format!("Offer {service_version}"),"defaultDurationMinutes":duration,"durationMinutesOverride":null,"effectiveDurationMinutes":duration,"serviceStatus":"active","status":"active","version":version,"serviceVersion":service_version,"createdAt":null,"updatedAt":null}),
    }
}
fn group(changes: Vec<SyncChange>) -> ChangeGroup {
    ChangeGroup {
        group_id: uuid::Uuid::now_v7().to_string().try_into().unwrap(),
        operation_id: uuid::Uuid::now_v7().to_string().try_into().unwrap(),
        accepted_at: "2026-11-03T12:00:00Z".into(),
        changes,
    }
}

#[test]
fn repeated_groups_and_equal_offer_versions_keep_newer_service_version() {
    let mut c = database();
    let access = access();
    let s = &access.context;
    let initial = group(vec![service(2), offer(s, 2, 2, 30)]);
    apply_groups(
        &mut c,
        s,
        std::slice::from_ref(&initial),
        &ServicesProjector,
    )
    .unwrap();
    apply_groups(&mut c, s, &[initial], &ServicesProjector).unwrap();
    apply_groups(
        &mut c,
        s,
        &[group(vec![offer(s, 2, 3, 45)])],
        &ServicesProjector,
    )
    .unwrap();
    apply_groups(
        &mut c,
        s,
        &[group(vec![
            service(1),
            offer(s, 1, 99, 60),
            offer(s, 2, 2, 30),
        ])],
        &ServicesProjector,
    )
    .unwrap();
    let repo = SqliteCatalogRepository { connection: &mut c };
    let v = &repo.list_offers(s).unwrap()[0];
    assert_eq!(
        (v.version, v.service_version, v.effective_duration_minutes),
        (2, 3, 45)
    );
    assert_eq!(v.name, "Offer 3");
    assert_eq!(repo.list_services(s).unwrap()[0].version, 2);
}

#[test]
fn remove_and_snapshot_reset_preserve_intentions_and_other_scopes() {
    let mut c = database();
    let access = access();
    let s = &access.context;
    seed_service(&c, s, SERVICE, 1);
    seed_offer(&c, s);
    let created = {
        let mut repo = SqliteCatalogRepository { connection: &mut c };
        CatalogUseCases {
            repository: &mut repo,
            access: &access,
            ids: &UuidV7Generator,
            clock: &SystemClock,
        }
        .update_service(
            SERVICE.to_owned().try_into().unwrap(),
            1,
            super::super::super::domain::ServicePatch::new(Some("Pending".into()), None).unwrap(),
        )
        .unwrap()
    };
    let mut other = access.clone();
    other.context.account_id = "019a1234-0000-7000-8000-000000000009"
        .to_owned()
        .try_into()
        .unwrap();
    seed(&mut c, &other);
    seed_service(&c, &other.context, SERVICE, 1);
    let tx = c.transaction().unwrap();
    ServicesProjector
        .project(
            &tx,
            s,
            &RemoteChange::Remove {
                entity_type: EntityType::Service,
                key: json!(SERVICE),
            },
        )
        .unwrap();
    ServicesProjector.reset(&tx, s).unwrap();
    tx.commit().unwrap();
    {
        let mut repo = SqliteCatalogRepository { connection: &mut c };
        assert!(repo.list_services(s).unwrap().is_empty());
        assert!(repo.list_offers(s).unwrap().is_empty());
        assert!(repo.list_professionals(s).unwrap().is_empty());
        assert_eq!(repo.list_services(&other.context).unwrap().len(), 1);
        assert_eq!(
            repo.catalog(&access).unwrap().services[0].local_changes[0].operation_id,
            created.operation_id
        );
    }
    apply_groups(
        &mut c,
        s,
        &[group(vec![service(1), offer(s, 1, 1, 30)])],
        &ServicesProjector,
    )
    .unwrap();
    let mut repo = SqliteCatalogRepository { connection: &mut c };
    let view = repo.catalog(&access).unwrap();
    assert!(view.services[0].accepted.is_some());
    assert!(view.services[0].is_provisional);
}

#[test]
fn invalid_projection_rolls_back_whole_group_and_cursor_inbox_effects() {
    let mut c = database();
    let access = access();
    let s = &access.context;
    let mut invalid = offer(s, 1, 1, 30);
    invalid.state["effectiveDurationMinutes"] = json!(99);
    assert!(
        apply_groups(
            &mut c,
            s,
            &[group(vec![service(1), invalid])],
            &ServicesProjector
        )
        .is_err()
    );
    let repo = SqliteCatalogRepository { connection: &mut c };
    assert!(repo.list_services(s).unwrap().is_empty());
    assert_eq!(
        c.query_row("SELECT count(*) FROM sync_inbox", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        0
    );
    assert_eq!(
        c.query_row("SELECT count(*) FROM sync_remote_items", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        0
    );
}

#[test]
fn professional_projection_is_typed_and_unowned_types_are_ignored() {
    let mut c = database();
    let access = access();
    let s = &access.context;
    let professional = SyncChange {
        entity_type: EntityType::Professional,
        entity_key: json!(PROFESSIONAL),
        entity_version: 2,
        change_kind: ChangeKind::Upsert,
        state: json!({"id":PROFESSIONAL,"userId":s.user_id,"publicName":"Updated professional","status":"active","version":2,"createdAt":null,"updatedAt":null}),
    };
    apply_groups(&mut c, s, &[group(vec![professional])], &ServicesProjector).unwrap();
    let tx = c.transaction().unwrap();
    ServicesProjector
        .project(
            &tx,
            s,
            &RemoteChange::Remove {
                entity_type: EntityType::Account,
                key: Value::Null,
            },
        )
        .unwrap();
    tx.commit().unwrap();
    let repo = SqliteCatalogRepository { connection: &mut c };
    assert_eq!(
        repo.list_professionals(s).unwrap()[0].public_name,
        "Updated professional"
    );
}
