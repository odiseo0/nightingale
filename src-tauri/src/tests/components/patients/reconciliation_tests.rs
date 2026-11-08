use super::test_support::*;
use crate::components::patients::{application::*, domain::*};
use crate::components::sync::remote::{
    ChangeGroup, ChangeKind, EntityType, SyncChange, apply_groups,
};
use rusqlite::params;
use serde_json::json;

fn profile_group(
    s: &crate::shared_kernel::id::OperationContext,
    patient: &str,
    version: i64,
    name: &str,
) -> ChangeGroup {
    ChangeGroup {
        group_id: uuid::Uuid::now_v7().to_string().try_into().unwrap(),
        operation_id: uuid::Uuid::now_v7().to_string().try_into().unwrap(),
        accepted_at: "2026-11-05T12:00:00Z".into(),
        changes: vec![SyncChange {
            entity_type: EntityType::PatientProfile,
            entity_key: json!(patient),
            entity_version: version,
            change_kind: ChangeKind::Upsert,
            state: json!({"accountId":s.account_id,"patientId":patient,"fullName":name,"birthDate":null,"phoneCode":null,"phone":null,"identificationBasis":null,"completionStatus":"draft","status":"active","version":version,"createdAt":null,"updatedAt":null}),
        }],
    }
}

/// Record the result a dispatch cycle would store for a sent operation.
fn accept(c: &mut rusqlite::Connection, operation_id: &str, entity_version: i64) {
    c.execute(
        "UPDATE sync_outbox SET status='accepted', result=?2 WHERE operation_id=?1",
        params![
            operation_id,
            json!({"operationId":operation_id,"status":"accepted","entityVersion":entity_version,"state":{},"error":Value::Null}).to_string()
        ],
    )
    .unwrap();
}

fn reject(c: &mut rusqlite::Connection, operation_id: &str) {
    c.execute(
        "UPDATE sync_outbox SET status='rejected', last_error='\"VERSION_CONFLICT\"', result=?2 WHERE operation_id=?1",
        params![
            operation_id,
            json!({"operationId":operation_id,"status":"rejected","entityVersion":null,"state":null,"error":{"code":"VERSION_CONFLICT","message":"stale","details":null}}).to_string()
        ],
    )
    .unwrap();
}

fn patient_id_of(result: &LocalMutationResult) -> String {
    result.entity_key.as_str().unwrap().to_owned()
}

fn patient(result: &LocalMutationResult) -> PatientId {
    result
        .entity_key
        .as_str()
        .unwrap()
        .to_owned()
        .try_into()
        .unwrap()
}

fn local_base(result: &LocalMutationResult) -> EditBase {
    EditBase::LocalOperation {
        operation_id: result.operation_id.clone().try_into().unwrap(),
    }
}

use serde_json::Value;

#[test]
fn accepted_receipt_with_reached_view_retires_only_that_provisional() {
    let mut c = database();
    let a = access();
    let ids = Ids::default();
    let registered = run(&mut c, &a, &ids, |cases| {
        cases.register_patient(data("Paciente local")).unwrap()
    });
    let edited = run(&mut c, &a, &ids, |cases| {
        cases
            .update_patient_profile(
                patient(&registered),
                local_base(&registered),
                ProfilePatch {
                    full_name: FieldChange::Set("Paciente editado".into()),
                    ..ProfilePatch::default()
                },
            )
            .unwrap()
    });

    let patient_key = patient_id_of(&registered);
    // Receipt alone does not retire anything; the download is still required.
    accept(&mut c, &registered.operation_id, 1);
    run(&mut c, &a, &ids, |cases| {
        let row = cases.get_patient(&patient(&registered)).unwrap();
        assert!(row.accepted.is_none());
        assert!(row.provisional.is_some());
    });

    // The accepted view reaches version 1: only that provisional retires.
    apply_groups(
        &mut c,
        &a.context.clone(),
        &[profile_group(&a.context, &patient_key, 1, "Paciente local")],
        &crate::components::patients::adapters::projector::PatientsProjector,
    )
    .unwrap();
    run(&mut c, &a, &ids, |cases| {
        let row = cases.get_patient(&patient(&registered)).unwrap();
        assert_eq!(row.accepted.as_ref().unwrap().version, 1);
        let provisional = row.provisional.as_ref().unwrap();
        assert_eq!(provisional.expected_version, 1);
        assert_eq!(
            provisional.state.full_name.as_deref(),
            Some("Paciente editado")
        );
        assert_eq!(row.operations.len(), 2);
        assert_eq!(row.operations[0].status, "accepted");
        assert_eq!(row.operations[1].status, "pending");
    });

    // The second step confirms too; the provisional disappears entirely.
    accept(&mut c, &edited.operation_id, 2);
    apply_groups(
        &mut c,
        &a.context,
        &[profile_group(
            &a.context,
            &patient_key,
            2,
            "Paciente editado",
        )],
        &crate::components::patients::adapters::projector::PatientsProjector,
    )
    .unwrap();
    run(&mut c, &a, &ids, |cases| {
        let row = cases.get_patient(&patient(&registered)).unwrap();
        assert!(row.provisional.is_none());
        assert_eq!(row.accepted.as_ref().unwrap().version, 2);
        assert_eq!(
            row.operations.len(),
            2,
            "operation and receipt stay available"
        );
    });
}

