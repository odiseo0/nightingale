use super::{
    application::{EditBase, SchedulingMutation, SchedulingRepository},
    domain::{BlockPatch, BlockValue, Instant, ReasonCode, Status},
    test_support::{
        FakeOccupancy, Ids, NoOccupancy, PROFESSIONAL, access, database_with_professional,
        owner_access, repository, FixedClock,
    },
};
use crate::components::appointments::application::OccupancyInterval;
use jiff::Timestamp;
use serde_json::{Value, json};
use std::str::FromStr;

const ACCOUNT: &str = "019a1234-0000-7000-8000-000000000001";
const OTHER_PROFESSIONAL: &str = "019a1234-0000-7000-8000-000000000009";

fn op(id: &str) -> crate::shared_kernel::id::OperationId {
    crate::shared_kernel::id::OperationId::try_from(id.to_owned()).unwrap()
}

fn account_block(starts: &str, ends: &str) -> BlockValue {
    BlockValue::new(
        PROFESSIONAL.to_owned().try_into().unwrap(),
        Some(ACCOUNT.to_owned().try_into().unwrap()),
        Instant::parse(starts).unwrap(),
        Instant::parse(ends).unwrap(),
        ReasonCode::Administrative,
    )
    .unwrap()
}

fn global_block(starts: &str, ends: &str) -> BlockValue {
    BlockValue::new(
        PROFESSIONAL.to_owned().try_into().unwrap(),
        None,
        Instant::parse(starts).unwrap(),
        Instant::parse(ends).unwrap(),
        ReasonCode::Vacation,
    )
    .unwrap()
}

/// Cita conocida 2026-05-06 09:30–10:00 en Caracas (13:30–14:00 UTC).
fn appointment() -> (String, OccupancyInterval) {
    (
        PROFESSIONAL.into(),
        OccupancyInterval::new(
            "019a1234-0000-7000-8000-000000000020".into(),
            Timestamp::from_str("2026-05-06T13:30:00Z").unwrap(),
            Timestamp::from_str("2026-05-06T14:00:00Z").unwrap(),
            "confirmed",
        )
        .unwrap(),
    )
}

fn outbox_payload_last(c: &rusqlite::Connection, action: &str) -> Value {
    serde_json::from_str(
        &c.query_row(
            "SELECT payload FROM sync_outbox WHERE action = ?1 ORDER BY operation_id DESC LIMIT 1",
            [action],
            |r| r.get::<_, String>(0),
        )
        .unwrap(),
    )
    .unwrap()
}

fn outbox_payload(c: &rusqlite::Connection, action: &str) -> Value {
    serde_json::from_str(
        &c.query_row(
            "SELECT payload FROM sync_outbox WHERE action = ?1",
            [action],
            |r| r.get::<_, String>(0),
        )
        .unwrap(),
    )
    .unwrap()
}

#[test]
fn owner_creates_account_and_global_blocks_with_contract_payloads() {
    let mut c = database_with_professional();
    let ids = Ids::default();

    repository(&mut c)
        .persist(
            &owner_access(),
            SchedulingMutation::CreateBlock {
                value: account_block("2026-04-01T13:00:00Z", "2026-04-01T17:00:00Z"),
            },
            &ids,
            &FixedClock,
            &NoOccupancy,
        )
        .unwrap();
    repository(&mut c)
        .persist(
            &owner_access(),
            SchedulingMutation::CreateBlock {
                value: global_block("2026-04-02T13:00:00Z", "2026-04-02T17:00:00Z"),
            },
            &ids,
            &FixedClock,
            &NoOccupancy,
        )
        .unwrap();

    assert_eq!(
        outbox_payload_last(&c, "CreateScheduleBlock"),
        json!({
            "professionalId": PROFESSIONAL,
            "accountId": Value::Null,
            "startsAt": "2026-04-02T13:00:00Z",
            "endsAt": "2026-04-02T17:00:00Z",
            "reasonCode": "vacation"
        })
    );

    // El asistente lista bloqueos fusionados: ámbito y motivo oculto según audiencia
    // son responsabilidad del proyector; aquí ambos provisionales existen.
    let rows = repository(&mut c).block_rows_list(&access()).unwrap();
    assert_eq!(rows.len(), 2);
    assert!(rows.iter().all(|row| row.operations.len() == 1));
}

