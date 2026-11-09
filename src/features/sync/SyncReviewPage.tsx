import { useCallback, useEffect, useRef, useState } from "react";
import { Link } from "react-router-dom";
import { CircleNotch, ArrowsClockwise } from "@phosphor-icons/react";
import { getAccountView } from "../../shared/api/account";
import {
  getCoreStatus, syncNow, type CoreStatus,
} from "../../shared/api/identity";
import { getPendingOperations, type PendingOperations } from "../../shared/api/sync";
import { listAppointments, type AppointmentState, type EntityRow, visibleOf } from "../../shared/api/appointments";
import { civilToday, dayRange, formatDateTime, formatTime, formatTimeEnd } from "../appointments/presentation";
import { actionLabel, entityLabel, outcomeLabel, syncWorkLabel } from "./presentation";

type Phase = "loading" | "ready" | "no-session" | "error";

/** S11 · Centro de cambios: pendientes, resultados desconocidos, reintentos,
 * rechazos y conflictos con vocabulario de trabajo y recuperación segura. */
export default function SyncReviewPage() {
  const [phase, setPhase] = useState<Phase>("loading");
  const [core, setCore] = useState<CoreStatus | null>(null);
  const [pending, setPending] = useState<PendingOperations | null>(null);
  const [conflictLinks, setConflictLinks] = useState<Map<string, string>>(new Map());
  const [conflictRows, setConflictRows] = useState<Map<string, EntityRow<AppointmentState>>>(new Map());
  const [syncing, setSyncing] = useState(false);
  const [notice, setNotice] = useState<string | null>(null);
  const [zone, setZone] = useState("UTC");
  const generation = useRef(0);

  useEffect(() => {
    let cancelled = false;
    getAccountView()
      .then((screen) => { if (!cancelled && screen.account) setZone(screen.account.timeZone); })
      .catch(() => { /* zona por defecto */ });
    return () => { cancelled = true; };
  }, []);

  const load = useCallback(async () => {
    const request = ++generation.current;
    try {
      const [status, queue] = await Promise.all([getCoreStatus(), getPendingOperations()]);
      if (request !== generation.current) return;
      setCore(status);
      setPending(queue);
      setPhase(status.signedIn ? "ready" : "no-session");
      /* Resolver enlaces de revisiones a citas locales conocidas. */
      if (status.reviews.length > 0) {
        const today = civilToday("UTC");
        const from = dayRange("UTC", today).from;
        const page = await listAppointments({
          from: new Date(Date.parse(from) - 365 * 86_400_000).toISOString(),
          until: new Date(Date.parse(from) + 60 * 86_400_000).toISOString(),
          shows: 500,
        });
        if (request !== generation.current) return;
        const links = new Map<string, string>();
        const rows = new Map<string, EntityRow<AppointmentState>>();
        for (const row of page.items) {
          for (const op of row.operations) {
            if (status.reviews.some((review) => review.operationId === op.operationId)) {
              const state = visibleOf(row);
              if (state) {
                links.set(op.operationId, state.appointmentId);
                rows.set(op.operationId, row);
              }
            }
          }
        }
        setConflictLinks(links);
        setConflictRows(rows);
      }
    } catch {
      if (request === generation.current) setPhase("error");
    }
  }, []);

  useEffect(() => {
    void load();
    const timer = window.setInterval(() => void load(), 10_000);
    return () => { generation.current += 1; window.clearInterval(timer); };
  }, [load]);

  async function runSync() {
    if (syncing) return;
    setSyncing(true);
    setNotice(null);
    try {
      await syncNow();
      setNotice("Sincronización solicitada; los resultados se muestran según se conozcan.");
    } catch {
      setNotice("No se pudo iniciar la sincronización ahora; el trabajo local se conserva.");
    } finally {
      setSyncing(false);
      void load();
    }
  }

  if (phase === "loading") {
    return (
      <div className="state-block" role="status">
        <CircleNotch size={22} aria-hidden="true" />
        <p>Preparando el centro de cambios…</p>
      </div>
    );
  }

  if (phase === "no-session") {
    return (
      <div className="state-block" role="alert">
        <p>No hay una sesión lista en este equipo.</p>
        <p className="muted">Las operaciones locales se conservan; activa una cuenta para sincronizar.</p>
      </div>
    );
  }

  if (phase === "error") {
    return (
      <div className="state-block" role="alert">
        <p>No se pudo leer el estado local de cambios.</p>
        <button type="button" className="btn" onClick={() => void load()}>Reintentar</button>
      </div>
    );
  }

  const outbox = core?.sync?.outbox ?? null;
  const workPending = outbox ? outbox.pending + outbox.retryable + outbox.unknown + outbox.inFlight : 0;
  const attention = outbox ? outbox.conflicts + outbox.rejected + outbox.reviewRequired + outbox.otherEpoch : 0;

  return (
    <>
      <div className="page-header">
        <div>
          <h1 className="page-heading">Cambios por revisar</h1>
          <p className="muted">
            {core?.lastSyncAt
              ? `Último intercambio en esta ejecución: ${formatDateTime("UTC", new Date(core.lastSyncAt).toISOString())} (UTC)`
              : "Sin intercambio completado en esta ejecución; los datos locales se conservan."}
            {" · "}{core ? outcomeLabel[core.outcome ?? ""] ?? "Estado de acceso disponible" : ""}
          </p>
        </div>
        <button type="button" className="btn" disabled={syncing} onClick={() => void runSync()}>
          <ArrowsClockwise size={16} aria-hidden="true" />
          {syncing ? "Sincronizando…" : "Sincronizar ahora"}
        </button>
      </div>

      {notice && <div className="notice" role="status"><p>{notice}</p></div>}

      <div className="toolbar" role="list" aria-label="Resumen de cambios" style={{ alignItems: "stretch" }}>
        <div className="card" role="listitem" style={{ padding: "12px 16px", minWidth: 180 }}>
          <p className="muted" style={{ fontSize: 13 }}>Pendientes de aceptación</p>
          <p className="num" style={{ fontSize: 22, fontWeight: 600 }}>{workPending}</p>
          <p className="muted" style={{ fontSize: 13 }}>incluye resultados por comprobar y reintentos</p>
        </div>
        <div className="card" role="listitem" style={{ padding: "12px 16px", minWidth: 180 }}>
          <p className="muted" style={{ fontSize: 13 }}>Necesitan revisión</p>
          <p className="num" style={{ fontSize: 22, fontWeight: 600 }}>{attention}</p>
          <p className="muted" style={{ fontSize: 13 }}>conflictos, rechazos y revisiones del ámbito de esta cuenta</p>
        </div>
        <div className="card" role="listitem" style={{ padding: "12px 16px", minWidth: 180 }}>
          <p className="muted" style={{ fontSize: 13 }}>Fase de sincronización</p>
          <p style={{ fontSize: 16, fontWeight: 600 }}>
            {core?.sync && core.sync.phase !== "idle" ? "Activa" : "En reposo"}
          </p>
          <p className="muted" style={{ fontSize: 13 }}>
            {core?.access === "active" ? "Acceso activo con últimos permisos válidos." : "Acceso no activo; revisa la sesión en Desarrollo."}
          </p>
        </div>
      </div>

      <section aria-labelledby="cola-titulo">
        <h2 id="cola-titulo" style={{ fontSize: 18, marginBottom: 8 }}>Cola de envío</h2>
        {!pending || pending.operations.length === 0 ? (
          <p className="muted">No hay operaciones esperando envío en este equipo.</p>
        ) : (
          <div className="table-scroll">
            <table className="data-table">
              <thead>
                <tr>
                  <th scope="col">Cambio</th>
                  <th scope="col">Entidad</th>
                  <th scope="col">Fecha local</th>
                </tr>
              </thead>
              <tbody>
                {pending.operations.map((operation) => (
                  <tr key={operation.operationId}>
                    <td>{actionLabel[operation.action] ?? operation.action}</td>
                    <td>{entityLabel[operation.entityType] ?? operation.entityType}</td>
                    <td className="num">
                      {operation.occurredAtLocal
                        ? operation.occurredAtLocal
                        : new Date(operation.createdAtMillis).toLocaleString("es")}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </section>

      <section aria-labelledby="revision-titulo">
        <h2 id="revision-titulo" style={{ fontSize: 18, marginBottom: 8 }}>Requieren revisión</h2>
        {!core || core.reviews.length === 0 ? (
          <p className="muted">No hay operaciones que requieran revisión en los datos conocidos.</p>
        ) : (
          <ul style={{ margin: 0, paddingLeft: 20, display: "grid", gap: 10 }}>
            {core.reviews.map((review) => {
              const appointmentId = conflictLinks.get(review.operationId);
              const row = appointmentId ? conflictRows.get(review.operationId) : undefined;
              const accepted = row?.accepted?.state ?? null;
              const proposed = row?.provisional?.state ?? null;
              return (
                <li key={review.operationId} className="card" style={{ padding: "12px 16px" }}>
                  <p style={{ fontWeight: 600 }}>{syncWorkLabel[review.status] ?? review.status}</p>
                  {review.code && <p className="muted" style={{ fontSize: 13 }}>Código: {review.code}</p>}
                  {review.status === "unknown" && (
                    <p className="muted" style={{ fontSize: 13 }}>
                      El resultado remoto se perdió. Antes de repetir la acción, comprueba esta misma operación
                      (mismo identificador) o sincroniza; no crees otra intención equivalente.
                    </p>
                  )}
                  {review.status === "rejected" && (
                    <p className="muted" style={{ fontSize: 13 }}>
                      No aceptada. La intención se conserva en este equipo; la API aplicó los permisos vigentes
                      al aceptar. Revisa el acceso o modifica la propuesta.
                    </p>
                  )}
                    {row && accepted && proposed && (
                    <div className="fact-grid" style={{ marginTop: 8 }}>
                      <div>
                        <dt>Estado compartido (aceptado)</dt>
                        <dd className="num">
                          {formatDateTime(zone, accepted.startsAt)} ·{" "}
                          {formatTime(zone, accepted.startsAt)}–{formatTimeEnd(zone, accepted.endsAt)}
                        </dd>
                      </div>
                      <div>
                        <dt>Tu propuesta (local)</dt>
                        <dd className="num">
                          {formatDateTime(zone, proposed.startsAt)} ·{" "}
                          {formatTime(zone, proposed.startsAt)}–{formatTimeEnd(zone, proposed.endsAt)}
                        </dd>
                      </div>
                    </div>
                  )}
                  <div className="toolbar" style={{ marginTop: 8 }}>
                    {appointmentId && (
                      <>
                        <Link className="btn" to={`/cita/${appointmentId}`}>Ver cita</Link>
                        <Link className="btn btn--primary" to={`/cita/${appointmentId}/reprogramar`}>
                          Elegir otra hora
                        </Link>
                      </>
                    )}
                  </div>
                </li>
              );
            })}
          </ul>
        )}
        <p className="muted" style={{ fontSize: 13, marginTop: 8 }}>
          No hay “forzar guardado” ni “último cambio gana”: la resolución usa los comandos existentes
          (por ejemplo, elegir otra hora) y la aceptación vuelve a comprobar reglas y permisos.
        </p>
      </section>
    </>
  );
}
