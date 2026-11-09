# Task List: P-20261109-03-appointment-reschedule-ui

**Plan:** [plan.md](plan.md)
**Updated:** 2026-11-09

## T01: Añadir ruta y comparación actual/propuesto

**Status:** Done (2026-11-09)

**Description:** Implementar `/cita/:appointmentId/reprogramar` con detalle actual, contexto de solo lectura y nuevo formulario.

**Acceptance criteria:**
- [x] Solo citas programadas/confirmadas muestran Reprogramar (botón en detalle); estados terminales muestran aviso sin acción (verificado con IPC real).
- [x] Actual y propuesto muestran fecha/inicio/fin/duración/zona; paciente/servicio/profesional como contexto de solo lectura sin campos editables.
- [x] Carga/error con retorno al detalle; fecha civil calculada en la zona de la cuenta (`civilOfInstant`, no se corta el UTC).

**Verification:**
- [x] Focused tests: `pnpm build`; navegación desde detalle y lectura real verificadas.
- [x] Build or broader check: `pnpm quality` OK.
- [x] Manual check: comparación legible en captura; 800×600 sin overflow.

**Dependencies:** P06

**Files likely touched:**
- `src/App.tsx`
- `src/features/appointments/AppointmentDetail.tsx`
- `src/features/appointments/ReschedulePage.tsx`
- `src/app/AppLayout.tsx`

**Estimated scope:** M

## T02: Seleccionar nuevo intervalo con disponibilidad real

**Status:** Done (2026-11-09)

**Description:** Consultar disponibilidad para la misma duración y seleccionar una nueva fecha/intervalo con resumen comparativo.

**Acceptance criteria:**
- [x] `getLocalAvailability` recibe profesional/servicio/día y la duración real de la cita (no la de oferta) y no se calculan huecos en React (verificado: lunes 16/11 devuelve ventana real).
- [x] Cobertura/errores con reintento no eliminan el resto de la propuesta; cambiar fecha invalida solo el nuevo slot.
- [x] El resumen compara actual/nuevo con duración conservada y explica la invalidación de la confirmación y la no liberación del horario anterior antes de guardar.

**Verification:**
- [x] Focused tests: `pnpm build`; selección/cambio de fecha con coberturas reales.
- [x] Build or broader check: `pnpm quality` OK.
- [x] Manual check: comparación legible a 800×600; teclado heredado; 200 % con las mismas reglas CSS verificadas en P06.

**Dependencies:** T01

**Files likely touched:**
- `src/features/appointments/ReschedulePage.tsx`
- `src/features/appointments/AvailabilityPicker.tsx`
- `src/features/appointments/BookingSummary.tsx`
- `src/features/appointments/presentation.ts`

**Estimated scope:** M

## C01: Después de T01–T02

- [x] `pnpm quality` pasa; comparación, duración real, cobertura y formulario verificados con IPC real.
- [~] Revisión del usuario: autorización continua; C01 técnico.

## T03: Guardar reprogramación, tratar errores y actualizar detalle/historia

**Status:** Done (2026-11-09)

**Description:** Ejecutar `rescheduleAppointment`, conservar intención ante fallos y actualizar detalle/historia con retorno contextual.

**Acceptance criteria:**
- [x] Base correcta (`baseOf`: versión aceptada u operación local), ref de intención única (doble clic sin duplicar, verificado), progreso y propuesta conservada ante errores (mapeo seguro por código).
- [x] Resultado presentado como reprogramación local pendiente; verificado con IPC real: cita reprogramada al lunes 16/11 00:00–00:30 local, estado `scheduled` y `confirmedAt` nulo (confirmación anterior invalidada según DTO), provisional.
- [x] Historia aceptada/operaciones separadas en el detalle; retorno al detalle con selección en Agenda (sessionStorage) y mensaje persistente.

**Verification:**
- [x] Focused tests: `pnpm build`; reprogramación real verificada de extremo a extremo (detalle→reprogramar→guardar→detalle).
- [x] Build or broader check: `pnpm quality` OK.
- [x] Manual check: 800×600 sin overflow; teclado heredado; lector de pantalla NOT CHECKED; corte de red NOT CHECKED.

**Dependencies:** C01

**Files likely touched:**
- `src/features/appointments/ReschedulePage.tsx`
- `src/features/appointments/AppointmentDetail.tsx`
- `src/features/appointments/AgendaPage.tsx`
- `src/features/appointments/presentation.ts`

**Estimated scope:** M

## C02: Después de T03

- [x] `pnpm quality` y reprogramación real pasan con errores mapeados y retorno verificado.
- [ ] NOT CHECKED — lector de pantalla (no instalado); persistencia tras reinicio no re-ejecutada para esta superficie (el ciclo provisional→aceptado→descargado se verificó en P06/R01); corte físico de red.
- [~] Revisión del usuario: autorización continua; aceptación final pendiente (mismo criterio P05–R01).
