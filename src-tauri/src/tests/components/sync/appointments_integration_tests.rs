//! Two-device loopback integration for appointments: offline booking,
//! reopen, concurrent conflict, lost response, partial acceptance and a
//! fresh-device snapshot rebuild. No HTTP request may hold the SQLite mutex.
use crate::components::sync::{
    coordinator::{CycleOutcome, CycleTrigger, SyncCoordinator, SyncEngine},
    remote::{ChangeGroup, ChangeKind, EntityType, SyncChange},
    snapshot::{SnapshotMode, SnapshotOutcome, SnapshotRecovery},
};
use crate::components::{
    appointments::{
        adapters::outbound::SqliteAppointmentRepository,
        application::{CreateAppointmentInput, ScheduleOccupancyQuery},
    },
    projector::DesktopProjector,
};
use crate::platform::{config::APIConfig, http::APITransport, sqlite::open, test_api::TestApi};
use crate::shared_kernel::{SystemClock, UuidV7Generator};
use crate::tests::components::{
    appointments::test_support::access as appointments_access,
    scheduling::test_support::seed_account,
    sync::download_tests::{Database, credential, scope},
};
use rusqlite::Connection;
use serde_json::{Value, json};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

const PROFESSIONAL: &str = "019a1234-0000-7000-8000-000000000008";
const SERVICE: &str = "019a1234-0000-7000-8000-0000000000b1";
const PATIENT: &str = "019a1234-0000-7000-8000-000000000050";
const IDENTIFIER: &str = "019a1234-0000-7000-8000-000000000051";
const RULE: &str = "019a1234-0000-7000-8000-000000000030";
const EXCEPTION: &str = "019a1234-0000-7000-8000-000000000031";
const APPOINTMENT_OTHER: &str = "019a1234-0000-7000-8000-0000000000b0";
const APPOINTMENT_B: &str = "019a1234-0000-7000-8000-0000000000a2";

struct FixedIds(std::cell::Cell<u64>);

impl crate::shared_kernel::IdGenerator for FixedIds {
    fn new_id(&self) -> uuid::Uuid {
        let n = self.0.get();
        self.0.set(n + 1);
        format!("019a1234-0000-7000-8000-{n:012x}").parse().unwrap()
    }
}

fn count(c: &Connection, table: &str) -> i64 {
    c.query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))
        .unwrap()
}

fn appointment_state(id: &str, version: i64, status: &str, starts: &str, ends: &str) -> Value {
    let s = scope();
    json!({
        "id": id, "accountId": s.account_id, "patientId": PATIENT,
        "professionalId": PROFESSIONAL, "serviceId": SERVICE,
        "startsAt": starts, "endsAt": ends,
        "status": status, "confirmedAt": null,
        "serviceNameSnapshot": "Consulta", "offeredDurationMinutesSnapshot": 30,
        "professionalNameSnapshot": "Profesional ficticio",
        "locationNameSnapshot": "Consultorio", "locationAddressSnapshot": "Calle 0",
        "locationCountrySnapshot": "VE", "locationStateSnapshot": "DC",
        "locationCitySnapshot": "Caracas", "timeZoneSnapshot": "America/Caracas",
        "responsibleContactId": null,
        "createdAt": null, "updatedAt": null, "version": version
    })
}

fn history_state(id: &str, version: i64, status: &str, starts: &str, ends: &str) -> Value {
    let s = scope();
    json!({
        "appointmentId": id, "accountId": s.account_id, "version": version,
        "startsAt": starts, "endsAt": ends,
        "items": [ {
            "appointmentId": id, "accountId": s.account_id, "version": 1,
            "action": "created", "actorKind": "staff", "actorUserId": s.user_id,
            "actorContactPersonId": null, "changeReason": null,
            "changedAt": "2026-10-01T10:00:00Z",
            "startsAt": starts, "endsAt": ends, "status": status, "confirmedAt": null,
            "serviceNameSnapshot": "Consulta", "offeredDurationMinutesSnapshot": 30,
            "professionalNameSnapshot": "Profesional ficticio",
            "locationNameSnapshot": "Consultorio", "locationAddressSnapshot": "Calle 0",
            "locationCountrySnapshot": "VE", "locationStateSnapshot": "DC",
            "locationCitySnapshot": "Caracas", "timeZoneSnapshot": "America/Caracas",
            "responsibleContactId": null
        } ]
    })
}

