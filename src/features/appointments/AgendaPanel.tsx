import { useEffect, useRef, useState } from "react";
import {
  getAppointment, getBookingContext, getHistory, listAppointments,
  type AppointmentState, type EntityRow, type HistoryView, visibleOf,
} from "../../shared/api/appointments";
import { errorMessage, statusLabel } from "./BookingPanel";
import AppointmentActions from "./AppointmentActions";

/** Civil midnight of a zone day as a UTC instant (presentation only; Rust
 * remains the authority on time rules). DST transitions may shift an hour. */
function civilMidnightUtc(zone: string, y: number, m: number, d: number): number {
  const guess = Date.UTC(y, m - 1, d);
  const parts = new Intl.DateTimeFormat("en-US", {
    timeZone: zone, hour12: false,
    year: "numeric", month: "2-digit", day: "2-digit",
    hour: "2-digit", minute: "2-digit",
  }).formatToParts(guess);
  const value = (type: string) => Number(parts.find((p) => p.type === type)?.value ?? 0);
  const civilHours = value("hour") % 24;
  const offset = ((civilHours - 0) * 60 + value("minute")) * 60_000;
  return guess - offset;
}

function dayRange(zone: string, civil: string): { from: string; until: string } {
  const [y, m, d] = civil.split("-").map(Number);
  const start = civilMidnightUtc(zone, y, m, d);
  return {
    from: new Date(start).toISOString(),
    until: new Date(start + 86_400_000).toISOString(),
  };
}

function civilToday(zone: string): string {
  const parts = new Intl.DateTimeFormat("en-CA", {
    timeZone: zone, year: "numeric", month: "2-digit", day: "2-digit",
  }).formatToParts(new Date());
  const value = (type: string) => parts.find((p) => p.type === type)?.value ?? "";
  return `${value("year")}-${value("month")}-${value("day")}`;
}

export default function AgendaPanel() {
  const [zone, setZone] = useState("UTC");
  const [civil, setCivil] = useState(() => civilToday("UTC"));
  const [rows, setRows] = useState<EntityRow<AppointmentState>[]>([]);
  const [total, setTotal] = useState(0);
  const [selected, setSelected] = useState<string | null>(null);
  const [message, setMessage] = useState("");
  const generation = useRef(0);

  useEffect(() => {
    void getBookingContext()
      .then((context) => {
        setZone(context.timeZone);
        setCivil(civilToday(context.timeZone));
      })
      .catch(() => { /* session not ready */ });
  }, []);

  async function load(day: string) {
    const request = ++generation.current;
    try {
      const page = await listAppointments({ ...dayRange(zone, day), shows: 500 });
      if (request !== generation.current) return;
      setRows(page.items);
      setTotal(page.total);
      setMessage("");
    } catch (error: unknown) {
      if (request === generation.current) setMessage(errorMessage(error));
    }
  }

  useEffect(() => { void load(civil); }, [zone, civil]);
  useEffect(() => {
    const timer = window.setInterval(() => void load(civil), 4000);
    return () => { ++generation.current; window.clearInterval(timer); };
  }, [zone, civil]);

  const today = civilToday(zone);
  const tomorrowDate = new Date(civilMidnightUtc(zone, ...today.split("-").map(Number) as [number, number, number]) + 86_400_000);
  const tomorrowParts = new Intl.DateTimeFormat("en-CA", {
    timeZone: zone, year: "numeric", month: "2-digit", day: "2-digit",
  }).formatToParts(tomorrowDate);
  const t = (type: string) => tomorrowParts.find((p) => p.type === type)?.value ?? "";
  const tomorrow = `${t("year")}-${t("month")}-${t("day")}`;

  return (
    <section aria-labelledby="agenda-heading" className="space-y-4">
      <h3 id="agenda-heading" className="text-lg font-semibold">Agenda de citas</h3>
      <p className="text-sm text-gray-600">
        Día civil según la zona de la cuenta ({zone}). Hoy y mañana se calculan con esa zona;
        un * marca citas provisionales, que no son aceptación remota.
      </p>
      <div className="flex flex-wrap items-center gap-2">
        <button type="button" onClick={() => setCivil(today)} disabled={civil === today}>Hoy</button>
        <button type="button" onClick={() => setCivil(tomorrow)} disabled={civil === tomorrow}>Mañana</button>
        <label className="flex items-center gap-2 text-sm">
          Día
          <input type="date" value={civil} onChange={(e) => setCivil(e.target.value)} />
        </label>
        <span className="text-sm text-gray-600">{total} cita(s) en el día.</span>
      </div>
      <p role="status">{message}</p>

      <ul className="space-y-2 text-sm">
        {rows.map((row) => {
          const state = visibleOf(row);
          if (!state) return null;
          return (
            <li key={state.appointmentId} className="space-y-1 border-b border-gray-100 pb-2">
              <button type="button" className="text-left" onClick={() => setSelected(state.appointmentId)}>
                <strong>{state.startsAt.slice(11, 16)}–{state.endsAt.slice(11, 16)}</strong>{" "}
                {state.serviceName} · {state.professionalName} ·{" "}
                {statusLabel[state.status] ?? state.status}
                {row.provisional && <em> *</em>}
              </button>
              {selected === state.appointmentId && (
                <AppointmentDetail appointmentId={state.appointmentId} onChanged={setMessage} />
              )}
            </li>
          );
        })}
        {!rows.length && <li>Sin citas en este día.</li>}
      </ul>
    </section>
  );
}

