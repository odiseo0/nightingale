-- Minimal accepted occupancy projection owned by the future appointments component.
-- Only occupying states are stored; cancelled appointments free occupancy and are
-- removed by the projector instead of being kept here.
CREATE TABLE appointment_occupancy_view (
    account_id TEXT NOT NULL, user_id TEXT NOT NULL, device_id TEXT NOT NULL,
    appointment_id TEXT NOT NULL,
    professional_id TEXT NOT NULL,
    starts_at TEXT NOT NULL,
    ends_at TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('scheduled','confirmed','completed','no_show')),
    version INTEGER NOT NULL CHECK (typeof(version) = 'integer' AND version > 0),
    CHECK (starts_at < ends_at),
    PRIMARY KEY (account_id, user_id, device_id, appointment_id)
);
CREATE INDEX appointment_occupancy_window ON appointment_occupancy_view
    (account_id, user_id, device_id, professional_id, starts_at, ends_at, status);
INSERT INTO schema_migrations (version, name) VALUES (9, '0009_appointment_occupancy');
PRAGMA user_version = 9;
