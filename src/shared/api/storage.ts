import { invoke } from "@tauri-apps/api/core";

export interface StorageStatus {
  schemaVersion: number;
  applicationId: number;
}

export function getStorageStatus(): Promise<StorageStatus> {
  return invoke<StorageStatus>("storage_get_status");
}
