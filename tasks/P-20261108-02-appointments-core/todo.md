# Task List: P-20261108-02-appointments-core

**Plan:** [plan.md](plan.md)  
**Status del plan:** Approved  
**Created:** 2026-11-08  
**Updated:** 2026-11-08

**Instrucción vigente:** el usuario aprobó el diseño y autorizó materializar estos archivos. Ninguna tarea ha comenzado. T01 se ejecutará después de `P-20261108-01-scheduling-availability/C06` y de una instrucción de inicio posterior.

**Migración reservada:** `0010_appointments`.  
**Responsable de integración:** quien ejecute el plan; una sola persona registra migraciones, composición, comandos, permisos, manifiestos y lockfile.  
**Orden:** secuencial, una tarea `In progress`; cada checkpoint requiere revisión del usuario antes de la fase siguiente.  
**Reglas comunes:** aplicar `minimal-code` antes de cada cambio y `docs-first-framework-code` para APIs de bibliotecas/frameworks; usar datos ficticios; preservar trabajo ajeno y no editar el contrato importado ni los planes anteriores.

## T01: Congelar contratos de Citas

**Status:** Todo

**Description:** Fijar DTO, seis acciones, historia, snapshots, puertos booking, exclusión de reprogramación, tablas `0010`, comandos, dependencias y puntos de integración antes de escribir dominio.

**Acceptance criteria:**
- [ ] Los contratos de las seis acciones, cita e historia coinciden con `api-v1.1`.
- [ ] Cobertura, datos provisionales, puertos booking, exclusión y resolución recibo+cita+historia quedan definidos.
- [ ] Migración, diez comandos, propietarios e integrador quedan reservados y las cinco huellas del bundle intactas.

**Verification:**
- [ ] Focused checks: cotejo estructural de payloads, modelos remotos, capabilities y OpenAPI.
- [ ] Build or broader check: `git diff --check`.
- [ ] Manual check: revisar create→confirm→reschedule→cancel y una historia consecutiva contra los contratos.

**Dependencies:** `P-20261108-01-scheduling-availability/C06` y autorización expresa de inicio.

**Files likely touched:**
- `tasks/P-20261108-02-appointments-core/plan.md`
- `tasks/P-20261108-02-appointments-core/todo.md`

**Estimated scope:** M

## T02: Dominio de cita y tiempo

**Status:** Todo

**Description:** Implementar estados, transiciones, intervalos de minuto, snapshots, edad a fecha local y errores estables sin almacenamiento ni framework.

**Acceptance criteria:**
- [ ] Las seis transiciones y los estados terminales respetan el contrato.
- [ ] Instantes, precisión de minuto y duración real 1–1440 se validan exactamente.
- [ ] Menor de 18, 29 de febrero y responsables se evalúan a la fecha local de la cita.

**Verification:**
- [ ] Focused tests: `cargo test --workspace --locked components::appointments::domain`.
- [ ] Build or broader check: `cargo check --workspace --all-targets --locked` y `git diff --check`.
- [ ] Manual check: ejecutar matriz pura de estados, tiempos, edad y responsables.

**Dependencies:** T01.

**Files likely touched:**
- `src-tauri/src/components/appointments/domain.rs`
- `src-tauri/src/components/appointments/mod.rs`

**Estimated scope:** M

## T03: Persistencia de Citas

**Status:** Todo

**Description:** Crear `0010_appointments`, vistas de cita/historia, intenciones e índices, conservando `appointment_occupancy_view`.

**Acceptance criteria:**
- [ ] La migración 9→10 conserva datos y reabre sin repetir cambios.
- [ ] Un fallo revierte archivo, historial y versión completos.
- [ ] Las filas aíslan cuenta/usuario/dispositivo y no hay cascadas hacia intenciones.

