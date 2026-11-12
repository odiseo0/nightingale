import { useCallback, useEffect, useRef, useState } from "react";
import { useNavigate } from "react-router-dom";
import { CaretLeft, CaretRight, CircleNotch, Funnel, Plus } from "@phosphor-icons/react";
import {
  getBookingContext, listAppointments,
  type AppointmentState, type AppointmentStatus, type BookingContext,
  type EntityRow, visibleOf,
} from "../../shared/api/appointments";
import { getAccountView } from "../../shared/api/account";
import {
  civilToday, dayRange, errorMessage, formatDateLong, formatTime, formatTimeEnd,
  patientNameOf, shiftCivil, showsProfessionalColumn, statusLabel,
} from "./presentation";
import AppointmentDetail from "./AppointmentDetail";
import WeekPage from "./WeekPage";
import AppointmentListPage from "./AppointmentListPage";

interface Filters {
  professionalId: string;
  serviceId: string;
  status: string;
}

type AgendaView = "day" | "week" | "list";

const NO_FILTER = "";
const STORAGE_KEY = "agenda:estado";
const MESSAGE_KEY = "agenda:mensaje";
const FOCUS_RETURN_KEY = "agenda:retornoFoco";
const WIDE_QUERY = "(min-width: 1024px)";

type SessionState = "loading" | "ready" | "no-session";