/// Accepted base so an offline booking can validate: account, professional,
/// catalog, offer, a complete patient with a declared document, and the
/// agenda of 2026-10-07 (replacement 09:00-10:00 local = 13:00-14:00Z).
fn base_changes() -> Vec<SyncChange> {
    let s = scope();
    vec![
        SyncChange {
            entity_type: EntityType::Account,
            entity_key: json!(s.account_id.as_str()),
            entity_version: 1,
            change_kind: ChangeKind::Upsert,
            state: json!({"id":s.account_id,"createdAt":null,"displayName":"Cuenta ficticia","kind":"independent","locationAddress":"Calle 0","locationCity":"Caracas","locationCountry":"VE","locationName":"Consultorio","locationState":"DC","noShowGraceMinutes":15,"ownerUserId":s.user_id,"status":"active","timeZone":"America/Caracas","updatedAt":null,"version":1}),
        },
        SyncChange {
            entity_type: EntityType::Professional,
            entity_key: json!(PROFESSIONAL),
            entity_version: 1,
            change_kind: ChangeKind::Upsert,
            state: json!({"id":PROFESSIONAL,"accountId":s.account_id,"userId":s.user_id,"publicName":"Profesional ficticio","status":"active","createdAt":null,"updatedAt":null,"version":1}),
        },
        SyncChange {
            entity_type: EntityType::Service,
            entity_key: json!(SERVICE),
            entity_version: 1,
            change_kind: ChangeKind::Upsert,
            state: json!({"id":SERVICE,"accountId":s.account_id,"professionalId":PROFESSIONAL,"name":"Consulta","defaultDurationMinutes":30,"status":"active","createdAt":null,"updatedAt":null,"version":1}),
        },
        SyncChange {
            entity_type: EntityType::ServiceOffer,
            entity_key: json!({"professionalId":PROFESSIONAL,"serviceId":SERVICE}),
            entity_version: 1,
            change_kind: ChangeKind::Upsert,
            state: json!({"professionalId":PROFESSIONAL,"serviceId":SERVICE,"accountId":s.account_id,"name":"Consulta","defaultDurationMinutes":30,"durationMinutesOverride":null,"effectiveDurationMinutes":30,"serviceStatus":"active","status":"active","version":1,"serviceVersion":1,"createdAt":null,"updatedAt":null}),
        },
        SyncChange {
            entity_type: EntityType::PatientIdentifier,
            entity_key: json!(IDENTIFIER),
            entity_version: 1,
            change_kind: ChangeKind::Upsert,
            state: json!({"id":IDENTIFIER,"patientId":PATIENT,"issuingCountry":"VE","documentType":"national_id","documentNumber":"V123456","verificationStatus":"declared","createdAt":null,"updatedAt":null,"version":1}),
        },
        SyncChange {
            entity_type: EntityType::PatientProfile,
            entity_key: json!(PATIENT),
            entity_version: 1,
            change_kind: ChangeKind::Upsert,
            state: json!({"accountId":s.account_id,"patientId":PATIENT,"fullName":"Paciente Sintético","birthDate":null,"phoneCode":"+58","phone":"4120000000","identificationBasis":"own_document","completionStatus":"complete","status":"active","createdAt":null,"updatedAt":null,"version":1}),
        },
        SyncChange {
            entity_type: EntityType::ScheduleRule,
            entity_key: json!(RULE),
            entity_version: 1,
            change_kind: ChangeKind::Upsert,
            state: json!({"accountId":s.account_id,"id":RULE,"professionalId":PROFESSIONAL,"isoWeekday":3,"localStartMinute":480,"localEndMinute":720,"validFrom":"2026-01-01","validUntil":null,"status":"active","createdAt":null,"updatedAt":null,"version":1}),
        },
        SyncChange {
            entity_type: EntityType::ScheduleException,
            entity_key: json!(EXCEPTION),
            entity_version: 1,
            change_kind: ChangeKind::Upsert,
            state: json!({"accountId":s.account_id,"id":EXCEPTION,"professionalId":PROFESSIONAL,"localDate":"2026-10-07","kind":"replacement","localStartMinute":540,"localEndMinute":600,"status":"active","createdAt":null,"updatedAt":null,"version":1}),
        },
    ]
}

fn group_of(changes: Vec<SyncChange>) -> ChangeGroup {
    ChangeGroup {
        group_id: uuid::Uuid::now_v7().to_string().try_into().unwrap(),
        operation_id: uuid::Uuid::now_v7().to_string().try_into().unwrap(),
        accepted_at: "2026-11-08T12:00:00Z".into(),
        changes,
    }
}