**Verification:**
- [ ] Focused tests: `cargo test --workspace --locked components::appointments` y `cargo test --workspace --locked platform::sqlite::tests`.
- [ ] Build or broader check: `cargo check --workspace --all-targets --locked` y `git diff --check`.
- [ ] Manual check: migrar una base temporal desde 9, reabrirla e inspeccionar vistas, historia e intenciones.

**Dependencies:** T02.

**Files likely touched:**
- `src-tauri/migrations/0010_appointments.sql`
- `src-tauri/src/platform/sqlite/mod.rs`
- `src-tauri/src/components/appointments/adapters/mod.rs`
- `src-tauri/src/components/appointments/adapters/outbound.rs`
- `src-tauri/src/components/appointments/repository_tests.rs`

**Estimated scope:** M

## T04: Puerto PatientBookingQuery

**Status:** Todo

**Description:** Añadir una lectura pública de ficha, documentos y responsables aceptados o provisionales utilizables con sus referencias, sin exponer el repositorio de Patients.

**Acceptance criteria:**
- [ ] La lectura aporta completitud, documentos, teléfono y responsables con vigencia.
- [ ] Las referencias aceptadas/provisionales incluyen versión u operación y excluyen estados no encadenables.
- [ ] El puerto mantiene aislamiento y no expone SQL, HTTP ni almacenamiento.

**Verification:**
- [ ] Focused tests: `cargo test --workspace --locked components::patients`.
- [ ] Build or broader check: `cargo check --workspace --all-targets --locked` y `git diff --check`.
- [ ] Manual check: resolver adulto, menor y responsable provisional con operaciones encadenables y rechazadas.

**Dependencies:** T03.

**Files likely touched:**
- `src-tauri/src/components/patients/application.rs`
- `src-tauri/src/components/patients/adapters/outbound.rs`
- `src-tauri/src/components/patients/adapters/outbound_contacts.rs`
- `src-tauri/src/components/patients/booking_tests.rs`

**Estimated scope:** M

## T05: Puerto CatalogBookingQuery

**Status:** Todo

**Description:** Añadir una lectura pública de servicio, oferta y profesional aceptados o provisionales con snapshots y referencias, sin cambiar `CatalogQuery`.

**Acceptance criteria:**
- [ ] Nombre y duración efectiva de referencia se calculan desde el estado elegido.
- [ ] Estados y referencias provisionales solo se ofrecen cuando pueden encadenarse.
- [ ] No se leen tablas ajenas ni se alteran las consultas aceptadas existentes.

**Verification:**
- [ ] Focused tests: `cargo test --workspace --locked components::services`.
- [ ] Build or broader check: `cargo check --workspace --all-targets --locked` y `git diff --check`.
- [ ] Manual check: crear servicio/oferta offline y resolverlos para una reserva antes de aceptación.

**Dependencies:** T04.

**Files likely touched:**
- `src-tauri/src/components/services/application.rs`
- `src-tauri/src/components/services/adapters/outbound.rs`
- `src-tauri/src/components/services/booking_tests.rs`

**Estimated scope:** M

## T06: Exclusión y ocupación provisional

**Status:** Todo

**Description:** Ampliar disponibilidad con `excludeAppointmentId` y la ocupación local de Appointments para bloquear provisionales sin autobloqueo al reprogramar.

**Acceptance criteria:**
- [ ] La exclusión retira únicamente la cita indicada del cálculo de ocupación.
- [ ] `scheduled`, `confirmed`, `completed` y `no_show` ocupan; `cancelled` libera también provisionalmente.
- [ ] La cobertura de Agenda permanece intacta y no se crea una segunda disponibilidad persistida.

**Verification:**
- [ ] Focused tests: `cargo test --workspace --locked components::scheduling` y `cargo test --workspace --locked components::appointments`.
- [ ] Build or broader check: `cargo check --workspace --all-targets --locked` y `git diff --check`.
- [ ] Manual check: reprogramar al mismo intervalo y comprobar conflicto con otra cita provisional.

