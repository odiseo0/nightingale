# Task List: P-20261109-11-requests-ui

**Plan:** [plan.md](plan.md)
**Updated:** 2026-11-09

Plan Blocked: ninguna tarea se ejecuta hasta contrato Hub/API, conciliación y R02.

## T01: Implementar lista/detalle de solicitudes

**Status:** Blocked

**Description:** Construir S07 con solicitudes reales, cita vigente, recepción, antigüedad y última vista local.

**Acceptance criteria:**
- [ ] Lista/detalle muestra paciente, petición, cita/horario vigente y estado conocido sin modificar la cita.
- [ ] Offline/antigüedad/ausencia de datos se diferencia de lista vacía.
- [ ] Revisar horario enlaza a R02 sin aceptación automática.

**Verification:**
- [ ] Focused tests: `pnpm build` y consultas reales del contrato disponible.
- [ ] Build or broader check: `pnpm quality`.
- [ ] Manual check: revisión junto a cita, teclado y viewports.

**Dependencies:** Contrato Hub/API y conciliación definida; R02

**Files likely touched:**
- `src/features/requests/RequestsPage.tsx`
- `src/features/requests/RequestDetailPage.tsx`
- Cliente contractual que defina el plan técnico correspondiente

**Estimated scope:** M

## T02: Integrar elección de horario con R02

**Status:** Blocked

**Description:** Abrir reprogramación desde la solicitud y retornar con solicitud/cita/resultado conservados.

**Acceptance criteria:**
- [ ] Nuevo horario se elige mediante R02 sin cambiar la solicitud por sí solo.
- [ ] Error/conflicto conserva solicitud y propuesta.
- [ ] Aceptación de cita y actualización de solicitud se presentan por separado.

**Verification:**
- [ ] Focused tests: `pnpm build`; recorrido real de cambio y retorno.
- [ ] Build or broader check: `pnpm quality`.
- [ ] Manual check: memoria de contexto y foco; revisión C01.

**Dependencies:** T01

**Files likely touched:**
- `src/features/requests/RequestDetailPage.tsx`
- `src/features/appointments/ReschedulePage.tsx`

**Estimated scope:** M

## C01: Después de T01–T02

- [ ] `pnpm quality` y revisión/cambio reales pasan; revisión del usuario.

## T03: Implementar conciliación, fallos parciales y vistas offline

**Status:** Blocked

**Description:** Presentar conciliación Hub/API, resultado desconocido y fallos parciales con última vista conocida.

**Acceptance criteria:**
- [ ] Cambio aceptado/solicitud pendiente y cambio pendiente se diferencian textualmente.
- [ ] Unknown/rechazo no elimina intenciones ni provoca duplicados.
- [ ] Offline muestra antigüedad y no simula resolución.

**Verification:**
- [ ] Focused tests: `pnpm build`; fallo parcial y recuperación real disponible.
- [ ] Build or broader check: `pnpm quality`.
- [ ] Manual check: offline, accesibilidad y NOT CHECKED registrados; revisión C02.

**Dependencies:** C01

**Files likely touched:**
- `src/features/requests/RequestDetailPage.tsx`
- `src/features/requests/presentation.ts`

**Estimated scope:** M

## C02: Después de T03

- [ ] `pnpm quality` y conciliación real pasan con evidencia disponible.
- [ ] Accesibilidad/offline/NOT CHECKED registrados; revisión del usuario.
