import { useEffect, useRef, useState, type FormEvent } from "react";
import { getLocalAvailability } from "../../shared/api/scheduling";
import {
  createAppointment, getBookingContext, listAppointments,
  type AppointmentState, type BookingContext, type EditBase, type EntityRow,
  type LocalMutationResult, visibleOf,
} from "../../shared/api/appointments";

export const statusLabel: Record<string, string> = {
  scheduled: "Programada",
  confirmed: "Confirmada",
  cancelled: "Cancelada",
  completed: "Atendida",
  no_show: "Ausencia",
};
const syncLabel: Record<string, string> = {
  pending: "Pendiente", in_flight: "Enviando", unknown: "Resultado desconocido",
  retryable: "Esperando reintento", accepted: "Aceptada; falta confirmar la vista",
  conflict: "Conflicto", rejected: "Rechazada", review_required: "Requiere revisión",
};

export function baseOf<T>(row: EntityRow<T>): EditBase {
  return row.provisional
    ? { operationId: row.provisional.operationId }
    : { acceptedVersion: row.accepted!.version };
}

export function Changes({ ops }: { ops: { operationId: string; status: string; error: string | null }[] }) {
  if (!ops.length) return null;
  return (
    <ul aria-label="Cambios locales" className="space-y-1">
      {ops.map((op) => (
        <li key={op.operationId} className="break-all">
          {syncLabel[op.status] ?? op.status}
          {op.error ? ` · ${op.error}` : ""} · <code>{op.operationId}</code>
          {["conflict", "rejected", "review_required"].includes(op.status) && (
            <> · <a href="#core-heading">Ver revisión de sincronización</a></>
          )}
        </li>
      ))}
    </ul>
  );
}

export function errorMessage(error: unknown): string {
  const code = typeof error === "object" && error !== null && "code" in error
    ? String((error as { code: string }).code)
    : "TEMPORARY_FAILURE";
  const text = typeof error === "object" && error !== null && "message" in error
    ? String((error as { message: string }).message)
    : "";
  if (code === "SLOT_UNAVAILABLE") return "El intervalo queda fuera del horario disponible o dentro de una ocupación; no se guardó.";
  if (code === "INSUFFICIENT_COVERAGE") return "La ventana de ocupación descargada no cubre el intervalo; sincroniza antes de reservar.";
  if (code === "INVALID_STATE_TRANSITION") return "La acción no está permitida desde el estado actual de la cita.";
  if (code === "VERSION_CONFLICT") return "La base local cambió; vuelve a cargar la cita e inténtalo de nuevo.";
  if (code === "DEPENDENCY_FAILED") return "Hay resultados previos que requieren revisión antes de editar.";
  if (code === "NO_CHANGES") return "El cambio no modifica el estado actual.";
  if (code === "PERMISSION_DENIED" || code === "AUTHENTICATION_REQUIRED") return "Tu acceso no permite esta acción; inicia sesión con una cuenta autorizada.";
  return `No se pudo completar la acción (${code}). ${text}`.trim();
}