**Dependencies:** T05.

**Files likely touched:**
- `src-tauri/src/components/scheduling/application.rs`
- `src-tauri/src/components/scheduling/availability.rs`
- `src-tauri/src/components/scheduling/availability_query_tests.rs`
- `src-tauri/src/components/appointments/adapters/outbound.rs`
- `src-tauri/src/components/appointments/occupancy_tests.rs`

**Estimated scope:** M

## C01: Base contractual y de datos

- [ ] Dominio, migración y puertos booking/disponibilidad pasan junto con `cargo test --workspace --locked` y `pnpm build:desktop`.
- [ ] El usuario revisa snapshots, provisionales, exclusión y cobertura.
- [ ] No hay escrituras de Citas ni cambios en contratos o migraciones anteriores.
- [ ] `git diff --check` pasa.

## T07: Secuencias y dependencias de operaciones

**Status:** Todo

**Description:** Implementar preparación común con base aceptada/local, versiones previstas, provisionales y Outbox atómico.

**Acceptance criteria:**
- [ ] Provisional y Outbox se confirman o revierten juntos.
- [ ] Las secuencias respetan ocho dependencias, sin duplicados, ciclos ni truncamiento.
- [ ] Bases antiguas o bloqueadas no crean operación y el trabajo previo se conserva.

**Verification:**
- [ ] Focused tests: `cargo test --workspace --locked components::appointments`.
- [ ] Build or broader check: `cargo check --workspace --all-targets --locked` y `git diff --check`.
- [ ] Manual check: encadenar operaciones offline, reabrir e inspeccionar bases y dependencias.

**Dependencies:** T06 y aprobación de C01.

**Files likely touched:**
- `src-tauri/src/components/appointments/application.rs`
- `src-tauri/src/components/appointments/adapters/intents.rs`
- `src-tauri/src/components/appointments/adapters/outbound.rs`
- `src-tauri/src/components/appointments/application_tests.rs`

**Estimated scope:** M

## T08: Crear cita

**Status:** Todo

**Description:** Implementar `CreateAppointment` con validación booking, snapshots, disponibilidad, dependencias y versión cero.

**Acceptance criteria:**
- [ ] Adulto, menor y responsable cumplen las reglas a la fecha local de la cita.
- [ ] Cita provisional, snapshots y Outbox se guardan atómicamente con `expectedVersion=0`.
- [ ] Cobertura insuficiente o solapamiento conocido bloquea sin efectos parciales.

**Verification:**
- [ ] Focused tests: `cargo test --workspace --locked components::appointments`.
- [ ] Build or broader check: `cargo check --workspace --all-targets --locked` y `git diff --check`.
- [ ] Manual check: crear con paciente/servicio/agenda provisionales y reabrir antes de sincronizar.

**Dependencies:** T07.

**Files likely touched:**
- `src-tauri/src/components/appointments/domain.rs`
- `src-tauri/src/components/appointments/application.rs`
- `src-tauri/src/components/appointments/adapters/outbound.rs`
- `src-tauri/src/components/appointments/create_tests.rs`

**Estimated scope:** M

## T09: Confirmar cita

**Status:** Todo

**Description:** Implementar `ConfirmAppointment` desde estado programado aceptado o provisional.

**Acceptance criteria:**
- [ ] El payload es `{}` y la base/dependencia pertenece a la misma cita.
- [ ] Solo `scheduled` pasa a `confirmed`; otra transición devuelve `INVALID_STATE_TRANSITION`.
- [ ] No se fabrica `confirmedAt` local ni una segunda confirmación.

**Verification:**
- [ ] Focused tests: `cargo test --workspace --locked components::appointments`.
- [ ] Build or broader check: `cargo check --workspace --all-targets --locked` y `git diff --check`.
- [ ] Manual check: ejecutar create→confirm offline, reabrir y revisar estado provisional.

**Dependencies:** T08.

