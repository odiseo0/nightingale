import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useBlocker, useNavigate, useSearchParams } from "react-router-dom";
import { CircleNotch } from "@phosphor-icons/react";
import Modal from "../../components/primitives/Modal";
import {
  createAppointment, getBookingContext,
  type BookingContext, type ProfileState, type ServiceOfferView, visibleOf,
} from "../../shared/api/appointments";
import { getLocalAvailability, type AvailabilityEvaluation } from "../../shared/api/scheduling";
import {
  listPatientContacts, type EntityRow as ContactRow, type RelationshipView,
} from "../../shared/api/patients";
import {
  civilToday, dayRange, errorMessage, formatDateLong, mutationErrorMessage,
} from "./presentation";
import BookingFields, { type ResponsibleState } from "./BookingFields";
import type { AvailabilityState } from "./AvailabilityPicker";
import BookingSummary from "./BookingSummary";

const AGENDA_STATE_KEY = "agenda:estado";
const AGENDA_MESSAGE_KEY = "agenda:mensaje";
const DRAFT_KEY = "cita:borrador";
const EDITOR_MESSAGE_KEY = "cita:mensaje";

type Phase = "loading" | "no-session" | "empty" | "ready";

/** S04 · Nueva cita con paciente existente. Solo lectura de contexto,
 * disponibilidad calculada por Rust y guardado local provisional. */
