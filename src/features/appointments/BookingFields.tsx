import { CircleNotch, Info, Plus, Warning } from "@phosphor-icons/react";
import type { ProfileState, ServiceOfferView } from "../../shared/api/appointments";
import type { EntityRow, RelationshipView } from "../../shared/api/patients";
import { civilToday, formatDateLong } from "./presentation";
import AvailabilityPicker, { type AvailabilityState } from "./AvailabilityPicker";

export interface ResponsibleState {
  required: boolean;
  status: "idle" | "loading" | "ready" | "error";
  links: EntityRow<RelationshipView>[];
}

interface Props {
  zone: string;
  patients: ProfileState[];
  offers: ServiceOfferView[];
  professionalName: (professionalId: string) => string;
  patientId: string | null;
  onPatientChange: (patientId: string | null) => void;
  offer: ServiceOfferView | null;
  onOfferChange: (offer: ServiceOfferView | null) => void;
  civil: string;
  onCivilChange: (civil: string) => void;
  responsibleId: string | null;
  onResponsibleChange: (contactPersonId: string | null) => void;
  responsible: ResponsibleState;
  onRetryContacts: () => void;
  availability: AvailabilityState;
  slot: { startsAt: string; endsAt: string } | null;
  onSlotChange: (slot: { startsAt: string; endsAt: string } | null) => void;
  onRetryAvailability: () => void;
  onRegisterPatient: () => void;
}

/** Campos del editor S04 en tres grupos: Paciente, Servicio y Horario.
 * Etiquetas persistentes, valores reales y sin identificadores técnicos. */
