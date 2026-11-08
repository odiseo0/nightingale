import { useState } from "react";
import {
  cancelAppointment, completeAppointment, confirmAppointment, markNoShow,
  rescheduleAppointment, type AppointmentState, type EntityRow, type LocalMutationResult,
  visibleOf,
} from "../../shared/api/appointments";
import { baseOf, Changes, errorMessage, statusLabel } from "./BookingPanel";

/** Allowed staff actions per business state; React only forwards them. */
const ACTIONS: Record<string, { key: string; label: string; reason: boolean }[]> = {
  scheduled: [
    { key: "confirm", label: "Confirmar", reason: false },
    { key: "reschedule", label: "Reprogramar", reason: false },
    { key: "cancel", label: "Cancelar", reason: true },
    { key: "complete", label: "Registrar atención", reason: true },
    { key: "no_show", label: "Marcar ausencia", reason: true },
  ],
  confirmed: [
    { key: "reschedule", label: "Reprogramar", reason: false },
    { key: "cancel", label: "Cancelar", reason: true },
    { key: "complete", label: "Registrar atención", reason: true },
    { key: "no_show", label: "Marcar ausencia", reason: true },
  ],
  cancelled: [],
  completed: [],
  no_show: [],
};

export default function AppointmentActions({
  row,
  onChanged,
}: {
  row: EntityRow<AppointmentState>;
  onChanged: (message: string) => void;
}) {
  const [busy, setBusy] = useState(false);
  const state = visibleOf(row);
  if (!state) return null;
  const actions = ACTIONS[state.status] ?? [];

  async function act(work: () => Promise<LocalMutationResult>) {
    setBusy(true);
    try {
      await work();
      onChanged("Cambio guardado en este dispositivo; pendiente de aceptación.");
    } catch (error: unknown) {
      onChanged(errorMessage(error));
    } finally {
      setBusy(false);
    }
  }

  function run(key: string, reason?: string) {
    const base = baseOf(row);
    const appointmentId = state!.appointmentId;
    if (key === "confirm") {
      void act(() => confirmAppointment(appointmentId, base));
    } else if (key === "cancel") {
      void act(() => cancelAppointment({ appointmentId, base, ...(reason ? { changeReason: reason } : {}) }));
    } else if (key === "complete") {
      void act(() => completeAppointment({ appointmentId, base, ...(reason ? { changeReason: reason } : {}) }));
    } else if (key === "no_show") {
      void act(() => markNoShow({ appointmentId, base, ...(reason ? { changeReason: reason } : {}) }));
    }
  }

  if (!actions.length) {
    return (
      <p className="text-sm">
        {statusLabel[state.status] ?? state.status} es un estado terminal: no admite más acciones.
      </p>
    );
  }

  return (
    <div className="space-y-2">
      <p className="text-sm">
        Estado: <strong>{statusLabel[state.status] ?? state.status}</strong>
        {row.provisional && <em> (provisional)</em>}
        {state.confirmedAt && <> · confirmada por el servidor el <code>{state.confirmedAt}</code></>}
      </p>
      <Changes ops={row.operations} />
      <div className="flex flex-wrap gap-2">
        {actions.map((action) =>
          action.key === "reschedule" ? (
            <RescheduleForm key={action.key}
              startsAt={state!.startsAt} endsAt={state!.endsAt}
              busy={busy}
              onSubmit={(startsAt, endsAt) =>
                void act(() =>
                  rescheduleAppointment({ appointmentId: state!.appointmentId, base: baseOf(row), startsAt, endsAt })
                ).then(() => undefined)
              }
            />
          ) : action.reason ? (
            <ReasonButton key={action.key} label={action.label} busy={busy}
              onSubmit={(reason) => run(action.key, reason)} />
          ) : (
            <button key={action.key} type="button" disabled={busy}
              onClick={() => run(action.key)}>
              {action.label}
            </button>
          )
        )}
      </div>
    </div>
  );
}

function ReasonButton({
  label, busy, onSubmit,
}: {
  label: string;
  busy: boolean;
  onSubmit: (reason: string) => void;
}) {
  const [open, setOpen] = useState(false);
  const [reason, setReason] = useState("");
  if (!open) {
    return (
      <button type="button" disabled={busy} onClick={() => setOpen(true)}>{label}</button>
    );
  }
  return (
    <span className="flex items-center gap-2">
      <input value={reason} onChange={(e) => setReason(e.target.value)}
        placeholder="Motivo (opcional)" className="w-48" />
      <button type="button" disabled={busy}
        onClick={() => { onSubmit(reason.trim() === "" ? "" : reason); setOpen(false); setReason(""); }}>
        {label}
      </button>
    </span>
  );
}

function RescheduleForm({
  startsAt, endsAt, busy, onSubmit,
}: {
  startsAt: string;
  endsAt: string;
  busy: boolean;
  onSubmit: (startsAt: string, endsAt: string) => void;
}) {
  const [open, setOpen] = useState(false);
  // Keep the real duration visible; Rust enforces the equal-duration rule.
  const durationMinutes = Math.round(
    (new Date(endsAt).getTime() - new Date(startsAt).getTime()) / 60_000
  );
  if (!open) {
    return (
      <button type="button" disabled={busy} onClick={() => setOpen(true)}>Reprogramar</button>
    );
  }
  return (
    <form
      className="flex items-center gap-2"
      onSubmit={(event) => {
        event.preventDefault();
        const d = new FormData(event.currentTarget);
        onSubmit(String(d.get("startsAt") ?? ""), String(d.get("endsAt") ?? ""));
        setOpen(false);
      }}
    >
      <input name="startsAt" type="datetime-local" required />
      <input name="endsAt" type="datetime-local" required />
      <span className="text-xs text-gray-600">duración {durationMinutes} min</span>
      <button type="submit" disabled={busy}>Guardar</button>
    </form>
  );
}