export default function AppointmentEditorPage() {
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();
  const presetPatientId = searchParams.get("paciente");
  const [presetMissing, setPresetMissing] = useState(false);
  const [notice] = useState<string | null>(() => {
    const message = sessionStorage.getItem(EDITOR_MESSAGE_KEY);
    if (message) sessionStorage.removeItem(EDITOR_MESSAGE_KEY);
    return message;
  });
  const [phase, setPhase] = useState<Phase>("loading");
  const [context, setContext] = useState<BookingContext | null>(null);
  const [patientId, setPatientId] = useState<string | null>(null);
  const [offer, setOffer] = useState<ServiceOfferView | null>(null);
  const [civil, setCivil] = useState<string | null>(null);
  const [responsibleId, setResponsibleId] = useState<string | null>(null);
  const [contacts, setContacts] = useState<ContactRow<RelationshipView>[]>([]);
  const [contactsStatus, setContactsStatus] = useState<ResponsibleState["status"]>("idle");
  const [availability, setAvailability] = useState<AvailabilityState>({ status: "idle" });
  const [slot, setSlot] = useState<{ startsAt: string; endsAt: string } | null>(null);
  const [saving, setSaving] = useState(false);
  const [formError, setFormError] = useState<string | null>(null);
  const [saved, setSaved] = useState(false);
  const generation = useRef(0);
  const contactsGeneration = useRef(0);
  const savingRef = useRef(false);
  const savedRef = useRef(false);
  const dirtyRef = useRef(false);
  const registeringRef = useRef(false);

  const loadContext = useCallback(async () => {
    setPhase("loading");
    try {
      const data = await getBookingContext();
      setContext(data);
      // Forma funcional: en modo estricto el efecto se ejecuta dos veces y la
      // segunda resolución no debe sobrescribir una fecha ya restaurada.
      setCivil((current) => current ?? civilToday(data.timeZone));
      setPhase("ready");
    } catch {
      setPhase("no-session");
    }
  }, []);

  useEffect(() => { void loadContext(); }, [loadContext]);

  const patients = useMemo(
    () => (context?.patients ?? [])
      .map((row) => visibleOf(row))
      .filter((p): p is ProfileState => p !== null),
    [context],
  );
  const professionals = useMemo(
    () => (context?.professionals ?? [])
      .map((row) => row.accepted ?? row.provisional)
      .filter((p): p is NonNullable<typeof p> => p !== null),
    [context],
  );
  const offers = useMemo(
    () => (context?.offers ?? [])
      .map((row) => row.accepted)
      .filter((o): o is ServiceOfferView => o !== null),
    [context],
  );

  /* Selección automática cuando la elección es única (texto contextual en campos). */
  useEffect(() => {
    if (phase !== "ready") return;
    if (presetPatientId) {
      const available = patients.some((p) => p.patientId === presetPatientId);
      if (available) {
        setPatientId(presetPatientId);
      } else {
        // No se sustituye silenciosamente: se informa y se deja elegir.
        setPresetMissing(true);
        setPatientId(null);
      }
      return;
    }
    if (patients.length === 1) setPatientId((current) => current ?? patients[0].patientId);
    if (offers.length === 1) setOffer((current) => current ?? offers[0]);
    if (professionals.length === 0 || offers.length === 0 || patients.length === 0) {
      setPhase("empty");
    }
  }, [phase, patients, offers, professionals, presetPatientId]);

  const professionalName = useCallback((professionalId: string) => {
    const professional = professionals.find((p) => p.professionalId === professionalId);
    return professional?.publicName ?? "Profesional no registrado";
  }, [professionals]);

  const zone = context?.timeZone ?? "UTC";
  const selectedPatient = patients.find((p) => p.patientId === patientId) ?? null;
  const responsibleRequired = selectedPatient?.identificationBasis === "minor_without_document";

  /* Vínculos responsables del paciente elegido (solo cuando aplica). */
  const loadContacts = useCallback(async () => {
    if (!patientId || !responsibleRequired) return;
    const request = ++contactsGeneration.current;
    setContactsStatus("loading");
    try {
      const rows = await listPatientContacts({ patientId, status: "active" });
      if (request !== contactsGeneration.current) return;
      setContacts(rows);
      setContactsStatus("ready");
    } catch {
      if (request !== contactsGeneration.current) return;
      setContacts([]);
      setContactsStatus("error");
    }
  }, [patientId, responsibleRequired]);

  useEffect(() => {
    contactsGeneration.current += 1;
    setResponsibleId(null);
    setContacts([]);
    setContactsStatus("idle");
    void loadContacts();
  }, [loadContacts]);

  /* Disponibilidad local calculada por Rust. */
  const loadAvailability = useCallback(async () => {
    if (!offer || !civil) return;
    const request = ++generation.current;
    setAvailability({ status: "loading" });
    try {
      const evaluation: AvailabilityEvaluation = await getLocalAvailability({
        professionalId: offer.professionalId,
        serviceId: offer.serviceId,
        ...dayRange(zone, civil),
        durationMinutes: offer.effectiveDurationMinutes,
      });
      if (request !== generation.current) return;
      setAvailability({ status: "ready", evaluation });
    } catch (loadError: unknown) {
      if (request !== generation.current) return;
      setAvailability({ status: "error", message: errorMessage(loadError) });
    }
  }, [offer, civil, zone]);

  useEffect(() => {
    if (skipSlotClear.current) {
      skipSlotClear.current = false;
    } else {
      setSlot(null);
    }
    void loadAvailability();
  }, [loadAvailability]);

  /* Restaurar el borrador conservado al salir a registrar un paciente. */
  const draftRestored = useRef(false);
  const skipSlotClear = useRef(false);
  useEffect(() => {
    if (phase !== "ready" || draftRestored.current) return;
    draftRestored.current = true;
    try {
      const raw = sessionStorage.getItem(DRAFT_KEY);
      if (!raw) return;
      sessionStorage.removeItem(DRAFT_KEY);
      const saved = JSON.parse(raw) as {
        offerKey?: string | null;
        civil?: string | null;
        slot?: { startsAt: string; endsAt: string } | null;
        responsibleId?: string | null;
      };
      /* La restauración no debe disparar la invalidación del slot propia del
         cambio de oferta/fecha. */
      if (saved.offerKey || saved.civil) skipSlotClear.current = true;
      if (saved.offerKey) {
        const restored = offers.find((o) => `${o.professionalId}:${o.serviceId}` === saved.offerKey);
        if (restored) setOffer(restored);
      }
      if (saved.civil) setCivil(saved.civil);
      if (saved.slot) setSlot(saved.slot);
      if (saved.responsibleId) setResponsibleId(saved.responsibleId);
    } catch { /* borrador ilegible: se ignora */ }
  }, [phase, offers]);

  function goRegisterPatient() {
    sessionStorage.setItem(DRAFT_KEY, JSON.stringify({
      offerKey: offer ? `${offer.professionalId}:${offer.serviceId}` : null,
      civil,
      slot,
      responsibleId,
    }));
    registeringRef.current = true;
    navigate("/pacientes/nueva?desde=cita");
  }

  const responsibleState: ResponsibleState = {
    required: responsibleRequired,
    status: contactsStatus,
    links: contacts,
  };

  const responsibleLinks = contacts
    .map((row) => row.provisional?.state ?? row.accepted?.state ?? null)
    .filter((v): v is RelationshipView =>
      v !== null && v.relationship.isResponsibleContact && v.relationship.status === "active");
  const selectedResponsible = responsibleLinks.find(
    (v) => v.relationship.contactPersonId === responsibleId,
  ) ?? null;

  /* El aviso de salida solo se activa por cambios del usuario: la precarga,
   * la selección automática única y la restauración del borrador no cuentan
   * como edición. El ref nace falso en cada montaje y solo los manejadores
   * de acción del usuario lo activan; resetearlo durante el render borraría
   * la marca en el primer re-render posterior a la interacción. */
  // Forma función: se evalúa en el momento de la navegación y lee los refs,
  // así el retorno tras guardar (savedRef=true) nunca se interpone y las
  // salidas con cambios sin guardar sí bloquean.
  // https://reactrouter.com/api/hooks/useBlocker
  const blocker = useBlocker(() => dirtyRef.current && !savedRef.current && !registeringRef.current);

  useEffect(() => {
    if (!dirtyRef.current || saved) return;
    const onBeforeUnload = (event: BeforeUnloadEvent) => {
      event.preventDefault();
    };
    window.addEventListener("beforeunload", onBeforeUnload);
    return () => window.removeEventListener("beforeunload", onBeforeUnload);
  }, [saved]);

  const canSave = !!patientId && !!offer && !!slot
    && (!responsibleRequired || (contactsStatus === "ready" && !!responsibleId))
    && !saving;

  async function save() {
    // Ref de intención única: evita una segunda operación por doble clic
    // aunque el estado aún no se haya re-renderizado.
    if (savingRef.current) return;
    if (!canSave || !patientId || !offer || !slot) return;
    savingRef.current = true;
    setSaving(true);
    setFormError(null);
    try {
      // Los intervalos pueden ser ventanas mayores que la duración efectiva
      // (p. ej. una regla de día completo); la cita usa la duración efectiva
      // de la oferta anclada al inicio del intervalo elegido.
      const startsAt = slot.startsAt;
      const endsAt = new Date(Math.min(
        Date.parse(slot.endsAt),
        Date.parse(startsAt) + offer.effectiveDurationMinutes * 60_000,
      )).toISOString();
      const result = await createAppointment({
        patientId,
        professionalId: offer.professionalId,
        serviceId: offer.serviceId,
        ...(responsibleRequired && responsibleId ? { responsibleContactId: responsibleId } : {}),
        startsAt,
        endsAt,
      });
      savedRef.current = true;
      setSaved(true);
      const newId = typeof result.entityKey === "string" ? result.entityKey : null;
      try {
        const raw = sessionStorage.getItem(AGENDA_STATE_KEY);
        const state = raw ? JSON.parse(raw) as Record<string, unknown> : {};
        sessionStorage.setItem(AGENDA_STATE_KEY, JSON.stringify({
          ...state,
          civil,
          ...(newId ? { selectedId: newId } : {}),
        }));
        sessionStorage.setItem(AGENDA_MESSAGE_KEY, "Cita guardada.");
      } catch { /* el mensaje persistente es opcional; la fecha y filtros ya se conservan */ }
      navigate("/agenda");
    } catch (saveError: unknown) {
      setFormError(mutationErrorMessage(saveError));
    } finally {
      setSaving(false);
    }
  }

  if (phase === "loading" || civil === null) {
    return (
      <div className="state-block" role="status">
        <CircleNotch size={22} aria-hidden="true" />
        <p>Preparando el editor de citas…</p>
      </div>
    );
  }

  if (phase === "no-session") {
    return (
      <div className="state-block" role="alert">
        <p>No hay una sesión lista en este equipo.</p>
        <p className="muted">Activa una cuenta para crear citas y vuelve a intentarlo.</p>
        <button type="button" className="btn" onClick={() => void loadContext()}>Reintentar</button>
      </div>
    );
  }

  if (phase === "empty") {
    return (
      <div className="state-block">
        <p>Esta cuenta aún no tiene pacientes, profesional u ofertas utilizables.</p>
        <p className="muted">Completa los datos de la cuenta y sincroniza antes de crear citas.</p>
        <button type="button" className="btn" onClick={() => void loadContext()}>Reintentar</button>
      </div>
    );
  }

  return (
    <>
      <div className="page-header">
        <div>
          <h1 className="page-heading">Nueva cita</h1>
          <p className="muted">{formatDateLong(zone, civil)} · zona {zone}</p>
        </div>
        <button type="button" className="btn" onClick={() => navigate("/agenda")}>
          Volver a la agenda
        </button>
      </div>

      <div className="editor-layout">
        <div style={{ display: "grid", gap: 20, minWidth: 0, flex: 1 }}>
          {notice && (
            <div className="notice" role="status"><p>{notice}</p></div>
          )}
          {presetMissing && (
            <div className="notice" role="alert">
              <p>
                El paciente indicado no está disponible en el contexto autorizado;
                selecciona un paciente para continuar.
              </p>
            </div>
          )}
          <BookingFields
            zone={zone}
            patients={patients}
            offers={offers}
            professionalName={professionalName}
            patientId={patientId}
            onPatientChange={(value) => { dirtyRef.current = true; setPatientId(value); }}
            offer={offer}
            onOfferChange={(value) => { dirtyRef.current = true; setOffer(value); }}
            civil={civil}
            onCivilChange={(value) => { if (value) { dirtyRef.current = true; setCivil(value); } }}
            responsibleId={responsibleId}
            onResponsibleChange={(value) => { dirtyRef.current = true; setResponsibleId(value); }}
            responsible={responsibleState}
            onRetryContacts={() => void loadContacts()}
            availability={availability}
            slot={slot}
            onSlotChange={(value) => { dirtyRef.current = true; setSlot(value); }}
            onRetryAvailability={() => void loadAvailability()}
            onRegisterPatient={goRegisterPatient}
          />
        </div>

        <div className="editor-summary">
          <BookingSummary
            zone={zone}
            civil={civil}
            patient={selectedPatient}
            offer={offer}
            professionalName={professionalName}
            slot={slot}
            responsible={selectedResponsible}
            canSave={canSave}
            saving={saving}
            error={formError}
            onSave={() => void save()}
          />
        </div>
      </div>

      <Modal open={blocker.state === "blocked"} onClose={() => blocker.reset?.()} alert labelledBy="descartar-titulo">
        <h2 id="descartar-titulo" style={{ fontSize: 20 }}>Cambios sin guardar</h2>
        <p className="muted">
          Si sales ahora, la cita no se guardará y los cambios de este formulario se perderán.
        </p>
        <div className="toolbar">
          <button type="button" className="btn btn--primary" onClick={() => blocker.reset?.()}>
            Seguir editando
          </button>
          <button type="button" className="btn" onClick={() => blocker.proceed?.()}>
            Descartar y salir
          </button>
        </div>
      </Modal>
    </>
  );
}
