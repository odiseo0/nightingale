import { invoke } from "@tauri-apps/api/core";

export type AccountStatus = "active" | "inactive";
export type Theme = "system" | "light" | "dark";
export interface Preferences { theme: Theme; servicesShowInactive: boolean }
export interface AccountView {
  accountId: string;
  displayName: string;
  timeZone: string;
  locationName: string;
  locationAddress: string;
  locationCity: string;
  locationState: string;
  locationCountry: string;
  policy: { noShowGraceMinutes: number };
  status: AccountStatus;
  version: number;
  updatedAt: string | null;
}
export interface AccountScreen { viewerRole: "owner" | "assistant"; account: AccountView | null; preferences: Preferences; canManage: boolean }
export type AccountDetailsPatch = Partial<Pick<AccountView,
  "displayName" | "timeZone" | "locationName" | "locationAddress" | "locationCity" | "locationState" | "locationCountry"
>>;
export interface AccountMutationResult {
  operationId: string;
  status: "accepted" | "conflict" | "rejected" | "retryable_failure";
  entityVersion: number | null;
  code: string | null;
  refreshStatus: "requested" | "failed" | "not_requested";
}
export interface AccountCommandError { code: string; message: string; operationId?: string; correlationId?: string }

export const getAccountView = () => invoke<AccountScreen>("account_get_view");
export const getPreferences = () => invoke<Preferences>("account_get_preferences");
export const setPreference = (request: { key: "theme"; value: Theme } | { key: "servicesShowInactive"; value: boolean }) =>
  invoke<Preferences>("account_set_preference", { request });
export const updateAccountDetails = (request: { expectedVersion: number; patch: AccountDetailsPatch }) =>
  invoke<AccountMutationResult>("account_update_details", { request });
export const updateAccountPolicy = (request: { expectedVersion: number; noShowGraceMinutes: number }) =>
  invoke<AccountMutationResult>("account_update_policy", { request });
export const setAccountStatus = (request: { expectedVersion: number; status: AccountStatus }) =>
  invoke<AccountMutationResult>("account_set_status", { request });