function AppointmentDetail({
  appointmentId, onChanged,
}: {
  appointmentId: string;
  onChanged: (message: string) => void;
}) {
  const [row, setRow] = useState<EntityRow<AppointmentState> | null>(null);
  const [history, setHistory] = useState<HistoryView | null>(null);

  useEffect(() => {
    let cancelled = false;
    const load = async () => {
      try {
        const [detail, hist] = await Promise.all([
          getAppointment(appointmentId),
          getHistory(appointmentId),
        ]);
        if (!cancelled) { setRow(detail); setHistory(hist); }
      } catch (error: unknown) {
        if (!cancelled) onChanged(errorMessage(error));
      }
    };
    void load();
    return () => { cancelled = true; };
  }, [appointmentId]);

  if (!row) return <p className="text-sm">Cargando detalle…</p>;
  const state = visibleOf(row);

  return (
    <div className="space-y-3 border-l-2 border-gray-200 pl-3">
      {state && (
        <dl className="grid gap-2 sm:grid-cols-2 text-sm">
          <div><dt className="text-gray-600">Paciente</dt><dd className="font-mono">{state.patientId}</dd></div>
          <div><dt className="text-gray-600">Servicio</dt><dd>{state.serviceName} ({state.offeredDurationMinutes} min ofrecidos)</dd></div>
          <div><dt className="text-gray-600">Profesional</dt><dd>{state.professionalName}</dd></div>
          <div><dt className="text-gray-600">Lugar</dt>
            <dd>{state.locationName} · {state.locationAddress} · {state.locationCity}, {state.locationState}, {state.locationCountry} · {state.timeZone}</dd></div>
          <div><dt className="text-gray-600">Intervalo (UTC)</dt><dd className="font-mono">{state.startsAt} – {state.endsAt}</dd></div>
          <div><dt className="text-gray-600">Responsable</dt>
            <dd className="font-mono">{state.responsibleContactId ?? "—"}</dd></div>
          <div><dt className="text-gray-600">Confirmación del servidor</dt>
            <dd>{state.confirmedAt ? <code>{state.confirmedAt}</code> : "sin confirmar (nunca se presenta la hora local como confirmación)"}</dd></div>
        </dl>
      )}
      <AppointmentActions row={row} onChanged={onChanged} />

      <div className="space-y-1">
        <h4 className="font-semibold">Historia aceptada (inmutable)</h4>
        {!history || !history.accepted.length ? (
          <p className="text-sm text-gray-600">
            Sin historia aceptada descargada; no se presenta una historia local como si fuera aceptada.
          </p>
        ) : (
          <ol className="list-decimal pl-5 text-sm">
            {history.accepted.map((entry) => (
              <li key={entry.version}>
                v{entry.version} · {entry.action} · {statusLabel[entry.status] ?? entry.status} ·{" "}
                <code>{entry.changedAt}</code>
                {entry.changeReason ? ` · “${entry.changeReason}”` : ""}
              </li>
            ))}
          </ol>
        )}
        {history && history.operations.length > 0 && (
          <p className="text-sm text-gray-600">
            Operaciones locales pendientes de aceptación: {history.operations.length} (separadas de la historia aceptada).
          </p>
        )}
      </div>
    </div>
  );
}
