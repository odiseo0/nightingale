import { useEffect, useRef, useState, type FormEvent } from "react";
import { useBlocker, useNavigate, useParams } from "react-router-dom";
import { CircleNotch, PencilSimple, Plus, Trash } from "@phosphor-icons/react";
import Modal from "../../components/primitives/Modal";
import { getAccountView } from "../../shared/api/account";
import {
  addPatientIdentifier, baseOfPatient, createContactPerson, createPatientContactRelationship,
  getPatient, listContacts, listPatientIdentifiers, registerPatient, updatePatientProfile,
  setPatientProfileStatus,
  visibleOf, type ContactState, type EntityRow, type ProfileState,
} from "../../shared/api/patients";
import {
  civilMidnightUtc, civilToday, mutationErrorMessage,
} from "../appointments/presentation";
import { documentTypeLabel, relationshipKindLabel, uuidV7 } from "./presentation";

const MESSAGE_KEY = "pacientes:mensaje";

interface IdentifierDraft {
  id: string;
  issuingCountry: string;
  documentType: string;
  documentNumber: string;
}

type Phase = "loading" | "ready" | "no-session";

/** Registro administrativo de pacientes (perfil propio o menor sin documento).
 * Rust valida completitud y reglas; React solo compone el formulario. */
export default function PatientEditorPage() {
  const navigate = useNavigate();
  const { patientId: editPatientId } = useParams<{ patientId: string }>();
  const [routeSearch] = useState(() => new URLSearchParams(window.location.hash.split("?")[1] ?? ""));
  const fromAppointment = routeSearch.get("desde") === "cita";
  const editMode = Boolean(editPatientId);
  const [phase, setPhase] = useState<Phase>("loading");
  const [zone, setZone] = useState("UTC");
  const [profileRow, setProfileRow] = useState<EntityRow<ProfileState> | null>(null);
  const [loadedIdentifiers, setLoadedIdentifiers] = useState<string[]>([]);
  const [statusConfirm, setStatusConfirm] = useState(false);
  const [statusMessage, setStatusMessage] = useState<string | null>(null);
  const [basis, setBasis] = useState<"own_document" | "minor_without_document">("own_document");
  const [fullName, setFullName] = useState("");
  const [birthDate, setBirthDate] = useState("");
  const [phoneCode, setPhoneCode] = useState("+58");
  const [phone, setPhone] = useState("");
  const [identifiers, setIdentifiers] = useState<IdentifierDraft[]>([
    { id: uuidV7(), issuingCountry: "VE", documentType: "national_id", documentNumber: "" },
  ]);
  /* Menor: responsable */
  const [contactMode, setContactMode] = useState<"new" | "existing">("new");
  const [contactQuery, setContactQuery] = useState("");
  const [contactResults, setContactResults] = useState<EntityRow<ContactState>[]>([]);
  const [contactResultsState, setContactResultsState] = useState<"idle" | "loading" | "ready" | "error">("idle");
  const [selectedContactId, setSelectedContactId] = useState<string | null>(null);
  const [contactName, setContactName] = useState("");
  const [contactPhoneCode, setContactPhoneCode] = useState("+58");
  const [contactPhone, setContactPhone] = useState("");
  const [relationshipKind, setRelationshipKind] = useState("mother");
  const [validFrom, setValidFrom] = useState(() => civilToday("UTC"));
  const [validUntil, setValidUntil] = useState("");
  const [saving, setSaving] = useState(false);
  const [formError, setFormError] = useState<string | null>(null);
  const [progress, setProgress] = useState<string | null>(null);
  /* Paciente ya creado si un envío multi-paso falló a mitad: no duplicar. */
  const [createdPatientId, setCreatedPatientId] = useState<string | null>(null);
  /* Contacto y vínculo ya creados en un intento previo: reintentar no los
   * duplica; se reutilizan para completar solo los pasos pendientes. */
  const [createdContactId, setCreatedContactId] = useState<string | null>(null);
  const [relationshipCreated, setRelationshipCreated] = useState(false);
  const savingRef = useRef(false);
  const savedRef = useRef(false);
  const dirtyRef = useRef(false);
  const contactGeneration = useRef(0);

  useEffect(() => {
    getAccountView()
      .then((screen) => {
        if (screen.account) setZone(screen.account.timeZone);
        return listContacts({ shows: 1 });
      })
      .then(() => setPhase("ready"))
      .catch((error: unknown) => {
        const code = typeof error === "object" && error !== null && "code" in error
          ? String((error as { code: string }).code)
          : "";
        setPhase(code === "AUTHENTICATION_REQUIRED" || code === "PERMISSION_DENIED" ? "no-session" : "ready");
      });
  }, []);

  /* Modo edición: precargar perfil e identificadores aceptados. */
  useEffect(() => {
    if (!editPatientId) return;
    let cancelled = false;
    Promise.all([getPatient(editPatientId), listPatientIdentifiers(editPatientId)])
      .then(([row, identifiers]) => {
        if (cancelled) return;
        setProfileRow(row);
        const state = visibleOf(row);
        if (state) {
          setFullName(state.fullName ?? "");
          setBirthDate(state.birthDate ?? "");
          setPhoneCode(state.phoneCode ?? "");
          setPhone(state.phone ?? "");
          if (state.identificationBasis === "minor_without_document") {
            setBasis("minor_without_document");
          }
        }
        setLoadedIdentifiers(
          identifiers.map((item) => visibleOf(item)).filter((d): d is NonNullable<typeof d> => d !== null)
            .map((d) => `${d.issuingCountry}:${d.documentType}:${d.documentNumber}`),
        );
        const existing = identifiers.map((item) => visibleOf(item))
          .filter((d): d is NonNullable<typeof d> => d !== null)
          .map((d) => ({
            id: uuidV7(),
            issuingCountry: d.issuingCountry,
            documentType: d.documentType,
            documentNumber: d.documentNumber,
          }));
        const nextIdentifiers = existing.length > 0
          ? [...existing, { id: uuidV7(), issuingCountry: "VE", documentType: "national_id", documentNumber: "" }]
          : existing;
        setIdentifiers(nextIdentifiers);
        /* La precarga fija la línea base: salir sin cambios no avisa. */
        if (state) {
          baselineRef.current = JSON.stringify([
            state.fullName ?? "", state.birthDate ?? "", state.phoneCode ?? "", state.phone ?? "",
            nextIdentifiers.map((i) => [i.issuingCountry, i.documentType, i.documentNumber]),
            "", null,
            state.identificationBasis === "minor_without_document" ? "minor_without_document" : "own_document",
          ]);
        }
      })
      .catch(() => { if (!cancelled) setFormError("No se pudo cargar la ficha para editar."); });
    return () => { cancelled = true; };
  }, [editPatientId]);

  /* Búsqueda de contactos existentes (modo "existing"). */
  useEffect(() => {
    if (contactMode !== "existing") return;
    const request = ++contactGeneration.current;
    if (!contactQuery.trim()) {
      setContactResults([]);
      setContactResultsState("idle");
      return;
    }
    setContactResultsState("loading");
    const timer = window.setTimeout(() => {
      listContacts({ q: contactQuery.trim(), status: "active", shows: 10 })
        .then((page) => {
          if (request !== contactGeneration.current) return;
          setContactResults(page.items);
          setContactResultsState("ready");
        })
        .catch(() => {
          if (request !== contactGeneration.current) return;
          setContactResults([]);
          setContactResultsState("error");
        });
    }, 300);
    return () => { contactGeneration.current += 1; window.clearTimeout(timer); };
  }, [contactMode, contactQuery]);

  /* La línea base separa precarga de edición: abrir una ficha cargada no
   * cuenta como cambio; modificar cualquier valor respecto a la línea base
   * sí activa el aviso de salida. */
  const baselineRef = useRef(JSON.stringify([
    "", "", "+58", "", [["VE", "national_id", ""]], "", null, "own_document",
  ]));
  const currentSnapshot = JSON.stringify([
    fullName, birthDate, phoneCode, phone,
    identifiers.map((i) => [i.issuingCountry, i.documentType, i.documentNumber]),
    contactName, selectedContactId, basis,
  ]);
  const dirty = currentSnapshot !== baselineRef.current;
  dirtyRef.current = dirty;
  const blocker = useBlocker(() => dirtyRef.current && !savedRef.current);

  useEffect(() => {
    if (!dirty || savedRef.current) return;
    const onBeforeUnload = (event: BeforeUnloadEvent) => { event.preventDefault(); };
    window.addEventListener("beforeunload", onBeforeUnload);
    return () => window.removeEventListener("beforeunload", onBeforeUnload);
  }, [dirty]);

  async function toggleStatus() {
    if (!editPatientId || !profileRow) return;
    savingRef.current = true;
    setSaving(true);
    setFormError(null);
    try {
      const state = visibleOf(profileRow);
      const next = state?.status === "active" ? "inactive" : "active";
      await setPatientProfileStatus({
        patientId: editPatientId,
        base: baseOfPatient(profileRow),
        status: next,
      });
      setProfileRow(await getPatient(editPatientId));
      setStatusMessage(next === "inactive"
        ? "Ficha desactivada en este equipo. Las citas existentes no se cancelan."
        : "Ficha activada en este equipo.");
    } catch (statusError: unknown) {
      setFormError(mutationErrorMessage(statusError));
    } finally {
      savingRef.current = false;
      setSaving(false);
      setStatusConfirm(false);
    }
  }

  function updateIdentifier(index: number, patch: Partial<IdentifierDraft>) {
    setIdentifiers((current) => current.map((item, i) => (i === index ? { ...item, ...patch } : item)));
  }

  function validate(): string | null {
    if (!fullName.trim()) return "Indica el nombre del paciente.";
    if (editMode) return null;
    if (basis === "own_document") {
      if (!phone.trim()) return "Indica el teléfono del paciente (código y número).";
      const docs = identifiers.filter((i) => i.documentNumber.trim());
      if (!docs.length) return "Indica al menos un documento declarado.";
    }
    if (basis === "minor_without_document") {
      if (!birthDate) return "Indica la fecha de nacimiento del menor.";
      if (contactMode === "new" && !contactName.trim()) return "Indica el nombre del responsable.";
      if (contactMode === "existing" && !selectedContactId) return "Selecciona un contacto existente.";
      if (contactMode === "new" && !contactPhone.trim()) return "Indica el teléfono del responsable.";
    }
    return null;
  }

  async function save(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (savingRef.current) return;
    const problem = validate();
    if (problem) { setFormError(problem); return; }
    savingRef.current = true;
    setSaving(true);
    setFormError(null);
    try {
      if (editMode && editPatientId && profileRow) {
        const original = visibleOf(profileRow);
        const base = baseOfPatient(profileRow);
        const patch: {
          fullName?: string | null;
          birthDate?: string | null;
          phoneCode?: string | null;
          phone?: string | null;
        } = {};
        if (original && fullName.trim() !== (original.fullName ?? "")) {
          patch.fullName = fullName.trim() || null;
        }
        const newBirth = birthDate || null;
        if (newBirth !== (original?.birthDate ?? null)) patch.birthDate = newBirth;
        if (basis === "own_document") {
          const newCode = phoneCode.trim() || null;
          const newPhone = phone.trim() || null;
          if (newCode !== (original?.phoneCode ?? null) || newPhone !== (original?.phone ?? null)) {
            patch.phoneCode = newCode;
            patch.phone = newPhone;
          }
        }
        if (Object.keys(patch).length > 0) {
          setProgress("Guardando los cambios del perfil…");
          await updatePatientProfile({ patientId: editPatientId, base, patch });
        }
        const pendingIdentifiers = identifiers.filter((i) => i.documentNumber.trim()
          && !loadedIdentifiers.includes(`${i.issuingCountry.trim() || "VE"}:${i.documentType}:${i.documentNumber.trim()}`));
        for (const identifier of pendingIdentifiers) {
          setProgress("Guardando documento nuevo…");
          await addPatientIdentifier({
            patientId: editPatientId,
            id: identifier.id,
            issuingCountry: identifier.issuingCountry.trim() || "VE",
            documentType: identifier.documentType,
            documentNumber: identifier.documentNumber.trim(),
          });
        }
        savedRef.current = true;
        sessionStorage.setItem(MESSAGE_KEY, "Cambios guardados en este equipo.");
        navigate(`/paciente/${editPatientId}`);
        return;
      }
      let patientId = createdPatientId;
      if (!patientId) {
        setProgress("Guardando la ficha del paciente…");
        const result = await registerPatient({
          fullName: fullName.trim(),
          ...(birthDate ? { birthDate } : {}),
          ...(basis === "own_document"
            ? {
                phoneCode: phoneCode.trim(),
                phone: phone.trim(),
                identificationBasis: "own_document",
                identifiers: identifiers
                  .filter((i) => i.documentNumber.trim())
                  .map((i) => ({
                    id: i.id,
                    issuingCountry: i.issuingCountry.trim() || "VE",
                    documentType: i.documentType,
                    documentNumber: i.documentNumber.trim(),
                  })),
              }
            : { identificationBasis: "minor_without_document" }),
        });
        patientId = typeof result.entityKey === "string" ? result.entityKey : null;
        if (!patientId) throw new Error("sin clave de paciente");
        setCreatedPatientId(patientId);
      }
      if (basis === "minor_without_document" && !relationshipCreated) {
        let contactPersonId: string;
        if (contactMode === "existing") {
          if (!selectedContactId) throw new Error("sin contacto seleccionado");
          contactPersonId = selectedContactId;
        } else if (createdContactId) {
          /* El contacto ya se creó en un intento previo fallido: se reutiliza
           * para no duplicarlo al reintentar. */
          contactPersonId = createdContactId;
        } else {
          setProgress("Guardando el contacto responsable…");
          const contact = await createContactPerson({
            fullName: contactName.trim(),
            phoneCode: contactPhoneCode.trim(),
            phone: contactPhone.trim(),
          });
          const key = contact.entityKey;
          let extracted: string | null = null;
          if (typeof key === "string") extracted = key;
          else if (key && typeof key === "object" && "contactPersonId" in key) {
            extracted = String((key as { contactPersonId: unknown }).contactPersonId);
          }
          if (!extracted) throw new Error("sin clave de contacto");
          setCreatedContactId(extracted);
          contactPersonId = extracted;
        }
        setProgress("Guardando el vínculo responsable…");
        const [fromY, fromM, fromD] = validFrom.split("-").map(Number);
        const [untilY, untilM, untilD] = validUntil.split("-").map(Number);
        await createPatientContactRelationship({
          patientId,
          contactPersonId,
          relationshipKind,
          isResponsibleContact: true,
          validFrom: new Date(civilMidnightUtc(zone, fromY, fromM, fromD)).toISOString(),
          ...(validUntil
            ? { validUntil: new Date(civilMidnightUtc(zone, untilY, untilM, untilD)).toISOString() }
            : {}),
        });
        setRelationshipCreated(true);
      }
      savedRef.current = true;
      sessionStorage.setItem(
        fromAppointment ? "cita:mensaje" : MESSAGE_KEY,
        "Paciente guardado en este equipo.",
      );
      if (fromAppointment && patientId) {
        navigate(`/cita/nueva?paciente=${patientId}`);
      } else {
        navigate(`/paciente/${patientId}`);
      }
    } catch (saveError: unknown) {
      const message = mutationErrorMessage(saveError);
      setFormError(createdPatientId
        ? `${message} La ficha ya quedó guardada en este equipo; vuelve a intentar solo los pasos pendientes.`
        : message);
    } finally {
      savingRef.current = false;
      setSaving(false);
      setProgress(null);
    }
  }

  if (phase === "loading") {
    return (
      <div className="state-block" role="status">
        <CircleNotch size={22} aria-hidden="true" />
        <p>Preparando el registro…</p>
      </div>
    );
  }

  if (phase === "no-session") {
    return (
      <div className="state-block" role="alert">
        <p>No hay una sesión lista en este equipo.</p>
        <p className="muted">Activa una cuenta para registrar pacientes y vuelve a intentarlo.</p>
      </div>
    );
  }

  return (
    <>
      <div className="page-header">
        <div>
          <h1 className="page-heading">{editMode ? "Editar paciente" : "Registrar paciente"}</h1>
          <p className="muted">La ficha se guarda en este equipo.</p>
        </div>
        <div className="toolbar">
          {editMode && profileRow && (
            <button type="button" className="btn" onClick={() => navigate(`/paciente/${editPatientId}`)}>
              Volver a la ficha
            </button>
          )}
          <button type="button" className="btn" onClick={() => navigate("/pacientes")}>
            Volver al directorio
          </button>
        </div>
      </div>

      <form className="editor-layout" onSubmit={(event) => void save(event)}>
        <div style={{ display: "grid", gap: 20, minWidth: 0, flex: 1 }}>
          <fieldset style={{ border: 0, padding: 0, margin: 0, display: "grid", gap: 16 }}>
            <legend className="muted" style={{ marginBottom: 8 }}>Perfil</legend>
            <label className="field-label">
              Base de identificación
              <select
                className="select"
                value={basis}
                disabled={editMode}
                onChange={(event) => setBasis(event.target.value as typeof basis)}
              >
                <option value="own_document">Persona con documento propio</option>
                <option value="minor_without_document">Menor sin documento</option>
              </select>
            </label>
            <label className="field-label">
              Nombre completo
              <input
                type="text"
                className="input-text"
                value={fullName}
                onChange={(event) => setFullName(event.target.value)}
                autoComplete="name"
              />
            </label>
            <label className="field-label">
              Fecha de nacimiento{basis === "minor_without_document" ? " (obligatoria para menores)" : ""}
              <input
                type="date"
                className="input-date"
                value={birthDate}
                max={civilToday("UTC")}
                onChange={(event) => setBirthDate(event.target.value)}
              />
            </label>
          </fieldset>

          {basis === "own_document" && (
            <fieldset style={{ border: 0, padding: 0, margin: 0, display: "grid", gap: 16 }}>
              <legend className="muted" style={{ marginBottom: 8 }}>Teléfono propio</legend>
              <div className="toolbar">
                <label className="field-label">
                  Código
                  <input
                    type="text"
                    className="input-text"
                    style={{ width: 80 }}
                    value={phoneCode}
                    onChange={(event) => setPhoneCode(event.target.value)}
                  />
                </label>
                <label className="field-label" style={{ flex: 1, minWidth: 180 }}>
                  Número
                  <input
                    type="tel"
                    className="input-text"
                    value={phone}
                    onChange={(event) => setPhone(event.target.value)}
                    autoComplete="tel-national"
                  />
                </label>
              </div>
            </fieldset>
          )}

          {basis === "own_document" && (
            <fieldset style={{ border: 0, padding: 0, margin: 0, display: "grid", gap: 12 }}>
              <legend className="muted" style={{ marginBottom: 8 }}>Documentos declarados</legend>
              {identifiers.map((identifier, index) => (
                <div className="toolbar" key={identifier.id} style={{ alignItems: "flex-end" }}>
                  <label className="field-label">
                    País emisor
                    <input
                      type="text"
                      className="input-text"
                      style={{ width: 72 }}
                      value={identifier.issuingCountry}
                      onChange={(event) => updateIdentifier(index, { issuingCountry: event.target.value })}
                    />
                  </label>
                  <label className="field-label">
                    Tipo
                    <select
                      className="select"
                      value={identifier.documentType}
                      onChange={(event) => updateIdentifier(index, { documentType: event.target.value })}
                    >
                      {Object.entries(documentTypeLabel).map(([value, label]) => (
                        <option key={value} value={value}>{label}</option>
                      ))}
                    </select>
                  </label>
                  <label className="field-label" style={{ flex: 1, minWidth: 160 }}>
                    Número de documento
                    <input
                      type="text"
                      className="input-text"
                      value={identifier.documentNumber}
                      onChange={(event) => updateIdentifier(index, { documentNumber: event.target.value })}
                    />
                  </label>
                  {identifiers.length > 1 && (
                    <button
                      type="button"
                      className="icon-btn"
                      aria-label="Quitar documento"
                      onClick={() => setIdentifiers((current) => current.filter((_, i) => i !== index))}
                    >
                      <Trash size={16} aria-hidden="true" />
                    </button>
                  )}
                </div>
              ))}
              <div>
                <button
                  type="button"
                  className="btn btn--subtle"
                  onClick={() => setIdentifiers((current) => [
                    ...current,
                    { id: uuidV7(), issuingCountry: "VE", documentType: "national_id", documentNumber: "" },
                  ])}
                >
                  <Plus size={16} aria-hidden="true" />
                  Añadir documento
                </button>
              </div>
            </fieldset>
          )}

          {basis === "minor_without_document" && (
            <fieldset style={{ border: 0, padding: 0, margin: 0, display: "grid", gap: 16 }}>
              <legend className="muted" style={{ marginBottom: 8 }}>Responsable</legend>
              <div className="toolbar" role="radiogroup" aria-label="Origen del responsable">
                <label className="badge" style={{ cursor: "pointer" }}>
                  <input
                    type="radio"
                    name="contact-mode"
                    checked={contactMode === "new"}
                    onChange={() => { setContactMode("new"); setSelectedContactId(null); }}
                  />
                  Nuevo contacto
                </label>
                <label className="badge" style={{ cursor: "pointer" }}>
                  <input
                    type="radio"
                    name="contact-mode"
                    checked={contactMode === "existing"}
                    onChange={() => setContactMode("existing")}
                  />
                  Contacto existente
                </label>
              </div>

              {contactMode === "new" ? (
                <>
                  <label className="field-label">
                    Nombre del responsable
                    <input
                      type="text"
                      className="input-text"
                      value={contactName}
                      onChange={(event) => setContactName(event.target.value)}
                    />
                  </label>
                  <div className="toolbar">
                    <label className="field-label">
                      Código
                      <input
                        type="text"
                        className="input-text"
                        style={{ width: 80 }}
                        value={contactPhoneCode}
                        onChange={(event) => setContactPhoneCode(event.target.value)}
                      />
                    </label>
                    <label className="field-label" style={{ flex: 1, minWidth: 180 }}>
                      Teléfono del responsable
                      <input
                        type="tel"
                        className="input-text"
                        value={contactPhone}
                        onChange={(event) => setContactPhone(event.target.value)}
                      />
                    </label>
                  </div>
                </>
              ) : (
                <div style={{ display: "grid", gap: 8 }}>
                  <label className="field-label">
                    Buscar contacto
                    <input
                      type="search"
                      className="input-text"
                      placeholder="Nombre del contacto"
                      value={contactQuery}
                      onChange={(event) => setContactQuery(event.target.value)}
                    />
                  </label>
                  {contactResultsState === "loading" && <p className="muted" role="status">Buscando contactos…</p>}
                  {contactResultsState === "error" && (
                    <p className="notice" role="alert">No se pudieron buscar contactos en este equipo.</p>
                  )}
                  {contactResultsState === "ready" && contactResults.length === 0 && (
                    <p className="muted">Sin coincidencias en datos locales.</p>
                  )}
                  {contactResultsState === "ready" && contactResults.length > 0 && (
                    <ul style={{ margin: 0, paddingLeft: 20 }}>
                      {contactResults.map((row) => {
                        const contact = visibleOf(row);
                        if (!contact) return null;
                        const label = `${contact.fullName}${contact.phone ? ` · ${contact.phoneCode ?? ""} ${contact.phone}` : ""}`;
                        return (
                          <li key={contact.contactPersonId} style={{ marginBottom: 4 }}>
                            <label style={{ display: "flex", gap: 8, alignItems: "center" }}>
                              <input
                                type="radio"
                                name="existing-contact"
                                checked={selectedContactId === contact.contactPersonId}
                                onChange={() => setSelectedContactId(contact.contactPersonId)}
                              />
                              {label}
                            </label>
                          </li>
                        );
                      })}
                    </ul>
                  )}
                </div>
              )}

              <div className="toolbar">
                <label className="field-label">
                  Relación
                  <select
                    className="select"
                    value={relationshipKind}
                    onChange={(event) => setRelationshipKind(event.target.value)}
                  >
                    {Object.entries(relationshipKindLabel).map(([value, label]) => (
                      <option key={value} value={value}>{label}</option>
                    ))}
                  </select>
                </label>
                <label className="field-label">
                  Válido desde
                  <input
                    type="date"
                    className="input-date"
                    value={validFrom}
                    onChange={(event) => setValidFrom(event.target.value)}
                  />
                </label>
                <label className="field-label">
                  Válido hasta (opcional)
                  <input
                    type="date"
                    className="input-date"
                    value={validUntil}
                    onChange={(event) => setValidUntil(event.target.value)}
                  />
                </label>
              </div>
              <p className="muted" style={{ fontSize: 13 }}>
                El vínculo se marca como responsable; la vigencia a la fecha de cada cita
                la comprueba el sistema.
              </p>
            </fieldset>
          )}
        </div>

        <div className="editor-summary">
          <section className="detail-panel" aria-labelledby="resumen-registro" style={{ width: "100%" }}>
            <h2 id="resumen-registro" style={{ fontSize: 20 }}>Resumen</h2>
            <dl className="fact-grid">
              <div>
                <dt>Nombre</dt>
                <dd>{fullName.trim() || "Sin indicar"}</dd>
              </div>
              {editMode && profileRow && (
                <div>
                  <dt>Estado de la ficha</dt>
                  <dd>{visibleOf(profileRow)?.status === "active" ? "Activo" : "Inactivo"}</dd>
                </div>
              )}
              <div>
                <dt>Base de identificación</dt>
                <dd>{basis === "own_document" ? "Documento propio" : "Menor sin documento"}</dd>
              </div>
              <div>
                <dt>Teléfono</dt>
                <dd className="num">
                  {basis === "own_document"
                    ? (phone ? `${phoneCode} ${phone}` : "Sin indicar")
                    : (contactMode === "new" ? (contactPhone ? `${contactPhoneCode} ${contactPhone}` : "Sin indicar") : (selectedContactId ? "Contacto existente" : "Sin seleccionar"))}
                </dd>
              </div>
              <div>
                <dt>Documentos</dt>
                <dd>{identifiers.filter((i) => i.documentNumber.trim()).length}</dd>
              </div>
            </dl>
            {progress && <p className="muted" role="status">{progress}</p>}
            {statusMessage && <div className="notice" role="status"><p>{statusMessage}</p></div>}
            {formError && (
              <div className="notice" role="alert"><p>{formError}</p></div>
            )}
            {editMode && profileRow && !statusConfirm && (
              <button
                type="button"
                className="btn"
                disabled={saving}
                onClick={() => setStatusConfirm(true)}
              >
                <PencilSimple size={16} aria-hidden="true" />
                {visibleOf(profileRow)?.status === "active" ? "Desactivar ficha" : "Activar ficha"}
              </button>
            )}
            {editMode && profileRow && statusConfirm && (
              <div className="notice" role="alertdialog" aria-label="Confirmar cambio de estado">
                <p>
                  {visibleOf(profileRow)?.status === "active"
                    ? "Desactivar la ficha no cancela sus citas. El cambio queda guardado en este equipo."
                    : "La ficha volverá a estar activa. El cambio queda guardado en este equipo."}
                </p>
                <div className="toolbar">
                  <button
                    type="button"
                    className="btn btn--primary"
                    disabled={saving}
                    onClick={() => void toggleStatus()}
                  >
                    Confirmar
                  </button>
                  <button type="button" className="btn" onClick={() => setStatusConfirm(false)}>
                    Cancelar
                  </button>
                </div>
              </div>
            )}
            <button type="submit" className="btn btn--primary" disabled={saving}>
              {saving ? <><CircleNotch size={16} aria-hidden="true" /> Guardando…</> : (editMode ? "Guardar cambios" : "Guardar paciente")}
            </button>
            <p className="muted" style={{ fontSize: 13 }}>
              Guardar conserva la ficha en este equipo; no confirma aceptación compartida.
              Las coincidencias de nombre o teléfono no fusionan personas.
            </p>
          </section>
        </div>
      </form>

      <Modal open={blocker.state === "blocked"} onClose={() => blocker.reset?.()} alert labelledBy="descartar-registro">
        <h2 id="descartar-registro" style={{ fontSize: 20 }}>Cambios sin guardar</h2>
        <p className="muted">Si sales ahora, el registro no se guardará y los cambios se perderán.</p>
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