**Files likely touched:**
- `src-tauri/src/components/appointments/application.rs`
- `src-tauri/src/components/appointments/adapters/outbound.rs`
- `src-tauri/src/components/appointments/confirm_tests.rs`

**Estimated scope:** M

## T10: Reprogramar cita

**Status:** Todo

**Description:** Implementar `RescheduleAppointment` con igual duración real, exclusión propia, snapshots y confirmación invalidada provisionalmente.

**Acceptance criteria:**
- [ ] El nuevo intervalo conserva duración real 1–1440 y referencias/snapshots.
- [ ] Disponibilidad excluye la cita actual y detecta otras ocupaciones/bloqueos.
- [ ] `scheduled` o `confirmed` vuelve a `scheduled` y `confirmedAt` queda nulo provisionalmente.

**Verification:**
- [ ] Focused tests: `cargo test --workspace --locked components::appointments`.
- [ ] Build or broader check: `cargo check --workspace --all-targets --locked` y `git diff --check`.
- [ ] Manual check: reprogramar estados programado/confirmado y conservar un conflicto conocido.

**Dependencies:** T09.

**Files likely touched:**
- `src-tauri/src/components/appointments/application.rs`
- `src-tauri/src/components/appointments/adapters/outbound.rs`
- `src-tauri/src/components/appointments/reschedule_tests.rs`

**Estimated scope:** M

## T11: Cancelar cita

**Status:** Todo

**Description:** Implementar `CancelAppointment` con motivo opcional, historia local separada y liberación de ocupación.

**Acceptance criteria:**
- [ ] Solo `scheduled` o `confirmed` puede cancelarse y el motivo presente no está vacío.
- [ ] La cancelación libera ocupación provisional sin borrar operaciones anteriores.
- [ ] La historia local permanece separada de la aceptada y sobrevive a reapertura.

**Verification:**
- [ ] Focused tests: `cargo test --workspace --locked components::appointments`.
- [ ] Build or broader check: `cargo check --workspace --all-targets --locked` y `git diff --check`.
- [ ] Manual check: cancelar y consultar disponibilidad e historia tras reapertura.

**Dependencies:** T10.

**Files likely touched:**
- `src-tauri/src/components/appointments/application.rs`
- `src-tauri/src/components/appointments/adapters/outbound.rs`
- `src-tauri/src/components/appointments/cancel_tests.rs`

**Estimated scope:** M

## T12: Registrar atención

**Status:** Todo

**Description:** Implementar `CompleteAppointment` con reloj inyectado y motivo opcional.

**Acceptance criteria:**
- [ ] Solo se completa cuando `clock.now >= startsAt` desde `scheduled` o `confirmed`.
- [ ] El estado terminal y motivo se conservan sin alterar ocupación ni historia anterior.
- [ ] Una llamada antes del inicio no crea operación ni modifica la cita.

**Verification:**
- [ ] Focused tests: `cargo test --workspace --locked components::appointments`.
- [ ] Build or broader check: `cargo check --workspace --all-targets --locked` y `git diff --check`.
- [ ] Manual check: ejercitar antes/en el inicio y reabrir el resultado terminal.

**Dependencies:** T11.

**Files likely touched:**
- `src-tauri/src/components/appointments/application.rs`
- `src-tauri/src/components/appointments/adapters/outbound.rs`
- `src-tauri/src/components/appointments/complete_tests.rs`

**Estimated scope:** M

## T13: Marcar ausencia

**Status:** Todo

**Description:** Implementar `MarkAppointmentNoShow` con inicio+margen aceptado vigente y acción staff.

**Acceptance criteria:**
- [ ] Solo se marca cuando `clock.now >= startsAt + noShowGraceMinutes` vigente.
- [ ] La ausencia nunca es automática por falta de respuesta y requiere acción del personal.
- [ ] El estado terminal conserva ocupación, motivo e historia.