#[test]
fn rejected_operations_keep_provisional_sequence_and_block_dependents() {
    let mut c = database();
    let a = access();
    let ids = Ids::default();
    let registered = run(&mut c, &a, &ids, |cases| {
        cases.register_patient(data("Paciente local")).unwrap()
    });
    let edited = run(&mut c, &a, &ids, |cases| {
        cases
            .update_patient_profile(
                patient(&registered),
                local_base(&registered),
                ProfilePatch {
                    full_name: FieldChange::Set("Cambio pendiente".into()),
                    ..ProfilePatch::default()
                },
            )
            .unwrap()
    });

    reject(&mut c, &registered.operation_id);
    run(&mut c, &a, &ids, |cases| {
        let row = cases.get_patient(&patient(&registered)).unwrap();
        assert!(row.accepted.is_none());
        // The whole sequence and its rejected first step stay for review.
        assert_eq!(row.operations.len(), 2);
        assert_eq!(row.operations[0].status, "rejected");
        assert_eq!(row.operations[0].error.as_deref(), Some("VERSION_CONFLICT"));
        assert!(row.provisional.is_some());
        // Editing on top of a blocked sequence is refused without changes.
        assert_eq!(
            cases
                .update_patient_profile(
                    patient(&registered),
                    local_base(&edited),
                    ProfilePatch {
                        full_name: FieldChange::Set("Otro intento".into()),
                        ..ProfilePatch::default()
                    }
                )
                .unwrap_err()
                .code,
            "DEPENDENCY_FAILED"
        );
        // A fresh, unrelated operation is still allowed.
        assert!(cases.register_patient(data("Otro paciente")).is_ok());
    });
}

#[test]
fn conflict_and_unknown_results_keep_provisional_for_later_recovery() {
    let mut c = database();
    let a = access();
    let ids = Ids::default();
    let registered = run(&mut c, &a, &ids, |cases| {
        cases.register_patient(data("Paciente incierto")).unwrap()
    });
    c.execute(
        "UPDATE sync_outbox SET status='unknown' WHERE operation_id=?1",
        params![registered.operation_id],
    )
    .unwrap();
    run(&mut c, &a, &ids, |cases| {
        let row = cases.get_patient(&patient(&registered)).unwrap();
        assert_eq!(row.operations[0].status, "unknown");
        assert!(row.provisional.is_some());
        // A distinct edit may still depend on the unknown original, keeping its ID.
        let other = cases
            .update_patient_profile(
                patient(&registered),
                local_base(&registered),
                ProfilePatch {
                    full_name: FieldChange::Set("Edición distinta".into()),
                    ..ProfilePatch::default()
                },
            )
            .unwrap();
        assert!(other.depends_on.contains(&registered.operation_id));
    });
}

fn data(name: &str) -> ProfileData {
    ProfileData {
        full_name: Some(name.into()),
        ..ProfileData::default()
    }
}
