import { useState } from "react";
import { getDiagnostics, type Diagnostics } from "../shared/api/diagnostics";
import { getStorageStatus, type StorageStatus } from "../shared/api/storage";
import { getPendingOperations, type PendingOperations } from "../shared/api/sync";
import CorePanel from "../CorePanel";
import SettingsPanel from "../features/settings/SettingsPanel";
import ServicesPanel from "../features/services/ServicesPanel";
import SchedulingPanel from "../features/scheduling/SchedulingPanel";
import AppointmentsPanel from "../features/appointments/AppointmentsPanel";

type DiagnosticState =
  | { status: "idle" | "loading" }
  | {
      status: "ready";
      data: Diagnostics;
      storage: StorageStatus;
      pending: PendingOperations;
    }
  | { status: "error" };

/** Esqueleto técnico original: acceso, sincronización y diagnóstico.
 *  Se conserva completo en /desarrollo fuera del trabajo diario. */
export default function DevelopmentPage() {
  const [diagnostic, setDiagnostic] = useState<DiagnosticState>({ status: "idle" });

  async function loadDiagnostics() {
    setDiagnostic({ status: "loading" });

    try {
      const [data, storage, pending] = await Promise.all([
        getDiagnostics(),
        getStorageStatus(),
        getPendingOperations(),
      ]);
      setDiagnostic({ status: "ready", data, storage, pending });
    } catch {
      setDiagnostic({ status: "error" });
    }
  }

  return (
    <div className="dev-panel" style={{ maxWidth: 768 }}>
      <section aria-labelledby="diagnostics-heading" className="space-y-4">
        <h2 id="diagnostics-heading" className="text-xl font-semibold">
          Diagnóstico local
        </h2>
        <p>Comprueba la comunicación entre esta interfaz y el proceso Rust.</p>
        <button
          type="button"
          onClick={loadDiagnostics}
          disabled={diagnostic.status === "loading"}
        >
          {diagnostic.status === "loading" ? "Consultando…" : "Consultar diagnóstico"}
        </button>
        <p role="status">
          {diagnostic.status === "idle" && "Sin consultar."}
          {diagnostic.status === "loading" && "Consultando el proceso local…"}
          {diagnostic.status === "ready" && "Diagnóstico recibido desde Rust."}
        </p>

        {diagnostic.status === "error" && (
          <p role="alert">
            No se pudo consultar el diagnóstico local. Comprueba que usas la
            aplicación de escritorio y vuelve a intentarlo.
          </p>
        )}

        {diagnostic.status === "ready" && (
          <dl className="grid gap-4 sm:grid-cols-3">
            <div>
              <dt className="text-sm text-gray-600">Versión de la app</dt>
              <dd className="font-mono">{diagnostic.data.appVersion}</dd>
            </div>
            <div>
              <dt className="text-sm text-gray-600">Sistema operativo</dt>
              <dd className="font-mono">{diagnostic.data.platform}</dd>
            </div>
            <div>
              <dt className="text-sm text-gray-600">Arquitectura</dt>
              <dd className="font-mono">{diagnostic.data.architecture}</dd>
            </div>
            <div>
              <dt className="text-sm text-gray-600">Versión del esquema local</dt>
              <dd className="font-mono">{diagnostic.storage.schemaVersion}</dd>
            </div>
            <div>
              <dt className="text-sm text-gray-600">Identificador de aplicación</dt>
              <dd className="font-mono">{diagnostic.storage.applicationId}</dd>
            </div>
            <div>
              <dt className="text-sm text-gray-600">Operaciones pendientes de sincronizar</dt>
              <dd className="font-mono">{diagnostic.pending.total}</dd>
            </div>
          </dl>
        )}
      </section>

      <CorePanel />
      <SettingsPanel />
      <ServicesPanel />
      <SchedulingPanel />
      <AppointmentsPanel />
      <section aria-labelledby="scope-heading" className="space-y-2 border-t border-gray-200 pt-6">
        <h2 id="scope-heading" className="text-xl font-semibold">
          Funciones pendientes
        </h2>
        <p>La gestión de pacientes tendrá su propio plan adicional.</p>
      </section>
    </div>
  );
}
