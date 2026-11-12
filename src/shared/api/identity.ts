import { invoke } from "@tauri-apps/api/core";
import type { ReviewSummary } from "./sync";

export interface AccountChoice { id: string; displayName: string; role: string; status: string }
export interface CoreStatus {
  lastSyncAt: number | null;
  reviews: ReviewSummary[];
  configured: boolean;
  signedIn: boolean;
  accountId: string | null;
  access: "active" | "inactive" | "revoked" | "registration_required" | null;
  outcome: "complete" | "offline" | "retry_waiting" | "access_denied" | "recovery_required" | "failed" | null;
  sync: {
    phase: "idle" | "access" | "recovering" | "sending" | "downloading" | "rebuilding";
    cursorReady: boolean;
    snapshotPending: boolean;
    outbox: { pending: number; inFlight: number; unknown: number; retryable: number; conflicts: number; rejected: number; reviewRequired: number; otherEpoch: number; nextAttemptAt: number | null };
  } | null;
}

export const getCoreStatus = () => invoke<CoreStatus>("identity_get_status");
export const beginLogin = () => invoke<void>("identity_begin_login");
export const completeLogin = () => invoke<AccountChoice[]>("identity_complete_login");
export const cancelLogin = () => invoke<void>("identity_cancel_login");
export const listAccounts = () => invoke<AccountChoice[]>("identity_list_accounts");
export const activateAccount = (accountId: string) => invoke<void>("identity_activate_account", { accountId });
export const logout = () => invoke<void>("identity_logout");
export const syncNow = () => invoke<unknown>("sync_now");
