//! Two-device loopback integration: offline writes, reopen, dispatch,
//! acceptance, projection, conflict, API outage and snapshot rebuild.
//! No HTTP request may hold the SQLite mutex (asserted inside the server).
use crate::components::sync::{
    coordinator::{CycleOutcome, CycleTrigger, SyncCoordinator, SyncEngine},
    remote::{ChangeGroup, ChangeKind, EntityType, SyncChange},
    snapshot::{SnapshotMode, SnapshotOutcome, SnapshotRecovery},
};
use crate::components::{
    appointments::{
        adapters::outbound::SqliteScheduleOccupancyQuery, application::ScheduleOccupancyQuery,
    },
    projector::DesktopProjector,
    scheduling::{
        adapters::outbound::SqliteSchedulingRepository,
        application::{
            AvailabilityRequest, EditBase, ScheduleAvailabilityQuery, SchedulingMutation,
            SchedulingRepository,
        },
        domain::{BlockValue, ExceptionKind, ExceptionValue, Instant, ReasonCode, RuleValue},
    },
};
use crate::platform::{config::APIConfig, http::APITransport, sqlite::open, test_api::TestApi};
use crate::shared_kernel::{SystemClock, UuidV7Generator};
use crate::tests::components::{
    scheduling::test_support::{access, seed_account},
    sync::download_tests::{Database, credential, cursor, scope},
};
use rusqlite::Connection;
use serde_json::{Value, json};
use std::str::FromStr;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

const PROFESSIONAL: &str = "019a1234-0000-7000-8000-000000000008";
const SERVICE: &str = "019a1234-0000-7000-8000-0000000000b1";
const RULE: &str = "019a1234-0000-7000-8000-000000000030";
const EXCEPTION: &str = "019a1234-0000-7000-8000-000000000031";
const BLOCK: &str = "019a1234-0000-7000-8000-000000000032";
const APPOINTMENT: &str = "019a1234-0000-7000-8000-000000000040";

fn count(c: &Connection, table: &str) -> i64 {
    c.query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))
        .unwrap()
}

