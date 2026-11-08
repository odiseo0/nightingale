import { invoke } from "@tauri-apps/api/core";

export type AppointmentStatus =
  | "scheduled"
  | "confirmed"
  | "cancelled"
  | "completed"
  | "no_show";
export type AppointmentSyncStatus =
  | "pending"
  | "in_flight"
  | "unknown"
  | "retryable"
  | "accepted"
  | "conflict"
  | "rejected"
  | "review_required";

export interface Accepted<T> { state: T; version: number }
export interface Provisional<T> { state: T; operationId: string; expectedVersion: number }
export interface OperationView {
  operationId: string;
  dependsOn: string[];
  status: AppointmentSyncStatus;
  error: string | null;
}
export interface EntityRow<T> {
  accepted: Accepted<T> | null;
  provisional: Provisional<T> | null;
  operations: OperationView[];
}

/** Fused view: the provisional state wins while one exists. */
export const visibleOf = <T,>(row: EntityRow<T>): T | null =>
  row.provisional?.state ?? row.accepted?.state ?? null;

export interface AppointmentState {
  appointmentId: string;
  patientId: string;
  professionalId: string;
  serviceId: string;
  responsibleContactId: string | null;
  startsAt: string;
  endsAt: string;
  status: AppointmentStatus;
  confirmedAt: string | null;
  serviceName: string;
  offeredDurationMinutes: number;
  professionalName: string;
  locationName: string;
  locationAddress: string;
  locationCountry: string;
  locationState: string;
  locationCity: string;
  timeZone: string;
}

export interface AppointmentError {
  code: string;
  message: string;
}

export interface EditBase {
  acceptedVersion?: number;
  operationId?: string;
}

export interface LocalMutationResult {
  operationId: string;
  entityType: string;
  entityKey: unknown;
  expectedVersion: number;
  dependsOn: string[];
  provisional: unknown;
  provisionalOperationId: string;
}

// ---- Writes ----

export interface CreateAppointmentRequest {
  patientId: string;
  professionalId: string;
  serviceId: string;
  responsibleContactId?: string;
  startsAt: string;
  endsAt: string;
}

export interface RescheduleAppointmentRequest {
  appointmentId: string;
  base: EditBase;
  startsAt: string;
  endsAt: string;
}

export interface LifecycleAppointmentRequest {
  appointmentId: string;
  base: EditBase;
  changeReason?: string;
}

export const createAppointment = (request: CreateAppointmentRequest) =>
  invoke<LocalMutationResult>("appointments_create_appointment", { request });

export const confirmAppointment = (appointmentId: string, base: EditBase) =>
  invoke<LocalMutationResult>("appointments_confirm_appointment", {
    request: { appointmentId, base },
  });

export const rescheduleAppointment = (request: RescheduleAppointmentRequest) =>
  invoke<LocalMutationResult>("appointments_reschedule_appointment", { request });

export const cancelAppointment = (request: LifecycleAppointmentRequest) =>
  invoke<LocalMutationResult>("appointments_cancel_appointment", { request });

export const completeAppointment = (request: LifecycleAppointmentRequest) =>
  invoke<LocalMutationResult>("appointments_complete_appointment", { request });

export const markNoShow = (request: LifecycleAppointmentRequest) =>
  invoke<LocalMutationResult>("appointments_mark_no_show", { request });

// ---- Queries ----

export interface ListAppointmentsRequest {
  from: string;
  until: string;
  professionalId?: string;
  serviceId?: string;
  patientId?: string;
  status?: AppointmentStatus;
  page?: number;
  shows?: number;
}

export interface AppointmentPage {
  items: EntityRow<AppointmentState>[];
  total: number;
}

export const listAppointments = (request: ListAppointmentsRequest) =>
  invoke<AppointmentPage>("appointments_list_appointments", { request });

export const getAppointment = (appointmentId: string) =>
  invoke<EntityRow<AppointmentState>>("appointments_get_appointment", {
    request: { appointmentId },
  });

export interface HistoryEntry {
  appointmentId: string;
  version: number;
  action: string;
  actorKind: string;
  actorUserId: string | null;
  actorContactPersonId: string | null;
  changeReason: string | null;
  changedAt: string;
  startsAt: string;
  endsAt: string;
  status: AppointmentStatus;
  confirmedAt: string | null;
  serviceNameSnapshot: string;
  offeredDurationMinutesSnapshot: number;
  professionalNameSnapshot: string;
  locationNameSnapshot: string;
  locationAddressSnapshot: string;
  locationCountrySnapshot: string;
  locationStateSnapshot: string;
  locationCitySnapshot: string;
  timeZoneSnapshot: string;
}

export interface HistoryView {
  accepted: HistoryEntry[];
  operations: OperationView[];
}

export const getHistory = (appointmentId: string, afterVersion?: number) =>
  invoke<HistoryView>("appointments_get_history", {
    request: { appointmentId, afterVersion },
  });

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

export interface ServiceOfferView {
  professionalId: string;
  serviceId: string;
  name: string;
  defaultDurationMinutes: number;
  durationMinutesOverride: number | null;
  effectiveDurationMinutes: number;
  serviceStatus: string;
  status: string;
  version: number;
  serviceVersion: number;
}

export interface ServiceOfferDraft {
  professionalId: string;
  serviceId: string;
  durationMinutesOverride: number | null;
  status: string;
}

export interface ProfessionalView {
  professionalId: string;
  userId: string;
  publicName: string;
  status: string;
  version: number;
}

export interface ProfessionalDraft {
  professionalId: string;
  publicName: string;
}

export interface LocalChange {
  operationId: string;
  action: string;
  expectedVersion: number;
  syncStatus: string;
  code: string | null;
}

export interface CatalogRow<Accepted, Draft> {
  accepted: Accepted | null;
  provisional: Draft | null;
  isProvisional: boolean;
  localChanges: LocalChange[];
}

export interface BookingContext {
  timeZone: string;
  noShowGraceMinutes: number;
  patients: EntityRow<ProfileState>[];
  offers: CatalogRow<ServiceOfferView, ServiceOfferDraft>[];
  professionals: CatalogRow<ProfessionalView, ProfessionalDraft>[];
}

export const getBookingContext = () =>
  invoke<BookingContext>("appointments_get_booking_context");
