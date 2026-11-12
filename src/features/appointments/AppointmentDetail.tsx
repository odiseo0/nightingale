import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { CalendarBlank, X } from "@phosphor-icons/react";
import {
  getAppointment, getHistory,
  type AppointmentState, type BookingContext, type EntityRow,
  type HistoryView, visibleOf,
} from "../../shared/api/appointments";
import { listPatientContacts, visibleOf as visibleContact } from "../../shared/api/patients";
import {
  errorMessage, formatAddressLine, formatDateFull, formatDateTime, formatTime,
  formatTimeEnd, historyActionLabel, patientNameOf, patientPhoneOf, statusLabel,
} from "./presentation";
import LifecycleActions from "./LifecycleActions";

interface Props {
  appointmentId: string;
  context: BookingContext | null;
  onClose?: () => void;
}

/** Detalle de cita (S03): prioriza paciente, horario y acciones; la actividad
 * se muestra como acciones comprensibles y las operaciones técnicas viven en
 * diagnóstico, no en la vista diaria. */
export default function AppointmentDetail({ appointmentId, context, onClose }: Props) {
  const navigate = useNavigate();
  const [row, setRow] = useState<EntityRow<AppointmentState> | null>(null);
  const [history, setHistory] = useState<HistoryView | null>(null);
  const [responsibleName, setResponsibleName] = useState<string | null>(null);
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
  const patientName = state ? patientNameOf(context, state.patientId) : "Cita";

  /* Resolver el nombre del responsable; nunca se muestra un UUID como
   * sustituto del nombre. Si no hay lectura autorizada, queda sin registrar. */
  useEffect(() => {
    if (!state?.responsibleContactId) return;
    let cancelled = false;
    listPatientContacts({ patientId: state.patientId })
      .then((relations) => {
        if (cancelled) return;
        const match = relations
          .map((relation) => visibleContact(relation))
          .find((view) => view?.relationship.contactPersonId === state.responsibleContactId);
        setResponsibleName(match?.contactFullName ?? null);
      })
      .catch(() => { /* sin lectura autorizada: se muestra como no registrado */ });
    return () => { cancelled = true; };
  }, [state?.responsibleContactId, state?.patientId]);

  const addressLine = state
    ? formatAddressLine({
        address: state.locationAddress,
        city: state.locationCity,
        state: state.locationState,
      })
    : "";

  return (
    <section
      className="detail-panel"
      style={{ width: "100%" }}
      aria-labelledby="detalle-cita-titulo"
      aria-busy={row === null && error === null}
    >
      <div className="detail-heading">
        <div style={{ minWidth: 0 }}>
          <h2 id="detalle-cita-titulo" style={{ fontSize: 20, overflowWrap: "anywhere" }}>
            {patientName}
          </h2>
          {state && (
            <p className="muted" style={{ fontSize: 14, marginTop: 2 }}>
              {state.serviceName} · {state.offeredDurationMinutes} min
            </p>
          )}
        </div>
        <div className="toolbar" style={{ gap: 8, flexShrink: 0 }}>
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
              <dt>Estado</dt>
              <dd>
                <span className="badge">{statusLabel[state.status] ?? state.status}</span>
              </dd>
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
              <dt>Teléfono</dt>
              <dd>{patientPhoneOf(context, state.patientId) ?? "No registrado"}</dd>
            </div>
            <div>
              <dt>Lugar</dt>
              <dd>
                {state.locationName}
                {addressLine && (
                  <>
                    <br />
                    {addressLine}
                  </>
                )}
              </dd>
            </div>
            <div>
              <dt>Profesional</dt>
              <dd>{state.professionalName}</dd>
            </div>
            <div>
              <dt>Confirmación</dt>
              <dd>
                {state.confirmedAt
                  ? `Confirmada el ${formatDateTime(zone, state.confirmedAt)}`
                  : "Sin confirmación registrada."}
              </dd>
            </div>
            {state.responsibleContactId && (
              <div>
                <dt>Contacto responsable</dt>
                <dd>{responsibleName ?? "No registrado"}</dd>
              </div>
            )}
          </dl>

          {row && (
            <LifecycleActions
              row={row}
              zone={zone}
              noShowGraceMinutes={context?.noShowGraceMinutes ?? 15}
              patientLabel={patientName}
              onMutated={(message) => { setNotice(message); reload(); }}
              onError={(message) => setNotice(message)}
            />
          )}

          <div>
            <h3 style={{ fontSize: 16, marginBottom: 8 }}>Actividad</h3>
            {!history || history.accepted.length === 0 ? (
              <p className="muted">
                Sin actividad aceptada descargada en este equipo.
              </p>
            ) : (
              <ol style={{ margin: 0, paddingLeft: 20 }}>
                {history.accepted.map((entry) => (
                  <li key={entry.version} style={{ marginBottom: 6 }}>
                    <span className="num">
                      {historyActionLabel[entry.action] ?? entry.action} ·{" "}
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
        </>
      )}
    </section>
  );
}
