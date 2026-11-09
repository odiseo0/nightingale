import { CircleNotch, Lock } from "@phosphor-icons/react";
import type { ProfileState, ServiceOfferView } from "../../shared/api/appointments";
import type { RelationshipView } from "../../shared/api/patients";
import { formatDateLong, formatTime, formatTimeEnd } from "./presentation";

interface Props {
  zone: string;
  civil: string;
  patient: ProfileState | null;
  offer: ServiceOfferView | null;
  professionalName: (professionalId: string) => string;
  slot: { startsAt: string; endsAt: string } | null;
  responsible: RelationshipView | null;
  canSave: boolean;
  saving: boolean;
  error: string | null;
  onSave: () => void;
}

/** Resumen visible antes de la acción final: paciente, servicio, fecha,
 * intervalo, duración, responsable si aplica y zona. */
export default function BookingSummary({
  zone, civil, patient, offer, professionalName, slot, responsible,
  canSave, saving, error, onSave,
}: Props) {
  const missing: string[] = [];
  if (!patient) missing.push("paciente");
  if (!offer) missing.push("servicio");
  if (!slot) missing.push("intervalo");

  return (
    <section className="detail-panel" aria-labelledby="resumen-titulo" style={{ width: "100%" }}>
      <h2 id="resumen-titulo" style={{ fontSize: 20 }}>Resumen</h2>
      <dl className="fact-grid">
        <div>
          <dt>Paciente</dt>
          <dd>{patient ? (patient.fullName?.trim() || "Paciente sin nombre registrado") : "Sin seleccionar"}</dd>
        </div>
        <div>
          <dt>Servicio</dt>
          <dd>{offer ? offer.name : "Sin seleccionar"}</dd>
        </div>
        <div>
          <dt>Profesional</dt>
          <dd>{offer ? professionalName(offer.professionalId) : "Sin seleccionar"}</dd>
        </div>
        <div>
          <dt>Fecha</dt>
          <dd>{formatDateLong(zone, civil)}</dd>
        </div>
        <div>
          <dt>Intervalo</dt>
          <dd className="num">
            {slot
              ? `${formatTime(zone, slot.startsAt)}–${formatTimeEnd(zone, slot.endsAt)}`
              : "Sin seleccionar"}
          </dd>
        </div>
        <div>
          <dt>Duración</dt>
          <dd className="num">{offer ? `${offer.effectiveDurationMinutes} min` : "—"}</dd>
        </div>
        {responsible && (
          <div>
            <dt>Responsable</dt>
            <dd>
              {responsible.contactFullName} · {responsible.relationship.relationshipKind}
            </dd>
          </div>
        )}
        <div>
          <dt>Zona</dt>
          <dd>{zone}</dd>
        </div>
      </dl>

      {error && (
        <div className="notice" role="alert">
          <p>{error}</p>
        </div>
      )}

      <button
        type="button"
        className="btn btn--primary"
        disabled={!canSave || saving}
        onClick={onSave}
      >
        {saving
          ? <><CircleNotch size={16} aria-hidden="true" /> Guardando…</>
          : <><Lock size={16} aria-hidden="true" /> Guardar cita</>}
      </button>
      {!canSave && !saving && (
        <p className="muted">Falta: {missing.join(", ")}.</p>
      )}
      <p className="muted" style={{ fontSize: 13 }}>
        Guardar crea una cita provisional en este equipo; no confirma una reserva
        aceptada ni envía ningún mensaje.
      </p>
    </section>
  );
}
