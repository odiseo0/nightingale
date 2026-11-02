-- Accepted account data and device preferences have separate writers.
-- https://www.sqlite.org/lang_createtable.html#check_constraints
CREATE TABLE account_view (
    account_id TEXT NOT NULL, user_id TEXT NOT NULL, device_id TEXT NOT NULL,
    display_name TEXT NOT NULL,
    time_zone TEXT NOT NULL,
    location_name TEXT NOT NULL,
    location_address TEXT NOT NULL,
    location_city TEXT NOT NULL,
    location_state TEXT NOT NULL,
    location_country TEXT NOT NULL,
    no_show_grace_minutes INTEGER NOT NULL
        CHECK (typeof(no_show_grace_minutes) = 'integer' AND no_show_grace_minutes BETWEEN 0 AND 1440),
    status TEXT NOT NULL CHECK (status IN ('active', 'inactive')),
    version INTEGER NOT NULL CHECK (version > 0),
    updated_at TEXT,
    PRIMARY KEY (account_id, user_id, device_id)
);
CREATE TABLE account_preferences (
    account_id TEXT NOT NULL, user_id TEXT NOT NULL, device_id TEXT NOT NULL,
    key TEXT NOT NULL CHECK (key IN ('theme', 'servicesShowInactive')),
    value TEXT NOT NULL CHECK (json_valid(value)),
    updated_at INTEGER NOT NULL,
    PRIMARY KEY (account_id, user_id, device_id, key)
);
INSERT INTO schema_migrations (version, name) VALUES (5, '0005_account');
PRAGMA user_version = 5;
