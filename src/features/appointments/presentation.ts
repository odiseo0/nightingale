import type {
  BookingContext, EditBase, EntityRow, ProfileState,
} from "../../shared/api/appointments";

export const statusLabel: Record<string, string> = {
  scheduled: "Programada",
  confirmed: "Confirmada",
  cancelled: "Cancelada",
  completed: "Atendida",
  no_show: "Ausencia",
};

export const syncLabel: Record<string, string> = {
  pending: "Guardada en este equipo · Pendiente de aceptación",
  in_flight: "Enviando cambio",
  unknown: "Resultado por comprobar",
  retryable: "Pendiente de reintento",
  accepted: "Aceptada; actualizando la vista local",
  conflict: "Necesita revisión por un cambio compartido",
  rejected: "No aceptada; requiere revisión",
  review_required: "Requiere revisión",
};

export function errorMessage(error: unknown): string {
  const code = typeof error === "object" && error !== null && "code" in error
    ? String((error as { code: string }).code)
    : "TEMPORARY_FAILURE";
  if (code === "AUTHENTICATION_REQUIRED" || code === "PERMISSION_DENIED") {
    return "Tu acceso no permite leer estos datos; inicia sesión con una cuenta autorizada.";
  }
  // No se interpolan mensajes arbitrarios (p. ej. errores internos de JS);
  // los códigos conocidos ya tienen texto útil y seguro.
  return `No se pudo completar la lectura local (${code}).`;
}

/** Medianoche civil de un día en una zona, como instante UTC (solo
 * presentación; Rust sigue siendo la autoridad en reglas de tiempo). */
export function civilMidnightUtc(zone: string, y: number, m: number, d: number): number {
  const guess = Date.UTC(y, m - 1, d);
  const parts = new Intl.DateTimeFormat("en-US", {
    timeZone: zone, hour12: false,
    year: "numeric", month: "2-digit", day: "2-digit",
    hour: "2-digit", minute: "2-digit",
  }).formatToParts(guess);
  const value = (type: string) => Number(parts.find((p) => p.type === type)?.value ?? 0);
  const civilHours = value("hour") % 24;
  // Diferencia zona: reloj civil del instante de referencia menos el UTC.
  // Para zonas al oeste de GMT el valor bruto supera 12 h y hay que envolverlo.
  let offsetMinutes = civilHours * 60 + value("minute");
  if (offsetMinutes > 720) offsetMinutes -= 1440;
  if (offsetMinutes <= -720) offsetMinutes += 1440;
  return guess - offsetMinutes * 60_000;
}

export function civilToday(zone: string): string {
  return civilToDateParts(zone, new Date());
}

/** Fecha civil (YYYY-MM-DD) de un instante en la zona indicada. */
export function civilOfInstant(zone: string, iso: string): string {
  return civilToDateParts(zone, new Date(iso));
}

function civilToDateParts(zone: string, instant: Date): string {
  const parts = new Intl.DateTimeFormat("en-CA", {
    timeZone: zone, year: "numeric", month: "2-digit", day: "2-digit",
  }).formatToParts(instant);
  const value = (type: string) => parts.find((p) => p.type === type)?.value ?? "";
  return `${value("year")}-${value("month")}-${value("day")}`;
}

export function shiftCivil(zone: string, civil: string, deltaDays: number): string {
  const [y, m, d] = civil.split("-").map(Number);
  return civilToDateParts(zone, new Date(civilMidnightUtc(zone, y, m, d) + deltaDays * 86_400_000));
}

export function dayRange(zone: string, civil: string): { from: string; until: string } {
  const [y, m, d] = civil.split("-").map(Number);
  const start = civilMidnightUtc(zone, y, m, d);
  return {
    from: new Date(start).toISOString(),
    until: new Date(start + 86_400_000).toISOString(),
  };
}

const timeFormatter = (zone: string) => new Intl.DateTimeFormat("es", {
  timeZone: zone, hour: "2-digit", minute: "2-digit", hourCycle: "h23",
});

export function formatTime(zone: string, iso: string): string {
  return timeFormatter(zone).format(new Date(iso));
}

/** Hora de fin de intervalo: una medianoche como final se presenta 24:00
 * (fin del día anterior), nunca 00:00. */
export function formatTimeEnd(zone: string, iso: string): string {
  const formatted = timeFormatter(zone).format(new Date(iso));
  return formatted === "00:00" ? "24:00" : formatted;
}

export function formatDateLong(zone: string, civil: string): string {
  const [y, m, d] = civil.split("-").map(Number);
  return new Intl.DateTimeFormat("es", {
    timeZone: zone, weekday: "long", day: "numeric", month: "long", year: "numeric",
  }).format(new Date(civilMidnightUtc(zone, y, m, d)));
}