export default function BookingFields({
  zone, patients, offers, professionalName,
  patientId, onPatientChange, offer, onOfferChange, civil, onCivilChange,
  responsibleId, onResponsibleChange, responsible, onRetryContacts,
  availability, slot, onSlotChange, onRetryAvailability, onRegisterPatient,
}: Props) {
  const today = civilToday(zone);
  const selectedPatient = patients.find((p) => p.patientId === patientId) ?? null;
  const patientIncomplete = selectedPatient?.completionStatus === "draft";

  const responsibleLinks = responsible.links
    .map((row) => {
      const view = row.provisional?.state ?? row.accepted?.state ?? null;
      return view && view.relationship.isResponsibleContact && view.relationship.status === "active"
        ? view
        : null;
    })
    .filter((v): v is RelationshipView => v !== null);

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 20, minWidth: 0, flex: 1 }}>
      <fieldset style={{ border: 0, padding: 0, margin: 0, display: "grid", gap: 16 }}>
        <legend className="muted" style={{ marginBottom: 8 }}>Paciente</legend>

        {patients.length === 1 && selectedPatient === null ? (
          <p>
            <span className="muted">Paciente: </span>{patients[0].fullName ?? "Paciente sin nombre registrado"}
          </p>
        ) : null}

        {patients.length === 0 ? (
          <div className="notice" role="alert">
            <Warning size={18} aria-hidden="true" />
            <p>No hay pacientes disponibles en el contexto de esta cuenta.</p>
          </div>
        ) : patients.length > 1 || selectedPatient !== null ? (
          <label className="field-label">
            Paciente
            <select
              className="select"
              value={patientId ?? ""}
              onChange={(event) => onPatientChange(event.target.value || null)}
            >
              <option value="">Selecciona un paciente</option>
              {patients.map((patient) => (
                <option key={patient.patientId} value={patient.patientId}>
                  {patient.fullName?.trim() || "Paciente sin nombre registrado"}
                  {patient.completionStatus === "draft" ? " (ficha incompleta)" : ""}
                </option>
              ))}
            </select>
          </label>
        ) : null}

        <div>
          <button type="button" className="btn btn--subtle" onClick={onRegisterPatient}>
            <Plus size={16} aria-hidden="true" />
            Registrar paciente
          </button>
        </div>

        {patientIncomplete && (
          <div className="notice">
            <Info size={18} aria-hidden="true" />
            <p>La ficha de este paciente está incompleta; el proceso local puede rechazar la cita.</p>
          </div>
        )}
      </fieldset>

      <fieldset style={{ border: 0, padding: 0, margin: 0, display: "grid", gap: 16 }}>
        <legend className="muted" style={{ marginBottom: 8 }}>Servicio</legend>

        {offers.length === 0 ? (
          <div className="notice" role="alert">
            <Warning size={18} aria-hidden="true" />
            <p>No hay servicios ofrecidos disponibles en esta cuenta.</p>
          </div>
        ) : offers.length === 1 && !offer ? (
          <p>
            <span className="muted">Servicio: </span>
            {offers[0].name} ({offers[0].effectiveDurationMinutes} min) con {professionalName(offers[0].professionalId)}
          </p>
        ) : (
          <label className="field-label">
            Servicio y profesional
            <select
              className="select"
              value={offer ? `${offer.professionalId}:${offer.serviceId}` : ""}
              onChange={(event) => {
                const chosen = offers.find(
                  (o) => `${o.professionalId}:${o.serviceId}` === event.target.value,
                );
                onOfferChange(chosen ?? null);
              }}
            >
              <option value="">Selecciona un servicio</option>
              {offers.map((o) => (
                <option key={`${o.professionalId}:${o.serviceId}`} value={`${o.professionalId}:${o.serviceId}`}>
                  {o.name} ({o.effectiveDurationMinutes} min) · {professionalName(o.professionalId)}
                </option>
              ))}
            </select>
          </label>
        )}
      </fieldset>

      <fieldset style={{ border: 0, padding: 0, margin: 0, display: "grid", gap: 12 }}>
        <legend className="muted" style={{ marginBottom: 8 }}>Horario</legend>
        <div className="toolbar">
          <button type="button" className="btn btn--subtle" onClick={() => onCivilChange(today)} disabled={civil === today}>
            Hoy
          </button>
          <label className="field-label">
            Fecha de la cita
            <input
              type="date"
              className="input-date"
              value={civil}
              min={today}
              onChange={(event) => onCivilChange(event.target.value)}
            />
          </label>
        </div>
        <p className="muted" aria-live="polite">{formatDateLong(zone, civil)}</p>
        <AvailabilityPicker
          zone={zone}
          availability={availability}
          slot={slot}
          onSlotChange={onSlotChange}
          onRetry={onRetryAvailability}
        />
      </fieldset>

      {responsible.required && (
        <fieldset style={{ border: 0, padding: 0, margin: 0, display: "grid", gap: 12 }}>
          <legend className="muted" style={{ marginBottom: 8 }}>Responsable</legend>
          <p className="muted">
            La ficha del paciente es de menor sin documento: se requiere un responsable
            vigente. La elegibilidad final a la fecha de la cita la valida el proceso local.
          </p>
          {responsible.status === "loading" && (
            <p className="muted" role="status"><CircleNotch size={16} aria-hidden="true" /> Consultando responsables…</p>
          )}
          {responsible.status === "error" && (
            <div className="notice" role="alert">
              <p>No se pudieron consultar los vínculos del paciente.</p>
              <button type="button" className="btn" onClick={onRetryContacts}>Reintentar</button>
            </div>
          )}
          {responsible.status === "ready" && responsibleLinks.length === 0 && (
            <div className="notice" role="alert">
              <Warning size={18} aria-hidden="true" />
              <p>
                No hay vínculos activos marcados como responsables para este paciente;
                no se puede guardar la cita hasta que exista uno.
              </p>
            </div>
          )}
          {responsible.status === "ready" && responsibleLinks.length > 0 && (
            <label className="field-label">
              Responsable de la cita
              <select
                className="select"
                value={responsibleId ?? ""}
                onChange={(event) => onResponsibleChange(event.target.value || null)}
              >
                <option value="">Selecciona un responsable</option>
                {responsibleLinks.map((view) => {
                  const phone = view.contactPhone
                    ? `${view.contactPhoneCode ? `${view.contactPhoneCode} ` : ""}${view.contactPhone}`
                    : "sin teléfono";
                  const validity = view.relationship.validUntil
                    ? `${view.relationship.validFrom} – ${view.relationship.validUntil}`
                    : `desde ${view.relationship.validFrom}`;
                  return (
                    <option key={view.relationship.contactPersonId} value={view.relationship.contactPersonId}>
                      {view.contactFullName} · {view.relationship.relationshipKind} · {phone} · {validity}
                    </option>
                  );
                })}
              </select>
            </label>
          )}
        </fieldset>
      )}
    </div>
  );
}
