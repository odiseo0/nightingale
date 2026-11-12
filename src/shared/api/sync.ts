import { invoke } from "@tauri-apps/api/core";

export interface ReviewSummary {
  operationId: string;
  status: string;
  code: string | null;
  action: string;
  entityType: string;
  entityKey: unknown;
  createdAtMillis: number;
}

export interface PendingOperation {
  operationId: string;
  action: string;
  entityType: string;
  expectedVersion: number;
  occurredAtLocal: string | null;
  createdAtMillis: number;
}

export interface PendingOperations {
  total: number;
  operations: PendingOperation[];
}

export function getPendingOperations(): Promise<PendingOperations> {
  return invoke<PendingOperations>("sync_list_pending");
}
