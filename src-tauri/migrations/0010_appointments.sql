-- Accepted appointment and history views plus local intentions, owned by the
-- appointments component. Views have no foreign keys to intentions: reset must
-- keep local work. appointment_occupancy_view (0009) is preserved unchanged.
-- https://www.sqlite.org/lang_createtable.html#check_constraints
-- https://www.sqlite.org/json1.html#the_json_valid_function
CREATE TABLE appointment_view (
    account_id TEXT NOT NULL, user_id TEXT NOT NULL, device_id TEXT NOT NULL,
    appointment_id TEXT NOT NULL,
    patient_id TEXT NOT NULL,
    professional_id TEXT NOT NULL,
    service_id TEXT NOT NULL,
    responsible_contact_id TEXT,
    starts_at TEXT NOT NULL,
    ends_at TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('scheduled','confirmed','cancelled','completed','no_show')),
    confirmed_at TEXT,
    service_name_snapshot TEXT NOT NULL,
    offered_duration_minutes_snapshot INTEGER NOT NULL
        CHECK (typeof(offered_duration_minutes_snapshot) = 'integer' AND offered_duration_minutes_snapshot BETWEEN 1 AND 1440),
    professional_name_snapshot TEXT NOT NULL,
    location_name_snapshot TEXT NOT NULL,
    location_address_snapshot TEXT NOT NULL,
    location_country_snapshot TEXT NOT NULL CHECK (length(location_country_snapshot) = 2),
    location_state_snapshot TEXT NOT NULL,
    location_city_snapshot TEXT NOT NULL,
    time_zone_snapshot TEXT NOT NULL,
    created_at TEXT,
    updated_at TEXT,
    version INTEGER NOT NULL CHECK (typeof(version) = 'integer' AND version > 0),
    CHECK (starts_at < ends_at),
    PRIMARY KEY (account_id, user_id, device_id, appointment_id)
);
CREATE INDEX appointment_professional_window ON appointment_view
    (account_id, user_id, device_id, professional_id, starts_at, ends_at, status);
CREATE INDEX appointment_patient ON appointment_view
    (account_id, user_id, device_id, patient_id, starts_at);
CREATE INDEX appointment_service ON appointment_view
    (account_id, user_id, device_id, service_id, starts_at);

-- One row per accepted appointment version. History is append-only: the
-- projector never rewrites or deletes ordinary entries.
CREATE TABLE appointment_history_view (
    account_id TEXT NOT NULL, user_id TEXT NOT NULL, device_id TEXT NOT NULL,
    appointment_id TEXT NOT NULL,
    version INTEGER NOT NULL CHECK (typeof(version) = 'integer' AND version > 0),
    action TEXT NOT NULL CHECK (action IN ('created','confirmed','rescheduled','cancelled','completed','no_show')),
    actor_kind TEXT NOT NULL CHECK (actor_kind IN ('staff','patient','contact')),
    actor_user_id TEXT,
    actor_contact_person_id TEXT,
    change_reason TEXT,
    changed_at TEXT NOT NULL,
    starts_at TEXT NOT NULL,
    ends_at TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('scheduled','confirmed','cancelled','completed','no_show')),
    confirmed_at TEXT,
    service_name_snapshot TEXT NOT NULL,
    offered_duration_minutes_snapshot INTEGER NOT NULL
        CHECK (typeof(offered_duration_minutes_snapshot) = 'integer' AND offered_duration_minutes_snapshot BETWEEN 1 AND 1440),
    professional_name_snapshot TEXT NOT NULL,
    location_name_snapshot TEXT NOT NULL,
    location_address_snapshot TEXT NOT NULL,
    location_country_snapshot TEXT NOT NULL CHECK (length(location_country_snapshot) = 2),
    location_state_snapshot TEXT NOT NULL,
    location_city_snapshot TEXT NOT NULL,
    time_zone_snapshot TEXT NOT NULL,
    CHECK (starts_at < ends_at),
    PRIMARY KEY (account_id, user_id, device_id, appointment_id, version)
);
CREATE INDEX appointment_history_appointment ON appointment_history_view
    (account_id, user_id, device_id, appointment_id, version);

-- Local intentions: the semantic payload travels in sync_outbox; the draft is
-- the provisional projection read by merged listings.
CREATE TABLE appointment_local (
    account_id TEXT NOT NULL, user_id TEXT NOT NULL, device_id TEXT NOT NULL,
    operation_id TEXT NOT NULL,
    appointment_id TEXT NOT NULL,
    action TEXT NOT NULL CHECK (action IN
        ('CreateAppointment','ConfirmAppointment','RescheduleAppointment','CancelAppointment','CompleteAppointment','MarkAppointmentNoShow')),
    expected_version INTEGER NOT NULL CHECK (typeof(expected_version) = 'integer' AND expected_version >= 0),
    base_operation_id TEXT,
    depends_on TEXT NOT NULL CHECK (json_valid(depends_on) AND json_type(depends_on) = 'array' AND json_array_length(depends_on) <= 8),
    payload TEXT NOT NULL CHECK (json_valid(payload) AND json_type(payload) = 'object'),
    draft TEXT NOT NULL CHECK (json_valid(draft) AND json_type(draft) = 'object'),
    created_at INTEGER NOT NULL,
    PRIMARY KEY (account_id, user_id, device_id, operation_id)
);
CREATE INDEX appointment_local_appointment ON appointment_local
    (account_id, user_id, device_id, appointment_id, created_at, operation_id);
INSERT INTO schema_migrations (version, name) VALUES (10, '0010_appointments');
PRAGMA user_version = 10;
