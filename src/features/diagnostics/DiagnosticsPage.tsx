import { useCallback, useEffect, useState } from "react";
import { CircleNotch, Info } from "@phosphor-icons/react";
import { getDiagnostics } from "../../shared/api/diagnostics";
import { getStorageStatus } from "../../shared/api/storage";
import { getCoreStatus } from "../../shared/api/identity";
import { getPendingOperations } from "../../shared/api/sync";
import type { Diagnostics } from "../../shared/api/diagnostics";
import type { StorageStatus } from "../../shared/api/storage";
import type { CoreStatus } from "../../shared/api/identity";
import type { PendingOperations } from "../../shared/api/sync";

type State = "loading" | "ready" | "error";

/** S13 · Diagnóstico de desarrollo: metadatos locales permitidos. No prueba
 * conexión con la API ni aceptación remota y no muestra secretos. */
export default function DiagnosticsPage() {
  const [state, setState] = useState<State>("loading");
  const [diagnostics, setDiagnostics] = useState<Diagnostics | null>(null);
  const [storage, setStorage] = useState<StorageStatus | null>(null);
  const [core, setCore] = useState<CoreStatus | null>(null);
  const [pending, setPending] = useState<PendingOperations | null>(null);

  const load = useCallback(async () => {
    setState("loading");
    try {
      const [diag, store, coreStatus, queue] = await Promise.all([
        getDiagnostics(), getStorageStatus(), getCoreStatus(), getPendingOperations(),
      ]);
      setDiagnostics(diag);
      setStorage(store);
      setCore(coreStatus);
      setPending(queue);
      setState("ready");
    } catch {
      setState("error");
    }
  }, []);

  useEffect(() => { void load(); }, [load]);

  if (state === "loading") {
    return (
      <div className="state-block" role="status">
        <CircleNotch size={22} aria-hidden="true" />
        <p>Consultando el proceso local…</p>
      </div>
    );
  }

  if (state === "error") {
    return (
      <div className="state-block" role="alert">
        <p>No se pudo consultar el diagnóstico local.</p>
        <p className="muted">Comprueba que usas la aplicación de escritorio y vuelve a intentarlo.</p>
        <button type="button" className="btn" onClick={() => void load()}>Reintentar</button>
      </div>
    );
  }

  return (
    <>
      <div className="page-header">
        <div>
          <h1 className="page-heading">Diagnóstico de desarrollo</h1>
          <p className="muted">Metadatos técnicos locales de esta instalación.</p>
        </div>
        <button type="button" className="btn" onClick={() => void load()}>Actualizar</button>
      </div>

      <div className="notice">
        <Info size={18} aria-hidden="true" />
        <p>
          Estos datos son locales: describen esta aplicación y su almacenamiento. No prueban que la API
          esté disponible, que la autenticación funcione ni que operaciones pendientes hayan sido aceptadas.
        </p>
      </div>

      <section className="card" style={{ padding: 16 }} aria-labelledby="app-titulo">
        <h2 id="app-titulo" style={{ fontSize: 18, marginBottom: 12 }}>Aplicación</h2>
        <dl className="fact-grid">
          <div><dt>Versión de la app</dt><dd className="num">{diagnostics?.appVersion ?? "—"}</dd></div>
          <div><dt>Sistema operativo</dt><dd className="num">{diagnostics?.platform ?? "—"}</dd></div>
          <div><dt>Arquitectura</dt><dd className="num">{diagnostics?.architecture ?? "—"}</dd></div>
          <div><dt>Versión del esquema local</dt><dd className="num">{storage?.schemaVersion ?? "—"}</dd></div>
          <div><dt>Identificador de aplicación</dt><dd className="num">{storage?.applicationId ?? "—"}</dd></div>
        </dl>
      </section>

      <section className="card" style={{ padding: 16 }} aria-labelledby="sesion-titulo">
        <h2 id="sesion-titulo" style={{ fontSize: 18, marginBottom: 12 }}>Sesión y sincronización local</h2>
        <dl className="fact-grid">
          <div>
            <dt>Sesión</dt>
            <dd>{core?.signedIn ? (core.accountId ? "Con cuenta activa" : "Sesión iniciada sin cuenta") : "Sin sesión"}</dd>
          </div>
          <div>
            <dt>Acceso</dt>
            <dd>{core?.access ?? "desconocido"}</dd>
          </div>
          <div>
            <dt>Resultado del último ciclo</dt>
            <dd>{core?.outcome ?? "sin ciclo en esta ejecución"}</dd>
          </div>
          <div>
            <dt>Operaciones esperando envío</dt>
            <dd className="num">{pending?.total ?? 0} (las pendientes no son un error)</dd>
          </div>
          <div>
            <dt>Último intercambio en esta ejecución</dt>
            <dd className="num">
              {core?.lastSyncAt ? new Date(core.lastSyncAt).toISOString() : "ninguno"}
            </dd>
          </div>
        </dl>
      </section>
    </>
  );
}
