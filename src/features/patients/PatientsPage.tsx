import { Link } from "react-router-dom";
import { useCallback, useEffect, useRef, useState, type FormEvent } from "react";
import { CaretLeft, CaretRight, CircleNotch, MagnifyingGlass } from "@phosphor-icons/react";
import {
  listPatients, visibleOf, type EntityRow, type ProfileState,
} from "../../shared/api/patients";
import { errorMessage } from "../appointments/presentation";

const PAGE_SIZE = 10;
const STORAGE_KEY = "pacientes:estado";
const FOCUS_RETURN_KEY = "pacientes:retornoFoco";
const MESSAGE_KEY = "pacientes:mensaje";

type Phase = "loading" | "ready" | "no-session" | "error";

interface DirectoryState {
  q: string;
  status: string;
  completionStatus: string;
  page: number;
}

const INITIAL: DirectoryState = { q: "", status: "", completionStatus: "", page: 1 };

function readStoredState(): DirectoryState {
  try {
    const raw = sessionStorage.getItem(STORAGE_KEY);
    if (raw) {
      const saved = JSON.parse(raw) as Partial<DirectoryState>;
      return {
        q: saved.q ?? "",
        status: saved.status ?? "",
        completionStatus: saved.completionStatus ?? "",
        page: saved.page ?? 1,
      };
    }
  } catch { /* estado previo ilegible: valores iniciales */ }
  return INITIAL;
}

/** S05 · Directorio de pacientes en modo lectura: búsqueda local, filtros
 * de estado y completitud, y paginación real. Sin acciones de escritura. */
