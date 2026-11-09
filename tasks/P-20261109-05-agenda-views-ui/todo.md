# Task List: P-20261109-05-agenda-views-ui

**Plan:** [plan.md](plan.md)
**Updated:** 2026-11-09

## T01: Implementar Semana con citas/bloqueos y detalle compartido

**Status:** Done (2026-11-09)

**Description:** Añadir vista Semana con escala horaria, eventos y bloqueos reales, acceso a detalle y alternativa de formulario.

**Acceptance criteria:**
- [x] Citas consultadas por periodo y dibujadas con hora local + paciente + estado en texto/badge; bloqueos distinguibles por etiqueta y doble borde (no solo color); verificado con IPC real (8 eventos del lunes con estados).
- [x] Scroll localizado (documento sin overflow a 800×600), día actual destacado, navegación semana anterior/actual/siguiente y acceso al detalle desde cada evento; día clicable para volver a la vista Día.
- [x] Sin drag-and-drop ni reserva desde celdas; aviso de que los huecos visibles no demuestran disponibilidad.

**Verification:**
- [x] Focused tests: `pnpm build`; semana real con citas y navegación de periodo.
- [x] Build or broader check: `pnpm quality` OK.
- [x] Manual check: 800×600 y 1280 sin overflow global; captura revisada; teclado heredado.

**Dependencies:** P05/P06

**Files likely touched:**
- `src/features/appointments/WeekPage.tsx`
- `src/features/appointments/AgendaPage.tsx`
- `src/features/appointments/presentation.ts`
- `src/App.tsx`, `src/App.css`

**Estimated scope:** M

## C01: Vista semanal

- [x] `pnpm quality` en verde; escala, scroll localizado, eventos y retorno verificados con IPC real; revisión del usuario: autorización continua.

## C02: Vistas de Agenda

- [x] `pnpm quality` en verde; Día/Semana/Lista con IPC real, tabs persistidas y detalle común.
- [ ] NOT CHECKED — lector de pantalla (no instalado); corte físico de red; 200 % no re-capturado (mismas reglas verificadas en P06).
- [~] Revisión del usuario: autorización continua; aceptación final pendiente.

## T02: Implementar Lista con búsqueda/periodo/filtros/paginación

**Status:** Done (2026-11-09)

**Description:** Añadir Lista de citas próximas/historial con búsqueda autorizada, periodo, filtros y paginación real.

**Acceptance criteria:**
- [x] Periodo (próximas 60 días / historial 366 días), profesional, servicio y estado funcionan con `listAppointments` (verificado: 9 citas, filtros aplicados al IPC); la búsqueda de texto filtra sobre resultados cargados (el contrato de citas no acepta `q` —limitación registrada).
- [x] Paginación Anterior/Siguiente con página/total y límite de 15 por página; totales reales del DTO, nada inventado.
- [x] Vacío/sin coincidencias/error distinguibles; cada fila abre el mismo S03 (ruta `/cita/:id`).

**Verification:**
- [x] Focused tests: `pnpm build`; búsqueda/periodo y apertura de detalle real.
- [x] Build or broader check: `pnpm quality` OK.
- [x] Manual check: nombres largos con wrapping; tabla con scroll localizado.

**Dependencies:** C01

**Files likely touched:**
- `src/features/appointments/AppointmentListPage.tsx`
- `src/features/appointments/AgendaPage.tsx`
- `src/features/appointments/presentation.ts`
- `src/App.css`

**Estimated scope:** M

## T03: Integrar tabs, contexto y comprobaciones responsive/accesibilidad

**Status:** Done (2026-11-09)

**Description:** Integrar Día/Semana/Lista con tabs semánticas y persistencia de fecha/vista/filtros/selección/foco entre detalle y retorno.

**Acceptance criteria:**
- [x] Tabs Día/Semana/Lista funcionales con `aria-selected` y persistencia de la vista en `agenda:estado` (verificado: al volver con atrás tras abrir un detalle desde Semana, la vista Semana se restaura); sin pestañas placeholder.
- [x] Cambiar de vista y volver del detalle restaura periodo/vista; respuestas obsoletas controladas por generación en Semana/Lista.
- [x] Viewports (1280/800×600 sin overflow global) verificados; 200 % con reglas CSS heredadas; lector de pantalla NOT CHECKED.

**Verification:**
- [x] Focused tests: `pnpm build`; cambio de vistas y atrás verificados con IPC real.
- [x] Build or broader check: `pnpm quality` OK.
- [x] Manual check: recorrido Semana→detalle→atrás con datos reales.

**Dependencies:** T02

**Files likely touched:**
- `src/features/appointments/AgendaPage.tsx`
- `src/features/appointments/WeekPage.tsx`
- `src/features/appointments/AppointmentListPage.tsx`
- `src/app/AppLayout.tsx`

**Estimated scope:** M

## C02: Después de T02–T03

- [x] `pnpm quality` en verde; Día/Semana/Lista con IPC real, tabs persistidas y detalle común.
- [ ] NOT CHECKED — lector de pantalla; corte físico de red; 200 % no re-capturado.
- [~] Revisión del usuario: autorización continua; aceptación final pendiente.
