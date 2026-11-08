use rusqlite::{Connection, Transaction, TransactionBehavior};
use std::{error::Error, io, path::Path};

pub const APPLICATION_ID: i64 = 1_313_297_480;

#[derive(Debug, Clone, Copy)]
struct Migration<'a> {
    version: i64,
    name: &'a str,
    sql: &'a str,
}

// Ordered and embedded. Each SQL file must set `PRAGMA user_version` to its own
// version; the runner verifies it so a file that forgets the pragma is rejected.
// Plans reserve their number in their todo.md before adding an entry here.
const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        name: "0001_schema",
        sql: include_str!("../../../migrations/0001_schema.sql"),
    },
    Migration {
        version: 2,
        name: "0002_sync_outbox",
        sql: include_str!("../../../migrations/0002_sync_outbox.sql"),
    },
    Migration {
        version: 3,
        name: "0003_identity_access",
        sql: include_str!("../../../migrations/0003_identity_access.sql"),
    },
    Migration {
        version: 4,
        name: "0004_sync_control",
        sql: include_str!("../../../migrations/0004_sync_control.sql"),
    },
    Migration {
        version: 5,
        name: "0005_account",
        sql: include_str!("../../../migrations/0005_account.sql"),
    },
    Migration {
        version: 6,
        name: "0006_services",
        sql: include_str!("../../../migrations/0006_services.sql"),
    },
    Migration {
        version: 7,
        name: "0007_patients",
        sql: include_str!("../../../migrations/0007_patients.sql"),
    },
    Migration {
        version: 8,
        name: "0008_scheduling",
        sql: include_str!("../../../migrations/0008_scheduling.sql"),
    },
    Migration {
        version: 9,
        name: "0009_appointment_occupancy",
        sql: include_str!("../../../migrations/0009_appointment_occupancy.sql"),
    },
    Migration {
        version: 10,
        name: "0010_appointments",
        sql: include_str!("../../../migrations/0010_appointments.sql"),
    },
];

type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub fn open(path: &Path) -> Result<Connection> {
    let mut connection = Connection::open(path)?;
    migrate(&mut connection, MIGRATIONS)?;
    connection.pragma_update(None, "foreign_keys", true)?;
    // A prior process can stop after sending but before saving the result.
    connection.execute(
        "UPDATE sync_outbox SET status = 'unknown', lease_until = NULL WHERE status = 'in_flight'",
        [],
    )?;

    Ok(connection)
}

/// Runs `f` inside a single SQLite transaction: commits on `Ok`, rolls back on `Err`.
/// Business changes and their outbox record must share one call so both land or neither
/// does (Master Plan §9.11).
pub fn in_transaction<T>(
    connection: &mut Connection,
    f: impl FnOnce(&Transaction) -> rusqlite::Result<T>,
) -> rusqlite::Result<T> {
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let output = f(&transaction)?;
    transaction.commit()?;

    Ok(output)
}

