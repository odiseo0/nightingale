-- Accepted views have no foreign keys to intentions. Reset must keep local work.
-- https://www.sqlite.org/lang_createtable.html#check_constraints
-- https://www.sqlite.org/json1.html#the_json_valid_function
CREATE TABLE schedule_rule_view (
    account_id TEXT NOT NULL, user_id TEXT NOT NULL, device_id TEXT NOT NULL,
    rule_id TEXT NOT NULL,
    professional_id TEXT NOT NULL,
    iso_weekday INTEGER NOT NULL CHECK (typeof(iso_weekday) = 'integer' AND iso_weekday BETWEEN 1 AND 7),
    local_start_minute INTEGER NOT NULL CHECK (typeof(local_start_minute) = 'integer' AND local_start_minute BETWEEN 0 AND 1439),
    local_end_minute INTEGER NOT NULL CHECK (typeof(local_end_minute) = 'integer' AND local_end_minute BETWEEN 1 AND 1440),
    valid_from TEXT NOT NULL,
    valid_until TEXT,
    status TEXT NOT NULL CHECK (status IN ('active','inactive')),
    created_at TEXT,
    updated_at TEXT,
    version INTEGER NOT NULL CHECK (typeof(version) = 'integer' AND version > 0),
    CHECK (local_start_minute < local_end_minute),
    CHECK (valid_until IS NULL OR valid_until >= valid_from),
    PRIMARY KEY (account_id, user_id, device_id, rule_id)
);
CREATE INDEX schedule_rule_professional ON schedule_rule_view
    (account_id, user_id, device_id, professional_id, iso_weekday, status, rule_id);
CREATE INDEX schedule_rule_validity ON schedule_rule_view
    (account_id, user_id, device_id, professional_id, valid_from, valid_until);

CREATE TABLE schedule_exception_view (
    account_id TEXT NOT NULL, user_id TEXT NOT NULL, device_id TEXT NOT NULL,
    exception_id TEXT NOT NULL,
    professional_id TEXT NOT NULL,
    local_date TEXT NOT NULL,
    kind TEXT NOT NULL CHECK (kind IN ('closed','replacement')),
    local_start_minute INTEGER,
    local_end_minute INTEGER,
    status TEXT NOT NULL CHECK (status IN ('active','inactive')),
    created_at TEXT,
    updated_at TEXT,
    version INTEGER NOT NULL CHECK (typeof(version) = 'integer' AND version > 0),
    CHECK (
        (kind = 'closed' AND local_start_minute IS NULL AND local_end_minute IS NULL)
        OR (kind = 'replacement'
            AND local_start_minute IS NOT NULL AND local_end_minute IS NOT NULL
            AND typeof(local_start_minute) = 'integer' AND local_start_minute BETWEEN 0 AND 1439
            AND typeof(local_end_minute) = 'integer' AND local_end_minute BETWEEN 1 AND 1440
            AND local_start_minute < local_end_minute)
    ),
    PRIMARY KEY (account_id, user_id, device_id, exception_id)
);
CREATE INDEX schedule_exception_date ON schedule_exception_view
    (account_id, user_id, device_id, professional_id, local_date, status, exception_id);

CREATE TABLE schedule_block_view (
    account_id TEXT NOT NULL, user_id TEXT NOT NULL, device_id TEXT NOT NULL,
    block_id TEXT NOT NULL,
    professional_id TEXT NOT NULL,
    block_account_id TEXT,
    starts_at TEXT NOT NULL,
    ends_at TEXT NOT NULL,
    reason_code TEXT CHECK (reason_code IN ('vacation','administrative','unavailable')),
    status TEXT NOT NULL CHECK (status IN ('active','inactive')),
    created_at TEXT,
    updated_at TEXT,
    version INTEGER NOT NULL CHECK (typeof(version) = 'integer' AND version > 0),
    CHECK (starts_at < ends_at),
    PRIMARY KEY (account_id, user_id, device_id, block_id)
);
-- NULL block_account_id marks a global block covering the professional's activity.
CREATE INDEX schedule_block_window ON schedule_block_view
    (account_id, user_id, device_id, professional_id, starts_at, ends_at, status, block_id);
CREATE INDEX schedule_block_account ON schedule_block_view
    (account_id, user_id, device_id, block_account_id, starts_at, ends_at);

CREATE TABLE schedule_local (
    account_id TEXT NOT NULL, user_id TEXT NOT NULL, device_id TEXT NOT NULL,
    operation_id TEXT NOT NULL,
    entity_type TEXT NOT NULL CHECK (entity_type IN
        ('schedule_rule','schedule_exception','schedule_block')),
    rule_id TEXT, exception_id TEXT, block_id TEXT,
    action TEXT NOT NULL,
    expected_version INTEGER NOT NULL CHECK (typeof(expected_version) = 'integer' AND expected_version >= 0),
    base_operation_id TEXT,
    depends_on TEXT NOT NULL CHECK (json_valid(depends_on) AND json_type(depends_on) = 'array' AND json_array_length(depends_on) <= 8),
    payload TEXT NOT NULL CHECK (json_valid(payload) AND json_type(payload) = 'object'),
    draft TEXT NOT NULL CHECK (json_valid(draft) AND json_type(draft) = 'object'),
    created_at INTEGER NOT NULL,
    CHECK (
        (entity_type = 'schedule_rule' AND rule_id IS NOT NULL AND exception_id IS NULL AND block_id IS NULL
            AND action IN ('CreateScheduleRule','UpdateScheduleRule','SetScheduleRuleStatus'))
        OR (entity_type = 'schedule_exception' AND rule_id IS NULL AND exception_id IS NOT NULL AND block_id IS NULL
            AND action IN ('CreateScheduleException','UpdateScheduleException','SetScheduleExceptionStatus'))
        OR (entity_type = 'schedule_block' AND rule_id IS NULL AND exception_id IS NULL AND block_id IS NOT NULL
            AND action IN ('CreateScheduleBlock','UpdateScheduleBlock','SetScheduleBlockStatus'))
    ),
    PRIMARY KEY (account_id, user_id, device_id, operation_id)
);
CREATE INDEX schedule_local_entity ON schedule_local
    (account_id, user_id, device_id, entity_type, rule_id, exception_id, block_id, created_at, operation_id);
INSERT INTO schema_migrations (version, name) VALUES (8, '0008_scheduling');
PRAGMA user_version = 8;