export default function BookingPanel() {
  const [context, setContext] = useState<BookingContext | null>(null);
  const [message, setMessage] = useState("");
  const [busy, setBusy] = useState(false);
  const [slots, setSlots] = useState<{ startsAt: string; endsAt: string }[] | null>(null);
  const generation = useRef(0);

  async function refresh() {
    const request = ++generation.current;
    try {
      const data = await getBookingContext();
      if (request === generation.current) setContext(data);
    } catch { /* session not ready yet */ }
  }

  useEffect(() => {
    void refresh();
    const timer = window.setInterval(() => void refresh(), 4000);
    return () => { ++generation.current; window.clearInterval(timer); };
  }, []);

  async function act(work: () => Promise<LocalMutationResult>, success = "Cambio guardado en este dispositivo; pendiente de aceptación.") {
    setBusy(true); setMessage("");
    try {
      await work(); setMessage(success); setSlots(null); void refresh();
    } catch (error: unknown) {
      setMessage(errorMessage(error));
    } finally { setBusy(false); }
  }

  // React never decides availability, versions or permissions: it only shows
  // what Rust returns and forwards the user's choices.
  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const form = event.currentTarget;
    const d = new FormData(form);
    const startsAt = String(d.get("startsAt") ?? "");
    const endsAt = String(d.get("endsAt") ?? "");
    const professionalId = String(d.get("professionalId") ?? "");
    const serviceId = String(d.get("serviceId") ?? "");
    const responsible = String(d.get("responsibleContactId") ?? "");
    await act(() => createAppointment({
      patientId: String(d.get("patientId") ?? ""),
      professionalId,
      serviceId,
      ...(responsible === "" ? {} : { responsibleContactId: responsible }),
      startsAt,
      endsAt,
    }), "Cita guardada localmente; pendiente de aceptación de la API.");
    form.reset();
  }

  async function checkAvailability(form: HTMLFormElement) {
    const d = new FormData(form);
    const startsAt = String(d.get("startsAt") ?? "");
    const endsAt = String(d.get("endsAt") ?? "");
    if (!startsAt || !endsAt) { setSlots(null); return; }
    const starts = new Date(startsAt).toISOString();
    const ends = new Date(endsAt).toISOString();
    try {
      const evaluation = await getLocalAvailability({
        professionalId: String(d.get("professionalId") ?? ""),
        serviceId: String(d.get("serviceId") ?? ""),
        from: starts,
        until: ends,
      });
      setSlots(evaluation.intervals);
      if (evaluation.coverage !== "within_synced_window") {
        setMessage(evaluation.coverage === "no_synced_window"
          ? "Sin captura previa: la comprobación usa solo datos locales conocidos."
          : "Cobertura insuficiente: la lista no prueba ausencia de huecos.");
      }
    } catch (error: unknown) {
      setMessage(errorMessage(error));
      setSlots(null);
    }
  }



  return (
    <section aria-labelledby="booking-heading" className="space-y-4">
      <h3 id="booking-heading" className="text-lg font-semibold">Reserva</h3>
      {!context && <p>Contexto de reserva no disponible todavía (sin sesión o sin datos).</p>}
      {context && (
        <form onSubmit={submit} className="grid gap-3 sm:grid-cols-3">
          <fieldset disabled={busy} className="contents">
            <label className="grid gap-1">Paciente
              <select name="patientId" required defaultValue="">
                <option value="" disabled>Elige un paciente…</option>
                {context.patients.map((row) => {
                  const state = visibleOf(row);
                  if (!state) return null;
                  return (
                    <option key={state.patientId} value={state.patientId}>
                      {state.fullName ?? state.patientId}{row.provisional ? " (provisional)" : ""}
                    </option>
                  );
                })}
              </select>
            </label>
            <label className="grid gap-1">Profesional
              <select name="professionalId" required defaultValue="">
                <option value="" disabled>Elige un profesional…</option>
                {context.professionals.map((row) => {
                  const accepted = row.accepted;
                  const provisional = row.provisional;
                  return (
                    <option key={provisional?.professionalId ?? accepted?.professionalId}
                      value={provisional?.professionalId ?? accepted?.professionalId}>
                      {provisional?.publicName ?? accepted?.publicName ?? ""}
                      {row.isProvisional ? " (provisional)" : ""}
                    </option>
                  );
                })}
              </select>
            </label>
            <label className="grid gap-1">Servicio
              <select name="serviceId" required defaultValue="">
                <option value="" disabled>Elige un servicio…</option>
                {context.offers.map((row) => {
                  const accepted = row.accepted;
                  const provisional = row.provisional;
                  return (
                    <option key={provisional?.serviceId ?? accepted?.serviceId}
                      value={provisional?.serviceId ?? accepted?.serviceId}>
                      {accepted?.name ?? provisional?.serviceId ?? ""}
                      {row.isProvisional ? " (provisional)" : ""}
                      {accepted ? ` · ${accepted.effectiveDurationMinutes} min` : ""}
                    </option>
                  );
                })}
              </select>
            </label>
            <label className="grid gap-1">Inicio
              <input name="startsAt" type="datetime-local" required
                onBlur={(e) => void checkAvailability(e.currentTarget.form!)} />
            </label>
            <label className="grid gap-1">Fin
              <input name="endsAt" type="datetime-local" required
                onBlur={(e) => void checkAvailability(e.currentTarget.form!)} />
            </label>
            <label className="grid gap-1">Responsable (opcional)
              <input name="responsibleContactId" placeholder="UUID del contacto responsable" />
            </label>
            <button type="submit">Crear cita local</button>
          </fieldset>
        </form>
      )}
      <p role="status">{message}</p>
      {slots && (
        <p className="text-sm">
          Intervalos disponibles (local, provisional):{" "}
          {slots.length ? slots.map((slot) => `${slot.startsAt} – ${slot.endsAt}`).join(" · ") : "sin huecos conocidos para ese rango"}
        </p>
      )}
      <p className="text-sm text-gray-600">
        Un intervalo que solo coincide con huecos conocidos aún no reserva: la API vuelve a
        validar permisos, referencias y solapamientos antes de aceptar. El responsable se exige
        solo cuando la ficha del paciente lo necesita; Rust valida la regla.
      </p>
      <p className="text-sm text-gray-600">
        Pacientes utilizables cargados: {context?.patients.length ?? 0}.
        <span className="ml-2" />
        <AppointmentsQuickList />
      </p>
    </section>
  );
}

function AppointmentsQuickList() {
  const [rows, setRows] = useState<EntityRow<AppointmentState>[]>([]);
  useEffect(() => {
    const load = async () => {
      const now = new Date();
      const from = new Date(now.getTime() - 7 * 86_400_000).toISOString();
      const until = new Date(now.getTime() + 31 * 86_400_000).toISOString();
      try {
        const page = await listAppointments({ from, until, shows: 5, page: 1 });
        setRows(page.items);
      } catch { /* session not ready */ }
    };
    void load();
    const timer = window.setInterval(() => void load(), 4000);
    return () => window.clearInterval(timer);
  }, []);
  if (!rows.length) return <span>Sin citas recientes.</span>;
  return (
    <span>
      Últimas citas:{" "}
      {rows.map((row) => {
        const state = visibleOf(row);
        if (!state) return null;
        return (
          <span key={state.appointmentId} className="mr-2">
            {state.startsAt} ({statusLabel[state.status] ?? state.status}){row.provisional ? "*" : ""}
          </span>
        );
      })}
    </span>
  );
}
