import { invoke } from "@tauri-apps/api/core";

export interface Diagnostics {
  appVersion: string;
  platform: string;
  architecture: string;
}

export function getDiagnostics(): Promise<Diagnostics> {
  return invoke<Diagnostics>("get_diagnostics");
}