export default function PatientsPage() {
  const [phase, setPhase] = useState<Phase>("loading");
  // Inicialización perezosa: evita que el efecto de persistencia (montaje en
  // modo estricto) sobrescriba el estado guardado antes de restaurarlo.
  const [applied, setApplied] = useState<DirectoryState>(readStoredState);
  const [draft, setDraft] = useState<DirectoryState>(readStoredState);
  const [rows, setRows] = useState<EntityRow<ProfileState>[] | null>(null);
  const [total, setTotal] = useState(0);
  const [error, setError] = useState<string | null>(null);
  const [notice] = useState<string | null>(() => {
    const message = sessionStorage.getItem(MESSAGE_KEY);
    if (message) sessionStorage.removeItem(MESSAGE_KEY);
    return message;
  });
  const generation = useRef(0);
  const rowsRef = useRef<EntityRow<ProfileState>[] | null>(null);
  const linkRefs = useRef(new Map<string, HTMLAnchorElement>());

  /* Restaurar búsqueda/filtros/página entre navegaciones (lazy init) y
     retorno de foco tras volver de la ficha. */
  useEffect(() => {
    sessionStorage.setItem(STORAGE_KEY, JSON.stringify(applied));
  }, [applied]);

  /* Retorno de foco tras volver de la ficha. */
  useEffect(() => {
    if (rows === null) return;
    const returnId = sessionStorage.getItem(FOCUS_RETURN_KEY);
    if (returnId) {
      sessionStorage.removeItem(FOCUS_RETURN_KEY);
      linkRefs.current.get(returnId)?.focus();
    }
  }, [rows]);

  const load = useCallback(async (state: DirectoryState) => {
    const request = ++generation.current;
    try {
      const page = await listPatients({
        page: state.page,
        shows: PAGE_SIZE,
        ...(state.q ? { q: state.q } : {}),
        ...(state.status ? { status: state.status } : {}),
        ...(state.completionStatus ? { completionStatus: state.completionStatus } : {}),
      });
      if (request !== generation.current) return;
      rowsRef.current = page.items;
      setRows(page.items);
      setTotal(page.total);
      setPhase("ready");
      setError(null);
    } catch (loadError: unknown) {
      if (request !== generation.current) return;
      const code = typeof loadError === "object" && loadError !== null && "code" in loadError
        ? String((loadError as { code: string }).code)
        : "";
      if (code === "AUTHENTICATION_REQUIRED" || code === "PERMISSION_DENIED") {
        setPhase("no-session");
      } else {
        setPhase(rowsRef.current === null ? "error" : "ready");
        setError(errorMessage(loadError));
      }
    }
  }, []);

  useEffect(() => { void load(applied); }, [load, applied]);

  function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setApplied({ ...draft, page: 1 });
  }

  function clearAll() {
    setDraft(INITIAL);
    setApplied(INITIAL);
  }

  const totalPages = Math.max(1, Math.ceil(total / PAGE_SIZE));
  const filterActive = applied.q !== "" || applied.status !== "" || applied.completionStatus !== "";
  const visibleRows = (rows ?? [])
    .map((row) => ({ row, state: visibleOf(row) }))
    .filter((item): item is { row: EntityRow<ProfileState>; state: ProfileState } =>
      item.state !== null);

  if (phase === "loading") {
    return (
      <div className="state-block" role="status">
        <CircleNotch size={22} aria-hidden="true" />
        <p>Cargando el directorio…</p>
      </div>
    );
  }

  if (phase === "no-session") {
    return (
      <div className="state-block" role="alert">
        <p>No hay una sesión lista en este equipo.</p>
        <p className="muted">Activa una cuenta para consultar pacientes y vuelve a intentarlo.</p>
        <button type="button" className="btn" onClick={() => void load(applied)}>Reintentar</button>
      </div>
    );
  }

  return (
    <>
      <div className="page-header">
        <div>
          <h1 className="page-heading">Pacientes</h1>
          <p className="muted">{total} ficha(s) en los datos disponibles en este equipo.</p>
        </div>
        <Link className="btn btn--primary" to="/pacientes/nueva">
          Registrar paciente
        </Link>
      </div>

      {notice && (
        <div className="notice" role="status"><p>{notice}</p></div>
      )}

      <form className="toolbar" role="search" aria-label="Búsqueda de pacientes" onSubmit={submit}>
        <MagnifyingGlass size={16} aria-hidden="true" style={{ color: "var(--color-text-tertiary)" }} />
        <label className="field-label">
          Buscar
          <input
            type="search"
            className="input-text"
            style={{ minWidth: 220 }}
            placeholder="Nombre del paciente"
            value={draft.q}
            onChange={(event) => setDraft({ ...draft, q: event.target.value })}
          />
        </label>
        <label className="field-label">
          Estado
          <select
            className="select"
            value={draft.status}
            onChange={(event) => setDraft({ ...draft, status: event.target.value })}
          >
            <option value="">Todos</option>
            <option value="active">Activos</option>
            <option value="inactive">Inactivos</option>
          </select>
        </label>
        <label className="field-label">
          Completitud
          <select
            className="select"
            value={draft.completionStatus}
            onChange={(event) => setDraft({ ...draft, completionStatus: event.target.value })}
          >
            <option value="">Todas</option>
            <option value="complete">Completas</option>
            <option value="draft">Borradores</option>
          </select>
        </label>
        <button type="submit" className="btn btn--primary">Buscar</button>
        {(filterActive || applied.page !== 1) && (
          <button type="button" className="btn" onClick={clearAll}>
            Limpiar
          </button>
        )}
      </form>

      <p role="status" className="muted" aria-live="polite">
        {error && `No se pudo actualizar el directorio. ${error}`}
        {!error && applied.q && `Resultados para “${applied.q}” en datos locales.`}
      </p>

      {phase === "error" && rows === null && (
        <div className="state-block" role="alert">
          <p>No se pudo leer el directorio local.</p>
          <p className="muted">{error}</p>
          <button type="button" className="btn" onClick={() => void load(applied)}>Reintentar</button>
        </div>
      )}

      {phase === "ready" && rows !== null && visibleRows.length === 0 && (
        <div className="state-block">
          {filterActive || applied.page !== 1 ? (
            <>
              <p>Ninguna ficha coincide con la búsqueda o los filtros activos.</p>
              <button type="button" className="btn" onClick={clearAll}>Limpiar búsqueda y filtros</button>
            </>
          ) : (
            <p>El directorio no tiene fichas en los datos disponibles en este equipo.</p>
          )}
        </div>
      )}

      {visibleRows.length > 0 && (
        <div className="table-scroll">
          <table className="data-table">
            <thead>
              <tr>
                <th scope="col">Paciente</th>
                <th scope="col">Teléfono</th>
                <th scope="col">Completitud</th>
                <th scope="col">Estado</th>
              </tr>
            </thead>
            <tbody>
              {visibleRows.map(({ state }) => (
                <tr key={state.patientId}>
                  <td>
                    <Link
                      to={`/paciente/${state.patientId}`}
                      ref={(element) => {
                        if (element) linkRefs.current.set(state.patientId, element);
                        else linkRefs.current.delete(state.patientId);
                      }}
                      onClick={() => sessionStorage.setItem(FOCUS_RETURN_KEY, state.patientId)}
                    >
                      {state.fullName?.trim() || "Paciente sin nombre registrado"}
                    </Link>
                  </td>
                  <td className="num">
                    {state.phone ? `${state.phoneCode ? `${state.phoneCode} ` : ""}${state.phone}` : "No registrado"}
                  </td>
                  <td>
                    <span className="badge">
                      {state.completionStatus === "complete" ? "Completa" : "Borrador"}
                    </span>
                  </td>
                  <td>
                    <span className="badge">
                      {state.status === "active" ? "Activo" : "Inactivo"}
                    </span>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {total > PAGE_SIZE && (
        <div className="toolbar" role="group" aria-label="Paginación">
          <button
            type="button"
            className="btn"
            disabled={applied.page <= 1}
            onClick={() => setApplied({ ...applied, page: applied.page - 1 })}
          >
            <CaretLeft size={16} aria-hidden="true" />
            Anterior
          </button>
          <span className="muted" aria-live="polite">
            Página {applied.page} de {totalPages}
          </span>
          <button
            type="button"
            className="btn"
            disabled={applied.page >= totalPages}
            onClick={() => setApplied({ ...applied, page: applied.page + 1 })}
          >
            Siguiente
            <CaretRight size={16} aria-hidden="true" />
          </button>
        </div>
      )}
    </>
  );
}
