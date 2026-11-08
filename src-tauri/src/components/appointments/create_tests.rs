//! CreateAppointment: booking validation matrix and the integrated local flow.
use super::{
    adapters::outbound::SqliteAppointmentRepository,
    application::{CreateAppointmentInput, validate_new_appointment},
    test_support::{Database, FixedClock, Ids, PATIENT, PROFESSIONAL, SERVICE, access},
};
use crate::components::account::domain::{AccountPolicy, AccountStatus, AccountView};
use crate::components::patients::application::{
    Accepted, BookingPatient, BookingReference, EntityRow, IdentifierState, ProfileState,
    Provisional, RelationshipState, RelationshipView,
};
use crate::components::scheduling::application::AvailabilityEvaluation;
use crate::components::services::application::{BookingOffer, CatalogReference};
use rusqlite::params;

const ACCOUNT: &str = "019a1234-0000-7000-8000-000000000001";
const CONTACT: &str = "019a1234-0000-7000-8000-000000000030";

fn account() -> AccountView {
    AccountView {
        account_id: ACCOUNT.to_owned().try_into().unwrap(),
        display_name: "Consultorio Sintético".into(),
        time_zone: "America/Caracas".into(),
        location_name: "Consultorio Central".into(),
        location_address: "Av. Principal 1".into(),
        location_city: "Caracas".into(),
        location_state: "Distrito Capital".into(),
        location_country: "VE".into(),
        policy: AccountPolicy::new(15).unwrap(),
        status: AccountStatus::Active,
        version: 1,
        updated_at: None,
    }
}

fn profile_row(basis: &str, birth: Option<&str>, phone: bool) -> EntityRow<ProfileState> {
    EntityRow {
        accepted: Some(Accepted {
            state: ProfileState {
                patient_id: PATIENT.into(),
                full_name: Some("Paciente Sintético".into()),
                birth_date: birth.map(str::to_owned),
                phone_code: phone.then(|| "58".into()),
                phone: phone.then(|| "4120000000".into()),
                identification_basis: Some(basis.into()),
                completion_status: "complete".into(),
                status: "active".into(),
            },
            version: 1,
        }),
        provisional: None,
        operations: vec![],
    }
}

fn identifier_row(verification: &str) -> EntityRow<IdentifierState> {
    EntityRow {
        accepted: Some(Accepted {
            state: IdentifierState {
                id: "019a1234-0000-7000-8000-000000000031".into(),
                patient_id: PATIENT.into(),
                issuing_country: "VE".into(),
                document_type: "national_id".into(),
                document_number: "V123456".into(),
                verification_status: verification.into(),
            },
            version: 1,
        }),
        provisional: None,
        operations: vec![],
    }
}

fn responsible_row(
    valid_from: &str,
    valid_until: Option<&str>,
    phone: bool,
) -> EntityRow<RelationshipView> {
    EntityRow {
        accepted: Some(Accepted {
            state: RelationshipView {
                relationship: RelationshipState {
                    patient_id: PATIENT.into(),
                    contact_person_id: CONTACT.into(),
                    relationship_kind: "mother".into(),
                    is_responsible_contact: true,
                    valid_from: valid_from.into(),
                    valid_until: valid_until.map(str::to_owned),
                    status: "active".into(),
                },
                patient_full_name: Some("Paciente Sintético".into()),
                contact_full_name: "Responsable Sintético".into(),
                contact_phone_code: phone.then(|| "58".into()),
                contact_phone: phone.then(|| "4140000000".into()),
            },
            version: 1,
        }),
        provisional: None,
        operations: vec![],
    }
}

fn patient(
    profile: EntityRow<ProfileState>,
    identifiers: Vec<EntityRow<IdentifierState>>,
    responsible: Vec<EntityRow<RelationshipView>>,
) -> BookingPatient {
    BookingPatient {
        profile,
        reference: BookingReference::Accepted { version: 1 },
        identifiers,
        responsible_contacts: responsible,
    }
}

fn offer() -> BookingOffer {
    BookingOffer {
        professional_id: PROFESSIONAL.into(),
        service_id: SERVICE.into(),
        service_name: "Consulta general".into(),
        professional_name: "Dra. Prueba".into(),
        effective_duration_minutes: 30,
        status: "active".into(),
        service_status: "active".into(),
        reference: CatalogReference::Accepted {
            version: 2,
            service_version: 7,
        },
    }
}

