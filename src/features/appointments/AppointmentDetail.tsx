import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { CalendarBlank, X } from "@phosphor-icons/react";
import {
  getAppointment, getHistory,
  type AppointmentState, type BookingContext, type EntityRow,
  type HistoryView, visibleOf,
} from "../../shared/api/appointments";
import {
  errorMessage, formatDateFull, formatDateTime, formatTime, formatTimeEnd,
  patientNameOf, patientPhoneOf, statusLabel, syncLabel,
} from "./presentation";
import LifecycleActions from "./LifecycleActions";

interface Props {
  appointmentId: string;
  context: BookingContext | null;
  onClose?: () => void;
}

/** Detalle de cita en modo lectura (S03): datos de la cita, paciente del
 * contexto, historia aceptada y operaciones locales, separadas. */
export default function AppointmentDetail({ appointmentId, context, onClose }: Props) {
  const navigate = useNavigate();
  const [row, setRow] = useState<EntityRow<AppointmentState> | null>(null);
  const [history, setHistory] = useState<HistoryView | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

  function reload() {
    void Promise.all([getAppointment(appointmentId), getHistory(appointmentId)])
      .then(([detail, hist]) => { setRow(detail); setHistory(hist); })
      .catch((loadError: unknown) => setError(errorMessage(loadError)));
  }

  useEffect(() => {
    let cancelled = false;
    setRow(null);
    setHistory(null);
    setError(null);
    setNotice(null);
    const load = async () => {
      try {
        const [detail, hist] = await Promise.all([
          getAppointment(appointmentId),
          getHistory(appointmentId),
        ]);
        if (!cancelled) { setRow(detail); setHistory(hist); }
      } catch (loadError: unknown) {
        if (!cancelled) setError(errorMessage(loadError));
      }
    };
    void load();
    return () => { cancelled = true; };
  }, [appointmentId]);

  const state = row ? visibleOf(row) : null;
  const zone = context?.timeZone ?? state?.timeZone ?? "UTC";

  return (
    <section
      className="detail-panel"
      style={{ width: "100%" }}
      aria-labelledby="detalle-cita-titulo"
      aria-busy={row === null && error === null}
    >
      <div className="detail-heading">
        <h2 id="detalle-cita-titulo" style={{ fontSize: 20 }}>
          Detalle de cita
        </h2>
        <div className="toolbar" style={{ gap: 8 }}>
          {state && (state.status === "scheduled" || state.status === "confirmed") && (
            <button
              type="button"
              className="btn btn--subtle"
              onClick={() => navigate(`/cita/${appointmentId}/reprogramar`)}
            >
              <CalendarBlank size={16} aria-hidden="true" />
              Reprogramar
            </button>
          )}
          {onClose && (
            <button
              type="button"
              className="icon-btn"
              aria-label="Cerrar detalle"
              onClick={onClose}
            >
              <X size={18} aria-hidden="true" />
            </button>
          )}
        </div>
      </div>

      {error && (
        <div className="notice" role="alert">
          <p>No se pudo leer la cita. {error}</p>
        </div>
      )}

      {notice && (
        <div className="notice" role="status"><p>{notice}</p></div>
      )}

      {row === null && !error && <p className="muted">Cargando detalle…</p>}

      {state && (
        <>
          <dl className="fact-grid">
            <div>
              <dt>Paciente</dt>
              <dd>{patientNameOf(context, state.patientId)}</dd>
            </div>
            <div>
              <dt>Teléfono</dt>
              <dd>{patientPhoneOf(context, state.patientId) ?? "No registrado"}</dd>
            </div>
            <div>
              <dt>Estado</dt>
              <dd>
                <span className="badge">{statusLabel[state.status] ?? state.status}</span>
                {row?.provisional && (
                  <p className="muted" style={{ marginTop: 4 }}>
                    Guardada en este equipo · pendiente de aceptación.
                  </p>
                )}
              </dd>
            </div>
            <div>
              <dt>Servicio</dt>
              <dd>{state.serviceName} ({state.offeredDurationMinutes} min)</dd>
            </div>
            <div>
              <dt>Profesional</dt>
              <dd>{state.professionalName}</dd>
            </div>
            <div>
              <dt>Fecha e intervalo</dt>
              <dd className="num">
                {formatDateFull(zone, state.startsAt)}
                <br />
                {formatTime(zone, state.startsAt)}–{formatTimeEnd(zone, state.endsAt)}
                {" "}{zone}
              </dd>
            </div>
            <div>
              <dt>Lugar</dt>
              <dd>
                {state.locationName}
                {state.locationAddress && <>, {state.locationAddress}</>}
                <br />
                {[state.locationCity, state.locationState, state.locationCountry]
                  .filter(Boolean).join(", ")}
              </dd>
            </div>
            <div>
              <dt>Confirmación del servidor</dt>
              <dd>
                {state.confirmedAt
                  ? `Confirmada el ${formatDateTime(zone, state.confirmedAt)}`
                  : "Sin confirmación del servidor."}
              </dd>
            </div>
            {state.responsibleContactId && (
              <div>
                <dt>Referencia de contacto responsable</dt>
                <dd>{state.responsibleContactId}</dd>
              </div>
            )}
          </dl>

          {row && (
            <LifecycleActions
              row={row}
              zone={zone}
              noShowGraceMinutes={context?.noShowGraceMinutes ?? 15}
              patientLabel={patientNameOf(context, state.patientId)}
              onMutated={(message) => { setNotice(message); reload(); }}
              onError={(message) => setNotice(message)}
            />
          )}

          <div>
            <h3 style={{ fontSize: 16, marginBottom: 8 }}>Historia aceptada</h3>
            {!history || history.accepted.length === 0 ? (
              <p className="muted">
                Sin historia aceptada descargada en este equipo.
              </p>
            ) : (
              <ol style={{ margin: 0, paddingLeft: 20 }}>
                {history.accepted.map((entry) => (
                  <li key={entry.version} style={{ marginBottom: 6 }}>
                    <span className="num">
                      v{entry.version} · {entry.action} ·{" "}
                      {statusLabel[entry.status] ?? entry.status} ·{" "}
                      {formatDateTime(zone, entry.changedAt)}
                    </span>
                    {entry.changeReason && (
                      <p className="muted" style={{ fontSize: 13 }}>
                        Motivo: {entry.changeReason}
                      </p>
                    )}
                  </li>
                ))}
              </ol>
            )}
          </div>

          {history && history.operations.length > 0 && (
            <div>
              <h3 style={{ fontSize: 16, marginBottom: 8 }}>Operaciones locales</h3>
              <ul style={{ margin: 0, paddingLeft: 20 }}>
                {history.operations.map((op) => (
                  <li key={op.operationId} style={{ marginBottom: 6 }}>
                    {syncLabel[op.status] ?? op.status}
                    {op.error && <span className="muted"> · {op.error}</span>}
                  </li>
                ))}
              </ul>
              <p className="muted" style={{ fontSize: 13 }}>
                Separadas de la historia aceptada; no son hechos compartidos.
              </p>
            </div>
          )}
        </>
      )}
    </section>
  );
}
