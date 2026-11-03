import { invoke } from "@tauri-apps/api/core";
export { getPreferences as getCatalogPreferences } from "./account";
export { syncNow as syncCatalog } from "./identity";

export type CatalogStatus = "active" | "inactive";
export interface ServiceView {
  serviceId: string; professionalId: string; name: string;
  defaultDurationMinutes: number; status: CatalogStatus; version: number;
}
export interface ServiceOfferView {
  professionalId: string; serviceId: string; name: string;
  defaultDurationMinutes: number; durationMinutesOverride: number | null;
  effectiveDurationMinutes: number; serviceStatus: CatalogStatus;
  status: CatalogStatus; version: number; serviceVersion: number;
}
export interface ProfessionalView {
  professionalId: string; userId: string; publicName: string; status: CatalogStatus; version: number;
}
export type ServiceDraft = Omit<ServiceView, "version">;
export type ServiceOfferDraft = Pick<ServiceOfferView, "professionalId" | "serviceId" | "durationMinutesOverride" | "status">;
export type ProfessionalDraft = Pick<ProfessionalView, "professionalId" | "publicName">;
export interface LocalChange {
  operationId: string; action: string; expectedVersion: number;
  syncStatus: "pending" | "in_flight" | "unknown" | "retryable" | "accepted" | "conflict" | "rejected" | "review_required";
  code: string | null;
}
export interface CatalogRow<T, D> {
  accepted: T | null; provisional: D | null; isProvisional: boolean; localChanges: LocalChange[];
}
export interface CatalogScreen {
  services: CatalogRow<ServiceView, ServiceDraft>[];
  offers: CatalogRow<ServiceOfferView, ServiceOfferDraft>[];
  professionals: CatalogRow<ProfessionalView, ProfessionalDraft>[];
  canManage: boolean;
}
export interface LocalMutationResult {
  operationId: string; entityType: "service" | "service_offer" | "professional";
  entityKey: string | { professionalId: string; serviceId: string }; syncStatus: "pending";
}
export type ServicePatch = Partial<Pick<ServiceView, "name" | "defaultDurationMinutes">>;

export const listCatalog = () => invoke<CatalogScreen>("services_list_catalog");
export const createService = (request: { professionalId: string; name: string; defaultDurationMinutes: number }) =>
  invoke<LocalMutationResult>("services_create_service", { request });
export const updateService = (request: { serviceId: string; expectedVersion: number; patch: ServicePatch }) =>
  invoke<LocalMutationResult>("services_update_service", { request });
export const setServiceStatus = (request: { serviceId: string; expectedVersion: number; status: CatalogStatus }) =>
  invoke<LocalMutationResult>("services_set_service_status", { request });
export const offerService = (request: { professionalId: string; serviceId: string; durationMinutesOverride?: number | null }) =>
  invoke<LocalMutationResult>("services_offer_service", { request });
export const updateServiceOffer = (request: { professionalId: string; serviceId: string; expectedVersion: number; durationMinutesOverride: number | null }) =>
  invoke<LocalMutationResult>("services_update_service_offer", { request });
export const setServiceOfferStatus = (request: { professionalId: string; serviceId: string; expectedVersion: number; status: CatalogStatus }) =>
  invoke<LocalMutationResult>("services_set_service_offer_status", { request });
export const updateProfessionalProfile = (request: { professionalId: string; expectedVersion: number; publicName: string }) =>
  invoke<LocalMutationResult>("services_update_professional_profile", { request });