#[test]
fn global_block_requires_owner_and_own_professional() {
    let mut c = database_with_professional();
    let ids = Ids::default();

    // Asistente: denegado aunque el profesional sea el propio.
    let denied = repository(&mut c).persist(
        &access(),
        SchedulingMutation::CreateBlock {
            value: global_block("2026-04-02T13:00:00Z", "2026-04-02T17:00:00Z"),
        },
        &ids,
        &FixedClock,
        &NoOccupancy,
    );
    assert_eq!(denied.unwrap_err().code, "PERMISSION_DENIED");

    // Propietario sobre profesional de otro usuario: denegado.
    let other = BlockValue::new(
        OTHER_PROFESSIONAL.to_owned().try_into().unwrap(),
        None,
        Instant::parse("2026-04-02T13:00:00Z").unwrap(),
        Instant::parse("2026-04-02T17:00:00Z").unwrap(),
        ReasonCode::Vacation,
    )
    .unwrap();
    let denied_other = repository(&mut c).persist(
        &owner_access(),
        SchedulingMutation::CreateBlock { value: other },
        &ids,
        &FixedClock,
        &NoOccupancy,
    );
    assert_eq!(denied_other.unwrap_err().code, "PERMISSION_DENIED");

    // Asistente con bloqueo de cuenta: permitido.
    let created = repository(&mut c)
        .persist(
            &access(),
            SchedulingMutation::CreateBlock {
                value: account_block("2026-04-01T13:00:00Z", "2026-04-01T17:00:00Z"),
            },
            &ids,
            &FixedClock,
            &NoOccupancy,
        )
        .unwrap();
    assert_eq!(created.entity_type, "schedule_block");
}

#[test]
fn block_scope_never_changes_by_patch() {
    let mut c = database_with_professional();
    let ids = Ids::default();

    let created = repository(&mut c)
        .persist(
            &access(),
            SchedulingMutation::CreateBlock {
                value: account_block("2026-04-01T13:00:00Z", "2026-04-01T17:00:00Z"),
            },
            &ids,
            &FixedClock,
            &NoOccupancy,
        )
        .unwrap();
    let block_id = created.entity_key.as_str().unwrap().to_owned();

    repository(&mut c)
        .persist(
            &access(),
            SchedulingMutation::UpdateBlock {
                block_id: block_id.clone(),
                base: EditBase::LocalOperation {
                    operation_id: op(&created.operation_id),
                },
                patch: BlockPatch::new(
                    Some(Instant::parse("2026-04-01T14:00:00Z").unwrap()),
                    Some(Instant::parse("2026-04-01T18:00:00Z").unwrap()),
                    Some(ReasonCode::Unavailable),
                )
                .unwrap(),
            },
            &ids,
            &FixedClock,
            &NoOccupancy,
        )
        .unwrap();

    let payload = outbox_payload(&c, "UpdateScheduleBlock");
    assert_eq!(
        payload,
        json!({
            "startsAt": "2026-04-01T14:00:00Z",
            "endsAt": "2026-04-01T18:00:00Z",
            "reasonCode": "unavailable"
        })
    );

    // El ámbito y el profesional se conservan en el provisional.
    let rows = repository(&mut c).block_rows_list(&access()).unwrap();
    let visible = rows[0].visible().unwrap();
    assert_eq!(visible.block_account_id.as_deref(), Some(ACCOUNT));
    assert_eq!(visible.professional_id, PROFESSIONAL);
}

