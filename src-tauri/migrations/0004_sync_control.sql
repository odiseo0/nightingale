-- Rebuild to change the status CHECK, inside the runner's transaction.
-- https://www.sqlite.org/lang_altertable.html#making_other_kinds_of_table_schema_changes
CREATE TABLE sync_outbox_new (
    operation_id TEXT PRIMARY KEY,
    contract_version INTEGER NOT NULL CHECK (contract_version > 0),
    action TEXT NOT NULL,
    entity_type TEXT NOT NULL,
    entity_key TEXT NOT NULL,
    expected_version INTEGER NOT NULL CHECK (expected_version >= 0),
    payload TEXT NOT NULL,
    occurred_at_local TEXT,
    status TEXT NOT NULL CHECK (status IN ('pending', 'in_flight', 'unknown', 'retryable', 'accepted', 'conflict', 'rejected')),
    attempts INTEGER NOT NULL DEFAULT 0 CHECK (attempts >= 0),
    last_error TEXT,
    created_at INTEGER NOT NULL,
    account_id TEXT,
    user_id TEXT,
    device_id TEXT,
    permission_epoch INTEGER,
    depends_on TEXT NOT NULL DEFAULT '[]' CHECK (json_valid(depends_on) AND json_type(depends_on) = 'array' AND json_array_length(depends_on) <= 8),
    semantic_fingerprint TEXT,
    result TEXT,
    lease_until INTEGER,
    next_attempt_at INTEGER,
    CHECK ((account_id IS NULL AND user_id IS NULL AND device_id IS NULL AND permission_epoch IS NULL)
        OR (account_id IS NOT NULL AND user_id IS NOT NULL AND device_id IS NOT NULL AND permission_epoch IS NOT NULL))
);
INSERT INTO sync_outbox_new (
    operation_id, contract_version, action, entity_type, entity_key,
    expected_version, payload, occurred_at_local, status, attempts, last_error, created_at
) SELECT operation_id, contract_version, action, entity_type, entity_key,
    expected_version, payload, occurred_at_local,
    CASE status WHEN 'sent' THEN 'unknown' WHEN 'failed' THEN 'retryable' ELSE status END,
    attempts, last_error, created_at FROM sync_outbox;
DROP TABLE sync_outbox;
ALTER TABLE sync_outbox_new RENAME TO sync_outbox;
CREATE INDEX sync_outbox_status_created ON sync_outbox (status, created_at);

CREATE TABLE sync_cursor (
    account_id TEXT NOT NULL, user_id TEXT NOT NULL, device_id TEXT NOT NULL,
    permission_epoch INTEGER NOT NULL,
    cursor TEXT NOT NULL,
    window TEXT NOT NULL CHECK (json_valid(window)),
    PRIMARY KEY (account_id, user_id, device_id)
);
CREATE TABLE sync_inbox (
    account_id TEXT NOT NULL, user_id TEXT NOT NULL, device_id TEXT NOT NULL,
    permission_epoch INTEGER NOT NULL,
    group_id TEXT NOT NULL, operation_id TEXT NOT NULL,
    PRIMARY KEY (account_id, user_id, device_id, permission_epoch, group_id),
    UNIQUE (account_id, user_id, device_id, permission_epoch, operation_id)
);
CREATE TABLE sync_remote_items (
    account_id TEXT NOT NULL, user_id TEXT NOT NULL, device_id TEXT NOT NULL,
    permission_epoch INTEGER NOT NULL,
    entity_type TEXT NOT NULL, entity_key TEXT NOT NULL CHECK (json_valid(entity_key)),
    entity_version INTEGER NOT NULL CHECK (entity_version >= 1),
    state TEXT NOT NULL CHECK (json_valid(state)),
    PRIMARY KEY (account_id, user_id, device_id, permission_epoch, entity_type, entity_key)
);
CREATE TABLE sync_snapshot_recovery (
    account_id TEXT NOT NULL, user_id TEXT NOT NULL, device_id TEXT NOT NULL,
    permission_epoch INTEGER NOT NULL,
    snapshot_id TEXT NOT NULL, base_cursor TEXT NOT NULL,
    window TEXT NOT NULL CHECK (json_valid(window)),
    expires_at TEXT NOT NULL, item_count INTEGER NOT NULL CHECK (item_count >= 0),
    next_page INTEGER NOT NULL DEFAULT 1 CHECK (next_page >= 1),
    PRIMARY KEY (account_id, user_id, device_id),
    UNIQUE (account_id, user_id, device_id, snapshot_id)
);
CREATE TABLE sync_snapshot_items (
    account_id TEXT NOT NULL, user_id TEXT NOT NULL, device_id TEXT NOT NULL,
    snapshot_id TEXT NOT NULL,
    entity_type TEXT NOT NULL, entity_key TEXT NOT NULL CHECK (json_valid(entity_key)),
    entity_version INTEGER NOT NULL CHECK (entity_version >= 1),
    state TEXT NOT NULL CHECK (json_valid(state)),
    PRIMARY KEY (account_id, user_id, device_id, snapshot_id, entity_type, entity_key),
    FOREIGN KEY (account_id, user_id, device_id, snapshot_id)
        REFERENCES sync_snapshot_recovery (account_id, user_id, device_id, snapshot_id) ON DELETE CASCADE
);
INSERT INTO schema_migrations (version, name) VALUES (4, '0004_sync_control');
PRAGMA user_version = 4;
