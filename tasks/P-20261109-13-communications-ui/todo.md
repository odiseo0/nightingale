# Task List: P-20261109-13-communications-ui

**Plan:** [plan.md](plan.md)
**Updated:** 2026-11-09

Plan Blocked: requiere workers/contrato de comunicaciones.

## T01: Implementar comunicaciones e historial con estados reales

**Status:** Blocked

**Description:** Construir lista de comunicaciones próximas/historial con paciente/cita, tipo, canal, fechas y estado real.

**Acceptance criteria:**
- [ ] Programado/enviado/entregado/respondido/fallo se distinguen de intención local y estado de cita.
- [ ] Cada fila muestra última actualización/antigüedad y enlaces reales permitidos.
- [ ] Vacío/sin datos/desactualizado/error no se confunden; no hay chat libre.

**Verification:**
- [ ] Focused tests: `pnpm build` y lectura del contrato disponible.
- [ ] Build or broader check: `pnpm quality`.
- [ ] Manual check: tabla, estados, teclado y viewports.

**Dependencies:** Workers/contrato de comunicaciones

**Files likely touched:**
- `src/features/communications/CommunicationsPage.tsx`
- Cliente contractual definido por el plan técnico

**Estimated scope:** M

## T02: Implementar detalle, contacto y reintento autorizado

**Status:** Blocked

**Description:** Añadir detalle de comunicación, problema de contacto y acción de reintento solo si el contrato la permite.

**Acceptance criteria:**
- [ ] Detalle muestra resultado conocido y no convierte entrega en asistencia.
- [ ] Revisar contacto enlaza a ficha real; reintento solo con capacidad/permiso explícitos.
- [ ] Errores conservan contexto y no provocan duplicados.

**Verification:**
- [ ] Focused tests: `pnpm build`; detalle y reintento permitido/rechazado.
- [ ] Build or broader check: `pnpm quality`.
- [ ] Manual check: acciones, teclado/foco y retorno; revisión C01.

**Dependencies:** T01

**Files likely touched:**
- `src/features/communications/CommunicationDetail.tsx`
- `src/features/patients/PatientRecordPage.tsx`

**Estimated scope:** M

## C01: Después de T01–T02

- [ ] `pnpm quality` y seguimiento real pasan; revisión del usuario.

## T03: Cubrir antigüedad, timeout, unknown y offline

**Status:** Blocked

**Description:** Presentar resultado desconocido/timeout y última vista local con antigüedad sin reenvío ciego.

**Acceptance criteria:**
- [ ] Timeout/unknown exige comprobar resultado antes de otra acción.
- [ ] Offline muestra última vista recibida y no simula estado actual.
- [ ] Fallo de comunicaciones no bloquea Agenda ni borra pendientes.

**Verification:**
- [ ] Focused tests: `pnpm build`; timeout/recuperación/offline disponible.
- [ ] Build or broader check: `pnpm quality`.
- [ ] Manual check: accesibilidad y NOT CHECKED registrados; revisión C02.

**Dependencies:** C01

**Files likely touched:**
- `src/features/communications/CommunicationDetail.tsx`
- `src/features/communications/presentation.ts`

**Estimated scope:** M

## C02: Después de T03

- [ ] `pnpm quality` y recuperación real pasan con evidencia disponible.
- [ ] Accesibilidad/offline/NOT CHECKED registrados; revisión del usuario.
