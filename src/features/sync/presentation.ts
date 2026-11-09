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