fn input(responsible: Option<&str>) -> CreateAppointmentInput {
    // 2026-05-04 is a Monday; 09:00-09:30 in Caracas.
    CreateAppointmentInput {
        patient_id: PATIENT.into(),
        professional_id: PROFESSIONAL.into(),
        service_id: SERVICE.into(),
        responsible_contact_id: responsible.map(str::to_owned),
        starts_at: "2026-05-04T13:00:00Z".into(),
        ends_at: "2026-05-04T13:30:00Z".into(),
    }
}

fn evaluation(coverage: &str, intervals: &[(&str, &str)]) -> AvailabilityEvaluation {
    AvailabilityEvaluation {
        time_zone: "America/Caracas".into(),
        reference_duration_minutes: 30,
        requested_duration_minutes: 30,
        account_version: 1,
        offer_version: 2,
        catalog_version: 7,
        agenda_versions: vec![],
        intervals: intervals
            .iter()
            .map(|(starts, ends)| {
                crate::components::scheduling::application::AvailabilityInterval {
                    starts_at: starts.to_string(),
                    ends_at: ends.to_string(),
                }
            })
            .collect(),
        origin: "local".into(),
        provisional: true,
        coverage: coverage.into(),
    }
}

fn ok_evaluation() -> AvailabilityEvaluation {
    evaluation(
        "within_synced_window",
        &[("2026-05-04T13:00:00Z", "2026-05-04T16:00:00Z")],
    )
}

#[test]
fn adult_with_document_declared_or_verified_books() {
    for verification in ["declared", "verified"] {
        let view = patient(
            profile_row("own_document", None, true),
            vec![identifier_row(verification)],
            vec![],
        );
        let created =
            validate_new_appointment(&account(), &view, &offer(), &input(None), &ok_evaluation())
                .unwrap();
        assert_eq!(created.snapshots.service_name, "Consulta general");
        assert_eq!(created.snapshots.offered_duration_minutes, 30);
        assert_eq!(created.snapshots.professional_name, "Dra. Prueba");
        assert_eq!(created.snapshots.location_name, "Consultorio Central");
        assert_eq!(created.snapshots.location_country, "VE");
        assert_eq!(created.snapshots.time_zone, "America/Caracas");
        assert_eq!(created.interval.duration_minutes(), 30);
    }
}

#[test]
fn adult_rules_reject_invalid_states() {
    // Only an invalidated document.
    let view = patient(
        profile_row("own_document", None, true),
        vec![identifier_row("invalidated")],
        vec![],
    );
    assert!(
        validate_new_appointment(&account(), &view, &offer(), &input(None), &ok_evaluation())
            .is_err()
    );

    // No document at all.
    let view = patient(profile_row("own_document", None, true), vec![], vec![]);
    assert!(
        validate_new_appointment(&account(), &view, &offer(), &input(None), &ok_evaluation())
            .is_err()
    );

    // No own phone and no responsible.
    let view = patient(
        profile_row("own_document", None, false),
        vec![identifier_row("declared")],
        vec![],
    );
    assert!(
        validate_new_appointment(&account(), &view, &offer(), &input(None), &ok_evaluation())
            .is_err()
    );

    // Own phone missing but a valid responsible is selected.
    let view = patient(
        profile_row("own_document", None, false),
        vec![identifier_row("declared")],
        vec![responsible_row("2020-01-01", None, true)],
    );
    assert!(
        validate_new_appointment(
            &account(),
            &view,
            &offer(),
            &input(Some(CONTACT)),
            &ok_evaluation()
        )
        .is_ok()
    );

    // Draft completion or inactive status cannot book.
    let mut view = patient(
        profile_row("own_document", None, true),
        vec![identifier_row("declared")],
        vec![],
    );
    view.profile
        .accepted
        .as_mut()
        .unwrap()
        .state
        .completion_status = "draft".into();
    assert!(
        validate_new_appointment(&account(), &view, &offer(), &input(None), &ok_evaluation())
            .is_err()
    );

    let mut view = patient(
        profile_row("own_document", None, true),
        vec![identifier_row("declared")],
        vec![],
    );
    view.profile.accepted.as_mut().unwrap().state.status = "inactive".into();
    assert!(
        validate_new_appointment(&account(), &view, &offer(), &input(None), &ok_evaluation())
            .is_err()
    );
}

