-- Accepted views have no foreign keys to intentions. Reset must keep local work.
-- https://www.sqlite.org/lang_createtable.html#check_constraints
-- https://www.sqlite.org/json1.html#the_json_valid_function
CREATE TABLE patient_profile_view (
    account_id TEXT NOT NULL, user_id TEXT NOT NULL, device_id TEXT NOT NULL,
    patient_id TEXT NOT NULL,
    full_name TEXT,
    birth_date TEXT,
    phone_code TEXT,
    phone TEXT,
    identification_basis TEXT CHECK (identification_basis IN ('own_document','minor_without_document')),
    completion_status TEXT NOT NULL CHECK (completion_status IN ('draft','complete')),
    status TEXT NOT NULL CHECK (status IN ('active','inactive')),
    created_at TEXT,
    updated_at TEXT,
    version INTEGER NOT NULL CHECK (typeof(version) = 'integer' AND version > 0),
    CHECK ((phone_code IS NULL) = (phone IS NULL)),
    PRIMARY KEY (account_id, user_id, device_id, patient_id)
);
CREATE TABLE patient_identifier_view (
    account_id TEXT NOT NULL, user_id TEXT NOT NULL, device_id TEXT NOT NULL,
    identifier_id TEXT NOT NULL,
    patient_id TEXT NOT NULL,
    issuing_country TEXT NOT NULL,
    document_type TEXT NOT NULL CHECK (document_type IN ('national_id','passport')),
    document_number TEXT NOT NULL CHECK (length(document_number) > 0),
    verification_status TEXT NOT NULL CHECK (verification_status IN ('declared','verified','invalidated')),
    created_at TEXT,
    updated_at TEXT,
    version INTEGER NOT NULL CHECK (typeof(version) = 'integer' AND version > 0),
    PRIMARY KEY (account_id, user_id, device_id, identifier_id)
);
CREATE TABLE contact_person_view (
    account_id TEXT NOT NULL, user_id TEXT NOT NULL, device_id TEXT NOT NULL,
    contact_person_id TEXT NOT NULL,
    full_name TEXT NOT NULL CHECK (length(trim(full_name)) > 0),
    phone_code TEXT,
    phone TEXT,
    issuing_country TEXT,
    document_type TEXT CHECK (document_type IN ('national_id','passport')),
    document_number TEXT,
    status TEXT NOT NULL CHECK (status IN ('active','inactive')),
    created_at TEXT,
    updated_at TEXT,
    version INTEGER NOT NULL CHECK (typeof(version) = 'integer' AND version > 0),
    CHECK ((phone_code IS NULL) = (phone IS NULL)),
    CHECK ((issuing_country IS NULL AND document_type IS NULL AND document_number IS NULL)
        OR (issuing_country IS NOT NULL AND document_type IS NOT NULL AND document_number IS NOT NULL)),
    PRIMARY KEY (account_id, user_id, device_id, contact_person_id)
);
CREATE TABLE patient_contact_view (
    account_id TEXT NOT NULL, user_id TEXT NOT NULL, device_id TEXT NOT NULL,
    patient_id TEXT NOT NULL,
    contact_person_id TEXT NOT NULL,
    relationship_kind TEXT NOT NULL CHECK (relationship_kind IN ('mother','father','guardian','other')),
    is_responsible_contact INTEGER NOT NULL CHECK (is_responsible_contact IN (0,1)),
    representation_status TEXT NOT NULL CHECK (representation_status IN ('unverified','verified')),
    verified_at TEXT,
    verified_by_user_id TEXT,
    valid_from TEXT NOT NULL,
    valid_until TEXT,
    status TEXT NOT NULL CHECK (status IN ('active','inactive')),
    created_at TEXT,
    updated_at TEXT,
    version INTEGER NOT NULL CHECK (typeof(version) = 'integer' AND version > 0),
    PRIMARY KEY (account_id, user_id, device_id, patient_id, contact_person_id)
);
CREATE INDEX patient_profile_filter ON patient_profile_view
    (account_id, user_id, device_id, status, completion_status, patient_id);
CREATE INDEX patient_identifier_patient ON patient_identifier_view
    (account_id, user_id, device_id, patient_id, identifier_id);
CREATE INDEX contact_person_status ON contact_person_view
    (account_id, user_id, device_id, status, contact_person_id);
CREATE INDEX patient_contact_contact ON patient_contact_view
    (account_id, user_id, device_id, contact_person_id, status, patient_id);
CREATE INDEX patient_contact_patient ON patient_contact_view
    (account_id, user_id, device_id, patient_id, status, contact_person_id);

CREATE TABLE patient_local (
    account_id TEXT NOT NULL, user_id TEXT NOT NULL, device_id TEXT NOT NULL,
    operation_id TEXT NOT NULL,
    entity_type TEXT NOT NULL CHECK (entity_type IN
        ('patient_profile','patient_identifier','contact_person','patient_contact')),
    patient_id TEXT, identifier_id TEXT, contact_person_id TEXT,
    action TEXT NOT NULL,
    expected_version INTEGER NOT NULL CHECK (typeof(expected_version) = 'integer' AND expected_version >= 0),
    base_operation_id TEXT,
    depends_on TEXT NOT NULL CHECK (json_valid(depends_on) AND json_type(depends_on) = 'array' AND json_array_length(depends_on) <= 8),
    payload TEXT NOT NULL CHECK (json_valid(payload) AND json_type(payload) = 'object'),
    draft TEXT NOT NULL CHECK (json_valid(draft) AND json_type(draft) = 'object'),
    created_at INTEGER NOT NULL,
    CHECK (
        (entity_type = 'patient_profile' AND patient_id IS NOT NULL AND identifier_id IS NULL AND contact_person_id IS NULL
            AND action IN ('RegisterPatient','UpdatePatientProfile','SetPatientCompletion','SetPatientProfileStatus'))
        OR (entity_type = 'patient_identifier' AND patient_id IS NOT NULL AND identifier_id IS NOT NULL AND contact_person_id IS NULL
            AND action = 'AddPatientIdentifier')
        OR (entity_type = 'contact_person' AND patient_id IS NULL AND identifier_id IS NULL AND contact_person_id IS NOT NULL
            AND action IN ('CreateContactPerson','UpdateContactPerson','SetContactPersonStatus'))
        OR (entity_type = 'patient_contact' AND patient_id IS NOT NULL AND identifier_id IS NULL AND contact_person_id IS NOT NULL
            AND action IN ('CreatePatientContactRelationship','UpdatePatientContactRelationship','SetPatientContactRelationshipStatus'))
    ),
    PRIMARY KEY (account_id, user_id, device_id, operation_id)
);
CREATE INDEX patient_local_entity ON patient_local
    (account_id, user_id, device_id, entity_type, patient_id, identifier_id, contact_person_id, created_at, operation_id);
INSERT INTO schema_migrations (version, name) VALUES (7, '0007_patients');
PRAGMA user_version = 7;