/// Accepted remote agenda plus one occupying appointment. Rule version is
/// parameterised so a second device can publish a newer accepted version.
fn agenda_changes(rule_version: i64, rule_start: i64) -> Vec<SyncChange> {
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
            entity_type: EntityType::ScheduleRule,
            entity_key: json!(RULE),
            entity_version: rule_version,
            change_kind: ChangeKind::Upsert,
            state: json!({"accountId":s.account_id,"id":RULE,"professionalId":PROFESSIONAL,"isoWeekday":3,"localStartMinute":rule_start,"localEndMinute":720,"validFrom":"2026-01-01","validUntil":null,"status":"active","createdAt":null,"updatedAt":null,"version":rule_version}),
        },
        SyncChange {
            entity_type: EntityType::ScheduleException,
            entity_key: json!(EXCEPTION),
            entity_version: 1,
            change_kind: ChangeKind::Upsert,
            state: json!({"accountId":s.account_id,"id":EXCEPTION,"professionalId":PROFESSIONAL,"localDate":"2026-10-07","kind":"replacement","localStartMinute":540,"localEndMinute":600,"status":"active","createdAt":null,"updatedAt":null,"version":1}),
        },
        SyncChange {
            entity_type: EntityType::ScheduleBlock,
            entity_key: json!(BLOCK),
            entity_version: 1,
            change_kind: ChangeKind::Upsert,
            state: json!({"accountId":s.account_id,"id":BLOCK,"professionalId":PROFESSIONAL,"startsAt":"2026-04-01T13:00:00Z","endsAt":"2026-04-01T17:00:00Z","reasonCode":"administrative","status":"active","createdAt":null,"updatedAt":null,"version":1}),
        },
        SyncChange {
            entity_type: EntityType::Appointment,
            entity_key: json!(APPOINTMENT),
            entity_version: 1,
            change_kind: ChangeKind::Upsert,
            state: json!({"id":APPOINTMENT,"accountId":s.account_id,"patientId":"019a1234-0000-7000-8000-000000000050","professionalId":PROFESSIONAL,"serviceId":SERVICE,"startsAt":"2026-10-07T13:30:00Z","endsAt":"2026-10-07T14:00:00Z","status":"scheduled","confirmedAt":null,"serviceNameSnapshot":"Consulta","offeredDurationMinutesSnapshot":30,"professionalNameSnapshot":"Profesional ficticio","locationNameSnapshot":"Consultorio","locationAddressSnapshot":"Calle 0","locationCountrySnapshot":"VE","locationStateSnapshot":"DC","locationCitySnapshot":"Caracas","timeZoneSnapshot":"America/Caracas","responsibleContactId":null,"createdAt":null,"updatedAt":null,"version":1}),
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

fn availability(c: &mut Connection) -> (String, Vec<(String, String)>) {
    let access = access();
    let request = AvailabilityRequest {
        professional_id: PROFESSIONAL.into(),
        service_id: SERVICE.into(),
        from: jiff::Timestamp::from_str("2026-10-06T04:00:00Z").unwrap(),
        until: jiff::Timestamp::from_str("2026-10-09T04:00:00Z").unwrap(),
        duration_minutes: None,
        exclude_appointment_id: None,
    };
    let known: Vec<_> = SqliteScheduleOccupancyQuery {
        connection: &*c,
        scope: &access.context,
    }
    .list_occupancy(PROFESSIONAL, request.from, request.until)
    .unwrap()
    .into_iter()
    .map(|interval| (PROFESSIONAL.to_owned(), interval))
    .collect();
    let evaluation = SqliteSchedulingRepository { connection: c }
        .local_availability(
            &access,
            &request,
            &crate::components::scheduling::adapters::outbound::KnownOccupancy(known),
            &crate::components::sync::adapters::control::SqliteSyncControlStore,
        )
        .unwrap();
    (
        evaluation.coverage,
        evaluation
            .intervals
            .into_iter()
            .map(|interval| (interval.starts_at, interval.ends_at))
            .collect(),
    )
}

#[test]
fn two_devices_offline_dispatch_accept_conflict_outage_and_snapshot() {
    tauri::async_runtime::block_on(async {
        let device_a = Database::new();
        let access = access();
        let _created = {
            let mut c = device_a.data.lock().unwrap();
            seed_account(&mut c);
            let ids = &UuidV7Generator;
            let mut repo = SqliteSchedulingRepository { connection: &mut c };
            let rule = repo
                .persist(
                    &access,
                    SchedulingMutation::CreateRule {
                        value: RuleValue::new(
                            PROFESSIONAL.to_owned().try_into().unwrap(),
                            3,
                            480,
                            720,
                            crate::components::scheduling::domain::parse_local_date("2026-01-01")
                                .unwrap(),
                            None,
                        )
                        .unwrap(),
                    },
                    ids,
                    &SystemClock,
                    &crate::components::scheduling::adapters::outbound::KnownOccupancy(vec![]),
                )
                .unwrap();
            repo.persist(
                &access,
                SchedulingMutation::CreateException {
                    value: ExceptionValue::new(
                        PROFESSIONAL.to_owned().try_into().unwrap(),
                        crate::components::scheduling::domain::parse_local_date("2026-10-07")
                            .unwrap(),
                        ExceptionKind::Replacement,
                        Some(540),
                        Some(600),
                    )
                    .unwrap(),
                },
                ids,
                &SystemClock,
                &crate::components::scheduling::adapters::outbound::KnownOccupancy(vec![]),
            )
            .unwrap();
            repo.persist(
                &access,
                SchedulingMutation::CreateBlock {
                    value: BlockValue::new(
                        PROFESSIONAL.to_owned().try_into().unwrap(),
                        Some(
                            "019a1234-0000-7000-8000-000000000001"
                                .to_owned()
                                .try_into()
                                .unwrap(),
                        ),
                        Instant::parse("2026-04-01T13:00:00Z").unwrap(),
                        Instant::parse("2026-04-01T17:00:00Z").unwrap(),
                        ReasonCode::Administrative,
                    )
                    .unwrap(),
                },
                ids,
                &SystemClock,
                &crate::components::scheduling::adapters::outbound::KnownOccupancy(vec![]),
            )
            .unwrap();
            rule
        };

        // Reopen antes de HTTP: intenciones y provisionales durables.
        {
            let c = open(&device_a.path).unwrap();
            assert_eq!(count(&c, "sync_outbox"), 3);
            assert_eq!(count(&c, "schedule_local"), 3);
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
                        // The server already accepted a newer rule version in
                        // stage 1, so the expected-version update conflicts.
                        let conflict = stage == 1 && action == "UpdateScheduleRule";
                        json!({"operationId":operation["operationId"],"status":if conflict {"conflict"} else {"accepted"},"entityVersion":if conflict {Value::Null} else {json!(1)},"state":null,"error":if conflict {json!({"code":"VERSION_CONFLICT","correlationId":"019a1234-0000-7000-8000-000000000099"})} else {Value::Null}})
                    })
                    .collect();
                return (
                    200,
                    json!({"correlationId":"019a1234-0000-7000-8000-000000000099","results":results}),
                );
            }
            if request.contains("/sync/changes?") {
                let changes = match stage {
                    0 => agenda_changes(1, 540),
                    1 => agenda_changes(2, 600),
                    _ => vec![],
                };
                return (
                    200,
                    json!({"groups":[group_of(changes)],"nextCursor":format!("stage-{stage}"),"hasMore":false}),
                );
            }
            if request.starts_with("DELETE ") {
                return (204, Value::Null);
            }
            if request.contains("/items?") {
                let items: Vec<_> = agenda_changes(2, 600)
                    .into_iter()
                    .map(|change| json!({"entityType":change.entity_type,"entityKey":change.entity_key,"entityVersion":change.entity_version,"state":change.state}))
                    .collect();
                return (200, json!({"items":items,"total":items.len()}));
            }
            assert!(request.contains("/sync/snapshots"));
            (
                200,
                json!({"snapshotId":"019a1234-0000-7000-8000-000000000300","accountId":scope().account_id,"contractVersion":1,"modelRevision":"api-v1.1","window":cursor("base").window,"baseCursor":"fresh-base","expiresAt":"2099-11-08T23:00:00Z","itemCount":8}),
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

        // Dispatch + download: acceptance retires provisionals; views project;
        // occupancy arrives; intentions stay.
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
        let availability_a = {
            let mut c = device_a.data.lock().unwrap();
            assert_eq!(count(&c, "schedule_local"), 3);
            assert_eq!(count(&c, "sync_outbox"), 3);
            assert_eq!(count(&c, "appointment_occupancy_view"), 1);
            let row = SqliteSchedulingRepository { connection: &mut c }
                .rule_row(&access, RULE)
                .unwrap();
            assert!(
                row.provisional.is_none(),
                "accepted + downloaded retires provisional"
            );
            assert_eq!(row.accepted.as_ref().unwrap().version, 1);
            availability(&mut c)
        };
        assert_eq!(availability_a.0, "within_synced_window");
        assert_eq!(
            availability_a.1,
            [(
                "2026-10-07T13:00:00Z".to_string(),
                "2026-10-07T13:30:00Z".to_string()
            )],
            "replacement 09:00-10:00 minus the known appointment 09:30-10:00 in Caracas"
        );

        // Conflict from a newer accepted version: intention kept, view moves on.
        let _conflict_op = {
            let mut c = device_a.data.lock().unwrap();
            SqliteSchedulingRepository { connection: &mut c }
                .persist(
                    &access,
                    SchedulingMutation::UpdateRule {
                        rule_id: RULE.into(),
                        base: EditBase::Accepted { version: 1 },
                        patch: crate::components::scheduling::domain::RulePatch::new(
                            None,
                            Some(660),
                            None,
                            None,
                            None,
                        )
                        .unwrap(),
                    },
                    &UuidV7Generator,
                    &SystemClock,
                    &crate::components::scheduling::adapters::outbound::KnownOccupancy(vec![]),
                )
                .unwrap()
                .operation_id
        };
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
            let mut c = device_a.data.lock().unwrap();
            let row = SqliteSchedulingRepository { connection: &mut c }
                .rule_row(&access, RULE)
                .unwrap();
            assert!(row.provisional.is_some(), "conflict keeps the provisional");
            assert!(row.operations.iter().any(|op| op.status == "conflict"));
            assert_eq!(row.accepted.as_ref().unwrap().version, 2);
            assert_eq!(count(&c, "schedule_local"), 4);
        }

        // API down with a pending operation: the cycle fails safe and the
        // local case keeps working.
        {
            let mut c = device_a.data.lock().unwrap();
            SqliteSchedulingRepository { connection: &mut c }
                .persist(
                    &access,
                    SchedulingMutation::SetExceptionStatus {
                        exception_id: EXCEPTION.into(),
                        base: EditBase::Accepted { version: 1 },
                        status: crate::components::scheduling::domain::Status::Inactive,
                    },
                    &UuidV7Generator,
                    &SystemClock,
                    &crate::components::scheduling::adapters::outbound::KnownOccupancy(vec![]),
                )
                .unwrap();
        }
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
            let mut c = device_a.data.lock().unwrap();
            assert_eq!(count(&c, "schedule_local"), 5, "outage keeps intentions");
            let before = count(&c, "schedule_local");
            // The conflicted rule blocks dependents; the block has clean deps.
            SqliteSchedulingRepository { connection: &mut c }
                .persist(
                    &access,
                    SchedulingMutation::SetBlockStatus {
                        block_id: BLOCK.into(),
                        base: EditBase::Accepted { version: 1 },
                        status: crate::components::scheduling::domain::Status::Inactive,
                    },
                    &UuidV7Generator,
                    &SystemClock,
                    &crate::components::scheduling::adapters::outbound::KnownOccupancy(vec![]),
                )
                .unwrap();
            assert_eq!(
                count(&c, "schedule_local"),
                before + 1,
                "local case works while the API is down"
            );
        }

        // Device B installs a fresh snapshot and computes the same availability.
        let device_b = Database::new();
        {
            let mut c = device_b.data.lock().unwrap();
            seed_account(&mut c);
        }
        stage.store(3, Ordering::SeqCst);
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
            let mut c = device_b.data.lock().unwrap();
            assert_eq!(
                count(&c, "schedule_local"),
                0,
                "fresh device has no intentions"
            );
            assert_eq!(count(&c, "appointment_occupancy_view"), 1);
            let availability_b = availability(&mut c);
            assert_eq!(availability_b.0, "within_synced_window");
            assert_eq!(
                availability_a.1, availability_b.1,
                "both devices agree inside the window"
            );
        }
        assert_eq!(device_b.cursor().cursor, "fresh-base");
    });
}
