import { useEffect, useState } from "react";
import { Link, useNavigate, useParams } from "react-router-dom";
import { ArrowLeft, CircleNotch, PencilSimple, Plus } from "@phosphor-icons/react";
import { getAccountView } from "../../shared/api/account";
import {
  listAppointments, type AppointmentState, type EntityRow as AppointmentRow, visibleOf as visibleAppointment,
} from "../../shared/api/appointments";
import {
  getPatient, listPatientContacts, listPatientIdentifiers,
  visibleOf, type EntityRow, type IdentifierState, type ProfileState, type RelationshipView,
} from "../../shared/api/patients";
import {
  civilToday, dayRange, errorMessage, formatDateFull, formatTime, formatTimeEnd,
  statusLabel,
} from "../appointments/presentation";
import { relationshipKindLabel } from "./presentation";

function formatValidity(zone: string, from: string, until: string | null): string {
  const dateOf = (iso: string) => new Intl.DateTimeFormat("es", {
    timeZone: zone, day: "numeric", month: "short", year: "numeric",
  }).format(new Date(iso));
  return until ? `${dateOf(from)} – ${dateOf(until)}` : `desde ${dateOf(from)}`;
}

const basisLabel: Record<string, string> = {
  own_document: "Documento propio",
  minor_without_document: "Menor sin documento",
};

const verificationLabel: Record<string, string> = {
  declared: "Declarado",
  verified: "Verificado",
  invalidated: "Invalidado",
};

type LoadState = "loading" | "ready" | "error";

interface AppointmentListProps {
  zone: string;
  title: string;
  items: AppointmentRow<AppointmentState>[];
  emptyText: string;
}

