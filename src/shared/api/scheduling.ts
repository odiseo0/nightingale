import { invoke } from "@tauri-apps/api/core";

export type ScheduleStatus = "active" | "inactive";
export type ScheduleSyncStatus =
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
  status: ScheduleSyncStatus;
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

export interface RuleState {
  ruleId: string;
  professionalId: string;
  isoWeekday: number;
  localStartMinute: number;
  localEndMinute: number;
  validFrom: string;
  validUntil: string | null;
  status: ScheduleStatus;
}

export type ExceptionKind = "closed" | "replacement";
export interface ExceptionState {
  exceptionId: string;
  professionalId: string;
  localDate: string;
  kind: ExceptionKind;
  localStartMinute: number | null;
  localEndMinute: number | null;
  status: ScheduleStatus;
}

export type BlockReasonCode = "vacation" | "administrative" | "unavailable";
export interface BlockState {
  blockId: string;
  professionalId: string;
  blockAccountId: string | null;
  startsAt: string;
  endsAt: string;
  reasonCode: BlockReasonCode | null;
  status: ScheduleStatus;
}

export interface LocalMutationResult {
  operationId: string;
  entityType: string;
  entityKey: unknown;
  expectedVersion: number;
  dependsOn: string[];
  provisional: unknown;
}

export interface SchedulingCommandError { code: string; message: string }

export interface EditBase {
  /** Positive accepted version; exclusive with operationId. */
  acceptedVersion?: number;
  /** Last local operation id; exclusive with acceptedVersion. */
  operationId?: string;
}

export type RulePatch = Partial<{
  isoWeekday: number;
  localStartMinute: number;
  localEndMinute: number;
  validFrom: string;
  /** Explicit null withdraws the open end; absent keeps it. */
  validUntil: string | null;
}>;

export const createRule = (request: {
  professionalId: string;
  isoWeekday: number;
  localStartMinute: number;
  localEndMinute: number;
  validFrom: string;
  validUntil?: string;
}) => invoke<LocalMutationResult>("scheduling_create_rule", { request });

export const updateRule = (request: {
  ruleId: string;
  base: EditBase;
  patch: RulePatch;
}) => invoke<LocalMutationResult>("scheduling_update_rule", { request });

export const setRuleStatus = (request: {
  ruleId: string;
  base: EditBase;
  status: ScheduleStatus;
}) => invoke<LocalMutationResult>("scheduling_set_rule_status", { request });

export type ExceptionPatch = Partial<{
  kind: ExceptionKind;
  localStartMinute: number;
  localEndMinute: number;
}>;

export const createException = (request: {
  professionalId: string;
  localDate: string;
  kind: ExceptionKind;
  localStartMinute?: number;
  localEndMinute?: number;
}) => invoke<LocalMutationResult>("scheduling_create_exception", { request });

export const updateException = (request: {
  exceptionId: string;
  base: EditBase;
  patch: ExceptionPatch;
}) => invoke<LocalMutationResult>("scheduling_update_exception", { request });

export const setExceptionStatus = (request: {
  exceptionId: string;
  base: EditBase;
  status: ScheduleStatus;
}) => invoke<LocalMutationResult>("scheduling_set_exception_status", { request });

export type BlockPatch = Partial<{
  startsAt: string;
  endsAt: string;
  reasonCode: BlockReasonCode;
}>;

export const createBlock = (request: {
  professionalId: string;
  /** Absent or null means a global block (owner + own professional only). */
  accountId?: string | null;
  startsAt: string;
  endsAt: string;
  reasonCode: BlockReasonCode;
}) => invoke<LocalMutationResult>("scheduling_create_block", { request });

export const updateBlock = (request: {
  blockId: string;
  base: EditBase;
  patch: BlockPatch;
}) => invoke<LocalMutationResult>("scheduling_update_block", { request });

export const setBlockStatus = (request: {
  blockId: string;
  base: EditBase;
  status: ScheduleStatus;
}) => invoke<LocalMutationResult>("scheduling_set_block_status", { request });

export const listRules = () => invoke<EntityRow<RuleState>[]>("scheduling_list_rules");
export const listExceptions = () => invoke<EntityRow<ExceptionState>[]>("scheduling_list_exceptions");
export const listBlocks = () => invoke<EntityRow<BlockState>[]>("scheduling_list_blocks");

export type AvailabilityCoverage =
  | "within_synced_window"
  | "no_synced_window"
  | "insufficient_coverage";

export interface AvailabilityEvaluation {
  timeZone: string;
  referenceDurationMinutes: number;
  requestedDurationMinutes: number;
  accountVersion: number;
  offerVersion: number;
  catalogVersion: number;
  agendaVersions: [string, number][];
  intervals: { startsAt: string; endsAt: string }[];
  origin: "local";
  provisional: boolean;
  coverage: AvailabilityCoverage;
}

export const getLocalAvailability = (request: {
  professionalId: string;
  serviceId: string;
  from: string;
  until: string;
  durationMinutes?: number;
}) => invoke<AvailabilityEvaluation>("scheduling_get_local_availability", { request });

export interface RemoteAvailability {
  accountId: string;
  professionalId: string;
  serviceId: string;
  referenceDurationMinutes: number;
  requestedDurationMinutes: number;
  intervals: { startsAt: string; endsAt: string }[];
  origin: "remote";
}

/**
 * Current API view of availability. It refreshes access, never reserves and
 * never becomes offline data; on error the local evaluation stays valid.
 */
export const checkRemoteAvailability = (request: {
  professionalId: string;
  serviceId: string;
  from: string;
  until: string;
  durationMinutes?: number;
}) => invoke<RemoteAvailability>("scheduling_check_remote_availability", { request });
