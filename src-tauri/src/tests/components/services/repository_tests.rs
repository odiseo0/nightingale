use crate::components::services::adapters::outbound::*;
use crate::components::{
    identity::domain::AccountAccess,
    services::{
        application::{CatalogMutation, CatalogQuery, CatalogRepository, CatalogUseCases},
        domain::{DurationMinutes, OfferKey, ServiceOfferPatch, ServicePatch, Status},
    },
    sync::adapters::outbound::SqliteOperationStore,
};
use crate::shared_kernel::IdGenerator;
use crate::shared_kernel::{SystemClock, UuidV7Generator};
use crate::tests::components::services::test_support::*;
use rusqlite::Connection;
use serde_json::json;

fn cases<'a>(
    repository: &'a mut SqliteCatalogRepository<'a>,
    access: &'a AccountAccess,
) -> CatalogUseCases<'a, SqliteCatalogRepository<'a>, UuidV7Generator, SystemClock> {
    CatalogUseCases {
        repository,
        access,
        ids: &UuidV7Generator,
        clock: &SystemClock,
    }
}
fn count(c: &Connection, table: &str) -> i64 {
    c.query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))
        .unwrap()
}

#[test]
fn offline_service_and_offer_survive_reopen_with_causal_dependency() {
    let path = std::env::temp_dir().join(format!(
        "nightingale-services-{}.sqlite",
        uuid::Uuid::now_v7()
    ));
    let access = access();
    let mut c = crate::platform::sqlite::open(&path).unwrap();
    seed(&mut c, &access);
    let service = {
        let mut repo = SqliteCatalogRepository { connection: &mut c };
        let mut cases = cases(&mut repo, &access);
        cases
            .create_service(
                PROFESSIONAL.to_owned().try_into().unwrap(),
                "Offline".into(),
                DurationMinutes::new(30).unwrap(),
            )
            .unwrap()
    };
    let service_id = service.entity_key.as_str().unwrap().to_owned();
    let offer = {
        let mut repo = SqliteCatalogRepository { connection: &mut c };
        let mut cases = cases(&mut repo, &access);
        cases
            .offer_service(
                OfferKey::new(PROFESSIONAL.into(), service_id.clone()).unwrap(),
                None,
            )
            .unwrap()
    };
    let stored = SqliteOperationStore
        .operation_result(
            &c,
            &access.context,
            &offer.operation_id.clone().try_into().unwrap(),
        )
        .unwrap()
        .unwrap();
    assert_eq!(
        stored.operation.depends_on[0].as_str(),
        service.operation_id
    );
    assert_eq!(count(&c, "service_catalog_view"), 0);
    drop(c);
    let mut c = crate::platform::sqlite::open(&path).unwrap();
    {
        let mut repo = SqliteCatalogRepository { connection: &mut c };
        assert!(repo.list_services(&access.context).unwrap().is_empty());
        let view = repo.catalog(&access).unwrap();
        assert!(view.services[0].is_provisional);
        assert!(view.services[0].accepted.is_none());
        assert_eq!(
            view.services[0].provisional.as_ref().unwrap().name,
            "Offline"
        );
        assert_eq!(view.offers[0].local_changes[0].sync_status, "pending");
        assert_eq!(
            view.offers[0]
                .provisional
                .as_ref()
                .unwrap()
                .duration_minutes_override,
            None
        );
    }
    drop(c);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn outbox_and_local_intent_roll_back_together_on_either_write_failure() {
    for table in ["service_local", "sync_outbox"] {
        let mut c = database();
        c.execute_batch(&format!("CREATE TRIGGER fail_write BEFORE INSERT ON {table} BEGIN SELECT RAISE(ABORT,'synthetic'); END;")).unwrap();
        let access = access();
        {
            let mut repo = SqliteCatalogRepository { connection: &mut c };
            assert!(
                cases(&mut repo, &access)
                    .create_service(
                        PROFESSIONAL.to_owned().try_into().unwrap(),
                        "Rollback".into(),
                        DurationMinutes::new(30).unwrap()
                    )
                    .is_err()
            );
        }
        assert_eq!(count(&c, "sync_outbox"), 0);
        assert_eq!(count(&c, "service_local"), 0);
    }
}

#[test]
fn accepted_receipt_keeps_draft_until_projection_then_retires_only_local_row() {
    let mut c = database();
    let access = access();
    let created = {
        let mut repo = SqliteCatalogRepository { connection: &mut c };
        cases(&mut repo, &access)
            .create_service(
                PROFESSIONAL.to_owned().try_into().unwrap(),
                "Accepted".into(),
                DurationMinutes::new(30).unwrap(),
            )
            .unwrap()
    };
    receipt(&c, &created.operation_id, "accepted", Some(1), None);
    {
        let mut repo = SqliteCatalogRepository { connection: &mut c };
        assert_eq!(
            repo.catalog(&access).unwrap().services[0].local_changes[0].sync_status,
            "accepted"
        );
    }
    assert_eq!(count(&c, "service_local"), 1);
    seed_service(&c, &access.context, created.entity_key.as_str().unwrap(), 1);
    {
        let mut repo = SqliteCatalogRepository { connection: &mut c };
        let view = repo.catalog(&access).unwrap();
        assert!(!view.services[0].is_provisional);
        assert_eq!(view.services[0].accepted.as_ref().unwrap().version, 1);
    }
    assert_eq!(count(&c, "service_local"), 0);
    assert_eq!(count(&c, "sync_outbox"), 1);
    c.execute("DELETE FROM service_catalog_view", []).unwrap();
    let mut repo = SqliteCatalogRepository { connection: &mut c };
    assert!(repo.catalog(&access).unwrap().services.is_empty());
}

struct Fixed(uuid::Uuid);
impl IdGenerator for Fixed {
    fn new_id(&self) -> uuid::Uuid {
        self.0
    }
}

#[test]
fn conflict_keeps_intent_and_correction_uses_a_new_immutable_operation() {
    let mut c = database();
    let access = access();
    seed_service(&c, &access.context, SERVICE, 1);
    let first = {
        let mut repo = SqliteCatalogRepository { connection: &mut c };
        cases(&mut repo, &access)
            .update_service(
                SERVICE.to_owned().try_into().unwrap(),
                1,
                ServicePatch::new(Some("First".into()), None).unwrap(),
            )
            .unwrap()
    };
    let id = first.operation_id.clone().try_into().unwrap();
    let before = SqliteOperationStore
        .operation_result(&c, &access.context, &id)
        .unwrap()
        .unwrap()
        .operation;
    receipt(
        &c,
        &first.operation_id,
        "conflict",
        Some(2),
        Some("VERSION_CONFLICT"),
    );
    {
        let mut repo = SqliteCatalogRepository { connection: &mut c };
        let view = repo.catalog(&access).unwrap();
        assert_eq!(
            view.services[0].local_changes[0].code.as_deref(),
            Some("VERSION_CONFLICT")
        );
        let fixed = Fixed(uuid::Uuid::parse_str(&first.operation_id).unwrap());
        let mut cases = CatalogUseCases {
            repository: &mut repo,
            access: &access,
            ids: &fixed,
            clock: &SystemClock,
        };
        let error = cases
            .update_service(
                SERVICE.to_owned().try_into().unwrap(),
                1,
                ServicePatch::new(Some("Changed fingerprint".into()), None).unwrap(),
            )
            .unwrap_err();
        assert_eq!(error.code, "IDEMPOTENCY_CONFLICT");
    }
    assert_eq!(count(&c, "service_local"), 1);
    let next = {
        let mut repo = SqliteCatalogRepository { connection: &mut c };
        cases(&mut repo, &access)
            .update_service(
                SERVICE.to_owned().try_into().unwrap(),
                1,
                ServicePatch::new(Some("Correction".into()), None).unwrap(),
            )
            .unwrap()
    };
    assert_ne!(first.operation_id, next.operation_id);
    let after = SqliteOperationStore
        .operation_result(&c, &access.context, &id)
        .unwrap()
        .unwrap()
        .operation;
    assert_eq!(before.payload, after.payload);
    assert_eq!(before.semantic_fingerprint, after.semantic_fingerprint);
    let mut repo = SqliteCatalogRepository { connection: &mut c };
    let view = repo.catalog(&access).unwrap();
    assert_eq!(view.services[0].local_changes.len(), 2);
    assert_eq!(
        view.services[0].provisional.as_ref().unwrap().name,
        "Correction"
    );
}

#[test]
fn status_override_and_profile_changes_use_owned_tables_and_exact_payloads() {
    let mutations = [
        CatalogMutation::SetServiceStatus {
            service: SERVICE.to_owned().try_into().unwrap(),
            version: 1,
            status: Status::Inactive,
        },
        CatalogMutation::UpdateServiceOffer {
            key: OfferKey::new(PROFESSIONAL.into(), SERVICE.into()).unwrap(),
            version: 1,
            patch: ServiceOfferPatch::new(Some(None)).unwrap(),
        },
        CatalogMutation::SetServiceOfferStatus {
            key: OfferKey::new(PROFESSIONAL.into(), SERVICE.into()).unwrap(),
            version: 1,
            status: Status::Inactive,
        },
        CatalogMutation::UpdateProfessionalProfile {
            professional: PROFESSIONAL.to_owned().try_into().unwrap(),
            version: 1,
            public_name: "Changed".into(),
        },
    ];
    for mutation in mutations {
        let mut c = database();
        let access = access();
        seed_service(&c, &access.context, SERVICE, 1);
        seed_offer(&c, &access.context);
        let expected = mutation.action();
        let result = {
            let mut repo = SqliteCatalogRepository { connection: &mut c };
            repo.persist(&access, mutation, &UuidV7Generator, &SystemClock)
                .unwrap()
        };
        let stored = SqliteOperationStore
            .operation_result(
                &c,
                &access.context,
                &result.operation_id.clone().try_into().unwrap(),
            )
            .unwrap()
            .unwrap();
        assert_eq!(stored.operation.action, expected);
        if expected == "UpdateServiceOffer" {
            assert_eq!(
                stored.operation.payload,
                json!({"durationMinutesOverride":null})
            );
        }
        assert_eq!(
            count(&c, "service_local") + count(&c, "service_offer_local"),
            1
        );
        assert_eq!(count(&c, "sync_outbox"), 1);
        let mut repo = SqliteCatalogRepository { connection: &mut c };
        let screen = repo.catalog(&access).unwrap();
        assert!(screen.can_manage);
        if expected == "UpdateProfessionalProfile" {
            assert_eq!(
                screen.professionals[0]
                    .provisional
                    .as_ref()
                    .unwrap()
                    .public_name,
                "Changed"
            );
        }
    }
}

#[test]
fn stale_permissions_and_other_professionals_cannot_queue_changes() {
    let mut c = database();
    let access = access();
    c.execute(
        "UPDATE professional_view SET professional_user_id='019a1234-0000-7000-8000-000000000009'",
        [],
    )
    .unwrap();
    {
        let mut repo = SqliteCatalogRepository { connection: &mut c };
        assert_eq!(
            cases(&mut repo, &access)
                .create_service(
                    PROFESSIONAL.to_owned().try_into().unwrap(),
                    "Foreign".into(),
                    DurationMinutes::new(30).unwrap()
                )
                .unwrap_err()
                .code,
            "PERMISSION_DENIED"
        );
    }
    c.execute("UPDATE identity_account_access SET status='revoked'", [])
        .unwrap();
    let mut repo = SqliteCatalogRepository { connection: &mut c };
    assert_eq!(
        cases(&mut repo, &access)
            .create_service(
                PROFESSIONAL.to_owned().try_into().unwrap(),
                "Revoked".into(),
                DurationMinutes::new(30).unwrap()
            )
            .unwrap_err()
            .code,
        "PERMISSION_DENIED"
    );
    assert_eq!(count(&c, "sync_outbox"), 0);
}
