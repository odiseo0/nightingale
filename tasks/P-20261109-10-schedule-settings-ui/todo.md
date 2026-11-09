# Task List: P-20261109-10-schedule-settings-ui

**Plan:** [plan.md](plan.md)
**Updated:** 2026-11-09

## T01: Implementar listado y editor de reglas semanales

**Status:** Done (2026-11-09)

**Description:** Construir reglas semanales con día, inicio/fin, vigencia y estado usando list/create/update/setStatus existentes.

**Acceptance criteria:**
- [x] Reglas legibles ("Lunes 08:00–12:00, 2026-01-01 en adelante, Activa") editadas con controles nativos (select de día, time, date); los minutos 0–1439 solo existen en la frontera DTO.
- [x] Crear (verificado: regla miércoles 14:00–18:00 real y aceptada tras sync) y activar/desactivar (verificado: miércoles inactive aceptado) con profesional real y base correcta (`acceptedVersion`/`operationId`).
- [x] Errores de intervalo validados en formulario (fin ≤ inicio) y errores IPC con campos conservados.

**Verification:**
- [x] Focused tests: `pnpm build`; CRUD/estado con IPC real (3 reglas reales tras las mutaciones).
- [x] Build or broader check: `pnpm quality` OK.
- [x] Manual check: formularios nativos y captura conforme.

**Dependencies:** R08

**Files likely touched:**
- `src/features/scheduling/RulesEditor.tsx`
- `src/features/scheduling/ScheduleSettingsPage.tsx`
- `src/shared/api/scheduling.ts`
- `src/App.css`

**Estimated scope:** M

## T02: Implementar excepciones y bloqueos

**Status:** Done (2026-11-09)

**Description:** Añadir excepciones closed/replacement y bloqueos de cuenta/global con motivo y estado permitido.

**Acceptance criteria:**
- [x] Excepciones distinguen cierre/reemplazo (verificado: excepción "closed" del 25-12 real y aceptada); bloqueos distinguen cuenta/global, intervalo y motivo.
- [x] Sin acciones inventadas para asistentes: el global se ofrece solo con nota "el proceso local valida el permiso"; motivo restringido no se infiere.
- [x] Crear con DTO/base real y provisionalidad conservada; sin alterar citas (aviso persistente en página).
- [x] Bug semántico corregido en verificación: el contrato usa `accountId` ausente/null = global; el bloqueo de cuenta exige el accountId activo (ahora se envía desde `account_get_view`). Verificado: bloqueo de cuenta con motivo "administrative" aceptado y visible.

**Verification:**
- [x] Focused tests: `pnpm build`; excepciones/bloqueos reales (1 excepción, 2 bloqueos: 1 global de una prueba previa sin motivo —artefacto registrado— y 1 de cuenta con motivo).
- [x] Build or broader check: `pnpm quality` OK.

**Dependencies:** T01

**Files likely touched:**
- `src/features/scheduling/ExceptionsEditor.tsx`
- `src/features/scheduling/BlocksEditor.tsx`
- `src/features/scheduling/ScheduleSettingsPage.tsx`

**Estimated scope:** M

## C01: Después de T01–T02

- [ ] `pnpm quality` y operaciones reales de agenda pasan; revisión del usuario.

## T03: Cubrir impacto en citas, permisos globales, versiones y errores

**Status:** Done (2026-11-09)

**Description:** Presentar efecto en citas existentes y tratamiento de SLOT_UNAVAILABLE/versiones/dependencias con retorno contextual.

**Acceptance criteria:**
- [x] Aviso permanente: los cambios no mueven ni cancelan citas y el rechazo conserva el formulario; cita que quede fuera de horario/bloqueo → rechazo de Rust con formulario intacto (flujo de errores verificado en R02/P06 con SLOT_UNAVAILABLE; en horarios los comandos validan y rechazan con el mismo mapeo).
- [x] `SLOT_UNAVAILABLE`, `VERSION_CONFLICT`, `DEPENDENCY_FAILED` y permisos con siguiente paso seguro (`mutationErrorMessage` compartido; no se fuerzan cambios).
- [x] Recibo sin descarga no presenta vista aceptada (badges de operación hasta la descarga, verificado en los ciclos pendiente→sync); persistencia tras reinicio verificada de forma transversal (SQLite/Outbox).

**Verification:**
- [x] Focused tests: `pnpm build`; cambios compatibles reales aceptados (regla, excepción, bloqueo) y rechazo de intervalo inválido en formulario.
- [x] Build or broader check: `pnpm quality` OK.
- [ ] NOT CHECKED — rechazo real por cita incompatible (crear esa condición exigiría dejar una cita fuera de horario de forma deliberada en el fixture; el texto y el flujo de conservación están verificados en R02), lector de pantalla.

**Dependencies:** C01

**Files likely touched:**
- `src/features/scheduling/ScheduleSettingsPage.tsx`
- `src/features/scheduling/RulesEditor.tsx`
- `src/features/scheduling/BlocksEditor.tsx`
- `src/features/scheduling/presentation.ts`

**Estimated scope:** M

## C02: Horarios y bloqueos

- [x] `pnpm quality` en verde; cambios locales reales con aceptación separada y compatibilidad explicada.
- [ ] NOT CHECKED — rechazo real por cita incompatible, restricción de rol asistente (sin segundo usuario), lector de pantalla.
- [~] Revisión del usuario: autorización continua; aceptación final pendiente.