**Verification:**
- [ ] Focused tests: `cargo test --workspace --locked components::appointments`.
- [ ] Build or broader check: `cargo check --workspace --all-targets --locked` y `git diff --check`.
- [ ] Manual check: ejercitar antes/después del margen y un cambio de política aceptada.

**Dependencies:** T12.

**Files likely touched:**
- `src-tauri/src/components/appointments/application.rs`
- `src-tauri/src/components/appointments/adapters/outbound.rs`
- `src-tauri/src/components/appointments/no_show_tests.rs`

**Estimated scope:** M

## C02: Ciclo de vida nativo

- [ ] Las seis acciones y secuencias offline/reapertura pasan con referencias provisionales y aceptadas.
- [ ] `cargo test --workspace --locked` y `pnpm build:desktop` pasan.
- [ ] El usuario revisa estados, tiempos y disponibilidad antes de sincronización.
- [ ] `git diff --check` pasa.

## T14: Proyector de cita

**Status:** Todo

**Description:** Implementar traducción y validación de `appointment`, ocupación y `remove/reset` sin tocar intenciones.

**Acceptance criteria:**
- [ ] Versiones, ámbito, estados y nueve snapshots proyectan sin retrocesos.
- [ ] `remove/reset` retira cita y ocupación aceptadas conservando intenciones y otras cuentas.
- [ ] Un fallo revierte grupo, proyección, cursor e Inbox.

**Verification:**
- [ ] Focused tests: `cargo test --workspace --locked components::appointments`.
- [ ] Build or broader check: `cargo check --workspace --all-targets --locked` y `git diff --check`.
- [ ] Manual check: aplicar upsert repetido, retirar y reconstruir una captura.

**Dependencies:** T13 y aprobación de C02.

**Files likely touched:**
- `src-tauri/src/components/appointments/adapters/projector.rs`
- `src-tauri/src/components/appointments/adapters/outbound.rs`
- `src-tauri/src/components/appointments/projection_tests.rs`
- `src-tauri/src/components/projector.rs`

**Estimated scope:** M

## T15: Proyector de historia

**Status:** Todo

**Description:** Implementar proyección agregada `appointment_history` con versiones consecutivas y estado actual coherente.

**Acceptance criteria:**
- [ ] La historia comienza en 1, no tiene huecos ni duplicados y termina en la versión agregada.
- [ ] Snapshots y entradas anteriores permanecen inmutables y cita/historia son coherentes.
- [ ] Estados inválidos revierten la transacción sin borrar historia válida anterior.

**Verification:**
- [ ] Focused tests: `cargo test --workspace --locked components::appointments`.
- [ ] Build or broader check: `cargo check --workspace --all-targets --locked` y `git diff --check`.
- [ ] Manual check: proyectar historia multipágina y llegada separada de cita.

**Dependencies:** T14.

**Files likely touched:**
- `src-tauri/src/components/appointments/adapters/projector.rs`
- `src-tauri/src/components/appointments/adapters/outbound.rs`
- `src-tauri/src/components/appointments/history_projection_tests.rs`

**Estimated scope:** M

## T16: Conciliación de provisionales

**Status:** Todo

**Description:** Implementar lecturas fusionadas y resolución solo por recibo+cita+historia, conservando conflictos y cancelaciones remotas.

**Acceptance criteria:**
- [ ] Un recibo aceptado sin cita e historia proyectadas no resuelve el provisional.
- [ ] Cuando ambas vistas alcanzan la versión confirmada se retira solo el provisional correspondiente.
- [ ] Una intención antigua nunca sobrescribe un estado remoto nuevo o una cancelación remota.

**Verification:**
- [ ] Focused tests: `cargo test --workspace --locked components::appointments` y `cargo test --workspace --locked components::sync`.
- [ ] Build or broader check: `cargo check --workspace --all-targets --locked` y `git diff --check`.
- [ ] Manual check: simular aceptación parcial, reapertura, `remove` y cancelación remota.

