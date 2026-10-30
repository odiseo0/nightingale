-- No credentials: these tables hold only local context and the last API access response.
CREATE TABLE identity_context (
    singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
    issuer TEXT NOT NULL,
    subject TEXT NOT NULL,
    user_id TEXT NOT NULL,
    selected_account_id TEXT
);
CREATE TABLE device_installation (
    device_id TEXT PRIMARY KEY,
    display_name TEXT NOT NULL,
    platform TEXT NOT NULL,
    client_version TEXT NOT NULL
);
CREATE TABLE identity_account_access (
    account_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    device_id TEXT NOT NULL,
    role TEXT NOT NULL CHECK (role IN ('owner', 'assistant')),
    status TEXT NOT NULL CHECK (status IN ('active', 'registration_required', 'revoked', 'inactive')),
    permission_epoch INTEGER NOT NULL,
    allowed_actions TEXT NOT NULL CHECK (json_valid(allowed_actions) AND json_type(allowed_actions) = 'array'),
    checked_at INTEGER NOT NULL,
    PRIMARY KEY (account_id, user_id, device_id)
);
INSERT INTO schema_migrations (version, name) VALUES (3, '0003_identity_access');
PRAGMA user_version = 3;