function AppointmentList({ zone, title, items, emptyText }: AppointmentListProps) {
  const visible = items
    .map((row) => ({ row, state: visibleAppointment(row) }))
    .filter((item): item is { row: AppointmentRow<AppointmentState>; state: AppointmentState } =>
      item.state !== null);
  if (!visible.length) return <p className="muted">{emptyText}</p>;
  return (
    <div>
      <p className="muted" style={{ margin: "0 0 8px" }}>{title}</p>
      <div className="table-scroll">
        <table className="data-table">
          <thead>
            <tr>
              <th scope="col">Fecha</th>
              <th scope="col">Intervalo</th>
              <th scope="col">Servicio</th>
              <th scope="col">Estado</th>
            </tr>
          </thead>
          <tbody>
            {visible.map(({ state }) => (
              <tr key={state.appointmentId}>
                <td>
                  <Link to={`/cita/${state.appointmentId}`}>{formatDateFull(zone, state.startsAt)}</Link>
                </td>
                <td className="num">
                  {formatTime(zone, state.startsAt)}–{formatTimeEnd(zone, state.endsAt)}
                </td>
                <td>{state.serviceName}</td>
                <td><span className="badge">{statusLabel[state.status] ?? state.status}</span></td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}

/** S06 · Ficha administrativa del paciente en modo lectura: perfil,
 * documentos, responsables y citas. Sin datos clínicos ni escrituras. */
export default function PatientRecordPage() {
  const { patientId } = useParams<{ patientId: string }>();
  const navigate = useNavigate();
  const [zone, setZone] = useState("UTC");
  const [profile, setProfile] = useState<EntityRow<ProfileState> | null>(null);
  const [profileState, setProfileState] = useState<LoadState>("loading");
  const [profileError, setProfileError] = useState<string | null>(null);
  const [identifiers, setIdentifiers] = useState<EntityRow<IdentifierState>[] | null>(null);
  const [identifiersState, setIdentifiersState] = useState<LoadState>("loading");
  const [contacts, setContacts] = useState<EntityRow<RelationshipView>[] | null>(null);
  const [contactsState, setContactsState] = useState<LoadState>("loading");
  const [upcoming, setUpcoming] = useState<AppointmentRow<AppointmentState>[] | null>(null);
  const [history, setHistory] = useState<AppointmentRow<AppointmentState>[] | null>(null);
  const [appointmentsState, setAppointmentsState] = useState<LoadState>("loading");
  const [historyPage, setHistoryPage] = useState(1);
  const [historyTotal, setHistoryTotal] = useState(0);
  const HISTORY_SIZE = 5;

  useEffect(() => {
    let cancelled = false;
    getAccountView()
      .then((screen) => { if (!cancelled && screen.account) setZone(screen.account.timeZone); })
      .catch(() => { /* zona por defecto; los datos llevan su propia zona */ });
    return () => { cancelled = true; };
  }, []);

  useEffect(() => {
    if (!patientId) return;
    let cancelled = false;
    setProfileState("loading");
    getPatient(patientId)
      .then((row) => { if (!cancelled) { setProfile(row); setProfileState("ready"); } })
      .catch((loadError: unknown) => {
        if (!cancelled) { setProfileState("error"); setProfileError(errorMessage(loadError)); }
      });
    return () => { cancelled = true; };
  }, [patientId]);

  useEffect(() => {
    if (!patientId) return;
    let cancelled = false;
    setIdentifiersState("loading");
    setContactsState("loading");
    listPatientIdentifiers(patientId)
      .then((rows) => { if (!cancelled) { setIdentifiers(rows); setIdentifiersState("ready"); } })
      .catch(() => { if (!cancelled) setIdentifiersState("error"); });
    listPatientContacts({ patientId })
      .then((rows) => { if (!cancelled) { setContacts(rows); setContactsState("ready"); } })
      .catch(() => { if (!cancelled) setContactsState("error"); });
    return () => { cancelled = true; };
  }, [patientId]);

  useEffect(() => {
    if (!patientId) return;
    let cancelled = false;
    setAppointmentsState("loading");
    const today = civilToday(zone);
    const futureRange = dayRange(zone, today);
    const futureUntil = new Date(Date.parse(futureRange.until) + 60 * 86_400_000).toISOString();
    // El contrato local limita los rangos a 366 días.
    const pastFrom = new Date(Date.parse(futureRange.from) - 365 * 86_400_000).toISOString();
    Promise.all([
      listAppointments({ patientId, from: futureRange.from, until: futureUntil, shows: 20 }),
      listAppointments({
        patientId,
        from: pastFrom,
        until: futureRange.from,
        page: historyPage,
        shows: HISTORY_SIZE,
      }),
    ])
      .then(([nextPage, historyPageResult]) => {
        if (cancelled) return;
        setUpcoming(nextPage.items);
        setHistory(historyPageResult.items);
        setHistoryTotal(historyPageResult.total);
        setAppointmentsState("ready");
      })
      .catch(() => {
        if (!cancelled) setAppointmentsState("error");
      });
    return () => { cancelled = true; };
  }, [patientId, zone, historyPage]);

  if (!patientId) {
    return (
      <div className="state-block" role="alert">
        <p>El identificador del paciente no es válido.</p>
        <Link className="btn" to="/pacientes">Volver al directorio</Link>
      </div>
    );
  }

  const state = profile ? visibleOf(profile) : null;
  const totalHistoryPages = Math.max(1, Math.ceil(historyTotal / HISTORY_SIZE));

  return (
    <>
      <div className="page-header">
        <div>
          <h1 className="page-heading">{state?.fullName?.trim() || "Paciente sin nombre registrado"}</h1>
          <p className="muted" style={{ display: "flex", gap: 8, flexWrap: "wrap", alignItems: "center" }}>
            {state && (
              <>
                <span className="badge">
                  {state.completionStatus === "complete" ? "Completa" : "Borrador"}
                </span>
                <span className="badge">
                  {state.status === "active" ? "Activo" : "Inactivo"}
                </span>
              </>
            )}
          </p>
        </div>
        <div className="toolbar" role="group" aria-label="Acciones de la ficha">
          <Link className="btn" to={`/paciente/${patientId}/editar`}>
            <PencilSimple size={16} aria-hidden="true" />
            Editar
          </Link>
          <button
            type="button"
            className="btn btn--primary"
            onClick={() => navigate(`/cita/nueva?paciente=${patientId}`)}
          >
            <Plus size={16} aria-hidden="true" />
            Nueva cita
          </button>
          <Link className="btn" to="/pacientes">
            <ArrowLeft size={16} aria-hidden="true" />
            Volver al directorio
          </Link>
        </div>
      </div>

      {profileState === "loading" && (
        <p className="muted" role="status"><CircleNotch size={16} aria-hidden="true" /> Cargando la ficha…</p>
      )}
      {profileState === "error" && (
        <div className="state-block" role="alert">
          <p>No se pudo leer la ficha del paciente.</p>
          <p className="muted">{profileError}</p>
          <Link className="btn" to="/pacientes">Volver al directorio</Link>
        </div>
      )}

      {state && (
        <div style={{ display: "grid", gap: 20 }}>
          <section className="card" style={{ padding: 16 }} aria-labelledby="perfil-titulo">
            <h2 id="perfil-titulo" style={{ fontSize: 18, marginBottom: 12 }}>Perfil administrativo</h2>
            <dl className="fact-grid">
              <div>
                <dt>Fecha de nacimiento</dt>
                <dd className="num">{state.birthDate ?? "No registrado"}</dd>
              </div>
              <div>
                <dt>Teléfono</dt>
                <dd className="num">
                  {state.phone ? `${state.phoneCode ? `${state.phoneCode} ` : ""}${state.phone}` : "No registrado"}
                </dd>
              </div>
              <div>
                <dt>Base de identificación</dt>
                <dd>{state.identificationBasis ? (basisLabel[state.identificationBasis] ?? state.identificationBasis) : "No registrado"}</dd>
              </div>
            </dl>
          </section>

          <section className="card" style={{ padding: 16 }} aria-labelledby="documentos-titulo">
            <h2 id="documentos-titulo" style={{ fontSize: 18, marginBottom: 12 }}>Documentos</h2>
            {identifiersState === "loading" && <p className="muted" role="status">Cargando documentos…</p>}
            {identifiersState === "error" && (
              <p className="notice" role="alert">No se pudieron leer los documentos de este equipo.</p>
            )}
            {identifiersState === "ready" && (identifiers ?? []).length === 0 && (
              <p className="muted">Sin documentos registrados en los datos disponibles.</p>
            )}
            {identifiersState === "ready" && (identifiers ?? []).length > 0 && (
              <ul style={{ margin: 0, paddingLeft: 20 }}>
                {(identifiers ?? []).map((row) => {
                  const doc = visibleOf(row);
                  if (!doc) return null;
                  return (
                    <li key={doc.id} style={{ marginBottom: 6 }}>
                      <span className="num">{doc.documentNumber}</span> · {doc.documentType} ·{" "}
                      {doc.issuingCountry} ·{" "}
                      {verificationLabel[doc.verificationStatus] ?? doc.verificationStatus}
                    </li>
                  );
                })}
              </ul>
            )}
          </section>

          <section className="card" style={{ padding: 16 }} aria-labelledby="responsables-titulo">
            <h2 id="responsables-titulo" style={{ fontSize: 18, marginBottom: 12 }}>Contactos y responsables</h2>
            {contactsState === "loading" && <p className="muted" role="status">Cargando contactos…</p>}
            {contactsState === "error" && (
              <p className="notice" role="alert">No se pudieron leer los vínculos de este equipo.</p>
            )}
            {contactsState === "ready" && (contacts ?? []).length === 0 && (
              <p className="muted">Sin contactos registrados en los datos disponibles.</p>
            )}
            {contactsState === "ready" && (contacts ?? []).length > 0 && (
              <ul style={{ margin: 0, paddingLeft: 20 }}>
                {(contacts ?? []).map((row) => {
                  const view = visibleOf(row);
                  if (!view) return null;
                  const phone = view.contactPhone
                    ? `${view.contactPhoneCode ? `${view.contactPhoneCode} ` : ""}${view.contactPhone}`
                    : "teléfono no registrado";
                  return (
                    <li key={view.relationship.contactPersonId} style={{ marginBottom: 6 }}>
                      {view.contactFullName} ·{" "}
                      {relationshipKindLabel[view.relationship.relationshipKind] ?? view.relationship.relationshipKind} ·{" "}
                      {phone} · {formatValidity(zone, view.relationship.validFrom, view.relationship.validUntil)}
                      {view.relationship.isResponsibleContact && (
                        <span className="badge" style={{ marginLeft: 8 }}>Responsable</span>
                      )}
                      {view.relationship.status !== "active" && (
                        <span className="badge" style={{ marginLeft: 8 }}>Inactivo</span>
                      )}
                    </li>
                  );
                })}
              </ul>
            )}
          </section>

          <section aria-labelledby="citas-titulo">
            <h2 id="citas-titulo" style={{ fontSize: 18, marginBottom: 12 }}>Citas del paciente</h2>
            {appointmentsState === "loading" && (
              <p className="muted" role="status"><CircleNotch size={16} aria-hidden="true" /> Cargando citas…</p>
            )}
            {appointmentsState === "error" && (
              <div className="notice" role="alert">
                <p>No se pudieron leer las citas locales de este paciente.</p>
              </div>
            )}
            {appointmentsState === "ready" && (
              <div style={{ display: "grid", gap: 16 }}>
                <AppointmentList
                  zone={zone}
                  title="Próximas citas"
                  items={upcoming ?? []}
                  emptyText="Sin próximas citas en los datos disponibles."
                />
                <div>
                  <AppointmentList
                    zone={zone}
                    title="Historial"
                    items={history ?? []}
                    emptyText="Sin citas anteriores en los datos disponibles."
                  />
                  {historyTotal > HISTORY_SIZE && (
                    <div className="toolbar" role="group" aria-label="Paginación del historial" style={{ marginTop: 8 }}>
                      <button
                        type="button"
                        className="btn"
                        disabled={historyPage <= 1}
                        onClick={() => setHistoryPage(historyPage - 1)}
                      >
                        Anterior
                      </button>
                      <span className="muted">Página {historyPage} de {totalHistoryPages}</span>
                      <button
                        type="button"
                        className="btn"
                        disabled={historyPage >= totalHistoryPages}
                        onClick={() => setHistoryPage(historyPage + 1)}
                      >
                        Siguiente
                      </button>
                    </div>
                  )}
                </div>
              </div>
            )}
          </section>
        </div>
      )}
    </>
  );
}
