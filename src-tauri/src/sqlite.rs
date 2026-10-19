use rusqlite::{Connection, TransactionBehavior};
use std::{error::Error, io, path::Path};

const APPLICATION_ID: i64 = 1_313_297_480;
const SCHEMA: &str = include_str!("../migrations/0001_schema.sql");
type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub fn open(path: &Path) -> Result<Connection> {
    let mut connection = Connection::open(path)?;
    migrate(&mut connection, SCHEMA)?;
    connection.pragma_update(None, "foreign_keys", true)?;

    Ok(connection)
}

fn migrate(connection: &mut Connection, sql: &str) -> Result<()> {
    // Lock before checking the version so two starts cannot both migrate.
    // https://docs.rs/rusqlite/0.40.2/rusqlite/struct.Connection.html#method.transaction_with_behavior
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let version: i64 = transaction.pragma_query_value(None, "user_version", |row| row.get(0))?;
    let application: i64 =
        transaction.pragma_query_value(None, "application_id", |row| row.get(0))?;

    match (version, application) {
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

            transaction.execute_batch(sql)?;
        }
        (1, APPLICATION_ID) => {
            let migrations: Vec<(i64, String)> = transaction
                .prepare("SELECT version, name FROM schema_migrations ORDER BY version")?
                .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
                .collect::<rusqlite::Result<_>>()?;

            if migrations != [(1, "0001_schema".to_owned())] {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid migration history",
                )
                .into());
            }
        }
        _ => {
            return Err(
                io::Error::new(io::ErrorKind::InvalidData, "unsupported database schema").into(),
            );
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

    #[test]
    fn creates_and_reopens_without_repeating_migration() {
        let fixture = Fixture::new();
        let path = fixture.database();
        let connection = open(&path).unwrap();
        let version: i64 = connection
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        assert_eq!(version, 1);
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
        assert_eq!(count, 1);
        drop(connection);
        assert_eq!(fs::read(path).unwrap(), before);
    }

    #[test]
    fn rejects_future_schema_without_changing_file() {
        let fixture = Fixture::new();
        let path = fixture.database();
        let connection = open(&path).unwrap();
        connection.pragma_update(None, "user_version", 2).unwrap();
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
            .execute("UPDATE schema_migrations SET name = 'unknown'", [])
            .unwrap();
        drop(connection);
        let before = fs::read(&path).unwrap();
        assert!(open(&path).is_err());
        assert_eq!(fs::read(path).unwrap(), before);
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
        assert!(migrate(&mut connection, &format!("{SCHEMA}\nINVALID SQL;")).is_err());
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
    fn reports_open_failure_without_replacing_directory() {
        let fixture = Fixture::new();
        assert!(open(&fixture.0).is_err());
        assert!(fixture.0.is_dir());
    }
}
