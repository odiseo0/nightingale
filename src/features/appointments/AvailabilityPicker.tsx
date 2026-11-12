import { CircleNotch, Info } from "@phosphor-icons/react";
import type { AvailabilityEvaluation } from "../../shared/api/scheduling";
import { coverageLabel, formatTime, formatTimeEnd } from "./presentation";

export type AvailabilityState =
  | { status: "idle" }
  | { status: "loading" }
  | { status: "error"; message: string }
  | { status: "ready"; evaluation: AvailabilityEvaluation };

interface Props {
  zone: string;
  availability: AvailabilityState;
  slot: { startsAt: string; endsAt: string } | null;
  onSlotChange: (slot: { startsAt: string; endsAt: string } | null) => void;
  onRetry: () => void;
}

/** Intervalos devueltos por Rust para el día y la duración efectiva elegidos.
 * React no calcula disponibilidad: solo presenta los intervalos del proceso local. */
export default function AvailabilityPicker({ zone, availability, slot, onSlotChange, onRetry }: Props) {
  if (availability.status === "idle") {
    return (
      <div className="notice">
        <Info size={18} aria-hidden="true" />
        <p>Elige servicio y fecha para consultar la disponibilidad local. Consultar no reserva.</p>
      </div>
    );
  }

  if (availability.status === "loading") {
    return (
      <p className="muted" role="status">
        <CircleNotch size={16} aria-hidden="true" /> Calculando disponibilidad…
      </p>
    );
  }

  if (availability.status === "error") {
    return (
      <div className="notice" role="alert">
        <p>No se pudo evaluar la disponibilidad local. {availability.message}</p>
        <button type="button" className="btn" onClick={onRetry}>Reintentar</button>
      </div>
    );
  }

  const { evaluation } = availability;
  const intervals = evaluation.intervals;

  return (
    <div style={{ display: "grid", gap: 12 }}>
      <p className="muted" role="status">
        {coverageLabel[evaluation.coverage] ?? evaluation.coverage}
      </p>

      {intervals.length === 0 ? (
        <div className="notice">
          <Info size={18} aria-hidden="true" />
          <p>
            {evaluation.coverage === "within_synced_window"
              ? "Sin huecos conocibles en este día para la duración del servicio."
              : "No hay intervalos calculables con los datos locales conocidos; esto no prueba que el día esté completo."}
          </p>
        </div>
      ) : (
        <div
          className="interval-list"
          role="group"
          aria-label="Intervalos disponibles"
        >
          {intervals.map((interval) => {
            const selected = slot?.startsAt === interval.startsAt && slot?.endsAt === interval.endsAt;
            return (
              <button
                key={interval.startsAt}
                type="button"
                className={`btn ${selected ? "btn--primary" : ""}`}
                aria-pressed={selected}
                onClick={() => onSlotChange(selected ? null : interval)}
              >
                <span className="num">
                  {formatTime(zone, interval.startsAt)}–{formatTimeEnd(zone, interval.endsAt)}
                </span>
              </button>
            );
          })}
        </div>
      )}
    </div>
  );
}
