-- Version 1 accepts only an empty database. Existing data stays unchanged.
-- The caller runs this migration in one transaction and rolls back on failure.
CREATE TABLE schema_migrations (
    version INTEGER PRIMARY KEY CHECK (version > 0),
    name TEXT NOT NULL UNIQUE
);
INSERT INTO schema_migrations (version, name) VALUES (1, '0001_schema');
PRAGMA application_id = 1313297480;
PRAGMA user_version = 1;
