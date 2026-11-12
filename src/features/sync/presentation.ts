/** Vocabulario de trabajo para el centro de cambios (S11). */
export const syncWorkLabel: Record<string, string> = {
  pending: "Guardada en este equipo · Pendiente de aceptación",
  in_flight: "Enviando cambio",
  unknown: "Resultado por comprobar",
  retryable: "Pendiente de reintento",
  accepted: "Aceptada; actualizando la vista local",
  conflict: "Necesita revisión por un cambio compartido",
  rejected: "No aceptada; requiere revisión",
  review_required: "Requiere revisión",
};

export const outcomeLabel: Record<string, string> = {
  complete: "Descarga completada",
  offline: "Sin conexión; se conserva el trabajo local",
  retry_waiting: "Servicio no disponible; puedes reintentar",
  access_denied: "Acceso denegado",
  recovery_required: "Se requiere recuperar la vista local",
  failed: "Servicio no disponible; puedes reintentar",
};

export const actionLabel: Record<string, string> = {
  CreateAppointment: "Crear cita",
  RescheduleAppointment: "Reprogramar cita",
  CancelAppointment: "Cancelar cita",
  ConfirmAppointment: "Confirmar cita",
  CompleteAppointment: "Registrar atención",
  MarkNoShow: "Registrar ausencia",
  RegisterPatient: "Registrar paciente",
  UpdatePatientProfile: "Editar perfil",
  SetPatientProfileStatus: "Cambiar estado de ficha",
  AddPatientIdentifier: "Añadir documento",
  CreateContactPerson: "Registrar contacto",
  CreatePatientContactRelationship: "Vincular responsable",
};

export const entityLabel: Record<string, string> = {
  appointment: "Cita",
  patient_profile: "Ficha de paciente",
  patient_identifier: "Documento",
  contact_person: "Contacto",
  patient_contact_relationship: "Vínculo",
  service: "Servicio",
  service_offer: "Oferta",
  schedule_rule: "Horario",
};

/** Solo estos estados requieren una decisión del personal; pendiente,
 * enviando, reintento y resultado desconocido siguen su recuperación
 * automática sin mostrarse en Por revisar. */
export function needsIntervention(status: string): boolean {
  return status === "conflict" || status === "rejected" || status === "review_required";
}

/** Ruta local de una revisión según su entidad y clave; null cuando no hay
 * ruta específica (la salida segura será su sección o Ayuda). No inventa
 * enlaces para entidades sin recorrido contratado. */
export function reviewRoute(entityType: string, entityKey: unknown): string | null {
  const key = typeof entityKey === "string" && entityKey !== ""
    ? entityKey
    : typeof entityKey === "object" && entityKey !== null
      ? ("appointmentId" in entityKey && typeof (entityKey as { appointmentId: unknown }).appointmentId === "string"
          ? (entityKey as { appointmentId: string }).appointmentId
          : "patientId" in entityKey && typeof (entityKey as { patientId: unknown }).patientId === "string"
            ? (entityKey as { patientId: string }).patientId
            : null)
      : null;
  if (key === null) return null;
  if (entityType === "appointment") return `/cita/${key}`;
  if (entityType === "patient_profile") return `/paciente/${key}`;
  return null;
}