#[test]
fn minor_rules_apply_at_the_local_appointment_date() {
    // Turns 18 the day of the appointment (2026-05-04): no longer a minor.
    let view = patient(
        profile_row("minor_without_document", Some("2008-05-04"), false),
        vec![],
        vec![responsible_row("2020-01-01", None, true)],
    );
    assert!(
        validate_new_appointment(
            &account(),
            &view,
            &offer(),
            &input(Some(CONTACT)),
            &ok_evaluation()
        )
        .is_err()
    );

    // Still 17 with a valid responsible: bookable.
    let view = patient(
        profile_row("minor_without_document", Some("2008-05-05"), false),
        vec![],
        vec![responsible_row("2020-01-01", None, true)],
    );
    assert!(
        validate_new_appointment(
            &account(),
            &view,
            &offer(),
            &input(Some(CONTACT)),
            &ok_evaluation()
        )
        .is_ok()
    );

    // Minor without responsible: rejected.
    let view = patient(
        profile_row("minor_without_document", Some("2015-03-10"), false),
        vec![],
        vec![],
    );
    assert!(
        validate_new_appointment(&account(), &view, &offer(), &input(None), &ok_evaluation())
            .is_err()
    );
}

#[test]
fn responsible_must_be_valid_and_have_phone_on_the_local_date() {
    let base = || {
        patient(
            profile_row("own_document", None, false),
            vec![identifier_row("declared")],
            vec![responsible_row("2020-01-01", None, true)],
        )
    };

    // Unknown contact.
    let err = validate_new_appointment(
        &account(),
        &base(),
        &offer(),
        &input(Some("019a1234-0000-7000-8000-000000000099")),
        &ok_evaluation(),
    )
    .unwrap_err();
    assert_eq!(err.code, "VALIDATION_ERROR");

    // Link starts after the appointment date.
    let view = patient(
        profile_row("own_document", None, false),
        vec![identifier_row("declared")],
        vec![responsible_row("2026-06-01", None, true)],
    );
    assert!(
        validate_new_appointment(
            &account(),
            &view,
            &offer(),
            &input(Some(CONTACT)),
            &ok_evaluation()
        )
        .is_err()
    );

    // Link ended before the appointment date.
    let view = patient(
        profile_row("own_document", None, false),
        vec![identifier_row("declared")],
        vec![responsible_row("2020-01-01", Some("2026-05-03"), true)],
    );
    assert!(
        validate_new_appointment(
            &account(),
            &view,
            &offer(),
            &input(Some(CONTACT)),
            &ok_evaluation()
        )
        .is_err()
    );

    // No phone on the contact.
    let view = patient(
        profile_row("own_document", None, false),
        vec![identifier_row("declared")],
        vec![responsible_row("2020-01-01", None, false)],
    );
    assert!(
        validate_new_appointment(
            &account(),
            &view,
            &offer(),
            &input(Some(CONTACT)),
            &ok_evaluation()
        )
        .is_err()
    );
}

#[test]
fn coverage_and_slot_rules_never_pretend_freedom() {
    let view = patient(
        profile_row("own_document", None, true),
        vec![identifier_row("declared")],
        vec![],
    );

    // Insufficient coverage blocks without an intention.
    let err = validate_new_appointment(
        &account(),
        &view,
        &offer(),
        &input(None),
        &evaluation("insufficient_coverage", &[]),
    )
    .unwrap_err();
    assert_eq!(err.code, "INSUFFICIENT_COVERAGE");

    // No synced window still works on known local data.
    assert!(
        validate_new_appointment(
            &account(),
            &view,
            &offer(),
            &input(None),
            &evaluation(
                "no_synced_window",
                &[("2026-05-04T13:00:00Z", "2026-05-04T16:00:00Z")]
            )
        )
        .is_ok()
    );

    // The interval must fit inside one available interval.
    let err = validate_new_appointment(
        &account(),
        &view,
        &offer(),
        &input(None),
        &evaluation(
            "within_synced_window",
            &[("2026-05-04T13:15:00Z", "2026-05-04T16:00:00Z")],
        ),
    )
    .unwrap_err();
    assert_eq!(err.code, "SLOT_UNAVAILABLE");

    // Occupied sub-interval leaves no fit either.
    let err = validate_new_appointment(
        &account(),
        &view,
        &offer(),
        &input(None),
        &evaluation(
            "within_synced_window",
            &[
                ("2026-05-04T13:00:00Z", "2026-05-04T13:05:00Z"),
                ("2026-05-04T13:25:00Z", "2026-05-04T16:00:00Z"),
            ],
        ),
    )
    .unwrap_err();
    assert_eq!(err.code, "SLOT_UNAVAILABLE");
}

