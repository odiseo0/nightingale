import { useRef, useState } from "react";
import { CircleNotch, Prohibit, CheckCircle, XCircle, UserCheck } from "@phosphor-icons/react";
import {
  cancelAppointment, completeAppointment, confirmAppointment, markNoShow,
  type AppointmentState, type EntityRow, visibleOf,
} from "../../shared/api/appointments";
import { baseOf, formatDateFull, formatTime, formatTimeEnd, mutationErrorMessage } from "./presentation";

interface Props {
  row: EntityRow<AppointmentState>;
  zone: string;
  noShowGraceMinutes: number;
  patientLabel: string;
  onMutated: (message: string) => void;
  onError: (message: string) => void;
}

type DialogKind = "cancel" | "complete" | "no_show" | null;

/** Acciones del personal sobre el ciclo de vida de la cita (S03). Rust decide
 * transición, momento y margen; la UI solo ofrece acciones válidas según el
 * estado conocido y aplica fricción proporcional. */
export default function LifecycleActions({
  row, zone, noShowGraceMinutes, patientLabel, onMutated, onError,
}: Props) {
  const [dialog, setDialog] = useState<DialogKind>(null);
  const [reason, setReason] = useState("");
  const [busyAction, setBusyAction] = useState<string | null>(null);
  const busyRef = useRef(false);

  const state = visibleOf(row);
  if (!state) return null;

  const now = Date.now();
  const start = Date.parse(state.startsAt);
  const reachedStart = now >= start;
  const reachedGrace = now >= start + noShowGraceMinutes * 60_000;
  const base = baseOf(row);

  async function run(action: string, work: () => Promise<unknown>) {
    if (busyRef.current) return;
    busyRef.current = true;
    setBusyAction(action);
    try {
      await work();
      setDialog(null);
      setReason("");
      onMutated(`${action} guardado en este equipo · Pendiente de aceptación.`);
    } catch (error: unknown) {
      onError(mutationErrorMessage(error));
    } finally {
      busyRef.current = false;
      setBusyAction(null);
    }
  }

  const busy = busyAction !== null;
  const confirmLabel = busyAction === "confirm" ? "Confirmando…" : "Confirmar";
  const dialogTexts: Record<Exclude<DialogKind, null>, { title: string; body: string; confirm: string; action: string }> = {
    cancel: {
      title: "Cancelar cita",
      body: `Vas a cancelar la cita de ${patientLabel} del ${formatDateFull(zone, state.startsAt)} a las ${formatTime(zone, state.startsAt)}. La historia se conserva; el horario solo se libera cuando la API acepte la cancelación y podrá ofrecerse a otra persona.`,
      confirm: "Confirmar cancelación",
      action: "Cancelación",
    },
    complete: {
      title: "Registrar atención",
      body: `Se registrará la atención de ${patientLabel} en la cita de ${formatDateFull(zone, state.startsAt)} ${formatTime(zone, state.startsAt)}–${formatTimeEnd(zone, state.endsAt)}. Es un estado terminal: la cita quedará atendida.`,
      confirm: "Registrar atención",
      action: "Atención",
    },
    no_show: {
      title: "Registrar ausencia",
      body: `Se registrará la ausencia de ${patientLabel} en la cita de ${formatDateFull(zone, state.startsAt)} ${formatTime(zone, state.startsAt)}. La ausencia se determina después del inicio más el margen de ${noShowGraceMinutes} min; es una acción explícita del personal, nunca automática.`,
      confirm: "Registrar ausencia",
      action: "Ausencia",
    },
  };

  return (
    <section aria-label="Acciones de la cita" style={{ display: "grid", gap: 10 }}>
      <div className="toolbar" role="group" aria-label="Acciones disponibles">
        {state.status === "scheduled" && (
          <button
            type="button"
            className="btn btn--primary"
            disabled={busy}
            onClick={() => void run("Confirmación", () => confirmAppointment(state.appointmentId, base))}
          >
            {busyAction === "confirm" ? <CircleNotch size={16} aria-hidden="true" /> : <CheckCircle size={16} aria-hidden="true" />}
            {confirmLabel}
          </button>
        )}
        {(state.status === "scheduled" || state.status === "confirmed") && (
          <>
            <button type="button" className="btn" disabled={busy || !reachedStart} onClick={() => setDialog("complete")}>
              <UserCheck size={16} aria-hidden="true" />
              Registrar atención
            </button>
            <button type="button" className="btn" disabled={busy || !reachedGrace} onClick={() => setDialog("no_show")}>
              <XCircle size={16} aria-hidden="true" />
              Registrar ausencia
            </button>
            <button type="button" className="btn" disabled={busy} onClick={() => setDialog("cancel")}>
              <Prohibit size={16} aria-hidden="true" />
              Cancelar
            </button>
          </>
        )}
      </div>

      {!reachedStart && (state.status === "scheduled" || state.status === "confirmed") && (
        <p className="muted" style={{ fontSize: 13 }}>
          La atención se puede registrar al alcanzar el inicio de la cita; la ausencia, tras el
          inicio más el margen de {noShowGraceMinutes} min.
        </p>
      )}

      {state.status === "cancelled" || state.status === "completed" || state.status === "no_show" ? (
        <p className="muted">
          {state.status === "cancelled" ? "Cancelada" : state.status === "completed" ? "Atendida" : "Ausencia"}
          {" "}es un estado terminal: la cita admite consulta e historia, sin más acciones.
        </p>
      ) : null}

      {dialog && (
        <div className="modal-backdrop" role="alertdialog" aria-modal="true" aria-labelledby="accion-titulo">
          <div className="modal-card">
            <h2 id="accion-titulo" style={{ fontSize: 20 }}>{dialogTexts[dialog].title}</h2>
            <p className="muted">{dialogTexts[dialog].body}</p>
            <label className="field-label">
              Motivo (opcional)
              <input
                type="text"
                className="input-text"
                value={reason}
                onChange={(event) => setReason(event.target.value)}
              />
            </label>
            <div className="toolbar">
              <button
                type="button"
                className="btn btn--primary"
                disabled={busy}
                onClick={() => {
                  const withReason = reason.trim() ? { changeReason: reason.trim() } : {};
                  if (dialog === "cancel") {
                    void run(dialogTexts.cancel.action, () =>
                      cancelAppointment({ appointmentId: state.appointmentId, base, ...withReason }));
                  } else if (dialog === "complete") {
                    void run(dialogTexts.complete.action, () =>
                      completeAppointment({ appointmentId: state.appointmentId, base, ...withReason }));
                  } else {
                    void run(dialogTexts.no_show.action, () =>
                      markNoShow({ appointmentId: state.appointmentId, base, ...withReason }));
                  }
                }}
              >
                {busy ? <CircleNotch size={16} aria-hidden="true" /> : null}
                {dialogTexts[dialog].confirm}
              </button>
              <button type="button" className="btn" disabled={busy} onClick={() => { setDialog(null); setReason(""); }}>
                {dialog === "cancel" ? "Conservar cita" : "Volver"}
              </button>
            </div>
          </div>
        </div>
      )}
    </section>
  );
}