#[test]
fn known_appointment_inside_block_blocks_the_write() {
    let mut c = database_with_professional();
    let ids = Ids::default();
    let occupancy = FakeOccupancy(vec![appointment()]);
    let ids0 = Ids::default();

    // Regla que cubre la cita: los bloqueos no deben exigir cobertura por sí solos.
    repository(&mut c)
        .persist(
            &access(),
            SchedulingMutation::CreateRule {
                value: super::domain::RuleValue::new(
                    PROFESSIONAL.to_owned().try_into().unwrap(),
                    3,
                    0,
                    1440,
                    super::domain::parse_local_date("2026-01-01").unwrap(),
                    None,
                )
                .unwrap(),
            },
            &ids0,
            &FixedClock,
            &occupancy,
        )
        .unwrap();
    drop(ids0);

    // Bloqueo de cuenta sobre la cita conocida: rechazado.
    let overlapping = repository(&mut c).persist(
        &access(),
        SchedulingMutation::CreateBlock {
            value: account_block("2026-05-06T13:00:00Z", "2026-05-06T15:00:00Z"),
        },
        &ids,
        &FixedClock,
        &occupancy,
    );
    assert_eq!(overlapping.unwrap_err().code, "SLOT_UNAVAILABLE");

    // Bloqueo global sobre la cita, propietario: rechazado igualmente.
    let global = repository(&mut c).persist(
        &owner_access(),
        SchedulingMutation::CreateBlock {
            value: global_block("2026-05-06T13:00:00Z", "2026-05-06T15:00:00Z"),
        },
        &ids,
        &FixedClock,
        &occupancy,
    );
    assert_eq!(global.unwrap_err().code, "SLOT_UNAVAILABLE");
    let count: i64 = c
        .query_row(
            "SELECT count(*) FROM sync_outbox WHERE entity_type = 'schedule_block'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 0, "ninguna intención de bloqueo creada");

    // Bloqueo que no toca la cita: permitido.
    let created = repository(&mut c)
        .persist(
            &access(),
            SchedulingMutation::CreateBlock {
                value: account_block("2026-05-06T15:00:00Z", "2026-05-06T16:00:00Z"),
            },
            &ids,
            &FixedClock,
            &occupancy,
        )
        .unwrap();

    // Ampliar el bloqueo hasta cubrir la cita: rechazado; el trabajo previo se conserva.
    let expanded = repository(&mut c).persist(
        &access(),
        SchedulingMutation::UpdateBlock {
            block_id: created.entity_key.as_str().unwrap().to_owned(),
            base: EditBase::LocalOperation {
                operation_id: op(&created.operation_id),
            },
            patch: BlockPatch::new(
                Some(Instant::parse("2026-05-06T13:45:00Z").unwrap()),
                None,
                None,
            )
            .unwrap(),
        },
        &ids,
        &FixedClock,
        &occupancy,
    );
    assert_eq!(expanded.unwrap_err().code, "SLOT_UNAVAILABLE");
    let rows = repository(&mut c).block_rows_list(&access()).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].operations.len(), 1);
}

#[test]
fn successive_block_edits_survive_reopen() {
    let dir = std::env::temp_dir().join(format!(
        "nightingale-scheduling-t08-{}.sqlite",
        uuid::Uuid::now_v7()
    ));
    let (block_id, first_op) = {
        let mut c = crate::platform::sqlite::open(&dir).unwrap();
        super::test_support::seed_account(&mut c);
        let tx = c.transaction().unwrap();
        tx.execute(
            "INSERT INTO professional_view (account_id,user_id,device_id,professional_id,professional_user_id,public_name,status,version) VALUES ('019a1234-0000-7000-8000-000000000001','019a1234-0000-7000-8000-000000000002','019a1234-0000-7000-8000-000000000003',?1,'019a1234-0000-7000-8000-000000000002','Profesional ficticio','active',1)",
            rusqlite::params![PROFESSIONAL],
        )
        .unwrap();
        tx.commit().unwrap();
        let ids = Ids::default();

        let created = repository(&mut c)
            .persist(
                &access(),
                SchedulingMutation::CreateBlock {
                    value: account_block("2026-04-01T13:00:00Z", "2026-04-01T17:00:00Z"),
                },
                &ids,
                &FixedClock,
                &NoOccupancy,
            )
            .unwrap();
        (
            created.entity_key.as_str().unwrap().to_owned(),
            created.operation_id,
        )
    };

    let mut c = crate::platform::sqlite::open(&dir).unwrap();
    let ids = Ids::default();
    repository(&mut c)
        .persist(
            &access(),
            SchedulingMutation::SetBlockStatus {
                block_id: block_id.clone(),
                base: EditBase::LocalOperation {
                    operation_id: op(&first_op),
                },
                status: Status::Inactive,
            },
            &ids,
            &FixedClock,
            &NoOccupancy,
        )
        .unwrap();

    let rows = repository(&mut c).block_rows_list(&access()).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].operations.len(), 2);
    assert_eq!(rows[0].visible().unwrap().status, "inactive");
    drop(c);
    std::fs::remove_file(&dir).unwrap();
}
