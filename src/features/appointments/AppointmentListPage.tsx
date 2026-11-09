import { useCallback, useEffect, useRef, useState, type FormEvent } from "react";
import { Link } from "react-router-dom";
import { CaretLeft, CaretRight, MagnifyingGlass } from "@phosphor-icons/react";
import {
  listAppointments,
  type AppointmentState, type AppointmentStatus, type BookingContext, type EntityRow,
  visibleOf,
} from "../../shared/api/appointments";
import {
  civilToday, dayRange, errorMessage, formatDateFull, formatTime, formatTimeEnd,
  patientNameOf, statusLabel, syncLabel,
} from "./presentation";

const PAGE_SIZE = 15;

interface Props {
  zone: string;
  context: BookingContext | null;
  professionals: { professionalId: string; publicName: string }[];
  serviceNames: Map<string, string>;
}

/** T02 · Vista Lista: citas próximas e historial con búsqueda, periodo,
 * filtros y paginación real. Canceladas/pasadas solo por filtro/contexto. */
export default function AppointmentListPage({ zone, context, professionals, serviceNames }: Props) {
  const [draft, setDraft] = useState({ q: "", status: "", window: "next" as "next" | "past", professionalId: "", serviceId: "" });
  const [applied, setApplied] = useState(draft);
  const [page, setPage] = useState(1);
  const [rows, setRows] = useState<EntityRow<AppointmentState>[] | null>(null);
  const [total, setTotal] = useState(0);
  const [refreshing, setRefreshing] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const generation = useRef(0);
  const rowsRef = useRef<EntityRow<AppointmentState>[] | null>(null);

  const rangeOf = useCallback((window: "next" | "past") => {
    const today = civilToday(zone);
    if (window === "next") {
      const start = dayRange(zone, today).from;
      const until = new Date(Date.parse(start) + 60 * 86_400_000).toISOString();
      return { from: start, until };
    }
    const end = dayRange(zone, today).from;
    const from = new Date(Date.parse(end) - 365 * 86_400_000).toISOString();
    return { from, until: end };
  }, [zone]);

  const load = useCallback(async () => {
    const request = ++generation.current;
    setRefreshing(true);
    try {
      const result = await listAppointments({
        ...rangeOf(applied.window),
        page,
        shows: PAGE_SIZE,
        ...(applied.status ? { status: applied.status as AppointmentStatus } : {}),
        ...(applied.professionalId ? { professionalId: applied.professionalId } : {}),
        ...(applied.serviceId ? { serviceId: applied.serviceId } : {}),
      });
      if (request !== generation.current) return;
      rowsRef.current = result.items;
      setRows(result.items);
      setTotal(result.total);
      setError(null);
    } catch (loadError: unknown) {
      if (request !== generation.current) return;
      if (rowsRef.current === null) setError(errorMessage(loadError));
    } finally {
      if (request === generation.current) setRefreshing(false);
    }
  }, [applied, page, rangeOf]);

  useEffect(() => { void load(); }, [load]);

  function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setPage(1);
    setApplied({ ...draft });
  }

  /* Búsqueda local por texto sobre las fichas descargadas (el contrato de
     citas no acepta q; se filtra solo dentro de los resultados cargados). */
  const visibleRows = (rows ?? [])
    .map((row) => ({ row, state: visibleOf(row) }))
    .filter((item): item is { row: EntityRow<AppointmentState>; state: AppointmentState } =>
      item.state !== null)
    .filter(({ state }) => !applied.q
      || patientNameOf(context, state.patientId).toLowerCase().includes(applied.q.toLowerCase())
      || state.serviceName.toLowerCase().includes(applied.q.toLowerCase()));

  const filterActive = applied.q !== "" || applied.status !== "" || applied.professionalId !== "" || applied.serviceId !== "";
  const totalPages = Math.max(1, Math.ceil(total / PAGE_SIZE));

  return (
    <>
      <form className="toolbar" role="search" aria-label="Filtros de la lista" onSubmit={submit}>
        <MagnifyingGlass size={16} aria-hidden="true" style={{ color: "var(--color-text-tertiary)" }} />
        <label className="field-label">
          Buscar
          <input
            type="search"
            className="input-text"
            style={{ minWidth: 200 }}
            placeholder="Paciente o servicio"
            value={draft.q}
            onChange={(event) => setDraft({ ...draft, q: event.target.value })}
          />
        </label>
        <label className="field-label">
          Periodo
          <select
            className="select"
            value={draft.window}
            onChange={(event) => setDraft({ ...draft, window: event.target.value as "next" | "past" })}
          >
            <option value="next">Próximas citas</option>
            <option value="past">Historial (último año)</option>
          </select>
        </label>
        <label className="field-label">
          Estado
          <select
            className="select"
            value={draft.status}
            onChange={(event) => setDraft({ ...draft, status: event.target.value })}
          >
            <option value="">Todos</option>
            {Object.entries(statusLabel).map(([key, label]) => (
              <option key={key} value={key}>{label}</option>
            ))}
          </select>
        </label>
        <label className="field-label">
          Profesional
          <select
            className="select"
            value={draft.professionalId}
            onChange={(event) => setDraft({ ...draft, professionalId: event.target.value })}
          >
            <option value="">Todos</option>
            {professionals.map((p) => (
              <option key={p.professionalId} value={p.professionalId}>{p.publicName}</option>
            ))}
          </select>
        </label>
        <label className="field-label">
          Servicio
          <select
            className="select"
            value={draft.serviceId}
            onChange={(event) => setDraft({ ...draft, serviceId: event.target.value })}
          >
            <option value="">Todos</option>
            {[...serviceNames].map(([id, name]) => (
              <option key={id} value={id}>{name}</option>
            ))}
          </select>
        </label>
        <button type="submit" className="btn btn--primary">Buscar</button>
        {filterActive && (
          <button type="button" className="btn" onClick={() => { const clean = { q: "", status: "", window: draft.window, professionalId: "", serviceId: "" }; setDraft(clean); setApplied(clean); setPage(1); }}>
            Limpiar
          </button>
        )}
      </form>

      <p role="status" className="muted" aria-live="polite">
        {refreshing && rows !== null && "Actualizando…"}
        {!refreshing && `${total} cita(s) en el periodo, datos disponibles en este equipo.`}
        {applied.q && " La búsqueda se aplica sobre los resultados cargados."}
      </p>

      {error && rows === null && (
        <div className="state-block" role="alert">
          <p>No se pudo leer la lista local.</p>
          <p className="muted">{error}</p>
          <button type="button" className="btn" onClick={() => void load()}>Reintentar</button>
        </div>
      )}

      {rows !== null && visibleRows.length === 0 && !refreshing && (
        <div className="state-block">
          {filterActive ? (
            <>
              <p>Ninguna cita coincide con la búsqueda en los datos cargados.</p>
              <button type="button" className="btn" onClick={() => { const clean = { q: "", status: "", window: applied.window, professionalId: "", serviceId: "" }; setDraft(clean); setApplied(clean); setPage(1); }}>
                Limpiar búsqueda y filtros
              </button>
            </>
          ) : (
            <p>Sin citas en este periodo en los datos disponibles en este equipo.</p>
          )}
        </div>
      )}

      {visibleRows.length > 0 && (
        <div className="table-scroll">
          <table className="data-table">
            <caption className="muted" style={{ textAlign: "left", padding: "8px 12px" }}>
              Lista de citas
            </caption>
            <thead>
              <tr>
                <th scope="col">Fecha</th>
                <th scope="col">Intervalo</th>
                <th scope="col">Paciente</th>
                <th scope="col">Servicio</th>
                <th scope="col">Profesional</th>
                <th scope="col">Estado</th>
                <th scope="col">Sincronización</th>
              </tr>
            </thead>
            <tbody>
              {visibleRows.map(({ row, state }) => (
                <tr key={state.appointmentId}>
                  <td>
                    <Link to={`/cita/${state.appointmentId}`}>{formatDateFull(zone, state.startsAt)}</Link>
                  </td>
                  <td className="num">
                    {formatTime(zone, state.startsAt)}–{formatTimeEnd(zone, state.endsAt)}
                  </td>
                  <td>{patientNameOf(context, state.patientId)}</td>
                  <td>{state.serviceName}</td>
                  <td>{state.professionalName}</td>
                  <td><span className="badge">{statusLabel[state.status] ?? state.status}</span></td>
                  <td>
                    {row.provisional && <span className="badge badge--selected">Guardada en este equipo</span>}
                    {row.operations.map((op) => (
                      <span key={op.operationId} className="badge badge--local">
                        {syncLabel[op.status] ?? op.status}
                      </span>
                    ))}
                    {!row.provisional && row.operations.length === 0 && (
                      <span className="muted" style={{ fontSize: 13 }}>Aceptada</span>
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {total > PAGE_SIZE && (
        <div className="toolbar" role="group" aria-label="Paginación">
          <button type="button" className="btn" disabled={page <= 1 || refreshing}
            onClick={() => setPage(page - 1)}>
            <CaretLeft size={16} aria-hidden="true" />
            Anterior
          </button>
          <span className="muted" aria-live="polite">Página {page} de {totalPages}</span>
          <button type="button" className="btn" disabled={page >= totalPages || refreshing}
            onClick={() => setPage(page + 1)}>
            Siguiente
            <CaretRight size={16} aria-hidden="true" />
          </button>
        </div>
      )}
    </>
  );
}
