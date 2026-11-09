# Task List: P-20261109-04-appointment-lifecycle-ui

**Plan:** [plan.md](plan.md)
**Updated:** 2026-11-09

## T01: Implementar Confirmar y Cancelar con consecuencia explícita

**Status:** Done (2026-11-09)

**Description:** Añadir acciones Confirmar/Cancelar a S03 según estado y permiso, con diálogo de cancelación y mutaciones locales reales. (Componente nuevo `LifecycleActions.tsx`; el panel técnico `AppointmentActions.tsx` de /desarrollo no se modifica.)

**Acceptance criteria:**
- [x] Confirmar solo para programada y cancelar para programada/confirmada, con base correcta (`baseOf`) y progreso sin duplicación (ref de intención única; verificado con IPC real: doble clic sin duplicar).
- [x] Cancelar muestra paciente, fecha/hora, conservación de historia y liberación solo tras aceptación, con alternativa "Conservar cita" (verificado: diálogo, conservar cierra, reintento y confirmación con motivo opcional → Cancelada provisional).
- [x] Errores con texto seguro y resultado provisional ("guardado en este equipo · Pendiente de aceptación"), sin afirmar aceptación ni mensajes.

**Verification:**
- [x] Focused tests: `pnpm build`; confirmar/cancelar con IPC real (verificado en app Tauri reiniciada limpiamente).
- [x] Build or broader check: `pnpm quality` OK.
- [x] Manual check: diálogo, 800×600 sin overflow, doble clic controlado y retorno con aviso persistente en el detalle.

**Dependencies:** P06

**Files likely touched:**
- `src/features/appointments/AppointmentActions.tsx`
- `src/features/appointments/AppointmentDetail.tsx`
- `src/features/appointments/presentation.ts`
- `src/App.css`

**Estimated scope:** M

## C01: Acciones iniciales

- [x] `pnpm quality` en verde; confirmar/cancelar transiciones reales verificadas; revisión del usuario: autorización continua.

## C02: Ciclo de cita

- [x] `pnpm quality` en verde; acciones reales (confirmar, cancelar, atención, ausencia con momento), estados terminales y retorno verificados.
- [ ] NOT CHECKED — lector de pantalla (no instalado); corte físico de red; algunos códigos de error no disparables con fixtures.
- [~] Revisión del usuario: autorización continua; aceptación final pendiente (mismo criterio P05–R02).

## T02: Implementar Registrar atención y Registrar ausencia

**Status:** Done (2026-11-09)

**Description:** Añadir atención/ausencia con explicación de momento/margen y motivo opcional según contrato.

**Acceptance criteria:**
- [x] Atención se ofrece deshabilitada hasta alcanzar el inicio y ausencia hasta inicio+margen de la política (`noShowGraceMinutes` del contexto), con explicación visible (§25.2); verificado: atención habilitada en cita iniciada y ausencia deshabilitada en cita futura.
- [x] Ninguna acción se deriva de falta de respuesta; diálogo de ausencia recuerda que es explícita del personal; sin estados terminales/reapertura (mensaje de terminal sin acciones, verificado tras atención).
- [x] Errores de transición/permiso con texto seguro y motivo conservado en el formulario.

**Verification:**
- [x] Focused tests: `pnpm build`; atención real y rechazo por momento (deshabilitada) verificados con IPC.
- [x] Build or broader check: `pnpm quality` OK.
- [x] Manual check: confirmación con consecuencia; lector de pantalla NOT CHECKED.

**Dependencies:** C01

**Files likely touched:**
- `src/features/appointments/AppointmentActions.tsx`
- `src/features/appointments/AppointmentDetail.tsx`
- `src/features/appointments/presentation.ts`

**Estimated scope:** M

## T03: Actualizar historia/Agenda y verificar estados/errores

**Status:** Done (2026-11-09)

**Description:** Refrescar detalle, historia y Agenda tras mutaciones, manteniendo separación aceptado/provisional/operaciones y contexto.

**Acceptance criteria:**
- [x] Estados terminales y versiones según DTO: tras cada acción el detalle se recarga (getAppointment+getHistory) y muestra el nuevo estado provisional sin convertir recibo sin descarga en vista aceptada (badges separados verificados en capturas anteriores).
- [x] Agenda conserva fecha/filtros/selección/foco (patrón sessionStorage verificado en P05) y muestra los estados actualizados tras refresh (verificado: Confirmada/Cancelada/Atendida reflejadas).
- [ ] Errores no disparables con fixtures quedan NOT CHECKED (VERSION_CONFLICT en acciones, PERMISSION_DENIED); no se simulan.

**Verification:**
- [x] Focused tests: `pnpm build`; actualización tras cada acción verificada en el flujo automatizado.
- [x] Build or broader check: `pnpm quality` OK.
- [x] Manual check: 800×600 sin overflow en agenda/detalle; 200 % con reglas CSS heredadas verificadas; lector de pantalla y corte de red NOT CHECKED.

**Dependencies:** T02

**Files likely touched:**
- `src/features/appointments/AppointmentDetail.tsx`
- `src/features/appointments/AgendaPage.tsx`
- `src/features/appointments/presentation.ts`

**Estimated scope:** M

## C02: Después de T02–T03

- [x] `pnpm quality` en verde y ciclo de acciones reales verificado.
- [ ] NOT CHECKED — lector de pantalla (no instalado); corte físico de red; códigos de error no disparables con fixtures.
- [~] Revisión del usuario: autorización continua; aceptación final pendiente.