**Dependencies:** T15.

**Files likely touched:**
- `src-tauri/src/components/appointments/adapters/intents.rs`
- `src-tauri/src/components/appointments/adapters/outbound.rs`
- `src-tauri/src/components/appointments/application.rs`
- `src-tauri/src/components/appointments/reconciliation_tests.rs`

**Estimated scope:** M

## C03: Sincronización e historia

- [ ] Cita, historia y ocupación se proyectan y reconcilian sin duplicados ni pérdida de intenciones.
- [ ] `cargo test --workspace --locked` y `pnpm build:desktop` pasan.
- [ ] El usuario revisa historia, cancelación remota y retiradas de ventana.
- [ ] `git diff --check` pasa.

## T17: Comandos de escritura

**Status:** Todo

**Description:** Añadir seis comandos Tauri con DTO específicos, bases, permisos y errores seguros.

**Acceptance criteria:**
- [ ] Acciones y payloads son exactos y no admiten ámbito, credenciales o envelope arbitrario.
- [ ] Campos extra, bases ambiguas y transiciones inválidas se rechazan sin efectos.
- [ ] Propietario y asistente se autorizan por acción efectiva y actor staff.

**Verification:**
- [ ] Focused tests: `cargo test --workspace --locked components::appointments::ipc`.
- [ ] Build or broader check: `cargo check --workspace --all-targets --locked` y `git diff --check`.
- [ ] Manual check: invocar comandos autorizados/denegados por el camino real de entrada.

**Dependencies:** T16 y aprobación de C03.

**Files likely touched:**
- `src-tauri/src/components/appointments/adapters/inbound_commands.rs`
- `src-tauri/src/components/appointments/adapters/mod.rs`
- `src-tauri/src/components/appointments/ipc_write_tests.rs`

**Estimated scope:** M

## T18: Comandos de consulta

**Status:** Todo

**Description:** Añadir listado, detalle, historia y contexto de reserva con aceptado/provisional separado.

**Acceptance criteria:**
- [ ] Filtros, intersección de rangos, paginación y `afterVersion` responden según contrato.
- [ ] Historia aceptada y operaciones locales permanecen separadas.
- [ ] Las salidas no exponen token, época, fingerprint, SQL ni estado remoto crudo.

**Verification:**
- [ ] Focused tests: `cargo test --workspace --locked components::appointments::ipc`.
- [ ] Build or broader check: `cargo check --workspace --all-targets --locked` y `git diff --check`.
- [ ] Manual check: invocar sin sesión, otro ámbito y consultas con cobertura insuficiente.

**Dependencies:** T17.

**Files likely touched:**
- `src-tauri/src/components/appointments/adapters/inbound_queries.rs`
- `src-tauri/src/components/appointments/adapters/mod.rs`
- `src-tauri/src/components/appointments/ipc_query_tests.rs`

**Estimated scope:** M

## T19: Registro y cliente TypeScript

**Status:** Todo

**Description:** Registrar diez comandos/permisos y crear `src/shared/api/appointments.ts`.

**Acceptance criteria:**
- [ ] Handler, manifiesto y capabilities coinciden para los diez comandos.
- [ ] Los DTO TypeScript cubren reserva, acciones, agenda, historia y contexto.
- [ ] La compilación pasa sin nuevas dependencias JavaScript.

**Verification:**
- [ ] Focused tests: prueba de coherencia de registro Appointments.
- [ ] Build or broader check: `pnpm build` y `pnpm build:desktop`.
- [ ] Manual check: abrir el ejecutable y verificar invocación autorizada y denegada mediante el arnés previsto.

**Dependencies:** T18.

**Files likely touched:**
- `src-tauri/src/lib.rs`
- `src-tauri/build.rs`
- `src-tauri/capabilities/default.json`
- `src/shared/api/appointments.ts`
- `src-tauri/src/components/appointments/ipc_registration_tests.rs`

