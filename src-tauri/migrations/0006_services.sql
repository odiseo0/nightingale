-- Views deliberately have no foreign keys to local intentions: remove/reset must
-- preserve pending work, including an offer whose catalog item is not visible.
CREATE TABLE service_catalog_view (
    account_id TEXT NOT NULL, user_id TEXT NOT NULL, device_id TEXT NOT NULL,
    service_id TEXT NOT NULL, professional_id TEXT NOT NULL,
    name TEXT NOT NULL,
    default_duration_minutes INTEGER NOT NULL
        CHECK (typeof(default_duration_minutes) = 'integer' AND default_duration_minutes BETWEEN 1 AND 1440),
    status TEXT NOT NULL CHECK (status IN ('active', 'inactive')),
    version INTEGER NOT NULL CHECK (version > 0),
    PRIMARY KEY (account_id, user_id, device_id, service_id)
);
CREATE TABLE service_offer_view (
    account_id TEXT NOT NULL, user_id TEXT NOT NULL, device_id TEXT NOT NULL,
    professional_id TEXT NOT NULL, service_id TEXT NOT NULL,
    name TEXT NOT NULL,
    default_duration_minutes INTEGER NOT NULL
        CHECK (typeof(default_duration_minutes) = 'integer' AND default_duration_minutes BETWEEN 1 AND 1440),
    duration_minutes_override INTEGER
        CHECK (duration_minutes_override IS NULL OR
            (typeof(duration_minutes_override) = 'integer' AND duration_minutes_override BETWEEN 1 AND 1440)),
    effective_duration_minutes INTEGER NOT NULL
        CHECK (typeof(effective_duration_minutes) = 'integer' AND effective_duration_minutes BETWEEN 1 AND 1440),
    service_status TEXT NOT NULL CHECK (service_status IN ('active', 'inactive')),
    status TEXT NOT NULL CHECK (status IN ('active', 'inactive')),
    version INTEGER NOT NULL CHECK (version > 0),
    service_version INTEGER NOT NULL CHECK (service_version > 0),
    CHECK (effective_duration_minutes = coalesce(duration_minutes_override, default_duration_minutes)),
    PRIMARY KEY (account_id, user_id, device_id, professional_id, service_id)
);
CREATE TABLE professional_view (
    account_id TEXT NOT NULL, user_id TEXT NOT NULL, device_id TEXT NOT NULL,
    professional_id TEXT NOT NULL, professional_user_id TEXT NOT NULL,
    public_name TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('active', 'inactive')),
    version INTEGER NOT NULL CHECK (version > 0),
    PRIMARY KEY (account_id, user_id, device_id, professional_id)
);
CREATE TABLE service_local (
    account_id TEXT NOT NULL, user_id TEXT NOT NULL, device_id TEXT NOT NULL,
    operation_id TEXT NOT NULL,
    entity_type TEXT NOT NULL CHECK (entity_type IN ('service', 'professional')),
    professional_id TEXT NOT NULL, service_id TEXT,
    action TEXT NOT NULL,
    expected_version INTEGER NOT NULL CHECK (expected_version >= 0),
    payload TEXT NOT NULL CHECK (json_valid(payload) AND json_type(payload) = 'object'),
    draft TEXT NOT NULL CHECK (json_valid(draft) AND json_type(draft) = 'object'),
    created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL,
    CHECK ((entity_type = 'service' AND service_id IS NOT NULL)
        OR (entity_type = 'professional' AND service_id IS NULL)),
    PRIMARY KEY (account_id, user_id, device_id, operation_id)
);
CREATE TABLE service_offer_local (
    account_id TEXT NOT NULL, user_id TEXT NOT NULL, device_id TEXT NOT NULL,
    operation_id TEXT NOT NULL,
    professional_id TEXT NOT NULL, service_id TEXT NOT NULL,
    action TEXT NOT NULL,
    expected_version INTEGER NOT NULL CHECK (expected_version >= 0),
    payload TEXT NOT NULL CHECK (json_valid(payload) AND json_type(payload) = 'object'),
    draft TEXT NOT NULL CHECK (json_valid(draft) AND json_type(draft) = 'object'),
    created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL,
    PRIMARY KEY (account_id, user_id, device_id, operation_id)
);
INSERT INTO schema_migrations (version, name) VALUES (6, '0006_services');
PRAGMA user_version = 6;