// ---- Integrated local flow over a real SQLite scope ----

pub(crate) fn seed_context(connection: &rusqlite::Connection) {
    let scope = access().context;
    connection
        .execute(
            "INSERT INTO account_view VALUES (?1,?2,?3,'Consultorio Sintético','America/Caracas','Consultorio Central','Av. Principal 1','Caracas','Distrito Capital','VE',15,'active',1,NULL)",
            params![
                scope.account_id.as_str(),
                scope.user_id.as_str(),
                scope.device_id.as_str()
            ],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO professional_view (account_id,user_id,device_id,professional_id,professional_user_id,public_name,status,version) VALUES (?1,?2,?3,?4,?2,'Dra. Prueba','active',1)",
            params![
                scope.account_id.as_str(),
                scope.user_id.as_str(),
                scope.device_id.as_str(),
                PROFESSIONAL
            ],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO service_offer_view (account_id,user_id,device_id,professional_id,service_id,name,default_duration_minutes,duration_minutes_override,effective_duration_minutes,service_status,status,version,service_version) VALUES (?1,?2,?3,?4,?5,'Consulta general',30,NULL,30,'active','active',2,7)",
            params![
                scope.account_id.as_str(),
                scope.user_id.as_str(),
                scope.device_id.as_str(),
                PROFESSIONAL,
                SERVICE
            ],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO service_catalog_view (account_id,user_id,device_id,service_id,professional_id,name,default_duration_minutes,status,version) VALUES (?1,?2,?3,?4,?5,'Consulta general',30,'active',7)",
            params![
                scope.account_id.as_str(),
                scope.user_id.as_str(),
                scope.device_id.as_str(),
                SERVICE,
                PROFESSIONAL
            ],
        )
        .unwrap();
    // Monday 08:00-12:00 local (12:00-16:00Z).
    connection
        .execute(
            "INSERT INTO schedule_rule_view (account_id,user_id,device_id,rule_id,professional_id,iso_weekday,local_start_minute,local_end_minute,valid_from,valid_until,status,version) VALUES (?1,?2,?3,'019a1234-0000-7000-8000-000000000040',?4,1,480,720,'2026-01-01',NULL,'active',1)",
            params![
                scope.account_id.as_str(),
                scope.user_id.as_str(),
                scope.device_id.as_str(),
                PROFESSIONAL
            ],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO patient_profile_view (account_id,user_id,device_id,patient_id,full_name,birth_date,phone_code,phone,identification_basis,completion_status,status,version) VALUES (?1,?2,?3,?4,'Paciente Sintético',NULL,'58','4120000000','own_document','complete','active',1)",
            params![
                scope.account_id.as_str(),
                scope.user_id.as_str(),
                scope.device_id.as_str(),
                PATIENT
            ],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO patient_identifier_view (account_id,user_id,device_id,identifier_id,patient_id,issuing_country,document_type,document_number,verification_status,version) VALUES (?1,?2,?3,'019a1234-0000-7000-8000-000000000041',?4,'VE','national_id','V123456','declared',1)",
            params![
                scope.account_id.as_str(),
                scope.user_id.as_str(),
                scope.device_id.as_str(),
                PATIENT
            ],
        )
        .unwrap();
    // Occupancy window fully covering the appointment day.
    connection
        .execute(
            "INSERT INTO sync_cursor (account_id,user_id,device_id,permission_epoch,cursor,window) VALUES (?1,?2,?3,1,'cursor-1','{\"appointmentsFrom\":\"2026-05-01T00:00:00Z\",\"appointmentsUntil\":\"2026-05-31T00:00:00Z\"}')",
            params![
                scope.account_id.as_str(),
                scope.user_id.as_str(),
                scope.device_id.as_str()
            ],
        )
        .unwrap();
}