**Estimated scope:** M

## C04: Interfaz nativa

- [ ] Diez comandos, DTO y permisos pasan con sesiones válidas y denegadas.
- [ ] `pnpm quality` y `pnpm build:desktop` pasan.
- [ ] El usuario revisa IPC antes de UI.
- [ ] `git diff --check` pasa.

## T20: Reserva y acciones

**Status:** Todo

**Description:** Crear formulario de cita y acciones por estado con contexto booking, disponibilidad y motivos.

**Acceptance criteria:**
- [ ] Responsable, edad, intervalo y cobertura se presentan antes de guardar.
- [ ] Provisionales y estados de sincronización no se presentan como aceptación remota.
- [ ] React no decide reglas críticas, versiones ni permisos.

**Verification:**
- [ ] Focused checks: `pnpm build`.
- [ ] Build or broader check: `pnpm build:desktop` y `git diff --check`.
- [ ] Manual check: recorrido Windows create→confirm→reschedule→cancel con datos sintéticos.

**Dependencies:** T19 y aprobación de C04.

**Files likely touched:**
- `src/features/appointments/BookingPanel.tsx`
- `src/features/appointments/AppointmentActions.tsx`
- `src/shared/api/appointments.ts`
- `src/App.tsx`

**Estimated scope:** M

## T21: Agenda, detalle e historia

**Status:** Todo

**Description:** Crear hoy/mañana por zona de cuenta, rango, detalle, snapshots e historia aceptada/local.

**Acceptance criteria:**
- [ ] Intersección de rangos, hoy/mañana y estados se muestran correctamente por zona de cuenta.
- [ ] Historia aceptada y operaciones locales permanecen separadas, incluida historia ausente.
- [ ] Conflictos y revisiones son visibles sin sobrescribir estado remoto.

**Verification:**
- [ ] Focused checks: `pnpm build`.
- [ ] Build or broader check: `pnpm build:desktop` y `git diff --check`.
- [ ] Manual check: recorrer offline/reapertura, detalle, historia y conflicto remoto.

**Dependencies:** T20.

**Files likely touched:**
- `src/features/appointments/AgendaPanel.tsx`
- `src/features/appointments/AppointmentDetail.tsx`
- `src/shared/api/appointments.ts`
- `src/App.tsx`

**Estimated scope:** M

## C05: Vertical usable

- [ ] Reserva, acciones, agenda e historia funcionan con datos sintéticos y estados claros.
- [ ] `pnpm quality` y `pnpm build:desktop` pasan; la revisión manual Windows queda registrada.
- [ ] El usuario revisa la vertical antes de pruebas finales.
- [ ] `git diff --check` pasa.

## T22: Fixtures del consumidor

**Status:** Todo

**Description:** Contrastar seis acciones, cita, historia y estados de error con el bundle fijo.

**Acceptance criteria:**
- [ ] PayloadModels, claves y disponibilidad offline coinciden con capabilities.
- [ ] Cita, historia y proyección agregada no presentan deriva estructural.
- [ ] Las cinco huellas del bundle permanecen intactas.

**Verification:**
- [ ] Focused tests: `cargo test --workspace --locked components::sync::contract_tests`.
- [ ] Build or broader check: `cargo check --workspace --all-targets --locked` y `git diff --check`.
- [ ] Manual check: revisar una fixture por acción y una historia consecutiva contra OpenAPI.

**Dependencies:** T21 y aprobación de C05.

**Files likely touched:**
- `src-tauri/src/components/sync/contract_tests.rs`
- `src-tauri/src/components/sync/fixtures/appointments-operations.json`
- `src-tauri/src/components/sync/fixtures/appointments-history.json`

**Estimated scope:** S

## T23: Integración loopback de dos perfiles

**Status:** Todo

