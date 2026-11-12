import { useCallback, useEffect, useRef, useState } from "react";
import { useBlocker, useNavigate, useParams } from "react-router-dom";
import { ArrowLeft, CircleNotch, Lock } from "@phosphor-icons/react";
import Modal from "../../components/primitives/Modal";
import {
  getAppointment, getBookingContext, rescheduleAppointment,
  type AppointmentState, type BookingContext, type EntityRow, visibleOf,
} from "../../shared/api/appointments";
import { getLocalAvailability, type AvailabilityEvaluation } from "../../shared/api/scheduling";
import AvailabilityPicker, { type AvailabilityState } from "./AvailabilityPicker";
import {
  baseOf, civilOfInstant, civilToday, dayRange, errorMessage, formatDateFull, formatTime, formatTimeEnd,
  mutationErrorMessage, patientNameOf,
} from "./presentation";

/** R02 · Reprogramar cita: comparación actual/propuesto, mismo profesional y
 * servicio, misma duración, disponibilidad calculada por Rust. */
export default function ReschedulePage() {
  const { appointmentId } = useParams<{ appointmentId: string }>();
  const navigate = useNavigate();
  const [row, setRow] = useState<EntityRow<AppointmentState> | null>(null);
  const [context, setContext] = useState<BookingContext | null>(null);
  const [loadState, setLoadState] = useState<"loading" | "ready" | "error">("loading");
  const [loadError, setLoadError] = useState<string | null>(null);
  const [civil, setCivil] = useState<string | null>(null);
  const [availability, setAvailability] = useState<AvailabilityState>({ status: "idle" });
  const [slot, setSlot] = useState<{ startsAt: string; endsAt: string } | null>(null);
  const [saving, setSaving] = useState(false);
  const [formError, setFormError] = useState<string | null>(null);
  const savingRef = useRef(false);
  const savedRef = useRef(false);
  const dirtyRef = useRef(false);
  const generation = useRef(0);

  useEffect(() => {
    if (!appointmentId) return;
    let cancelled = false;
    Promise.all([getAppointment(appointmentId), getBookingContext()])
      .then(([detail, booking]) => {
        if (cancelled) return;
        const state = visibleOf(detail);
        setRow(detail);
        setContext(booking);
        if (state) {
          setCivil(civilOfInstant(booking.timeZone, state.startsAt));
        }
        setLoadState("ready");
      })
      .catch((loadError: unknown) => {
        if (cancelled) return;
        setLoadState("error");
        setLoadError(errorMessage(loadError));
      });
    return () => { cancelled = true; };
  }, [appointmentId]);

  const state = row ? visibleOf(row) : null;
  const zone = context?.timeZone ?? state?.timeZone ?? "UTC";
  const durationMinutes = state
    ? Math.round((Date.parse(state.endsAt) - Date.parse(state.startsAt)) / 60_000)
    : 0;

  const loadAvailability = useCallback(async () => {
    if (!state || !civil || durationMinutes <= 0) return;
    const request = ++generation.current;
    setAvailability({ status: "loading" });
    try {
      const evaluation: AvailabilityEvaluation = await getLocalAvailability({
        professionalId: state.professionalId,
        serviceId: state.serviceId,
        ...dayRange(zone, civil),
        durationMinutes,
      });
      if (request !== generation.current) return;
      setAvailability({ status: "ready", evaluation });
    } catch (availabilityError: unknown) {
      if (request !== generation.current) return;
      setAvailability({ status: "error", message: errorMessage(availabilityError) });
    }
  }, [state, civil, zone, durationMinutes]);

  /* El slot propuesto solo se invalida cuando deja de pertenecer al día
   * elegido; un error de disponibilidad o un reintento no borran la
   * propuesta del usuario. */
  useEffect(() => {
    setSlot((current) => {
      if (!current || !civil) return current;
      return civilOfInstant(zone, current.startsAt) === civil ? current : null;
    });
    void loadAvailability();
  }, [loadAvailability, zone, civil]);

  const dirty = slot !== null;
  dirtyRef.current = dirty;
  const blocker = useBlocker(() => dirtyRef.current && !savedRef.current);

  useEffect(() => {
    if (!dirty || savedRef.current) return;
    const onBeforeUnload = (event: BeforeUnloadEvent) => { event.preventDefault(); };
    window.addEventListener("beforeunload", onBeforeUnload);
    return () => window.removeEventListener("beforeunload", onBeforeUnload);
  }, [dirty]);

  if (!appointmentId) {
    return (
      <div className="state-block" role="alert">
        <p>El identificador de cita no es válido.</p>
        <button type="button" className="btn" onClick={() => navigate("/agenda")}>Volver a la agenda</button>
      </div>
    );
  }

  if (loadState === "loading") {
    return (
      <div className="state-block" role="status">
        <CircleNotch size={22} aria-hidden="true" />
        <p>Preparando la reprogramación…</p>
      </div>
    );
  }

  if (loadState === "error") {
    return (
      <div className="state-block" role="alert">
        <p>No se pudo leer la cita para reprogramar.</p>
        <p className="muted">{loadError}</p>
        <button type="button" className="btn" onClick={() => navigate(`/cita/${appointmentId}`)}>
          Volver al detalle
        </button>
      </div>
    );
  }

  if (!state) {
    return (
      <div className="state-block" role="alert">
        <p>La cita no está disponible en los datos locales.</p>
        <button type="button" className="btn" onClick={() => navigate("/agenda")}>Volver a la agenda</button>
      </div>
    );
  }

  const canReschedule = state.status === "scheduled" || state.status === "confirmed";
  const today = civilToday(zone);
  const sameSlot = slot
    && slot.startsAt === state.startsAt
    && slot.endsAt === state.endsAt;

  async function save() {
    if (savingRef.current || !slot || !row || sameSlot) return;
    savingRef.current = true;
    setSaving(true);
    setFormError(null);
    try {
      const startsAt = slot.startsAt;
      const endsAt = new Date(Math.min(
        Date.parse(slot.endsAt),
        Date.parse(startsAt) + durationMinutes * 60_000,
      )).toISOString();
      await rescheduleAppointment({
        appointmentId: state!.appointmentId,
        base: baseOf(row),
        startsAt,
        endsAt,
      });
      savedRef.current = true;
      try {
        const raw = sessionStorage.getItem("agenda:estado");
        const stored = raw ? JSON.parse(raw) as Record<string, unknown> : {};
        sessionStorage.setItem("agenda:estado", JSON.stringify({ ...stored, selectedId: state!.appointmentId }));
        sessionStorage.setItem("agenda:mensaje", "Reprogramación guardada.");
      } catch { /* el mensaje persistente es opcional */ }
      navigate(`/cita/${state!.appointmentId}`);
    } catch (saveError: unknown) {
      setFormError(mutationErrorMessage(saveError));
    } finally {
      savingRef.current = false;
      setSaving(false);
    }
  }

  return (
    <>
      <div className="page-header">
        <div>
          <h1 className="page-heading">Reprogramar cita</h1>
          <p className="muted">
            {patientNameOf(context, state.patientId)} · {state.serviceName} · {state.professionalName}
          </p>
        </div>
        <button type="button" className="btn" onClick={() => navigate(`/cita/${appointmentId}`)}>
          <ArrowLeft size={16} aria-hidden="true" />
          Volver al detalle
        </button>
      </div>

      {!canReschedule && (
        <div className="notice" role="alert">
          <p>Esta cita está {state.status === "cancelled" ? "cancelada" : state.status === "completed" ? "atendida" : "marcada como ausencia"}; no admite reprogramación.</p>
        </div>
      )}

      <div className="editor-layout">
        <div style={{ display: "grid", gap: 20, minWidth: 0, flex: 1 }}>
          <section className="card" style={{ padding: 16 }} aria-labelledby="actual-titulo">
            <h2 id="actual-titulo" style={{ fontSize: 18, marginBottom: 12 }}>Horario actual</h2>
            <dl className="fact-grid">
              <div>
                <dt>Fecha</dt>
                <dd>{formatDateFull(zone, state.startsAt)}</dd>
              </div>
              <div>
                <dt>Intervalo</dt>
                <dd className="num">
                  {formatTime(zone, state.startsAt)}–{formatTimeEnd(zone, state.endsAt)} {zone}
                </dd>
              </div>
              <div>
                <dt>Duración</dt>
                <dd className="num">{durationMinutes} min</dd>
              </div>
              <div>
                <dt>Estado de cita</dt>
                <dd>{state.status === "confirmed" ? "Confirmada" : "Programada"}
                  {state.confirmedAt && " · confirmada anteriormente"}</dd>
              </div>
            </dl>
          </section>

          {canReschedule && (
            <fieldset style={{ border: 0, padding: 0, margin: 0, display: "grid", gap: 12 }}>
              <legend className="muted" style={{ marginBottom: 8 }}>Nuevo horario</legend>
              <div className="toolbar">
                <button type="button" className="btn btn--subtle" onClick={() => setCivil(today)} disabled={civil === today}>
                  Hoy
                </button>
                <label className="field-label">
                  Fecha propuesta
                  <input
                    type="date"
                    className="input-date"
                    value={civil ?? today}
                    min={today}
                    onChange={(event) => setCivil(event.target.value)}
                  />
                </label>
              </div>
              <AvailabilityPicker
                zone={zone}
                availability={availability}
                slot={slot}
                onSlotChange={setSlot}
                onRetry={() => void loadAvailability()}
              />
            </fieldset>
          )}
        </div>

        <div className="editor-summary">
          <section className="detail-panel" aria-labelledby="comparacion-titulo" style={{ width: "100%" }}>
            <h2 id="comparacion-titulo" style={{ fontSize: 20 }}>Comparación</h2>
            <dl className="fact-grid">
              <div>
                <dt>Horario actual</dt>
                <dd>
                  {formatDateFull(zone, state.startsAt)}
                  <br />
                  <span className="num">{formatTime(zone, state.startsAt)}–{formatTimeEnd(zone, state.endsAt)}</span>
                </dd>
              </div>
              <div>
                <dt>Horario propuesto</dt>
                <dd>
                  {slot && civil ? formatDateFull(zone, slot.startsAt) : "Sin seleccionar"}
                  <br />
                  <span className="num">
                    {slot ? `${formatTime(zone, slot.startsAt)}–${formatTimeEnd(zone, slot.endsAt)}` : "—"}
                  </span>
                </dd>
              </div>
              <div>
                <dt>Duración conservada</dt>
                <dd className="num">{durationMinutes} min</dd>
              </div>
            </dl>
            {formError && (
              <div className="notice" role="alert"><p>{formError}</p></div>
            )}
            <button
              type="button"
              className="btn btn--primary"
              disabled={!canReschedule || !slot || sameSlot || saving}
              onClick={() => void save()}
            >
              {saving
                ? <><CircleNotch size={16} aria-hidden="true" /> Guardando…</>
                : <><Lock size={16} aria-hidden="true" /> Guardar reprogramación</>}
            </button>
            {!slot && canReschedule && <p className="muted">Selecciona un intervalo disponible.</p>}
            {sameSlot && <p className="muted">El intervalo propuesto coincide con el actual; elige otro horario.</p>}
            <p className="muted" style={{ fontSize: 13 }}>
              Al aceptarse la reprogramación, la confirmación anterior queda invalidada y la cita
              vuelve a estar programada. El horario anterior no queda libre hasta que el cambio
              sea aceptado. Guardar ahora solo guarda la propuesta en este equipo.
            </p>
          </section>
        </div>
      </div>

      <Modal open={blocker.state === "blocked"} onClose={() => blocker.reset?.()} alert labelledBy="descartar-reprogramacion">
        <h2 id="descartar-reprogramacion" style={{ fontSize: 20 }}>Cambios sin guardar</h2>
        <p className="muted">Si sales ahora, la propuesta de reprogramación se perderá.</p>
        <div className="toolbar">
          <button type="button" className="btn btn--primary" onClick={() => blocker.reset?.()}>
            Seguir editando
          </button>
          <button type="button" className="btn" onClick={() => blocker.proceed?.()}>
            Descartar y salir
          </button>
        </div>
      </Modal>
    </>
  );
}