fn create_via_repository(
    connection: &mut rusqlite::Connection,
    ids: &Ids,
    starts: &str,
    ends: &str,
) -> super::application::AppResult<super::application::LocalMutationResult> {
    SqliteAppointmentRepository { connection }.create_appointment(
        &access(),
        CreateAppointmentInput {
            patient_id: PATIENT.into(),
            professional_id: PROFESSIONAL.into(),
            service_id: SERVICE.into(),
            responsible_contact_id: None,
            starts_at: starts.into(),
            ends_at: ends.into(),
        },
        ids,
        &FixedClock,
    )
}

#[test]
fn create_persists_provisional_and_blocks_double_booking() {
    let database = Database::new();
    let mut connection = crate::platform::sqlite::open(&database.0).unwrap();
    {
        let tx = connection.transaction().unwrap();
        use crate::components::identity::repository::AccessStore as _;
        crate::components::identity::repository::SqliteAccessStore
            .save_access(&tx, &access())
            .unwrap();
        tx.commit().unwrap();
    }
    seed_context(&connection);
    let ids = Ids::default();

    let created = create_via_repository(
        &mut connection,
        &ids,
        "2026-05-04T13:00:00Z",
        "2026-05-04T13:30:00Z",
    )
    .unwrap();
    assert_eq!(created.expected_version, 0);
    let draft = &created.provisional;
    assert_eq!(draft["status"], "scheduled");
    assert_eq!(draft["serviceName"], "Consulta general");
    assert_eq!(draft["offeredDurationMinutes"], 30);
    assert_eq!(draft["professionalName"], "Dra. Prueba");
    assert_eq!(draft["locationName"], "Consultorio Central");
    assert_eq!(draft["timeZone"], "America/Caracas");
    assert_eq!(draft["confirmedAt"], serde_json::Value::Null);

    // Outbox row carries the contract payload.
    let payload: String = connection
        .query_row(
            "SELECT payload FROM sync_outbox WHERE operation_id=?1",
            params![created.operation_id],
            |row| row.get(0),
        )
        .unwrap();
    let payload: serde_json::Value = serde_json::from_str(&payload).unwrap();
    assert_eq!(payload["patientId"], PATIENT);
    assert_eq!(payload["professionalId"], PROFESSIONAL);
    assert_eq!(payload["serviceId"], SERVICE);
    assert!(payload.get("responsibleContactId").is_none());

    // The provisional occupancy blocks a second booking in the same slot.
    let err = create_via_repository(
        &mut connection,
        &ids,
        "2026-05-04T13:00:00Z",
        "2026-05-04T13:30:00Z",
    )
    .unwrap_err();
    assert_eq!(err.code, "SLOT_UNAVAILABLE");

    // An adjacent slot is still free (half-open intervals).
    create_via_repository(
        &mut connection,
        &ids,
        "2026-05-04T13:30:00Z",
        "2026-05-04T14:00:00Z",
    )
    .unwrap();

    // Outside the schedule: rejected before any write.
    let before: i64 = connection
        .query_row("SELECT count(*) FROM appointment_local", [], |row| {
            row.get(0)
        })
        .unwrap();
    let err = create_via_repository(
        &mut connection,
        &ids,
        "2026-05-04T17:00:00Z",
        "2026-05-04T17:30:00Z",
    )
    .unwrap_err();
    assert_eq!(err.code, "SLOT_UNAVAILABLE");
    let after: i64 = connection
        .query_row("SELECT count(*) FROM appointment_local", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(before, after, "rejected creates leave no intention");
}

#[test]
fn create_with_provisional_patient_declares_dependency() {
    let database = Database::new();
    let mut connection = crate::platform::sqlite::open(&database.0).unwrap();
    {
        let tx = connection.transaction().unwrap();
        use crate::components::identity::repository::AccessStore as _;
        crate::components::identity::repository::SqliteAccessStore
            .save_access(&tx, &access())
            .unwrap();
        tx.commit().unwrap();
    }
    seed_context(&connection);

    // A complete minor whose profile and responsible link exist only as
    // chainable provisional intentions (same offline session).
    let minor_id = "019a1234-0000-7000-8000-000000000050";
    let patient_op = "019a1234-0000-7000-8000-000000000051";
    let scope = access().context;
    connection
        .execute(
            "INSERT INTO sync_outbox (operation_id,contract_version,action,entity_type,entity_key,expected_version,payload,occurred_at_local,created_at,account_id,user_id,device_id,permission_epoch,depends_on,semantic_fingerprint,status,attempts,last_error,result,lease_until,next_attempt_at)
             VALUES (?1,1,'SetPatientCompletion','patient_profile',?5,1,'{}','2026-11-08T10:00:00Z',1,?2,?3,?4,1,'[]',NULL,'pending',0,NULL,NULL,NULL,NULL)",
            params![patient_op, scope.account_id.as_str(), scope.user_id.as_str(), scope.device_id.as_str(), format!("\"{}\"", minor_id)],
        )
        .unwrap();
    let draft = serde_json::json!({
        "patientId": minor_id,
        "fullName": "Menor Sintético",
        "birthDate": "2015-03-10",
        "phoneCode": null,
        "phone": null,
        "identificationBasis": "minor_without_document",
        "completionStatus": "complete",
        "status": "active"
    });
    connection
        .execute(
            "INSERT INTO patient_local (account_id,user_id,device_id,operation_id,entity_type,patient_id,identifier_id,action,expected_version,base_operation_id,depends_on,payload,draft,created_at) VALUES (?1,?2,?3,?4,'patient_profile',?5,NULL,'SetPatientCompletion',1,NULL,'[]','{}',?6,1)",
            rusqlite::params![scope.account_id.as_str(), scope.user_id.as_str(), scope.device_id.as_str(), patient_op, minor_id, draft.to_string()],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO contact_person_view (account_id,user_id,device_id,contact_person_id,full_name,phone_code,phone,status,version) VALUES (?1,?2,?3,?4,'Responsable Sintético','58','4140000000','active',1)",
            params![
                scope.account_id.as_str(),
                scope.user_id.as_str(),
                scope.device_id.as_str(),
                CONTACT
            ],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO patient_contact_view (account_id,user_id,device_id,patient_id,contact_person_id,relationship_kind,is_responsible_contact,representation_status,valid_from,status,version) VALUES (?1,?2,?3,?4,?5,'mother',1,'unverified','2020-01-01','active',1)",
            params![
                scope.account_id.as_str(),
                scope.user_id.as_str(),
                scope.device_id.as_str(),
                minor_id,
                CONTACT
            ],
        )
        .unwrap();

    let ids = Ids(std::cell::Cell::new(500));
    let created = SqliteAppointmentRepository {
        connection: &mut connection,
    }
    .create_appointment(
        &access(),
        CreateAppointmentInput {
            patient_id: minor_id.into(),
            professional_id: PROFESSIONAL.into(),
            service_id: SERVICE.into(),
            responsible_contact_id: Some(CONTACT.into()),
            starts_at: "2026-05-04T13:00:00Z".into(),
            ends_at: "2026-05-04T13:30:00Z".into(),
        },
        &ids,
        &FixedClock,
    )
    .unwrap();

    // The appointment depends on the provisional completion operation.
    assert_eq!(created.depends_on, [patient_op.to_string()]);
    let stored: String = connection
        .query_row(
            "SELECT depends_on FROM appointment_local WHERE operation_id=?1",
            params![created.operation_id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(stored, format!("[\"{patient_op}\"]"));
}

#[test]
fn insufficient_coverage_rejects_without_side_effects() {
    let database = Database::new();
    let mut connection = crate::platform::sqlite::open(&database.0).unwrap();
    {
        let tx = connection.transaction().unwrap();
        use crate::components::identity::repository::AccessStore as _;
        crate::components::identity::repository::SqliteAccessStore
            .save_access(&tx, &access())
            .unwrap();
        tx.commit().unwrap();
    }
    seed_context(&connection);
    // A partial occupancy window: the request is only partly covered.
    connection
        .execute(
            "UPDATE sync_cursor SET window='{\"appointmentsFrom\":\"2026-05-04T00:00:00Z\",\"appointmentsUntil\":\"2026-05-04T13:15:00Z\"}'",
            [],
        )
        .unwrap();

    let err = create_via_repository(
        &mut connection,
        &Ids::default(),
        "2026-05-04T13:00:00Z",
        "2026-05-04T13:30:00Z",
    )
    .unwrap_err();
    assert_eq!(err.code, "INSUFFICIENT_COVERAGE");
    let count: i64 = connection
        .query_row("SELECT count(*) FROM appointment_local", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(count, 0);
    let count: i64 = connection
        .query_row(
            "SELECT count(*) FROM sync_outbox WHERE entity_type='appointment'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 0);
}

#[allow(dead_code)]
fn _use_provisional(_: Provisional<ProfileState>) {}
