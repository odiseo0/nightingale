import { invoke } from "@tauri-apps/api/core";

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
