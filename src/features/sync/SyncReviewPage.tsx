import { useCallback, useEffect, useRef, useState } from "react";
import { Link } from "react-router-dom";
import { CircleNotch, Question } from "@phosphor-icons/react";
import { getAccountView } from "../../shared/api/account";
import { getCoreStatus } from "../../shared/api/identity";
import type { ReviewSummary } from "../../shared/api/sync";
import { getAppointment, type AppointmentState, type EntityRow } from "../../shared/api/appointments";
import { formatDateTime, formatTime, formatTimeEnd } from "../appointments/presentation";
import { actionLabel, entityLabel, needsIntervention, reviewRoute } from "./presentation";

type Phase = "loading" | "ready" | "no-session" | "error";

/** S11 · Por revisar: solo asuntos que requieren una decisión (conflicto,
 * rechazo o revisión requerida). El proceso normal es invisible y el
 * resultado desconocido se recupera automáticamente con la misma intención.
 * Los enlaces usan las claves de entidad de la revisión, sin barridos por
 * fechas. */
export default function SyncReviewPage() {
  const [phase, setPhase] = useState<Phase>("loading");
  const [reviews, setReviews] = useState<ReviewSummary[]>([]);
  const [appointmentRows, setAppointmentRows] = useState<Map<string, EntityRow<AppointmentState>>>(new Map());
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
      const status = await getCoreStatus();
      if (request !== generation.current) return;
      /* Solo intervención; lo demás sigue su proceso automático. */
      const attention = status.reviews.filter((review) => needsIntervention(review.status));
      setReviews(attention);
      setPhase(status.signedIn ? "ready" : "no-session");
      /* Detalle solo de las citas enlazadas (por clave de entidad), nunca un
         barrido de citas por rango de fechas. */
      const linked = attention
        .map((review) => ({ review, route: reviewRoute(review.entityType, review.entityKey) }))
        .filter((item): item is { review: ReviewSummary; route: string } =>
          item.route !== null && item.route.startsWith("/cita/"));
      const rows = new Map<string, EntityRow<AppointmentState>>();
      await Promise.all(linked.map(async ({ review, route }) => {
        const appointmentId = route.slice("/cita/".length);
        try {
          const row = await getAppointment(appointmentId);
          if (request === generation.current) rows.set(review.operationId, row);
        } catch { /* la cita no está disponible localmente: se muestra sin comparación */ }
      }));
      if (request === generation.current) setAppointmentRows(rows);
    } catch {
      if (request === generation.current) setPhase("error");
    }
  }, []);

  useEffect(() => {
    void load();
    const timer = window.setInterval(() => void load(), 15_000);
    return () => { generation.current += 1; window.clearInterval(timer); };
  }, [load]);

  if (phase === "loading") {
    return (
      <div className="state-block" role="status">
        <CircleNotch size={22} aria-hidden="true" />
        <p>Preparando Por revisar…</p>
      </div>
    );
  }

  if (phase === "no-session") {
    return (
      <div className="state-block" role="alert">
        <p>No hay una sesión lista en este equipo.</p>
        <p className="muted">Los datos locales se conservan; activa una cuenta para continuar.</p>
      </div>
    );
  }

  if (phase === "error") {
    return (
      <div className="state-block" role="alert">
        <p>No se pudo leer el estado local.</p>
        <button type="button" className="btn" onClick={() => void load()}>Reintentar</button>
      </div>
    );
  }

  return (
    <>
      <div className="page-header">
        <div>
          <h1 className="page-heading">Por revisar</h1>
          <p className="muted">
            Asuntos que requieren tu decisión. El resto del proceso sigue en segundo plano sin avisos.
          </p>
        </div>
      </div>

      {reviews.length === 0 ? (
        <div className="state-block">
          <p>No hay asuntos que requieran tu revisión.</p>
          <p className="muted">
            Cuando un cambio necesite una decisión, aparecerá aquí y junto al dato afectado.
          </p>
        </div>
      ) : (
        <ul style={{ margin: 0, paddingLeft: 20, display: "grid", gap: 10 }}>
          {reviews.map((review) => {
            const route = reviewRoute(review.entityType, review.entityKey);
            const row = appointmentRows.get(review.operationId);
            const accepted = row?.accepted?.state ?? null;
            const proposed = row?.provisional?.state ?? null;
            return (
              <li key={review.operationId} className="card" style={{ padding: "12px 16px" }}>
                <p style={{ fontWeight: 600 }}>
                  {review.status === "conflict"
                    ? "Un cambio compartido pide tu decisión"
                    : review.status === "rejected"
                      ? "Un cambio no fue aceptado"
                      : "Un cambio requiere revisión"}
                  {" · "}{entityLabel[review.entityType] ?? review.entityType}
                  {" · "}{actionLabel[review.action] ?? review.action}
                </p>
                {review.code && <p className="muted" style={{ fontSize: 13 }}>Referencia: {review.code}</p>}
                <p className="muted" style={{ fontSize: 13 }}>
                  Registrado el {formatDateTime(zone, new Date(review.createdAtMillis).toISOString())}.
                </p>
                {review.status === "conflict" && (
                  <p className="muted" style={{ fontSize: 13 }}>
                    Tu propuesta se conserva en este equipo y no se eligió nada por ti.
                    Compara con el estado compartido y decide con las acciones disponibles.
                  </p>
                )}
                {review.status === "rejected" && (
                  <p className="muted" style={{ fontSize: 13 }}>
                    La intención se conserva en este equipo. Revisa el acceso o modifica la
                    propuesta; los permisos vigentes vuelven a comprobarse al aceptar.
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
                        {formatDateTime(zone, proposed!.startsAt)} ·{" "}
                        {formatTime(zone, proposed!.startsAt)}–{formatTimeEnd(zone, proposed!.endsAt)}
                      </dd>
                    </div>
                  </div>
                )}
                <div className="toolbar" style={{ marginTop: 8 }}>
                  {route ? (
                    <>
                      <Link className="btn" to={route}>Ver {entityLabel[review.entityType] ?? "detalle"}</Link>
                      {route.startsWith("/cita/") && (
                        <Link className="btn btn--primary" to={`${route}/reprogramar`}>
                          Elegir otra hora
                        </Link>
                      )}
                    </>
                  ) : (
                    <Link className="btn" to="/agenda">
                      <Question size={16} aria-hidden="true" />
                      Ir a su sección
                    </Link>
                  )}
                </div>
              </li>
            );
          })}
        </ul>
      )}

      <p className="muted" style={{ fontSize: 13, marginTop: 12 }}>
        No hay «forzar guardado» ni «último cambio gana»: la resolución usa los comandos existentes
        (por ejemplo, elegir otra hora) y la aceptación vuelve a comprobar reglas y permisos.
      </p>
    </>
  );
}
