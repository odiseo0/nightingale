-- Version 2 adds the local sync outbox owned by the sync component.
-- entity_key and payload hold JSON: the contract accepts a UUID string or a
-- composite object as entityKey, and an object as payload.
-- created_at is milliseconds since Unix epoch (no timezone ambiguity).
CREATE TABLE sync_outbox (
    operation_id TEXT PRIMARY KEY,
    contract_version INTEGER NOT NULL CHECK (contract_version > 0),
    action TEXT NOT NULL,
    entity_type TEXT NOT NULL,
    entity_key TEXT NOT NULL,
    expected_version INTEGER NOT NULL CHECK (expected_version >= 0),
    payload TEXT NOT NULL,
    occurred_at_local TEXT,
    status TEXT NOT NULL CHECK (status IN ('pending', 'sent', 'failed', 'conflict')),
    attempts INTEGER NOT NULL DEFAULT 0 CHECK (attempts >= 0),
    last_error TEXT,
    created_at INTEGER NOT NULL
);
CREATE INDEX sync_outbox_status_created ON sync_outbox (status, created_at);
INSERT INTO schema_migrations (version, name) VALUES (2, '0002_sync_outbox');
PRAGMA user_version = 2;
