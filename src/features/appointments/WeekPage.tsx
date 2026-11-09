import { useCallback, useEffect, useRef, useState } from "react";
import { Link } from "react-router-dom";
import { CaretLeft, CaretRight, CircleNotch } from "@phosphor-icons/react";
import {
  listAppointments,
  type AppointmentState, type EntityRow, visibleOf,
} from "../../shared/api/appointments";
import { listBlocks, type BlockState } from "../../shared/api/scheduling";
import {
  civilMidnightUtc, dayRange, errorMessage, formatTime,
  shiftCivil, statusLabel,
} from "./presentation";

interface WeekEvent {
  key: string;
  top: number;
  height: number;
  label: string;
  status: string;
  provisional: boolean;
  appointmentId: string;
}

const DAY_START_MINUTE = 7 * 60;
const DAY_END_MINUTE = 20 * 60;
const HOUR_HEIGHT = 44;

interface Props {
  zone: string;
  civil: string;
  onCivilChange: (civil: string) => void;
  patientName: (patientId: string) => string;
}

/** T01 · Vista Semana: citas por periodo con escala horaria local y bloqueos
 * con etiqueta propia. Sin drag-and-drop ni reserva desde celdas. */
export default function WeekPage({ zone, civil, onCivilChange, patientName }: Props) {
  const [rows, setRows] = useState<EntityRow<AppointmentState>[] | null>(null);
  const [blocks, setBlocks] = useState<EntityRow<BlockState>[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [refreshing, setRefreshing] = useState(false);
  const generation = useRef(0);

  /* Lunes de la semana que contiene `civil` (zona de la cuenta). */
  const weekStartCivil = (() => {
    const instant = new Date(civilMidnightUtc(zone, ...civil.split("-").map(Number) as [number, number, number]));
    const weekdayName = new Intl.DateTimeFormat("en-US", { timeZone: zone, weekday: "short" }).format(instant);
    const weekday = ({ Sun: 0, Mon: 1, Tue: 2, Wed: 3, Thu: 4, Fri: 5, Sat: 6 } as Record<string, number>)[weekdayName] ?? 0;
    const monday = new Date(instant.getTime() - ((weekday + 6) % 7) * 86_400_000);
    const parts = new Intl.DateTimeFormat("en-CA", { timeZone: zone, year: "numeric", month: "2-digit", day: "2-digit" }).formatToParts(monday);
    const v = (t: string) => parts.find((p) => p.type === t)?.value ?? "";
    return `${v("year")}-${v("month")}-${v("day")}`;
  })();

  const days = Array.from({ length: 7 }, (_, i) =>
    shiftCivil(zone, weekStartCivil, i));

  const load = useCallback(async () => {
    const request = ++generation.current;
    setRefreshing(true);
    try {
      const from = dayRange(zone, weekStartCivil).from;
      const until = dayRange(zone, days[6]).until;
      const [page, blockRows] = await Promise.all([
        listAppointments({ from, until, shows: 500 }),
        listBlocks(),
      ]);
      if (request !== generation.current) return;
      setRows(page.items);
      setBlocks(blockRows);
      setError(null);
    } catch (loadError: unknown) {
      if (request !== generation.current) return;
      setError(errorMessage(loadError));
    } finally {
      if (request === generation.current) setRefreshing(false);
    }
  }, [zone, weekStartCivil, days]);

  useEffect(() => { void load(); }, [load]);

  const dayDate = (civilDay: string) => {
    const instant = new Date(civilMidnightUtc(zone, ...civilDay.split("-").map(Number) as [number, number, number]));
    return new Intl.DateTimeFormat("es", { timeZone: zone, weekday: "short", day: "numeric", month: "short" }).format(instant);
  };

  const eventsOfDay = (civilDay: string): WeekEvent[] => {
    const { from, until } = dayRange(zone, civilDay);
    const fromMs = Date.parse(from);
    const untilMs = Date.parse(until);
    const visible: WeekEvent[] = [];
    for (const row of rows ?? []) {
      const state = visibleOf(row);
      if (!state) continue;
      const start = Date.parse(state.startsAt);
      const end = Date.parse(state.endsAt);
      if (end <= fromMs || start >= untilMs) continue;
      const startMin = Math.max(DAY_START_MINUTE, Math.round((start - fromMs) / 60_000));
      const endMin = Math.min(DAY_END_MINUTE, Math.round((end - fromMs) / 60_000));
      visible.push({
        key: state.appointmentId,
        top: (startMin - DAY_START_MINUTE) * (HOUR_HEIGHT / 60),
        height: Math.max(18, (endMin - startMin) * (HOUR_HEIGHT / 60)),
        label: `${formatTime(zone, state.startsAt)} ${patientName(state.patientId)}`,
        status: state.status,
        provisional: Boolean(row.provisional),
        appointmentId: state.appointmentId,
      });
    }
    for (const row of blocks) {
      const block = row.provisional?.state ?? row.accepted?.state ?? null;
      if (!block || block.status !== "active") continue;
      const start = Date.parse(block.startsAt);
      const end = Date.parse(block.endsAt);
      if (end <= fromMs || start >= untilMs) continue;
      const startMin = Math.max(DAY_START_MINUTE, Math.round((start - fromMs) / 60_000));
      const endMin = Math.min(DAY_END_MINUTE, Math.round((end - fromMs) / 60_000));
      visible.push({
        key: block.blockId,
        top: (startMin - DAY_START_MINUTE) * (HOUR_HEIGHT / 60),
        height: Math.max(16, (endMin - startMin) * (HOUR_HEIGHT / 60)),
        label: `Bloqueado${block.reasonCode ? `: ${block.reasonCode}` : ""}`,
        status: "block",
        provisional: Boolean(row.provisional),
        appointmentId: "",
      });
    }
    return visible;
  };

  const hours = Array.from(
    { length: (DAY_END_MINUTE - DAY_START_MINUTE) / 60 },
    (_, i) => DAY_START_MINUTE / 60 + i,
  );

  return (
    <>
      <div className="toolbar" role="group" aria-label="Navegación de semana">
        <button type="button" className="btn" onClick={() => onCivilChange(shiftCivil(zone, weekStartCivil, -7))}>
          <CaretLeft size={16} aria-hidden="true" />
          Semana anterior
        </button>
        <button type="button" className="btn btn--subtle" onClick={() => onCivilChange(civil)}>
          Semana actual
        </button>
        <button type="button" className="btn" onClick={() => onCivilChange(shiftCivil(zone, weekStartCivil, 7))}>
          Semana siguiente
          <CaretRight size={16} aria-hidden="true" />
        </button>
        <span className="muted" role="status" aria-live="polite">
          {refreshing ? "Actualizando…" : `${(rows ?? []).length} cita(s) en la semana (datos locales).`}
        </span>
      </div>

      {error && (
        <div className="notice" role="alert">
          <p>No se pudo leer la semana. {error}</p>
          <button type="button" className="btn" onClick={() => void load()}>Reintentar</button>
        </div>
      )}

      {!error && rows === null && (
        <div className="state-block" role="status">
          <CircleNotch size={22} aria-hidden="true" />
          <p>Cargando la semana…</p>
        </div>
      )}

      {rows !== null && (
        <div className="table-scroll" role="region" aria-label="Vista semanal" tabIndex={0}>
          <div className="week-grid">
            <div className="week-hours" aria-hidden="true">
              <div style={{ height: HOUR_HEIGHT }} />
              {hours.map((hour) => (
                <div key={hour} className="week-hour-label num">
                  {String(hour).padStart(2, "0")}:00
                </div>
              ))}
            </div>
            {days.map((day) => {
              const isToday = day === civil;
              return (
                <div key={day} className="week-day" data-today={isToday || undefined}>
                  <div className="week-day-header">
                    <button
                      type="button"
                      className="btn btn--subtle"
                      style={{ padding: "2px 8px" }}
                      onClick={() => onCivilChange(day)}
                      aria-label={`Ver ${dayDate(day)} en la vista de día`}
                    >
                      {dayDate(day)}
                    </button>
                  </div>
                  <div className="week-day-body">
                    {eventsOfDay(day).map((event) => event.appointmentId ? (
                      <Link
                        key={event.key}
                        to={`/cita/${event.appointmentId}`}
                        className={`week-event${event.provisional ? " week-event--provisional" : ""}`}
                        style={{ top: event.top, height: event.height }}
                      >
                        <span className="week-event-text">{event.label}</span>
                        <span className="badge">{statusLabel[event.status] ?? event.status}</span>
                      </Link>
                    ) : (
                      <div
                        key={event.key}
                        className="week-event week-event--block"
                        style={{ top: event.top, height: event.height }}
                        aria-label={event.label}
                      >
                        <span className="week-event-text">{event.label}</span>
                      </div>
                    ))}
                  </div>
                </div>
              );
            })}
          </div>
        </div>
      )}
      <p className="muted" style={{ fontSize: 13 }}>
        Los huecos visibles no demuestran disponibilidad: la ocupación real la calcula el proceso
        local y los filtros no afectan a esta vista.
      </p>
    </>
  );
}
