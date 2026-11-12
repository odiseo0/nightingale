import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { Link } from "react-router-dom";
import { CaretLeft, CaretRight, CircleNotch } from "@phosphor-icons/react";
import {
  listAppointments,
  type AppointmentState, type EntityRow, visibleOf,
} from "../../shared/api/appointments";
import { listBlocks, type BlockState } from "../../shared/api/scheduling";
import {
  civilMidnightUtc, civilToday, dayRange, errorMessage, formatTime, formatTimeEnd,
  shiftCivil, statusLabel,
} from "./presentation";
import {
  assignColumns, eventBox, visibleMinuteRange, WEEK_HOUR_HEIGHT,
} from "./week-layout";

interface WeekEvent {
  key: string;
  startMin: number;
  endMin: number;
  label: string;
  detail: string;
  status: string;
  provisional: boolean;
  appointmentId: string;
}

interface Props {
  zone: string;
  civil: string;
  onCivilChange: (civil: string) => void;
  patientName: (patientId: string) => string;
}

const PX_PER_MINUTE = WEEK_HOUR_HEIGHT / 60;

/** Vista Semana: escala 96 px/hora, columnas sin solapamiento y rango visible
 * que incluye los eventos conocidos. Sin drag-and-drop ni reserva desde
 * celdas. */