**Description:** Recorrer dependencias offline→reapertura→dispatch→accepted→cita+historia en dos SQLite.

**Acceptance criteria:**
- [ ] Una reserva concurrente produce un solo efecto y el otro dispositivo conserva su conflicto.
- [ ] Respuesta perdida, aceptación parcial y captura no duplican efectos ni borran intenciones.
- [ ] API/SSE caídos no bloquean casos locales y HTTP nunca mantiene SQLite bloqueada.

**Verification:**
- [ ] Focused tests: `cargo test --workspace --locked appointments_integration_tests`.
- [ ] Build or broader check: `cargo test --workspace --locked` y `git diff --check`.
- [ ] Manual check: recorrer la matriz de dos perfiles, conflicto, recuperación e historia.

**Dependencies:** T22.

**Files likely touched:**
- `src-tauri/src/components/sync/appointments_integration_tests.rs`
- `src-tauri/src/components/sync/mod.rs`
- `src-tauri/src/components/appointments/test_support.rs`
- `src-tauri/src/components/appointments/mod.rs`

**Estimated scope:** M

## T24: Guías y calidad

**Status:** Todo

**Description:** Documentar ciclo de vida, snapshots, booking, cobertura, historia, comandos y límites sin editar especificaciones.

**Acceptance criteria:**
- [ ] Las guías coinciden con implementación y separan lo verificado de lo pendiente.
- [ ] Calidad, build y diff pasan sobre la entrega completa.
- [ ] No quedan artefactos de prueba, bases, secretos ni logs en Git.

**Verification:**
- [ ] Focused tests: `cargo test --workspace --locked components::appointments`, `cargo test --workspace --locked components::sync::contract_tests` y `cargo test --workspace --locked appointments_integration_tests`.
- [ ] Build or broader check: `pnpm quality`, `pnpm build:desktop`, `git diff --check` y `git status`.
- [ ] Manual check: reproducir recorridos documentados y revisar enlaces y límites.

**Dependencies:** T23.

**Files likely touched:**
- `docs/technical/components/appointments.md`
- `docs/operations/core-next-tracks.md`
- `docs/operations/development.md`
- `docs/README.md`
- `tasks/P-20261108-02-appointments-core/todo.md`

**Estimated scope:** M

## T25: API/OIDC real de desarrollo

**Status:** Todo

**Description:** Ejecutar prueba opt-in con dos dispositivos, referencias sintéticas y ciclo completo posible sin esperas artificiales.

**Acceptance criteria:**
- [ ] Las seis acciones y descarga de cita+historia se verifican con adaptadores reales.
- [ ] Reserva concurrente, respuesta perdida y conflicto no duplican efectos ni pierden intenciones.
- [ ] Comandos, datos sintéticos y límites quedan registrados sin secretos; un bloqueo no cuenta como éxito.

**Verification:**
- [ ] Focused tests: `cargo test --workspace --locked -- --ignored components::appointments::development_tests` con el entorno documentado en T24.
- [ ] Build or broader check: `cargo test --workspace --locked components::appointments` y `git diff --check`.
- [ ] Manual check: completar login OIDC y revisar agenda/historia tras reapertura en ambos dispositivos.

**Dependencies:** T24.

**Files likely touched:**
- `src-tauri/src/components/appointments/development_tests.rs`
- `src-tauri/src/components/appointments/mod.rs`
- `tasks/P-20261108-02-appointments-core/todo.md`
- `tasks/P-20261108-02-appointments-core/evidence-c06.md`

**Estimated scope:** M

## C06: Aceptación de Citas

- [ ] Suite, contrato, integración loopback, calidad y build de escritorio pasan.
- [ ] API/OIDC real y recorrido UI tienen evidencia; todo límite no ejecutado permanece explícito.
- [ ] El usuario revisa y acepta el cierre. Si T25 no pasa, el plan permanece abierto.
- [ ] `git diff --check` y `git status` se revisan antes del cierre.