export function formatDateFull(zone: string, iso: string): string {
  return new Intl.DateTimeFormat("es", {
    timeZone: zone, weekday: "long", day: "numeric", month: "long", year: "numeric",
  }).format(new Date(iso));
}

export function formatDateTime(zone: string, iso: string): string {
  return new Intl.DateTimeFormat("es", {
    timeZone: zone, day: "numeric", month: "short", year: "numeric",
    hour: "2-digit", minute: "2-digit", hour12: false,
  }).format(new Date(iso));
}

export function baseOf<T>(row: EntityRow<T>): EditBase {
  return row.provisional
    ? { operationId: row.provisional.operationId }
    : { acceptedVersion: row.accepted!.version };
}

export function mutationErrorMessage(error: unknown): string {
  const code = typeof error === "object" && error !== null && "code" in error
    ? String((error as { code: string }).code)
    : "TEMPORARY_FAILURE";
  // Textos estables y orientados a la acción; nunca se interpolan objetos
  // arbitrarios de error en la interfaz.
  switch (code) {
    case "SLOT_UNAVAILABLE":
      return "El intervalo queda fuera del horario disponible o dentro de una ocupación; no se guardó. Elige otro intervalo.";
    case "INSUFFICIENT_COVERAGE":
      return "La ventana de ocupación descargada no cubre el intervalo; sincroniza antes de reservar.";
    case "INVALID_STATE_TRANSITION":
      return "La acción no está permitida desde el estado actual de la cita.";
    case "VERSION_CONFLICT":
      return "La base local cambió; vuelve a cargar la cita e inténtalo de nuevo.";
    case "DEPENDENCY_FAILED":
      return "Hay resultados previos que requieren revisión antes de editar.";
    case "NO_CHANGES":
      return "El cambio no modifica el estado actual.";
    case "PERMISSION_DENIED":
    case "AUTHENTICATION_REQUIRED":
      return "Tu acceso no permite esta acción; inicia sesión con una cuenta autorizada.";
    default:
      return `No se pudo completar la acción (${code}). Inténtalo de nuevo.`;
  }
}

export const coverageLabel: Record<string, string> = {
  within_synced_window: "Dentro de la ventana de ocupación descargada.",
  no_synced_window: "Sin ventana descargada: se usan solo los datos locales conocidos.",
  insufficient_coverage:
    "Cobertura insuficiente: la ausencia de intervalos no prueba que no haya huecos.",
};

export function patientNameOf(context: BookingContext | null, patientId: string): string {
  const row = context?.patients.find((p) =>
    (p.provisional?.state ?? p.accepted?.state)?.patientId === patientId,
  );
  const profile: ProfileState | null = row?.provisional?.state ?? row?.accepted?.state ?? null;
  return profile?.fullName?.trim() || "Paciente sin nombre registrado";
}

export function patientPhoneOf(context: BookingContext | null, patientId: string): string | null {
  const row = context?.patients.find((p) =>
    (p.provisional?.state ?? p.accepted?.state)?.patientId === patientId,
  );
  const profile = row?.provisional?.state ?? row?.accepted?.state;
  if (!profile) return null;
  if (profile.phoneCode && profile.phone) return `${profile.phoneCode} ${profile.phone}`;
  return profile.phone ?? null;
}

/** La columna y el filtro Profesional se ocultan solo para el dueño con un
 * único profesional disponible; asistentes y cuentas con varios profesionales
 * lo conservan. El rol nulo (contexto no disponible) nunca oculta. */
export function showsProfessionalColumn(
  viewerRole: "owner" | "assistant" | null,
  professionalCount: number,
): boolean {
  return viewerRole !== "owner" || professionalCount > 1;
}

/** Dirección de lectura: termina en ciudad y estado, sin país ni separadores
 * vacíos cuando falta un dato. El país almacenado se conserva fuera de esta
 * lectura. */
export function formatAddressLine(parts: {
  address?: string | null;
  city?: string | null;
  state?: string | null;
}): string {
  return [parts.address, parts.city, parts.state]
    .filter((part): part is string => typeof part === "string" && part.trim().length > 0)
    .join(", ");
}

/** Actividad traducida a acciones comprensibles; códigos desconocidos se
 * muestran tal cual (son cadenas contractuales estables y seguras). */
export const historyActionLabel: Record<string, string> = {
  created: "Cita creada",
  confirmed: "Cita confirmada",
  rescheduled: "Cita reprogramada",
  cancelled: "Cita cancelada",
  completed: "Atención registrada",
  no_show: "Ausencia registrada",
};