export default function WeekPage({ zone, civil, onCivilChange, patientName }: Props) {
  const [rows, setRows] = useState<EntityRow<AppointmentState>[] | null>(null);
  const [blocks, setBlocks] = useState<EntityRow<BlockState>[]>([]);
  const [error, setError] = useState<string | null>(null);
  const generation = useRef(0);
  const scrollRef = useRef<HTMLDivElement>(null);
  const SCROLL_KEY = "semana:scroll";

  /* Lunes de la semana que contiene `civil` (zona de la cuenta). Memorizado
   * para que la lectura dependa de valores estables y no se repita en cada
   * render. */
  const weekStartCivil = useMemo(() => {
    const instant = new Date(civilMidnightUtc(zone, ...civil.split("-").map(Number) as [number, number, number]));
    const weekdayName = new Intl.DateTimeFormat("en-US", { timeZone: zone, weekday: "short" }).format(instant);
    const weekday = ({ Sun: 0, Mon: 1, Tue: 2, Wed: 3, Thu: 4, Fri: 5, Sat: 6 } as Record<string, number>)[weekdayName] ?? 0;
    const monday = new Date(instant.getTime() - ((weekday + 6) % 7) * 86_400_000);
    const parts = new Intl.DateTimeFormat("en-CA", { timeZone: zone, year: "numeric", month: "2-digit", day: "2-digit" }).formatToParts(monday);
    const v = (t: string) => parts.find((p) => p.type === t)?.value ?? "";
    return `${v("year")}-${v("month")}-${v("day")}`;
  }, [zone, civil]);

  const days = useMemo(
    () => Array.from({ length: 7 }, (_, i) => shiftCivil(zone, weekStartCivil, i)),
    [zone, weekStartCivil],
  );

  /* Conservar la posición de lectura al volver del detalle. */
  useEffect(() => {
    return () => {
      const element = scrollRef.current;
      if (element) sessionStorage.setItem(SCROLL_KEY, String(element.scrollTop));
    };
  }, []);

  useEffect(() => {
    const element = scrollRef.current;
    if (!element || rows === null) return;
    const saved = sessionStorage.getItem(SCROLL_KEY);
    if (saved !== null) {
      sessionStorage.removeItem(SCROLL_KEY);
      element.scrollTop = Number(saved) || 0;
    }
  }, [rows]);

  const load = useCallback(async () => {
    const request = ++generation.current;
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
      const timeRange = `${formatTime(zone, state.startsAt)}–${formatTimeEnd(zone, state.endsAt)}`;
      const name = patientName(state.patientId);
      visible.push({
        key: state.appointmentId,
        startMin: Math.max(0, Math.round((start - fromMs) / 60_000)),
        endMin: Math.min(24 * 60, Math.round((end - fromMs) / 60_000)),
        label: `${timeRange} ${name}`,
        detail: `${name} · ${state.serviceName} · ${timeRange} · ${statusLabel[state.status] ?? state.status}`,
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
      const startMin = Math.max(0, Math.round((start - fromMs) / 60_000));
      const endMin = Math.min(24 * 60, Math.round((end - fromMs) / 60_000));
      visible.push({
        key: block.blockId,
        startMin,
        endMin,
        label: `Bloqueado${block.reasonCode ? `: ${block.reasonCode}` : ""}`,
        detail: `Bloqueado ${formatTime(zone, block.startsAt)}–${formatTimeEnd(zone, block.endsAt)}`,
        status: "block",
        provisional: Boolean(row.provisional),
        appointmentId: "",
      });
    }
    return visible;
  };

  /* Rango visible: horario base ampliado con los eventos y bloqueos de la
   * semana, para que nada aparezca bajo una hora distinta de la real. */
  const allMinuteSpans = days.flatMap((day) =>
    eventsOfDay(day).map((event) => ({ startMin: event.startMin, endMin: event.endMin })));
  const range = visibleMinuteRange(allMinuteSpans);
  const bodyHeight = (range.endMinute - range.startMinute) * PX_PER_MINUTE;
  const hours = Array.from(
    { length: (range.endMinute - range.startMinute) / 60 },
    (_, i) => range.startMinute / 60 + i,
  );

  return (
    <>
      <div className="toolbar" role="group" aria-label="Navegación de semana">
        <button type="button" className="btn" onClick={() => onCivilChange(shiftCivil(zone, weekStartCivil, -7))}>
          <CaretLeft size={16} aria-hidden="true" />
          Semana anterior
        </button>
        <button type="button" className="btn btn--subtle" onClick={() => onCivilChange(civilToday(zone))}>
          Semana actual
        </button>
        <button type="button" className="btn" onClick={() => onCivilChange(shiftCivil(zone, weekStartCivil, 7))}>
          Semana siguiente
          <CaretRight size={16} aria-hidden="true" />
        </button>
        <span className="muted" role="status" aria-live="polite">
          {rows === null ? " " : `${rows.length} cita(s) en la semana (datos locales).`}
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
        <div className="table-scroll table-scroll--week" role="region" aria-label="Vista semanal" tabIndex={0} ref={scrollRef}>
          <div className="week-grid">
            <div className="week-corner" aria-hidden="true" />
            {days.map((day) => (
              <div key={day} className="week-day-header" data-today={day === civil || undefined}>
                <button
                  type="button"
                  className="btn btn--subtle"
                  style={{ padding: "2px 8px", minHeight: 30 }}
                  onClick={() => onCivilChange(day)}
                  aria-label={`Ver ${dayDate(day)} en la vista de día`}
                >
                  {dayDate(day)}
                </button>
              </div>
            ))}
            <div className="week-hours" style={{ height: bodyHeight }} aria-hidden="true">
              {hours.map((hour) => (
                <span
                  key={hour}
                  className="week-hour-label num"
                  style={{ top: (hour * 60 - range.startMinute) * PX_PER_MINUTE }}
                >
                  {String(hour).padStart(2, "0")}:00
                </span>
              ))}
            </div>
            {days.map((day) => {
              const placed = assignColumns(eventsOfDay(day));
              return (
                <div key={day} className="week-day-body" style={{ height: bodyHeight }}>
                  {placed.map(({ item: event, column, columns }) => {
                    const box = eventBox(event.startMin, event.endMin, range);
                    const width = 100 / columns;
                    const style = {
                      top: box.top,
                      height: Math.max(box.height, 18),
                      left: `calc(${width * column}% + 2px)`,
                      width: `calc(${width}% - 4px)`,
                    };
                    const brief = box.height < 40;
                    if (!event.appointmentId) {
                      return (
                        <div
                          key={event.key}
                          className="week-event week-event--block"
                          style={style}
                          aria-label={event.detail}
                          title={event.detail}
                        >
                          <span className="week-event-text">{event.label}</span>
                        </div>
                      );
                    }
                    return (
                      <Link
                        key={event.key}
                        to={`/cita/${event.appointmentId}`}
                        className={[
                          "week-event",
                          event.provisional ? "week-event--provisional" : "",
                          event.status === "cancelled" ? "week-event--cancelled" : "",
                          brief ? "week-event--brief" : "",
                        ].filter(Boolean).join(" ")}
                        style={style}
                        title={event.detail}
                      >
                        {brief ? (
                          <span className="week-event-text">{event.label}</span>
                        ) : (
                          <>
                            <span className="week-event-text">{event.label}</span>
                            {!brief && box.height >= 60 && (
                              <span className="week-event-status">
                                {statusLabel[event.status] ?? event.status}
                              </span>
                            )}
                          </>
                        )}
                      </Link>
                    );
                  })}
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
