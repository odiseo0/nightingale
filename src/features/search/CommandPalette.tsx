import { useCallback, useEffect, useRef, useState } from "react";
import { useNavigate } from "react-router-dom";
import { CircleNotch, MagnifyingGlass, User, CalendarBlank } from "@phosphor-icons/react";
import { listAppointments, getBookingContext, type AppointmentState, type BookingContext, type EntityRow as AppointmentRow, visibleOf as visibleAppointment } from "../../shared/api/appointments";
import { listPatients, type EntityRow as PatientRow, type ProfileState } from "../../shared/api/patients";
import { civilToday, dayRange, errorMessage, formatDateFull, formatTime, formatTimeEnd, patientNameOf, statusLabel } from "../appointments/presentation";

interface SearchResult {
  key: string;
  type: "paciente" | "cita";
  title: string;
  context: string;
  to: string;
}

interface Props {
  open: boolean;
  onClose: () => void;
  zone: string;
}

/** Búsqueda global (command palette): pacientes y citas de los datos locales
 * autorizados. Sin coincidencias locales no prueba inexistencia global. */
export default function CommandPalette({ open, onClose, zone }: Props) {
  const navigate = useNavigate();
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<SearchResult[] | null>(null);
  const [state, setState] = useState<"idle" | "loading" | "ready" | "error">("idle");
  const [error, setError] = useState<string | null>(null);
  const generation = useRef(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const listRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (open) {
      setQuery("");
      setResults(null);
      setState("idle");
      window.setTimeout(() => inputRef.current?.focus(), 30);
    }
  }, [open]);

  const search = useCallback(async (q: string) => {
    const request = ++generation.current;
    setState("loading");
    try {
      const trimmed = q.trim();
      const [patientPage, appointmentPage, context] = await Promise.all([
        listPatients({ q: trimmed, shows: 8 }),
        listAppointments({
          /* El contrato limita el rango a 366 días: 300 pasados + 60 futuros. */
          from: new Date(Date.parse(dayRange(zone, civilToday(zone)).from) - 300 * 86_400_000).toISOString(),
          until: new Date(Date.parse(dayRange(zone, civilToday(zone)).from) + 60 * 86_400_000).toISOString(),
          shows: 50,
        }),
        getBookingContext(),
      ]);
      if (request !== generation.current) return;
      const found: SearchResult[] = [];
      for (const row of patientPage.items as PatientRow<ProfileState>[]) {
        const profile = row.provisional?.state ?? row.accepted?.state ?? null;
        if (!profile) continue;
        found.push({
          key: `paciente:${profile.patientId}`,
          type: "paciente",
          title: profile.fullName?.trim() || "Paciente sin nombre registrado",
          context: profile.phone ? `Teléfono ${profile.phoneCode ?? ""} ${profile.phone}` : "Paciente",
          to: `/paciente/${profile.patientId}`,
        });
      }
      const lower = trimmed.toLowerCase();
      const seenPatients = new Set<string>();
      for (const row of appointmentPage.items as AppointmentRow<AppointmentState>[]) {
        const state = visibleAppointment(row);
        if (!state) continue;
        const patientName = patientNameOf(context as BookingContext | null, state.patientId);
        if (trimmed && !patientName.toLowerCase().includes(lower) && !state.serviceName.toLowerCase().includes(lower)) {
          continue;
        }
        if (trimmed && patientName.toLowerCase().includes(lower) && seenPatients.has(state.patientId)) continue;
        seenPatients.add(state.patientId);
        found.push({
          key: `cita:${state.appointmentId}`,
          type: "cita",
          title: patientName,
          context: `${formatDateFull(zone, state.startsAt)} · ${formatTime(zone, state.startsAt)}–${formatTimeEnd(zone, state.endsAt)} · ${statusLabel[state.status] ?? state.status}`,
          to: `/cita/${state.appointmentId}`,
        });
        if (found.length >= 12) break;
      }
      setResults(found);
      setState("ready");
    } catch (searchError: unknown) {
      if (request !== generation.current) return;
      setState("error");
      setError(errorMessage(searchError));
    }
  }, [zone]);

  useEffect(() => {
    if (!open) return;
    if (query.trim().length < 2) {
      generation.current += 1;
      setResults(null);
      setState("idle");
      return;
    }
    const timer = window.setTimeout(() => void search(query), 300);
    return () => { generation.current += 1; window.clearTimeout(timer); };
  }, [query, open, search]);

  if (!open) return null;

  function openResult(to: string) {
    onClose();
    navigate(to);
  }

  return (
    <div className="modal-backdrop" role="dialog" aria-modal="true" aria-label="Búsqueda global" onClick={onClose}>
      <div className="modal-card" style={{ maxWidth: 560 }} onClick={(event) => event.stopPropagation()}>
        <label className="field-label">
          <span className="muted"><MagnifyingGlass size={14} aria-hidden="true" /> Buscar pacientes y citas</span>
          <input
            ref={inputRef}
            type="search"
            className="input-text"
            placeholder="Escribe al menos 2 caracteres"
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            onKeyDown={(event) => {
              if (event.key === "Escape") {
                event.stopPropagation();
                onClose();
              } else if (event.key === "ArrowDown" || event.key === "ArrowUp") {
                event.preventDefault();
                const buttons = listRef.current?.querySelectorAll("button") ?? [];
                if (!buttons.length) return;
                const current = buttons[0].ownerDocument.activeElement;
                const index = [...buttons].indexOf(current as HTMLButtonElement);
                const next = event.key === "ArrowDown"
                  ? buttons[Math.min(buttons.length - 1, index + 1 < 0 ? 0 : index + 1)]
                  : buttons[Math.max(0, index <= 0 ? 0 : index - 1)];
                (next as HTMLButtonElement).focus();
              }
            }}
          />
        </label>
        <p className="muted" style={{ fontSize: 13 }}>
          Búsqueda solo en los datos disponibles en este equipo; sin coincidencias locales no prueba que no exista en otra copia autorizada.
        </p>
        <div ref={listRef} role="listbox" aria-label="Resultados" style={{ display: "grid", gap: 6, maxHeight: 320, overflowY: "auto" }}>
          {state === "loading" && <p className="muted" role="status"><CircleNotch size={16} aria-hidden="true" /> Buscando…</p>}
          {state === "error" && <p className="notice" role="alert">No se pudo buscar. {error}</p>}
          {state === "ready" && results !== null && results.length === 0 && (
            <p className="muted">Sin coincidencias en los datos locales de este equipo.</p>
          )}
          {results?.map((result) => (
            <button
              key={result.key}
              type="button"
              role="option"
              aria-selected={false}
              className="btn"
              style={{ justifyContent: "flex-start", textAlign: "left" }}
              onClick={() => openResult(result.to)}
            >
              {result.type === "paciente" ? <User size={16} aria-hidden="true" /> : <CalendarBlank size={16} aria-hidden="true" />}
              <span>
                <strong>{result.title}</strong>
                <span className="muted" style={{ fontSize: 13, display: "block" }}>
                  {result.type === "paciente" ? "Paciente" : "Cita"} · {result.context}
                </span>
              </span>
            </button>
          ))}
        </div>
        <p className="muted" style={{ fontSize: 12 }}>Escape cierra y devuelve el foco al disparador.</p>
      </div>
    </div>
  );
}
