import { invoke } from "@tauri-apps/api/core";

export interface ProfileState {
  patientId: string;
  fullName: string | null;
  birthDate: string | null;
  phoneCode: string | null;
  phone: string | null;
  identificationBasis: string | null;
  completionStatus: string;
  status: string;
}

export interface PatientPage {
  items: EntityRow<ProfileState>[];
  total: number;
}

export const listPatients = (request: {
  q?: string;
  status?: string;
  completionStatus?: string;
  page?: number;
  shows?: number;
}) => invoke<PatientPage>("patients_list_patients", { request });

export const getPatient = (patientId: string) =>
  invoke<EntityRow<ProfileState>>("patients_get_patient", { request: { patientId } });

export interface IdentifierState {
  id: string;
  patientId: string;
  issuingCountry: string;
  documentType: string;
  documentNumber: string;
  verificationStatus: string;
}

export const listPatientIdentifiers = (patientId: string) =>
  invoke<EntityRow<IdentifierState>[]>("patients_list_patient_identifiers", {
    request: { patientId },
  });

export interface RelationshipState {
  patientId: string;
  contactPersonId: string;
  relationshipKind: string;
  isResponsibleContact: boolean;
  validFrom: string;
  validUntil: string | null;
  status: string;
}

/** Read-side join of a relationship with current names and phone. */
export interface RelationshipView {
  relationship: RelationshipState;
  patientFullName: string | null;
  contactFullName: string;
  contactPhoneCode: string | null;
  contactPhone: string | null;
}

export interface EntityRow<T> {
  accepted: { state: T; version: number } | null;
  provisional: { state: T; operationId: string; expectedVersion: number } | null;
  operations: { operationId: string; status: string; error: string | null }[];
}

/** Fused view: the provisional state wins while one exists. */
export const visibleOf = <T,>(row: EntityRow<T>): T | null =>
  row.provisional?.state ?? row.accepted?.state ?? null;

export interface PatientMutationResult {
  operationId: string;
  entityType: string;
  entityKey: unknown;
  expectedVersion: number;
  dependsOn: string[];
  provisional: unknown;
}

export interface PatientEditBase {
  acceptedVersion?: number;
  operationId?: string;
}

export const updatePatientProfile = (request: {
  patientId: string;
  base: PatientEditBase;
  patch: {
    fullName?: string | null;
    birthDate?: string | null;
    phoneCode?: string | null;
    phone?: string | null;
    identificationBasis?: string | null;
  };
}) => invoke<PatientMutationResult>("patients_update_patient_profile", { request });

export const setPatientProfileStatus = (request: {
  patientId: string;
  base: PatientEditBase;
  status: string;
}) => invoke<PatientMutationResult>("patients_set_patient_profile_status", { request });

export const addPatientIdentifier = (request: {
  patientId: string;
  id: string;
  issuingCountry?: string;
  documentType: string;
  documentNumber: string;
}) => invoke<PatientMutationResult>("patients_add_patient_identifier", { request });

export const baseOfPatient = (
  row: EntityRow<ProfileState>,
): PatientEditBase =>
  row.provisional
    ? { operationId: row.provisional.operationId }
    : { acceptedVersion: row.accepted!.version };

export const registerPatient = (request: {
  fullName?: string;
  birthDate?: string;
  phoneCode?: string;
  phone?: string;
  identificationBasis?: string;
  identifiers?: { id: string; issuingCountry?: string; documentType: string; documentNumber: string }[];
}) => invoke<PatientMutationResult>("patients_register_patient", { request });

export const createContactPerson = (request: {
  fullName: string;
  phoneCode?: string;
  phone?: string;
  issuingCountry?: string;
  documentType?: string;
  documentNumber?: string;
}) => invoke<PatientMutationResult>("patients_create_contact_person", { request });

export const createPatientContactRelationship = (request: {
  patientId: string;
  contactPersonId: string;
  relationshipKind: string;
  isResponsibleContact?: boolean;
  validFrom: string;
  validUntil?: string;
}) => invoke<PatientMutationResult>("patients_create_patient_contact_relationship", { request });

export interface ContactState {
  contactPersonId: string;
  fullName: string;
  phoneCode: string | null;
  phone: string | null;
  issuingCountry: string | null;
  documentType: string | null;
  documentNumber: string | null;
  status: string;
}

export const listContacts = (request: { q?: string; status?: string; page?: number; shows?: number }) =>
  invoke<{ items: EntityRow<ContactState>[]; total: number }>("patients_list_contacts", { request });

export const listPatientContacts = (request: { patientId: string; status?: string }) =>
  invoke<EntityRow<RelationshipView>[]>("patients_list_patient_contacts", { request });
