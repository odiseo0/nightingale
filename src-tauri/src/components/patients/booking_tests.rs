//! PatientBookingQuery: booking views over accepted and chainable provisional data.
use super::{
    adapters::outbound::SqlitePatientRepository,
    application::{BookingReference, PatientBookingQuery, PatientRepository},
    domain::PatientId,
    test_support,
};
use rusqlite::params;

const PATIENT: &str = "019a1234-0000-7000-8000-000000000010";
const CONTACT: &str = "019a1234-0000-7000-8000-000000000020";

fn scope() -> crate::shared_kernel::id::OperationContext {
    test_support::access().context
}

fn insert_accepted_profile(connection: &rusqlite::Connection, basis: Option<&str>) {
    connection
        .execute(
            "INSERT INTO patient_profile_view (account_id,user_id,device_id,patient_id,full_name,birth_date,phone_code,phone,identification_basis,completion_status,status,version) VALUES (?1,?2,?3,?4,'Paciente Sintético','1990-06-15','58','4120000000',?5,'complete','active',2)",
            params![
                scope().account_id.as_str(),
                scope().user_id.as_str(),
                scope().device_id.as_str(),
                PATIENT,
                basis
            ],
        )
        .unwrap();
}

fn insert_accepted_identifier(connection: &rusqlite::Connection, verification: &str) {
    connection
        .execute(
            "INSERT INTO patient_identifier_view (account_id,user_id,device_id,identifier_id,patient_id,issuing_country,document_type,document_number,verification_status,version) VALUES (?1,?2,?3,'019a1234-0000-7000-8000-000000000011',?4,'VE','national_id','V123456','declared',1)",
            params![
                scope().account_id.as_str(),
                scope().user_id.as_str(),
                scope().device_id.as_str(),
                PATIENT,
            ],
        )
        .unwrap();
    connection
        .execute(
            "UPDATE patient_identifier_view SET verification_status=?1 WHERE identifier_id='019a1234-0000-7000-8000-000000000011'",
            params![verification],
        )
        .unwrap();
}

fn insert_accepted_responsible(connection: &rusqlite::Connection) {
    connection
        .execute(
            "INSERT INTO contact_person_view (account_id,user_id,device_id,contact_person_id,full_name,phone_code,phone,status,version) VALUES (?1,?2,?3,?4,'Responsable Sintético','58','4140000000','active',1)",
            params![
                scope().account_id.as_str(),
                scope().user_id.as_str(),
                scope().device_id.as_str(),
                CONTACT
            ],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO patient_contact_view (account_id,user_id,device_id,patient_id,contact_person_id,relationship_kind,is_responsible_contact,representation_status,valid_from,status,version) VALUES (?1,?2,?3,?4,?5,'mother',1,'unverified','2020-01-01','active',1)",
            params![
                scope().account_id.as_str(),
                scope().user_id.as_str(),
                scope().device_id.as_str(),
                PATIENT,
                CONTACT
            ],
        )
        .unwrap();
}

fn booking(
    connection: &mut rusqlite::Connection,
    patient: &PatientId,
) -> Option<super::application::BookingPatient> {
    let access = test_support::access();
    let mut repository = SqlitePatientRepository { connection };
    repository.booking_patient(&access, patient).unwrap()
}

fn patient_id() -> PatientId {
    PATIENT.to_owned().try_into().unwrap()
}

#[test]
fn accepted_patient_resolves_with_version_and_responsible_contact() {
    let mut connection = test_support::database();
    insert_accepted_profile(&connection, Some("own_document"));
    insert_accepted_identifier(&connection, "declared");
    insert_accepted_responsible(&connection);

    let view = booking(&mut connection, &patient_id()).unwrap();
    assert_eq!(view.reference, BookingReference::Accepted { version: 2 });
    assert!(view.profile.accepted.is_some());
    assert!(view.profile.provisional.is_none());
    assert_eq!(view.identifiers.len(), 1);
    assert_eq!(
        view.identifiers[0]
            .accepted
            .as_ref()
            .unwrap()
            .state
            .verification_status,
        "declared"
    );
    // Only the responsible relationship appears, with the contact's current phone.
    assert_eq!(view.responsible_contacts.len(), 1);
    let relationship = &view.responsible_contacts[0]
        .accepted
        .as_ref()
        .unwrap()
        .state;
    assert!(relationship.relationship.is_responsible_contact);
    assert_eq!(relationship.contact_phone.as_deref(), Some("4140000000"));

    // Paciente inexistente: sin vista.
    let missing: PatientId = "019a1234-0000-7000-8000-000000000099"
        .to_owned()
        .try_into()
        .unwrap();
    assert!(booking(&mut connection, &missing).is_none());
}

#[test]
fn provisional_patient_resolves_with_operation_reference() {
    let mut connection = test_support::database();
    let access = test_support::access();
    let ids = test_support::Ids::default();
    let created = test_support::run(&mut connection, &access, &ids, |cases| {
        cases
            .repository
            .persist(
                &access,
                super::application::PatientMutation::Register {
                    profile: super::domain::ProfileData {
                        full_name: Some("Menor Sintético".into()),
                        birth_date: Some("2015-03-10".into()),
                        phone: None,
                        identification_basis: Some(
                            super::domain::IdentificationBasis::MinorWithoutDocument,
                        ),
                    },
                    identifiers: vec![],
                },
                &ids,
                &test_support::FixedClock,
            )
            .unwrap()
    });

    let patient: PatientId = created
        .entity_key
        .as_str()
        .unwrap()
        .to_owned()
        .try_into()
        .unwrap();
    let view = booking(&mut connection, &patient).unwrap();
    let BookingReference::Provisional {
        operation_id,
        expected_version,
    } = view.reference
    else {
        panic!("se esperaba una referencia provisional");
    };
    assert_eq!(operation_id, created.operation_id);
    assert_eq!(expected_version, 0);
    assert!(view.profile.provisional.is_some());
    assert_eq!(view.profile.operations.len(), 1);
    assert_eq!(view.profile.operations[0].status, "pending");
}

#[test]
fn conflicted_provisional_is_excluded_from_booking() {
    let mut connection = test_support::database();
    let access = test_support::access();
    let ids = test_support::Ids::default();
    let created = test_support::run(&mut connection, &access, &ids, |cases| {
        cases
            .repository
            .persist(
                &access,
                super::application::PatientMutation::Register {
                    profile: super::domain::ProfileData {
                        full_name: Some("Paciente en conflicto".into()),
                        birth_date: None,
                        phone: None,
                        identification_basis: None,
                    },
                    identifiers: vec![],
                },
                &ids,
                &test_support::FixedClock,
            )
            .unwrap()
    });

    // A dispatch cycle returned conflict: the reference stops being chainable.
    connection
        .execute(
            "UPDATE sync_outbox SET status='conflict', last_error='\"VERSION_CONFLICT\"' WHERE operation_id=?1",
            params![created.operation_id],
        )
        .unwrap();

    let patient: PatientId = created
        .entity_key
        .as_str()
        .unwrap()
        .to_owned()
        .try_into()
        .unwrap();
    assert!(booking(&mut connection, &patient).is_none());
}