fn migrate(connection: &mut Connection, migrations: &[Migration<'_>]) -> Result<()> {
    // Lock before checking the version so two starts cannot both migrate.
    // https://docs.rs/rusqlite/0.40.2/rusqlite/struct.Connection.html#method.transaction_with_behavior
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let version: i64 = transaction.pragma_query_value(None, "user_version", |row| row.get(0))?;
    let application: i64 =
        transaction.pragma_query_value(None, "application_id", |row| row.get(0))?;

    let pending: &[Migration] = match (version, application) {
        (0, 0) => {
            let objects: i64 = transaction.query_row(
                "SELECT count(*) FROM sqlite_schema WHERE name NOT GLOB 'sqlite_*'",
                [],
                |row| row.get(0),
            )?;

            if objects != 0 {
                return Err(
                    io::Error::new(io::ErrorKind::InvalidData, "unrecognized database").into(),
                );
            }

            migrations
        }

        (version, APPLICATION_ID) if version >= 1 && (version as usize) <= migrations.len() => {
            let applied: Vec<(i64, String)> = transaction
                .prepare("SELECT version, name FROM schema_migrations ORDER BY version")?
                .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
                .collect::<rusqlite::Result<_>>()?;
            let expected: Vec<(i64, String)> = migrations[..version as usize]
                .iter()
                .map(|migration| (migration.version, migration.name.to_owned()))
                .collect();

            if applied != expected {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid migration history",
                )
                .into());
            }

            &migrations[version as usize..]
        }
        _ => {
            return Err(
                io::Error::new(io::ErrorKind::InvalidData, "unsupported database schema").into(),
            );
        }
    };

    for migration in pending {
        transaction.execute_batch(migration.sql)?;

        let recorded: i64 =
            transaction.pragma_query_value(None, "user_version", |row| row.get(0))?;

        if recorded != migration.version {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "migration {} did not set user_version to {}",
                    migration.name, migration.version
                ),
            )
            .into());
        }
    }

    transaction.commit()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    static NEXT: AtomicU64 = AtomicU64::new(0);

    struct Fixture(PathBuf);

    impl Fixture {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "nightingale-sqlite-{}-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&path).unwrap();

            Self(path)
        }

        fn database(&self) -> PathBuf {
            self.0.join("test.sqlite")
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }

    const SCHEMA: &str = include_str!("../../../migrations/0001_schema.sql");

    const TEST_MIGRATION: Migration<'static> = Migration {
        version: 2,
        name: "0002_test_table",
        sql: "CREATE TABLE test_table (id INTEGER PRIMARY KEY);\nINSERT INTO schema_migrations (version, name) VALUES (2, '0002_test_table');\nPRAGMA user_version = 2;",
    };

    fn test_migrations() -> Vec<Migration<'static>> {
        vec![
            Migration {
                version: 1,
                name: "0001_schema",
                sql: SCHEMA,
            },
            TEST_MIGRATION,
        ]
    }

    #[test]
    fn creates_and_reopens_without_repeating_migration() {
        let fixture = Fixture::new();
        let path = fixture.database();
        let connection = open(&path).unwrap();
        let version: i64 = connection
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        assert_eq!(version, MIGRATIONS.last().unwrap().version);
        let foreign_keys: bool = connection
            .pragma_query_value(None, "foreign_keys", |row| row.get(0))
            .unwrap();
        assert!(foreign_keys);
        drop(connection);
        let before = fs::read(&path).unwrap();
        let connection = open(&path).unwrap();
        assert_eq!(connection.total_changes(), 0);
        let count: i64 = connection
            .query_row("SELECT count(*) FROM schema_migrations", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(count, MIGRATIONS.len() as i64);
        drop(connection);
        assert_eq!(fs::read(path).unwrap(), before);
    }

    #[test]
    fn applies_pending_migrations_in_order() {
        let fixture = Fixture::new();
        let mut connection = Connection::open(fixture.database()).unwrap();
        migrate(&mut connection, &test_migrations()[..1]).unwrap();
        migrate(&mut connection, &test_migrations()).unwrap();
        migrate(&mut connection, &test_migrations()).unwrap();

        let version: i64 = connection
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        assert_eq!(version, 2);
        let tables: i64 = connection
            .query_row(
                "SELECT count(*) FROM sqlite_schema WHERE name = 'test_table'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(tables, 1);
    }

    fn assert_account_services_schema(connection: &Connection) {
        assert_eq!(
            connection
                .pragma_query_value::<i64, _>(None, "user_version", |r| r.get(0))
                .unwrap(),
            MIGRATIONS.last().unwrap().version
        );
        for table in [
            "account_view",
            "account_preferences",
            "service_catalog_view",
            "service_offer_view",
            "professional_view",
            "service_local",
            "service_offer_local",
        ] {
            assert!(connection.table_exists(None, table).unwrap(), "{table}");
        }
        let history: Vec<(i64, String)> = connection
            .prepare("SELECT version, name FROM schema_migrations ORDER BY version")
            .unwrap()
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap();
        assert_eq!(
            history,
            MIGRATIONS
                .iter()
                .map(|m| (m.version, m.name.to_owned()))
                .collect::<Vec<_>>()
        );
        assert_eq!(
            connection
                .query_row("PRAGMA integrity_check", [], |r| r.get::<_, String>(0))
                .unwrap(),
            "ok"
        );
    }

    #[test]
    fn account_services_open_new_database_and_upgrade_schema_four() {
        for prior_version in [0, 4] {
            let fixture = Fixture::new();
            let path = fixture.database();
            let mut connection = Connection::open(&path).unwrap();
            if prior_version == 4 {
                migrate(&mut connection, &MIGRATIONS[..4]).unwrap();
                connection.execute("INSERT INTO identity_context VALUES (1, 'synthetic-issuer', 'synthetic-subject', 'synthetic-user', 'synthetic-account')", []).unwrap();
            }
            drop(connection);
            let connection = open(&path).unwrap();
            assert_account_services_schema(&connection);
            if prior_version == 4 {
                assert_eq!(
                    connection
                        .query_row("SELECT subject FROM identity_context", [], |r| r
                            .get::<_, String>(0))
                        .unwrap(),
                    "synthetic-subject"
                );
            }
            drop(connection);
            let bytes = fs::read(&path).unwrap();
            let connection = open(&path).unwrap();
            assert_account_services_schema(&connection);
            assert_eq!(connection.total_changes(), 0);
            drop(connection);
            assert_eq!(fs::read(&path).unwrap(), bytes);
        }
    }

    #[test]
    fn failed_services_migration_rolls_back_account_migration_from_four() {
        let fixture = Fixture::new();
        let path = fixture.database();
        let mut connection = seed_0002(&path);
        migrate(&mut connection, &MIGRATIONS[..4]).unwrap();
        let before = legacy_rows(&connection);
        let broken_sql = format!("{}\nINVALID SQL;", MIGRATIONS[5].sql);
        let mut migrations = MIGRATIONS.to_vec();
        migrations[5].sql = &broken_sql;
        assert!(migrate(&mut connection, &migrations).is_err());
        assert_eq!(legacy_rows(&connection), before);
        assert_eq!(
            connection
                .pragma_query_value::<i64, _>(None, "user_version", |r| r.get(0))
                .unwrap(),
            4
        );
        assert!(!connection.table_exists(None, "account_view").unwrap());
        assert!(
            !connection
                .table_exists(None, "service_catalog_view")
                .unwrap()
        );
        assert_eq!(
            connection
                .query_row("SELECT count(*) FROM schema_migrations", [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            4
        );
        drop(connection);
        assert_account_services_schema(&open(&path).unwrap());
    }

    #[test]
    fn account_services_schema_keeps_scope_and_intentions_separate() {
        let fixture = Fixture::new();
        let connection = open(&fixture.database()).unwrap();
        for account in ["account-a", "account-b"] {
            connection.execute("INSERT INTO account_preferences VALUES (?1, 'user', 'device', 'theme', '\"dark\"', 1)", [account]).unwrap();
        }
        assert_eq!(
            connection
                .query_row("SELECT count(*) FROM account_preferences", [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            2
        );
        assert!(connection.execute("INSERT INTO account_preferences VALUES ('account-a', 'user', 'device', 'timeZone', '\"UTC\"', 1)", []).is_err());
        assert!(connection.execute("INSERT INTO service_catalog_view VALUES ('a','u','d','s','p','Synthetic',1.5,'active',1)", []).is_err());
        connection.execute("INSERT INTO service_catalog_view VALUES ('a','u','d','s','p','Synthetic',30,'active',1)", []).unwrap();
        connection.execute("INSERT INTO service_local VALUES ('a','u','d','op','service','p','s','CreateService',0,'{}','{}',1,1)", []).unwrap();
        connection.execute("INSERT INTO service_local VALUES ('a','u','d','profile-op','professional','p',NULL,'UpdateProfessionalProfile',1,'{}','{}',1,1)", []).unwrap();
        connection
            .execute("DELETE FROM service_catalog_view", [])
            .unwrap();
        assert_eq!(
            connection
                .query_row("SELECT count(*) FROM service_local", [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            2
        );
    }

    #[test]
    fn rejects_future_schema_without_changing_file() {
        let fixture = Fixture::new();
        let path = fixture.database();
        let connection = open(&path).unwrap();
        let future = MIGRATIONS.last().unwrap().version + 1;
        connection
            .pragma_update(None, "user_version", future)
            .unwrap();
        drop(connection);
        let before = fs::read(&path).unwrap();
        assert!(open(&path).is_err());
        assert_eq!(fs::read(path).unwrap(), before);
    }

    #[test]
    fn rejects_unknown_unversioned_database_without_changing_file() {
        let fixture = Fixture::new();
        let path = fixture.database();
        let connection = Connection::open(&path).unwrap();
        connection
            .execute_batch(
                "CREATE TABLE sqlitex (value TEXT); INSERT INTO sqlitex VALUES ('synthetic');",
            )
            .unwrap();
        drop(connection);
        let before = fs::read(&path).unwrap();
        assert!(open(&path).is_err());
        assert_eq!(fs::read(path).unwrap(), before);
    }

    #[test]
    fn rejects_invalid_history_without_changing_file() {
        let fixture = Fixture::new();
        let path = fixture.database();
        let connection = open(&path).unwrap();
        connection
            .execute(
                "UPDATE schema_migrations SET name = 'unknown' WHERE version = 1",
                [],
            )
            .unwrap();
        drop(connection);
        let before = fs::read(&path).unwrap();
        assert!(open(&path).is_err());
        assert_eq!(fs::read(path).unwrap(), before);
    }

    #[test]
    fn rejects_history_with_gap_without_changing_file() {
        let fixture = Fixture::new();
        let path = fixture.database();
        let mut connection = Connection::open(&path).unwrap();
        migrate(&mut connection, &test_migrations()).unwrap();
        connection
            .execute("DELETE FROM schema_migrations WHERE version = 2", [])
            .unwrap();
        drop(connection);
        let before = fs::read(&path).unwrap();
        assert!(open(&path).is_err());
        assert_eq!(fs::read(path).unwrap(), before);
    }

    #[test]
    fn rejects_migration_that_forgets_user_version() {
        let fixture = Fixture::new();
        let mut connection = Connection::open(fixture.database()).unwrap();
        let broken = [Migration {
            version: 1,
            name: "0001_broken",
            sql: "CREATE TABLE broken (id INTEGER);",
        }];

        assert!(migrate(&mut connection, &broken).is_err());
    }

    #[test]
    fn preserves_corrupt_file() {
        let fixture = Fixture::new();
        let path = fixture.database();
        let before = b"synthetic invalid database";
        fs::write(&path, before).unwrap();
        assert!(open(&path).is_err());
        assert_eq!(fs::read(path).unwrap(), before);
    }

    #[test]
    fn rolls_back_failed_migration() {
        let fixture = Fixture::new();
        let path = fixture.database();
        let mut connection = Connection::open(&path).unwrap();
        let broken_sql = format!("{SCHEMA}\nINVALID SQL;");
        let broken = [Migration {
            version: 1,
            name: "0001_schema",
            sql: &broken_sql,
        }];
        assert!(migrate(&mut connection, &broken).is_err());
        let version: i64 = connection
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        assert_eq!(version, 0);
        let tables: i64 = connection
            .query_row("SELECT count(*) FROM sqlite_schema", [], |row| row.get(0))
            .unwrap();
        assert_eq!(tables, 0);
        drop(connection);
        assert!(open(&path).is_ok());
    }

    #[test]
    fn transaction_commits_on_success_and_rolls_back_on_error() {
        let fixture = Fixture::new();
        let mut connection = Connection::open(fixture.database()).unwrap();

        let committed: rusqlite::Result<i64> = in_transaction(&mut connection, |tx| {
            tx.execute("CREATE TABLE committed (id INTEGER)", [])?;
            tx.query_row("SELECT count(*) FROM committed", [], |row| row.get(0))
        });
        assert_eq!(committed.unwrap(), 0);

        let failed: rusqlite::Result<()> = in_transaction(&mut connection, |tx| {
            tx.execute("CREATE TABLE rolled_back (id INTEGER)", [])?;
            Err(rusqlite::Error::ExecuteReturnedResults)
        });
        assert!(failed.is_err());
        let tables: i64 = connection
            .query_row(
                "SELECT count(*) FROM sqlite_schema WHERE name = 'rolled_back'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(tables, 0);
    }

    #[test]
    fn reports_open_failure_without_replacing_directory() {
        let fixture = Fixture::new();
        assert!(open(&fixture.0).is_err());
        assert!(fixture.0.is_dir());
    }

    fn seed_patients_schema_six(path: &Path) -> Connection {
        let mut connection = Connection::open(path).unwrap();
        migrate(&mut connection, &MIGRATIONS[..6]).unwrap();
        connection.execute("INSERT INTO service_catalog_view VALUES ('a','u','d','s','p','Synthetic',30,'active',1)", []).unwrap();
        connection
            .execute(
                "INSERT INTO account_preferences VALUES ('b','u','d','theme','\"dark\"',1)",
                [],
            )
            .unwrap();
        connection
    }

    #[test]
    fn patients_upgrade_six_keeps_existing_data_and_reopens() {
        let fixture = Fixture::new();
        let path = fixture.database();
        drop(seed_patients_schema_six(&path));
        let connection = open(&path).unwrap();
        assert_eq!(
            connection
                .pragma_query_value::<i64, _>(None, "user_version", |r| r.get(0))
                .unwrap(),
            MIGRATIONS.last().unwrap().version
        );
        for table in [
            "patient_profile_view",
            "patient_identifier_view",
            "contact_person_view",
            "patient_contact_view",
            "patient_local",
        ] {
            assert!(connection.table_exists(None, table).unwrap());
        }
        assert_eq!(
            connection
                .query_row(
                    "SELECT name FROM service_catalog_view WHERE account_id='a'",
                    [],
                    |r| r.get::<_, String>(0)
                )
                .unwrap(),
            "Synthetic"
        );
        assert_eq!(
            connection
                .query_row(
                    "SELECT value FROM account_preferences WHERE account_id='b'",
                    [],
                    |r| r.get::<_, String>(0)
                )
                .unwrap(),
            "\"dark\""
        );
        drop(connection);
        let before = fs::read(&path).unwrap();
        let connection = open(&path).unwrap();
        assert_eq!(connection.total_changes(), 0);
        assert_eq!(
            connection
                .query_row("PRAGMA integrity_check", [], |r| r.get::<_, String>(0))
                .unwrap(),
            "ok"
        );
        drop(connection);
        assert_eq!(fs::read(path).unwrap(), before);
    }

    #[test]
    fn patients_failed_seventh_migration_restores_six_byte_for_byte() {
        let fixture = Fixture::new();
        let path = fixture.database();
        drop(seed_patients_schema_six(&path));
        let before = fs::read(&path).unwrap();
        let mut connection = Connection::open(&path).unwrap();
        let broken_sql = format!("{}\nINVALID SQL;", MIGRATIONS[6].sql);
        let mut migrations = MIGRATIONS.to_vec();
        migrations[6].sql = &broken_sql;
        assert!(migrate(&mut connection, &migrations).is_err());
        assert_eq!(
            connection
                .pragma_query_value::<i64, _>(None, "user_version", |r| r.get(0))
                .unwrap(),
            6
        );
        assert_eq!(
            connection
                .query_row("SELECT count(*) FROM schema_migrations", [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            6
        );
        assert!(!connection.table_exists(None, "patient_local").unwrap());
        assert!(
            !connection
                .table_exists(None, "patient_profile_view")
                .unwrap()
        );
        drop(connection);
        assert_eq!(fs::read(&path).unwrap(), before);
        assert!(open(&path).is_ok());
    }

    fn legacy_rows(connection: &Connection) -> Vec<Vec<rusqlite::types::Value>> {
        connection.prepare("SELECT operation_id, contract_version, action, entity_type, entity_key, expected_version, payload, occurred_at_local, attempts, last_error, created_at FROM sync_outbox ORDER BY operation_id")
            .unwrap().query_map([], |row| (0..11).map(|index| row.get(index)).collect()).unwrap()
            .collect::<rusqlite::Result<_>>().unwrap()
    }

    fn seed_0002(path: &Path) -> Connection {
        let mut connection = Connection::open(path).unwrap();
        migrate(&mut connection, &MIGRATIONS[..2]).unwrap();

        for (index, status) in ["pending", "sent", "failed", "conflict"].iter().enumerate() {
            connection.execute("INSERT INTO sync_outbox VALUES (?1,1,'RegisterPatient','patient_profile',?2,0,?3,?4,?5,3,'prior error',42)", rusqlite::params![
                format!("019a1234-0000-7000-8000-00000000000{}", index + 1),
                r#""019a1234-0000-7000-8000-000000000009""#,
                r#"{"fullName":"Synthetic","omitted":null}"#,
                "2026-10-29T10:00:00-04:00", status]).unwrap();
        }

        connection
    }

    #[test]
    fn upgrades_0002_without_losing_or_assigning_operations() {
        use crate::components::sync::application::OperationStore;
        let fixture = Fixture::new();
        let path = fixture.database();
        let connection = seed_0002(&path);
        let before = legacy_rows(&connection);
        drop(connection);
        let connection = open(&path).unwrap();
        assert_eq!(legacy_rows(&connection), before);
        let statuses: Vec<String> = connection
            .prepare("SELECT status FROM sync_outbox ORDER BY operation_id")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap();
        assert_eq!(statuses, ["pending", "unknown", "retryable", "conflict"]);
        let unassigned: i64 = connection.query_row("SELECT count(*) FROM sync_outbox WHERE account_id IS NULL AND user_id IS NULL AND device_id IS NULL AND permission_epoch IS NULL AND semantic_fingerprint IS NULL AND depends_on = '[]'", [], |row| row.get(0)).unwrap();
        assert_eq!(unassigned, 4);
        let pending = crate::components::sync::adapters::outbound::SqliteOperationStore
            .list_pending(&connection, 10)
            .unwrap();
        assert_eq!(pending.len(), 1);
        assert!(pending[0].context.is_none());
        assert!(pending[0].semantic_fingerprint.is_none());
        assert!(pending[0].envelope().is_err());
        let tables: Vec<String> = connection
            .prepare("SELECT name FROM sqlite_schema WHERE type = 'table'")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap();
        for expected in [
            "identity_context",
            "device_installation",
            "identity_account_access",
            "sync_inbox",
            "sync_cursor",
            "sync_remote_items",
            "sync_snapshot_recovery",
            "sync_snapshot_items",
        ] {
            assert!(tables.iter().any(|table| table == expected));
        }
        for table in tables {
            let columns: Vec<String> = connection
                .prepare("SELECT name FROM pragma_table_info(?1)")
                .unwrap()
                .query_map([table], |row| row.get(0))
                .unwrap()
                .collect::<rusqlite::Result<_>>()
                .unwrap();
            assert!(!columns.iter().any(|name| name.contains("token")
                || name.contains("secret")
                || name.contains("credential")));
        }
        assert_eq!(
            connection
                .query_row("PRAGMA integrity_check", [], |row| row.get::<_, String>(0))
                .unwrap(),
            "ok"
        );
        drop(connection);
        let bytes = fs::read(&path).unwrap();
        let connection = open(&path).unwrap();
        assert_eq!(connection.total_changes(), 0);
        assert_eq!(legacy_rows(&connection), before);
        drop(connection);
        assert_eq!(fs::read(&path).unwrap(), bytes);
    }

    #[test]
    fn failed_0004_restores_0002_file_history_and_outbox() {
        let fixture = Fixture::new();
        let path = fixture.database();
        let connection = seed_0002(&path);
        let before = legacy_rows(&connection);
        drop(connection);
        let bytes = fs::read(&path).unwrap();
        let mut connection = Connection::open(&path).unwrap();
        let broken_sql = format!("{}\nINVALID SQL;", MIGRATIONS[3].sql);
        let mut migrations = MIGRATIONS.to_vec();
        migrations[3].sql = &broken_sql;
        assert!(migrate(&mut connection, &migrations).is_err());
        assert_eq!(legacy_rows(&connection), before);
        assert_eq!(
            connection
                .pragma_query_value::<i64, _>(None, "user_version", |row| row.get(0))
                .unwrap(),
            2
        );
        assert_eq!(
            connection
                .query_row("SELECT count(*) FROM schema_migrations", [], |row| row
                    .get::<_, i64>(0))
                .unwrap(),
            2
        );
        assert_eq!(
            connection
                .query_row(
                    "SELECT count(*) FROM sqlite_schema WHERE name = 'identity_context'",
                    [],
                    |row| row.get::<_, i64>(0)
                )
                .unwrap(),
            0
        );
        drop(connection);
        assert_eq!(fs::read(&path).unwrap(), bytes);
        assert!(open(&path).is_ok());
    }

    #[test]
    fn reopening_recovers_in_flight_without_changing_intention() {
        let fixture = Fixture::new();
        let path = fixture.database();
        drop(seed_0002(&path));
        let connection = open(&path).unwrap();
        connection.execute("UPDATE sync_outbox SET status = 'in_flight', lease_until = 100 WHERE status = 'pending'", []).unwrap();
        let before = legacy_rows(&connection);
        drop(connection);
        let connection = open(&path).unwrap();
        assert_eq!(legacy_rows(&connection), before);
        assert_eq!(connection.query_row("SELECT count(*) FROM sync_outbox WHERE status = 'in_flight' OR lease_until IS NOT NULL", [], |row| row.get::<_, i64>(0)).unwrap(), 0);
        assert_eq!(
            connection
                .query_row(
                    "SELECT count(*) FROM sync_outbox WHERE status = 'unknown'",
                    [],
                    |row| row.get::<_, i64>(0)
                )
                .unwrap(),
            2
        );
    }
}