export default function AgendaPage() {
  const navigate = useNavigate();
  const [session, setSession] = useState<SessionState>("loading");
  const [context, setContext] = useState<BookingContext | null>(null);
  const [viewerRole, setViewerRole] = useState<"owner" | "assistant" | null>(null);
  const [zone, setZone] = useState("UTC");
  const [civil, setCivil] = useState<string | null>(null);
  const [filters, setFilters] = useState<Filters>({
    professionalId: NO_FILTER, serviceId: NO_FILTER, status: NO_FILTER,
  });
  const [rows, setRows] = useState<EntityRow<AppointmentState>[] | null>(null);
  const [total, setTotal] = useState(0);
  const [initialLoading, setInitialLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [view, setView] = useState<AgendaView>("day");
  const [isWide, setIsWide] = useState(() => window.matchMedia(WIDE_QUERY).matches);
  const generation = useRef(0);
  const rowsRef = useRef<EntityRow<AppointmentState>[] | null>(null);
  const rowRefs = useRef(new Map<string, HTMLButtonElement>());
  const focusReturnIdRef = useRef<string | null>(null);

  /* Restaurar fecha/filtros/selección entre navegaciones. */
  useEffect(() => {
    try {
      const raw = sessionStorage.getItem(STORAGE_KEY);
      if (raw) {
        const saved = JSON.parse(raw) as {
          civil?: string;
          filters?: Filters;
          selectedId?: string | null;
          view?: AgendaView;
        };
        if (saved.civil) setCivil(saved.civil);
        if (saved.filters) setFilters(saved.filters);
        if (saved.selectedId) setSelectedId(saved.selectedId);
        if (saved.view) setView(saved.view);
      }
      const message = sessionStorage.getItem(MESSAGE_KEY);
      if (message) {
        sessionStorage.removeItem(MESSAGE_KEY);
        setNotice(message);
      }
    } catch { /* estado previo ilegible: se ignora */ }
  }, []);

  useEffect(() => {
    if (civil === null) return;
    sessionStorage.setItem(STORAGE_KEY, JSON.stringify({ civil, filters, selectedId, view }));
  }, [civil, filters, selectedId, view]);

  useEffect(() => {
    const media = window.matchMedia(WIDE_QUERY);
    const onChange = () => setIsWide(media.matches);
    media.addEventListener("change", onChange);
    return () => media.removeEventListener("change", onChange);
  }, []);

  const loadContext = useCallback(async () => {
    setSession("loading");
    try {
      const [data, screen] = await Promise.all([getBookingContext(), getAccountView()]);
      setContext(data);
      setViewerRole(screen.viewerRole ?? null);
      setZone(data.timeZone);
      setCivil((current) => current ?? civilToday(data.timeZone));
      setSession("ready");
    } catch {
      setSession("no-session");
    }
  }, []);

  useEffect(() => { void loadContext(); }, [loadContext]);

  const loadDay = useCallback(async () => {
    if (civil === null) return;
    const request = ++generation.current;
    const hadContent = rowsRef.current !== null;
    if (!hadContent) setInitialLoading(true);
    try {
      const page = await listAppointments({
        ...dayRange(zone, civil),
        shows: 500,
        ...(filters.professionalId ? { professionalId: filters.professionalId } : {}),
        ...(filters.serviceId ? { serviceId: filters.serviceId } : {}),
        ...(filters.status ? { status: filters.status as AppointmentStatus } : {}),
      });
      if (request !== generation.current) return;
      rowsRef.current = page.items;
      setRows(page.items);
      setTotal(page.total);
      setError(null);
    } catch (loadError: unknown) {
      if (request !== generation.current) return;
      setError(errorMessage(loadError));
    } finally {
      if (request === generation.current) {
        setInitialLoading(false);
      }
    }
  }, [zone, civil, filters]);

  useEffect(() => {
    if (session !== "ready" || civil === null) return;
    void loadDay();
    const timer = window.setInterval(() => void loadDay(), 15_000);
    return () => {
      generation.current += 1;
      window.clearInterval(timer);
    };
  }, [loadDay]);

  /* Retorno de foco tras volver del detalle propio (ventana estrecha). */
  useEffect(() => {
    if (rows === null) return;
    const returnId = focusReturnIdRef.current ?? sessionStorage.getItem(FOCUS_RETURN_KEY);
    if (returnId) {
      sessionStorage.removeItem(FOCUS_RETURN_KEY);
      focusReturnIdRef.current = null;
      const target = rowRefs.current.get(returnId);
      if (target) {
        target.focus();
      } else {
        setSelectedId(returnId);
      }
    }
  }, [rows]);

  const today = civilToday(zone);
  const filterActive = filters.professionalId !== NO_FILTER
    || filters.serviceId !== NO_FILTER
    || filters.status !== NO_FILTER;

  function openRow(appointmentId: string) {
    if (isWide) {
      setSelectedId(appointmentId);
    } else {
      focusReturnIdRef.current = appointmentId;
      sessionStorage.setItem(FOCUS_RETURN_KEY, appointmentId);
      navigate(`/cita/${appointmentId}`);
    }
  }

  function closeDetail() {
    const previous = selectedId;
    setSelectedId(null);
    if (previous) rowRefs.current.get(previous)?.focus();
  }

  const professionals = (context?.professionals ?? [])
    .map((row) => row.accepted ?? row.provisional)
    .filter((p): p is NonNullable<typeof p> => p !== null);
  const showProfessional = showsProfessionalColumn(viewerRole, professionals.length);
  const offers = (context?.offers ?? [])
    .map((row) => row.accepted)
    .filter((o): o is NonNullable<typeof o> => o !== null);
  const serviceNames = new Map<string, string>();
  for (const offer of offers) {
    if (!serviceNames.has(offer.serviceId)) serviceNames.set(offer.serviceId, offer.name);
  }
  const usablePatients = (context?.patients ?? [])
    .map((row) => visibleOf(row))
    .filter((p): p is NonNullable<typeof p> => p !== null);
  const canCreate = usablePatients.length > 0 && professionals.length > 0 && offers.length > 0;

  if (session === "loading") {
    return (
      <div className="state-block" role="status">
        <CircleNotch size={22} aria-hidden="true" />
        <p>Preparando la agenda…</p>
      </div>
    );
  }

  if (session === "no-session" || civil === null) {
    return (
      <div className="state-block" role="alert">
        <p>No hay una sesión lista en este equipo.</p>
        <p className="muted">
          La agenda solo muestra datos locales de una cuenta activa; no se presenta
          como vacía globalmente. Activa una cuenta y vuelve a intentarlo.
        </p>
        <button type="button" className="btn" onClick={() => void loadContext()}>
          Reintentar
        </button>
      </div>
    );
  }

  const visibleRows = (rows ?? [])
    .map((row) => ({ row, state: visibleOf(row) }))
    .filter((item): item is { row: EntityRow<AppointmentState>; state: AppointmentState } =>
      item.state !== null);

  return (
    <>
      <div className="tabs" role="tablist" aria-label="Vistas de agenda">
        {(["day", "week", "list"] as const).map((candidate) => (
          <button
            key={candidate}
            type="button"
            role="tab"
            aria-selected={view === candidate}
            className="tab"
            onClick={() => setView(candidate)}
          >
            {candidate === "day" ? "Día" : candidate === "week" ? "Semana" : "Lista"}
          </button>
        ))}
      </div>

      {view === "week" && (
        <WeekPage
          zone={zone}
          civil={civil}
          onCivilChange={(next) => { setCivil(next); setView("day"); }}
          patientName={(patientId) => patientNameOf(context, patientId)}
        />
      )}

      {view === "list" && (
        <AppointmentListPage
          zone={zone}
          context={context}
          professionals={professionals}
          serviceNames={serviceNames}
          showProfessional={showProfessional}
        />
      )}

      {view === "day" && (
      <>
      <div className="page-header">
        <div>
          <p className="muted" style={{ fontSize: 13, margin: "0 0 4px" }}>Agenda · zona {zone}</p>
          <h1 className="page-heading" style={{ textTransform: "capitalize" }}>{formatDateLong(zone, civil)}</h1>
        </div>
        {canCreate && (
          <button
            type="button"
            className="btn btn--primary"
            onClick={() => navigate("/cita/nueva")}
          >
            <Plus size={16} aria-hidden="true" />
            Nueva cita
          </button>
        )}
      </div>

      <div className="toolbar" style={{ justifyContent: "space-between" }}>
        <div className="toolbar" role="group" aria-label="Navegación de fecha" style={{ gap: 8, margin: 0 }}>
          <button
            type="button"
            className="btn"
            onClick={() => setCivil(shiftCivil(zone, civil, -1))}
            aria-label="Día anterior"
          >
            <CaretLeft size={16} aria-hidden="true" />
            Anterior
          </button>
          <button
            type="button"
            className="btn btn--subtle"
            onClick={() => setCivil(today)}
            disabled={civil === today}
          >
            Hoy
          </button>
          <button
            type="button"
            className="btn"
            onClick={() => setCivil(shiftCivil(zone, civil, 1))}
            aria-label="Día siguiente"
          >
            Siguiente
            <CaretRight size={16} aria-hidden="true" />
          </button>
          <label className="field-label">
            Fecha
            <input
              type="date"
              className="input-date"
              value={civil}
              onChange={(event) => setCivil(event.target.value)}
            />
          </label>
        </div>
        <div className="toolbar" role="group" aria-label="Filtros de citas" style={{ gap: 8, margin: 0 }}>
          <Funnel size={16} aria-hidden="true" style={{ color: "var(--color-text-tertiary)", alignSelf: "center" }} />
          {showProfessional && (
            <label className="field-label">
              Profesional
              <select
                className="select"
                value={filters.professionalId}
                onChange={(e) => setFilters({ ...filters, professionalId: e.target.value })}
              >
                <option value="">Todos</option>
                {professionals.map((p) => (
                  <option key={p.professionalId} value={p.professionalId}>{p.publicName}</option>
                ))}
              </select>
            </label>
          )}
          <label className="field-label">
            Servicio
            <select
              className="select"
              value={filters.serviceId}
              onChange={(e) => setFilters({ ...filters, serviceId: e.target.value })}
            >
              <option value="">Todos</option>
              {[...serviceNames].map(([id, name]) => (
                <option key={id} value={id}>{name}</option>
              ))}
            </select>
          </label>
          <label className="field-label">
            Estado
            <select
              className="select"
              value={filters.status}
              onChange={(e) => setFilters({ ...filters, status: e.target.value })}
            >
              <option value="">Todos</option>
              {Object.entries(statusLabel).map(([key, label]) => (
                <option key={key} value={key}>{label}</option>
              ))}
            </select>
          </label>
          {filterActive && (
            <button
              type="button"
              className="btn"
              style={{ alignSelf: "flex-end" }}
              onClick={() => setFilters({ professionalId: NO_FILTER, serviceId: NO_FILTER, status: NO_FILTER })}
            >
              Limpiar filtros
            </button>
          )}
        </div>
      </div>

      {notice && (
        <div className="notice" role="status">
          <p>{notice}</p>
        </div>
      )}

      <p role="status" className="muted" aria-live="polite">
        {`${total} cita(s) este día en los datos disponibles en este equipo.`}
      </p>

      {initialLoading && rows === null && !error && (
        <div className="state-block" role="status">
          <CircleNotch size={22} aria-hidden="true" />
          <p>Cargando citas…</p>
        </div>
      )}

      {error && (
        <div className="state-block" role="alert">
          <p>No se pudieron leer las citas locales.</p>
          <p className="muted">{error}</p>
          <button type="button" className="btn" onClick={() => void loadDay()}>
            Reintentar
          </button>
        </div>
      )}

      {!initialLoading && !error && rows !== null && visibleRows.length === 0 && (
        <div className="state-block">
          {filterActive ? (
            <>
              <p>Ninguna cita coincide con los filtros activos.</p>
              <button
                type="button"
                className="btn"
                onClick={() => setFilters({ professionalId: NO_FILTER, serviceId: NO_FILTER, status: NO_FILTER })}
              >
                Limpiar filtros
              </button>
            </>
          ) : (
            <p>Sin citas este día en los datos disponibles en este equipo.</p>
          )}
        </div>
      )}

      {visibleRows.length > 0 && (
        <div style={{ display: "flex", gap: 16, alignItems: "flex-start" }}>
          <div className="table-scroll" style={{ flex: 1, minWidth: 0 }}>
            <table className="data-table">
              <thead>
                <tr>
                  <th scope="col">Hora</th>
                  <th scope="col">Paciente</th>
                  <th scope="col">Servicio</th>
                  {showProfessional && <th scope="col">Profesional</th>}
                  <th scope="col">Estado</th>
                </tr>
              </thead>
              <tbody>
                {visibleRows.map(({ state }) => (
                  <tr key={state.appointmentId}>
                    <td className="num">
                      <button
                        type="button"
                        ref={(element) => {
                          if (element) rowRefs.current.set(state.appointmentId, element);
                          else rowRefs.current.delete(state.appointmentId);
                        }}
                        className="row-button"
                        aria-expanded={selectedId === state.appointmentId}
                        onClick={() => openRow(state.appointmentId)}
                      >
                        {formatTime(zone, state.startsAt)}–{formatTimeEnd(zone, state.endsAt)}
                      </button>
                    </td>
                    <td>
                      <button
                        type="button"
                        className="row-button"
                        onClick={() => openRow(state.appointmentId)}
                      >
                        {patientNameOf(context, state.patientId)}
                      </button>
                    </td>
                    <td>{state.serviceName}</td>
                    {showProfessional && <td>{state.professionalName}</td>}
                    <td>
                      <span className="badge">{statusLabel[state.status] ?? state.status}</span>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>

          {isWide && selectedId && (
            <div style={{ width: "min(420px, 100%)", flexShrink: 0 }}>
              <AppointmentDetail
                appointmentId={selectedId}
                context={context}
                onClose={closeDetail}
              />
            </div>
          )}
        </div>
      )}
      </>
      )}
    </>
  );
}