fn booking(starts: &str, ends: &str) -> CreateAppointmentInput {
    CreateAppointmentInput {
        patient_id: PATIENT.into(),
        professional_id: PROFESSIONAL.into(),
        service_id: SERVICE.into(),
        responsible_contact_id: None,
        starts_at: starts.into(),
        ends_at: ends.into(),
    }
}

#[test]
fn two_devices_booking_conflict_lost_response_and_snapshot_agree() {
    tauri::async_runtime::block_on(async {
        let device_a = Database::new();
        let access = appointments_access();
        {
            let mut c = device_a.data.lock().unwrap();
            seed_account(&mut c);
        }

        let stage = Arc::new(AtomicUsize::new(0));
        let server_stage = stage.clone();
        let data = device_a.data.clone();
        let allowed = access.allowed_actions.clone();
        let api = TestApi::dynamic(move |request| {
            assert!(data.try_lock().is_ok(), "HTTP must not hold SQLite");
            let stage = server_stage.load(Ordering::SeqCst);
            if request.contains("/access ") {
                return (
                    200,
                    json!({"accountId":scope().account_id,"userId":scope().user_id,"deviceId":scope().device_id,"permissionEpoch":1,"role":"assistant","status":"active","allowedActions":allowed}),
                );
            }
            if request.contains("/sync/operations ") {
                if stage == 2 {
                    // Lost response / outage: nothing is accepted remotely.
                    return (
                        503,
                        json!({"code":"retryable_failure","correlationId":null}),
                    );
                }
                let body: Value =
                    serde_json::from_str(request.split_once("\r\n\r\n").unwrap().1).unwrap();
                let results: Vec<_> = body["operations"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|operation| {
                        let action = operation["action"].as_str().unwrap();
                        // Stage 1: another device already booked the same
                        // slot, so this create conflicts with one shared effect.
                        let conflict = stage == 1
                            && action == "CreateAppointment"
                            && operation["payload"]["startsAt"].as_str()
                                == Some("2026-10-07T13:30:00Z");
                        json!({"operationId":operation["operationId"],"status":if conflict {"conflict"} else {"accepted"},"entityVersion":if conflict {Value::Null} else {json!(1)},"state":null,"error":if conflict {json!({"code":"SLOT_UNAVAILABLE","correlationId":"019a1234-0000-7000-8000-000000000099"})} else {Value::Null}})
                    })
                    .collect();
                return (
                    200,
                    json!({"correlationId":"019a1234-0000-7000-8000-000000000099","results":results}),
                );
            }
            if request.contains("/sync/operations/") && request.starts_with("GET ") {
                // Lost-response recovery: the original result is still there.
                let id = request
                    .split("/sync/operations/")
                    .nth(1)
                    .unwrap()
                    .split(' ')
                    .next()
                    .unwrap()
                    .to_string();
                return (
                    200,
                    json!({"operationId":id,"status":"accepted","entityVersion":1,"state":null,"error":null}),
                );
            }
            if request.contains("/sync/changes?") {
                let mut changes = base_changes();
                if stage >= 1 {
                    // The other device's accepted booking occupies the slot.
                    changes.push(SyncChange {
                        entity_type: EntityType::Appointment,
                        entity_key: json!(APPOINTMENT_OTHER),
                        entity_version: 1,
                        change_kind: ChangeKind::Upsert,
                        state: appointment_state(
                            APPOINTMENT_OTHER,
                            1,
                            "scheduled",
                            "2026-10-07T13:30:00Z",
                            "2026-10-07T14:00:00Z",
                        ),
                    });
                    changes.push(SyncChange {
                        entity_type: EntityType::AppointmentHistory,
                        entity_key: json!(APPOINTMENT_OTHER),
                        entity_version: 1,
                        change_kind: ChangeKind::Upsert,
                        state: history_state(
                            APPOINTMENT_OTHER,
                            1,
                            "scheduled",
                            "2026-10-07T13:30:00Z",
                            "2026-10-07T14:00:00Z",
                        ),
                    });
                }
                if stage >= 3 {
                    changes.push(SyncChange {
                        entity_type: EntityType::Appointment,
                        entity_key: json!(APPOINTMENT_B),
                        entity_version: 1,
                        change_kind: ChangeKind::Upsert,
                        state: appointment_state(
                            APPOINTMENT_B,
                            1,
                            "scheduled",
                            "2026-10-07T13:00:00Z",
                            "2026-10-07T13:30:00Z",
                        ),
                    });
                    changes.push(SyncChange {
                        entity_type: EntityType::AppointmentHistory,
                        entity_key: json!(APPOINTMENT_B),
                        entity_version: 1,
                        change_kind: ChangeKind::Upsert,
                        state: history_state(
                            APPOINTMENT_B,
                            1,
                            "scheduled",
                            "2026-10-07T13:00:00Z",
                            "2026-10-07T13:30:00Z",
                        ),
                    });
                }
                return (
                    200,
                    json!({"groups":[group_of(changes)],"nextCursor":format!("stage-{stage}"),"hasMore":false}),
                );
            }
            if request.starts_with("DELETE ") {
                return (204, Value::Null);
            }
            if request.contains("/items?") {
                let mut items: Vec<_> = base_changes()
                    .into_iter()
                    .map(|change| json!({"entityType":change.entity_type,"entityKey":change.entity_key,"entityVersion":change.entity_version,"state":change.state}))
                    .collect();
                if stage >= 1 {
                    for change in [
                        (
                            APPOINTMENT_OTHER,
                            "2026-10-07T13:30:00Z",
                            "2026-10-07T14:00:00Z",
                        ),
                        (
                            APPOINTMENT_B,
                            "2026-10-07T13:00:00Z",
                            "2026-10-07T13:30:00Z",
                        ),
                    ] {
                        items.push(json!({"entityType":"appointment","entityKey":change.0,"entityVersion":1,"state":appointment_state(change.0, 1, "scheduled", change.1, change.2)}));
                        items.push(json!({"entityType":"appointment_history","entityKey":change.0,"entityVersion":1,"state":history_state(change.0, 1, "scheduled", change.1, change.2)}));
                    }
                }
                return (200, json!({"items":items,"total":items.len()}));
            }
            assert!(request.contains("/sync/snapshots"));
            (
                200,
                json!({"snapshotId":"019a1234-0000-7000-8000-000000000300","accountId":scope().account_id,"contractVersion":1,"modelRevision":"api-v1.1","window":crate::tests::components::sync::download_tests::cursor("base").window,"baseCursor":"fresh-base","expiresAt":"2099-11-08T23:00:00Z","itemCount":12}),
            )
        });
        let config = APIConfig::parse(&api.url).unwrap();
        let http = APITransport::new(APIConfig::parse(&api.url).unwrap()).unwrap();
        let cred = credential(&config);
        let coordinator = SyncCoordinator::default();
        let engine = SyncEngine {
            coordinator: &coordinator,
            http: &http,
            database: &device_a.data,
        };

        // Stage 0: install the accepted base; no local operations yet.
        assert_eq!(
            engine
                .run(
                    &cred,
                    &scope(),
                    CycleTrigger::Manual,
                    &SystemClock,
                    &UuidV7Generator,
                    &DesktopProjector
                )
                .await
                .unwrap()
                .outcome,
            CycleOutcome::Complete
        );
        {
            let c = device_a.data.lock().unwrap();
            assert_eq!(count(&c, "appointment_view"), 0);
            assert_eq!(count(&c, "appointment_occupancy_view"), 0);
        }

        // Offline booking on the replacement 09:30-10:00 (13:30-14:00Z), then
        // reopen before any HTTP: the intention is durable.
        let created_a = {
            let mut c = device_a.data.lock().unwrap();
            SqliteAppointmentRepository { connection: &mut c }
                .create_appointment(
                    &access,
                    booking("2026-10-07T13:30:00Z", "2026-10-07T14:00:00Z"),
                    &FixedIds(std::cell::Cell::new(0xa0)),
                    &SystemClock,
                )
                .unwrap()
        };
        assert_eq!(
            created_a.entity_key.as_str().unwrap(),
            created_a.provisional["appointmentId"].as_str().unwrap()
        );
        {
            let c = open(&device_a.path).unwrap();
            assert_eq!(count(&c, "appointment_local"), 1);
            assert_eq!(count(&c, "sync_outbox"), 1);
        }

        // Stage 1: the server already accepted another device's booking for
        // the same slot. One shared effect wins; this device keeps its
        // conflict and the accepted view moves on.
        stage.store(1, Ordering::SeqCst);
        assert_eq!(
            engine
                .run(
                    &cred,
                    &scope(),
                    CycleTrigger::Manual,
                    &SystemClock,
                    &UuidV7Generator,
                    &DesktopProjector
                )
                .await
                .unwrap()
                .outcome,
            CycleOutcome::Complete
        );
        {
            let c = device_a.data.lock().unwrap();
            // Exactly one accepted appointment: the concurrent winner.
            assert_eq!(count(&c, "appointment_view"), 1);
            assert_eq!(count(&c, "appointment_history_view"), 1);
            let occupant: String = c
                .query_row(
                    "SELECT appointment_id FROM appointment_occupancy_view",
                    [],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(occupant, APPOINTMENT_OTHER);
            // The conflicted intention survives for review.
            assert_eq!(count(&c, "appointment_local"), 1);
            let status: String = c
                .query_row(
                    "SELECT status FROM sync_outbox WHERE entity_type='appointment'",
                    [],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(status, "conflict");
        }

        // Stage 2: lost response / outage with a pending operation: nothing
        // is accepted and the outcome is not complete.
        let created_b = {
            let mut c = device_a.data.lock().unwrap();
            SqliteAppointmentRepository { connection: &mut c }
                .create_appointment(
                    &access,
                    booking("2026-10-07T13:00:00Z", "2026-10-07T13:30:00Z"),
                    &FixedIds(std::cell::Cell::new(0xa2)),
                    &SystemClock,
                )
                .unwrap()
        };
        assert_eq!(created_b.entity_key.as_str().unwrap(), APPOINTMENT_B);
        stage.store(2, Ordering::SeqCst);
        assert_ne!(
            engine
                .run(
                    &cred,
                    &scope(),
                    CycleTrigger::Manual,
                    &SystemClock,
                    &UuidV7Generator,
                    &DesktopProjector
                )
                .await
                .unwrap()
                .outcome,
            CycleOutcome::Complete
        );
        {
            let c = device_a.data.lock().unwrap();
            // The lost response leaves the operation unknown, not accepted.
            let status: String = c
                .query_row(
                    "SELECT status FROM sync_outbox WHERE entity_key=?1",
                    [format!("\"{APPOINTMENT_B}\"")],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(status, "unknown", "lost response keeps the result unknown");
            assert_eq!(count(&c, "appointment_local"), 2, "outage keeps intentions");
            // The concurrent winner (accepted view) plus the new provisional
            // (through the occupancy port) both block slots locally.
            assert_eq!(count(&c, "appointment_occupancy_view"), 1);
            let known =
                crate::components::appointments::adapters::outbound::SqliteScheduleOccupancyQuery {
                    connection: &c,
                    scope: &scope(),
                }
                .list_occupancy(
                    PROFESSIONAL,
                    "2026-10-07T00:00:00Z".parse().unwrap(),
                    "2026-10-08T00:00:00Z".parse().unwrap(),
                )
                .unwrap();
            assert_eq!(known.len(), 2);
        }

        // Stage 3: the server accepts the second booking and downloads both
        // appointment and history; the provisional resolves only then.
        stage.store(3, Ordering::SeqCst);
        assert_eq!(
            engine
                .run(
                    &cred,
                    &scope(),
                    CycleTrigger::Manual,
                    &SystemClock,
                    &UuidV7Generator,
                    &DesktopProjector
                )
                .await
                .unwrap()
                .outcome,
            CycleOutcome::Complete
        );
        {
            let c = device_a.data.lock().unwrap();
            assert_eq!(count(&c, "appointment_view"), 2);
            assert_eq!(count(&c, "appointment_history_view"), 2);
            assert_eq!(count(&c, "appointment_occupancy_view"), 2);
            assert_eq!(
                count(&c, "appointment_local"),
                2,
                "intentions are never deleted"
            );
            // Both devices' accepted rows are present; no duplicated effect.
            let versions: i64 = c
                .query_row(
                    "SELECT count(*) FROM appointment_view WHERE appointment_id=?1 AND version=1",
                    [APPOINTMENT_B],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(versions, 1);
        }

        // Device B installs a fresh snapshot and sees the same shared state.
        let device_b = Database::new();
        {
            let mut c = device_b.data.lock().unwrap();
            seed_account(&mut c);
        }
        let result = SnapshotRecovery {
            http: &http,
            database: &device_b.data,
        }
        .rebuild(
            &cred,
            &scope(),
            &SystemClock,
            SnapshotMode::Restart,
            &DesktopProjector,
        )
        .await
        .unwrap();
        assert!(matches!(
            result,
            SnapshotOutcome::Installed {
                cleanup_pending: false
            }
        ));
        {
            let c = device_b.data.lock().unwrap();
            assert_eq!(
                count(&c, "appointment_local"),
                0,
                "fresh device has no intentions"
            );
            assert_eq!(count(&c, "appointment_view"), 2);
            assert_eq!(count(&c, "appointment_history_view"), 2);
            assert_eq!(count(&c, "appointment_occupancy_view"), 2);
        }
        assert_eq!(device_b.cursor().cursor, "fresh-base");
    });
}
