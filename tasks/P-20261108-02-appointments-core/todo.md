# Task List: P-20261108-02-appointments-core

**Plan:** [plan.md](plan.md)  
**Status del plan:** Approved  
**Created:** 2026-11-08  
**Updated:** 2026-11-08

**Instrucción vigente:** el usuario aprobó C01 el 2026-11-08, consolidó T01–T06 en el commit `43ec89e` (con un pase de formato y comentarios propio) y autorizó continuar con el desarrollo. Convención registrada por el usuario: **los comentarios de código nuevos se escriben en inglés**. `P-20261108-01-scheduling-availability/C06` sigue pendiente de revisión por decisión explícita del usuario ("es algo que revisaremos después").

**Migración reservada:** `0010_appointments`.  
**Responsable de integración:** quien ejecute el plan; una sola persona registra migraciones, composición, comandos, permisos, manifiestos y lockfile.  
**Orden:** secuencial, una tarea `In progress`; cada checkpoint requiere revisión del usuario antes de la fase siguiente.  
**Reglas comunes:** aplicar `minimal-code` antes de cada cambio y `docs-first-framework-code` para APIs de bibliotecas/frameworks; usar datos ficticios; preservar trabajo ajeno y no editar el contrato importado ni los planes anteriores.

## T01: Congelar contratos de Citas

**Status:** Done

**Description:** Fijar DTO, seis acciones, historia, snapshots, puertos booking, exclusión de reprogramación, tablas `0010`, comandos, dependencias y puntos de integración antes de escribir dominio.

**Acceptance criteria:**
- [x] Los contratos de las seis acciones, cita e historia coinciden con `api-v1.1`.
- [x] Cobertura, datos provisionales, puertos booking, exclusión y resolución recibo+cita+historia quedan definidos.
- [x] Migración, diez comandos, propietarios e integrador quedan reservados y las cinco huellas del bundle intactas.

**Evidencia 2026-11-08 (T01):**

- **Bundle:** las cinco huellas SHA-256 de `manifest.json` verificadas contra `accepted-operations.schema.json`, `capabilities.json`, `openapi.json`, `public-models.schema.json` y `sse.schema.json`; todas OK, sin ediciones.
- **Seis acciones congeladas** (capabilities `payloadModels`, todas en `operationCatalog` y **ausentes de `onlineOnlyActions`** → offline-capables): `CreateAppointment` (`CreateAppointmentPayload`), `ConfirmAppointment` (`EmptyAppointmentPayload`), `RescheduleAppointment` (`AppointmentInterval`), `CancelAppointment`/`CompleteAppointment`/`MarkAppointmentNoShow` (`AppointmentReasonPayload` con `changeReason` opcional minLength 1, por defecto null).
- **Payloads remotos** (esquemas verificados): create `{patientId,professionalId,serviceId,startsAt,endsAt,responsibleContactId?}`; reschedule `{startsAt,endsAt}`; confirm `{}`; motivo `{changeReason?}`. `additionalProperties:false` en todos.
- **`AppointmentResponse`**: `id`, `accountId`, `patientId`, `professionalId`, `serviceId`, `responsibleContactId` (nullable), `startsAt`, `endsAt`, `status ∈ {scheduled,confirmed,cancelled,completed,no_show}`, `confirmedAt` nullable, `version>=1`, nueve snapshots (`serviceNameSnapshot`, `offeredDurationMinutesSnapshot` 1–1440, `professionalNameSnapshot`, `locationNameSnapshot/Address/Country(2 letras)/State/City`, `timeZoneSnapshot`), `createdAt`/`updatedAt` nullable.
- **Historia**: `AppointmentHistoryResponse` (una entrada por versión: `action ∈ {created,confirmed,rescheduled,cancelled,completed,no_show}`, `actorKind ∈ {staff,patient,contact}` — escritorio solo `staff` —, `actorUserId`/`actorContactPersonId` nullable, `changeReason`, `changedAt`, intervalo, `confirmedAt`, estado y nueve snapshots) y `AppointmentHistoryProjection` (`{appointmentId,accountId,version,startsAt,endsAt,items}`; la proyección conserva el intervalo de su agregado).
- **Transiciones** (contrato §5.4, verificadas contra el plan): `Create→scheduled(v1)`; `Confirm scheduled→confirmed` (hora de confirmación del servidor); `Reschedule scheduled/confirmed→scheduled` con igual duración y `confirmedAt` nulo; `Cancel scheduled/confirmed→cancelled` libera ocupación; `Complete scheduled/confirmed→completed` exige inicio alcanzado; `NoShow scheduled/confirmed→no_show` exige inicio + margen vigente (0–1440, base 15) y acción staff. `cancelled`/`completed`/`no_show` terminales; repetir la misma operación recupera su recibo, no crea otro efecto. `scheduled`/`confirmed`/`completed`/`no_show` ocupan; solo `cancelled` libera; contigüidad fin/inicio permitida; exclusión por profesional entre cuentas.
- **OpenAPI**: `GET /v1/accounts/{id}/appointments` (listado, paginación máx 500), `GET .../appointments/{id}`, `GET .../appointments/{id}/history`. `DefaultAppointmentWindow` = `{daysBefore:30, daysAfter:180, maxDays:366}` (los comandos locales limitan el rango a 366 días).
- **Resolución de provisionales congelada:** un provisional se resuelve solo con recibo `accepted` **y** cita e historia alcanzando la versión confirmada; recibo solo no retira nada; intenciones jamás se borran por conciliación.
- **Puertos booking congelados (aditivos):** `PatientBookingQuery` (Patients; ficha/documentos/responsables aceptados o provisionales encadenables con versión/operación y teléfono/vigencia), `CatalogBookingQuery` (Services; servicio/oferta/profesional con nombre y duración efectiva para snapshot), `AccountViewQuery` sin cambios (solo aceptado), `ScheduleAvailabilityQuery` ampliado con `excludeAppointmentId?: AppointmentId`, `ScheduleOccupancyQuery` combinando vista aceptada + operaciones locales.
- **Migración reservada:** `0010_appointments` (`appointment_view`, `appointment_history_view`, `appointment_local`; `appointment_occupancy_view` se conserva). Sin colisión: registro actual termina en `0009_appointment_occupancy`.
- **Comandos congelados (10):** cuatro consultas (`appointments_list_appointments`, `appointments_get_appointment`, `appointments_get_history`, `appointments_get_booking_context`) y seis escrituras (`appointments_create_appointment`, `appointments_confirm_appointment`, `appointments_reschedule_appointment`, `appointments_cancel_appointment`, `appointments_complete_appointment`, `appointments_mark_no_show`).
- **Propietarios/integrador:** según plan §Arquitectura y `core-next-tracks.md`; integrador único registra `lib.rs`, `build.rs`, `components/mod.rs`, `platform/sqlite/mod.rs`, `components/projector.rs`, manifiestos/lockfile y `capabilities/default.json`.

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

**Status:** Done

**Description:** Implementar estados, transiciones, intervalos de minuto, snapshots, edad a fecha local y errores estables sin almacenamiento ni framework.

**Acceptance criteria:**
- [x] Las seis transiciones y los estados terminales respetan el contrato.
- [x] Instantes, precisión de minuto y duración real 1–1440 se validan exactamente.
- [x] Menor de 18, 29 de febrero y responsables se evalúan a la fecha local de la cita.

**Evidencia 2026-11-08 (T02):**

- Nuevo `src-tauri/src/components/appointments/domain.rs` registrado en `mod.rs`; sin almacenamiento, HTTP ni Tauri.
- `AppointmentId` UUIDv7 con macro local (mismo patrón que Scheduling; `shared_kernel` no se amplía). `AppointmentsError` con códigos estables `VALIDATION_ERROR`/`INVALID_STATE_TRANSITION`/`INTERNAL_ERROR`.
- `AppointmentStatus` (5 estados contractuales), terminales y `occupies()` (solo `cancelled` libera). `HistoryAction` y `ActorKind` (staff/patient/contact) con strings contractuales. `ChangeReason` opcional no vacío.
- `MinuteInstant`: RFC 3339 con offset, normalizado a UTC, precisión de minuto exacta (segundos y subsegundos nulos). `AppointmentInterval` `[start,end)` con duración real 1–1440 vía `Span::total(Unit::Minute)` (fuente: [jiff 0.2.38 Span](https://docs.rs/jiff/0.2.38/jiff/struct.Span.html), [`Timestamp`](https://docs.rs/jiff/0.2.38/jiff/struct.Timestamp.html)).
- `Snapshots`: nueve referencias con validación contractual (textos no vacíos, país ISO de 2 letras, duración ofrecida 1–1440); comentario de dominio: el servidor los recalcula al aceptar y el cliente jamás los recalcula al reprogramar o proyectar historia.
- `apply_transition`: seis transiciones como intenciones; `Create` exige ausencia de cita; `Confirm` solo desde `scheduled`; `Reschedule` desde `scheduled`/`confirmed` → `scheduled`; `Cancel`/`Complete`/`NoShow` desde `scheduled`/`confirmed` → terminal; terminales rechazan con `INVALID_STATE_TRANSITION`. La repetición de una operación se reconcilia con su recibo en sync (no aquí).
- `age_on`/`is_minor_at`: edad administrativa a fecha civil; 29 de febrero cumple el 1 de marzo en años no bisiestos; menor de 18 límite exacto el día del cumpleaños.
- Verificación: `cargo test --locked components::appointments` → **14/14** (8 de dominio: estados/transiciones, intervalos/minuto/duración, historia/actores, motivo, snapshots, edad/feb-29, IDs); `cargo check --workspace --all-targets --locked` sin warnings; `git diff --check` OK.

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

**Status:** Done

**Description:** Crear `0010_appointments`, vistas de cita/historia, intenciones e índices, conservando `appointment_occupancy_view`.

**Acceptance criteria:**
- [x] La migración 9→10 conserva datos y reabre sin repetir cambios.
- [x] Un fallo revierte archivo, historial y versión completos.
- [x] Las filas aíslan cuenta/usuario/dispositivo y no hay cascadas hacia intenciones.

**Evidencia 2026-11-08 (T03):**

- `src-tauri/migrations/0010_appointments.sql` registrada como versión 10 en `platform/sqlite/mod.rs` (`0001`–`0009` intactas). Tablas:
  - `appointment_view` (PK ámbito+`appointment_id`): referencias, `responsible_contact_id` NULL, `starts_at<ends_at`, estado limitado a los 5 contractuales, `confirmed_at` NULL, nueve snapshots con CHECK (duración ofrecida 1–1440, país 2 letras), `created_at`/`updated_at` NULL, `version>0`. Índices por profesional/ventana, paciente y servicio.
  - `appointment_history_view` (PK ámbito+`appointment_id`+`version`): acción/actor contractuales (actor `staff|patient|contact`), motivo e intervalo por versión, `confirmed_at` y nueve snapshots; append-only por diseño. Índice por cita/versión.
  - `appointment_local`: intenciones con las seis acciones con CHECK, `expected_version>=0`, `base_operation_id`, `depends_on` JSON ≤8, `payload`/`draft` JSON objeto, `created_at`. Sin columna `provisional` (se deriva de `sync_outbox`, igual que Agenda).
  - `appointment_occupancy_view` (0009) se conserva sin cambios.
- Adaptadores: `components/appointments/adapters/outbound.rs` amplía con `AppointmentRecord`/`AppointmentHistoryRecord`/`LocalChange`, `save_*` sobre `&Transaction`, lecturas sobre `&Connection`, aisladas por ámbito triple; sin SQL expuesto al consumidor.
- `repository_tests.rs` (+3 pruebas): ida y vuelta aceptado+historia+intención, aislamiento de ámbito (otra cuenta no ve nada), supervivencia de intenciones al vaciar vistas y a reapertura; CHECK rechazan estado `reschedule_requested`, país de 3 letras, intervalo invertido, acción de historia `deleted`, acción de intención ajena y 9 dependencias; migración 9→10 conserva datos de Scheduling con reapertura sin cambios (`total_changes()==0`). Prueba previa de Agenda actualizada a `user_version=10`.
- El rollback byte a byte de un fallo de migración queda cubierto por la suite común del runner (`rolls_back_failed_migration`, `failed_*_restores_*`), que ahora ejerce la cadena completa hasta la 10 (19/19 en `platform::sqlite`).
- Verificación: `cargo test --locked components::appointments` → **17/17**; `platform::sqlite` → **19/19**; `cargo check --workspace --all-targets --locked` sin warnings; `git diff --check` OK.

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

**Status:** Done

**Description:** Añadir una lectura pública de ficha, documentos y responsables aceptados o provisionales utilizables con sus referencias, sin exponer el repositorio de Patients.

**Acceptance criteria:**
- [x] La lectura aporta completitud, documentos, teléfono y responsables con vigencia.
- [x] Las referencias aceptadas/provisionales incluyen versión u operación y excluyen estados no encadenables.
- [x] El puerto mantiene aislamiento y no expone SQL, HTTP ni almacenamiento.

**Evidencia 2026-11-08 (T04):**

- `patients/application.rs`: `BookingReference` (`Accepted{version}` | `Provisional{operationId,expectedVersion}`), `BookingPatient` (perfil fusionado `EntityRow<ProfileState>`, documentos `EntityRow<IdentifierState>`, responsables `EntityRow<RelationshipView>` con teléfono actual y vigencia) y el puerto público `PatientBookingQuery::booking_patient`. Ampliación aditiva: `PatientQuery` y `PatientRepository` no cambian.
- `patients/adapters/outbound.rs`: implementación sobre `SqlitePatientRepository` reutilizando las lecturas fusionadas existentes (`profile`, `identifiers`, `patient_contacts` activos); sin SQL nuevo expuesto y sin tocar tablas ajenas. Reglas: sin estado visible o con operaciones `conflict`/`rejected` → `Ok(None)` (referencia no encadenable excluida); `NOT_FOUND` del ámbito se mapea a `Ok(None)`; la referencia toma la versión aceptada o la operación provisional vigente.
- `patients/booking_tests.rs` (3 pruebas): adulto aceptado con documento `declared` resuelve con `Accepted{version:2}`, teléfono propio, identificadores y responsable activo con teléfono; paciente inexistente → `None`; paciente creado offline (`Register`) resuelve como `Provisional{operationId,expectedVersion:0}` con operación `pending`; operación en `conflict` (simulado vía `sync_outbox`, patrón `reconciliation_tests`) excluye al paciente (`None`).
- Verificación: `cargo test --locked components::patients` → **72/72**, 1 ignored preexistente (prueba de desarrollo real); `cargo check --workspace --all-targets --locked` sin warnings.

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

**Status:** Done

**Description:** Añadir una lectura pública de servicio, oferta y profesional aceptados o provisionales con snapshots y referencias, sin cambiar `CatalogQuery`.

**Acceptance criteria:**
- [x] Nombre y duración efectiva de referencia se calculan desde el estado elegido.
- [x] Estados y referencias provisionales solo se ofrecen cuando pueden encadenarse.
- [x] No se leen tablas ajenas ni se alteran las consultas aceptadas existentes.

**Evidencia 2026-11-08 (T05):**

- `services/application.rs`: `CatalogReference` (`Accepted{version,serviceVersion}` — ambas versiones contractuales — | `Provisional{operationId}`), `BookingOffer` (profesional, servicio, nombre, nombre profesional, duración efectiva de referencia y estados) y puerto público `CatalogBookingQuery::booking_offer`. Ampliación aditiva: `CatalogQuery` no cambia.
- `services/adapters/outbound.rs`: implementación sobre `SqliteCatalogRepository` reutilizando `intents::resolve` + `intents::screen` (misma fusión aceptado/provisional de la pantalla de catálogo, transacción `IMMEDIATE` y commit como `catalog()`); sin SQL nuevo ni tablas ajenas. Reglas: profesional activo no bloqueado; oferta visible con cambios `conflict`/`rejected` → excluida (`None`); oferta aceptada → referencia `Accepted` con ambas versiones y duración efectiva; oferta con borrador provisional (ajuste o creación offline) → referencia `Provisional` con la operación pendiente/`unknown`, duración `override` o por defecto del servicio (vista aceptada o borrador del servicio offline) y nombre del servicio (vista o borrador); oferta o servicio inactivos → `None`.
- `services/booking_tests.rs` (4 pruebas): oferta aceptada resuelve con `Accepted{1,1}`, duración efectiva 30 y nombre; servicio+oferta creados offline encadenan (`Provisional{operationId}` de la oferta, nombre "Consulta offline", override 45, estados activos); conflicto sobre la operación de la oferta excluye la reserva (`None`); oferta inactiva o `service_status` inactivo excluyen.
- Verificación: `cargo test --locked components::services` → **23/23**; `cargo check --workspace --all-targets --locked` sin warnings.

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

**Status:** Done

**Description:** Ampliar disponibilidad con `excludeAppointmentId` y la ocupación local de Appointments para bloquear provisionales sin autobloqueo al reprogramar.

**Acceptance criteria:**
- [x] La exclusión retira únicamente la cita indicada del cálculo de ocupación.
- [x] `scheduled`, `confirmed`, `completed` y `no_show` ocupan; `cancelled` libera también provisionalmente.
- [x] La cobertura de Agenda permanece intacta y no se crea una segunda disponibilidad persistida.

**Evidencia 2026-11-08 (T06):**

- `scheduling/application.rs`: `AvailabilityRequest` amplía con `exclude_appointment_id: Option<String>` (aditivo; no se envía a la API — la validación compartida la hace el servidor con el estado completo) y `validate_availability_request` exige UUIDv7 canónico cuando está presente. Seis sitios de construcción actualizados con `None` (IPC, tests, remoto, integración).
- `scheduling/adapters/outbound.rs`: `local_availability` valida el exclusión y filtra **solo** la cita indicada antes de restar la ocupación; la cobertura por cursor y el resto del cálculo no cambian (no se persiste ninguna segunda disponibilidad).
- `appointments/adapters/outbound.rs`: `SqliteScheduleOccupancyQuery` combina la vista aceptada con la ocupación provisional de `appointment_local`: el último borrador local por cita define el estado provisional (pendiente bloquea, cancelación provisional libera); intenciones con operaciones `conflict`/`rejected`/revisión excluidas; la vista aceptada gana ante duplicados; borradores ilegibles se ignoran de forma defensiva; resultado ordenado por inicio.
- `appointments/occupancy_tests.rs` (3 pruebas, intención + Outbox en la misma transacción vía `record_operation`): cita pendiente bloquea y su cancelación provisional libera; profesional distinto aislado; operación en conflicto excluye; aceptada + provisional combinan sin duplicados y sobreviven a reapertura.
- `scheduling/availability_query_tests.rs` (+1): reprogramar excluyendo la propia cita devuelve su hueco (12:00–14:00Z) mientras la otra cita sigue ocupando (el hueco 14:00–14:30Z no reaparece); exclusión de cita desconocida no cambia nada; exclusión no canónica rechazada.
- Puntos de integración por el registro de la migración 10 (esperados): `identity/activation_tests.rs` (`localSchemaVersion` 9→10), `scheduling/repository_tests.rs` (`user_version` 9→10) y `tests/core_persistence.rs` (conteo de migraciones 9→10). `0001`–`0009` intactas.
- Verificación: `cargo test --workspace --locked` → **331 passed, 0 failed** (325 lib + 5 core_persistence + 1 bin/doc), 8 ignored preexistentes; `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings` y `git diff --check` OK.

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

**Status:** Aprobado por el usuario el 2026-11-08 (commit `43ec89e`).

- [ ] Dominio, migración y puertos booking/disponibilidad pasan junto con `cargo test --workspace --locked` y `pnpm build:desktop`.
- [ ] El usuario revisa snapshots, provisionales, exclusión y cobertura.
- [ ] No hay escrituras de Citas ni cambios en contratos o migraciones anteriores.
- [ ] `git diff --check` pasa.

**Evidencia 2026-11-08 (C01, pendiente de revisión del usuario):**

- `cargo test --workspace --locked`: **331 passed, 0 failed** (325 lib + 5 core_persistence + 1 bin/doc), 8 ignored preexistentes (pruebas de desarrollo con entorno real). Citas 20/20 (8 dominio, 6 repositorio, 3 ocupación, 3 proyectores previos de Agenda), Patients 72/72 (incl. 3 booking), Services 23/23 (incl. 4 booking), Scheduling 53/53 (incl. exclusión), plataforma 19/19.
- `pnpm build:desktop`: OK (`target/debug/nightingale.exe`). `cargo fmt --all -- --check` y `cargo clippy --workspace --all-targets --locked -- -D warnings`: OK. `git diff --check`: OK.
- **Huellas del bundle:** las cinco SHA-256 de `manifest.json` verificadas contra los cinco archivos → todas OK; contrato importado sin ediciones.
- **Archivos creados en esta fase:** migración `0010_appointments.sql`, `appointments/domain.rs`, `appointments/occupancy_tests.rs`, `patients/booking_tests.rs`, `services/booking_tests.rs` y evidencia en el todo del plan. Sin logs, bases temporales ni artefactos huérfanos en Git. Lo demás no rastreado (`scheduling/*`, `projector.rs`, fixtures, `features/scheduling/`, etc.) es trabajo sin commit del plan de Agenda, preservado intacto.
- **Puertos booking demostrados con aceptados y provisionales:** adulto aceptado (versión 2, documento `declared`, responsable activo con teléfono) y paciente offline (`Provisional{operationId, expectedVersion:0}`); oferta aceptada (`Accepted{version, serviceVersion}`, duración efectiva) y servicio+oferta creados offline (override 45 encadenado); conflictos excluyen en ambos puertos (`None`).
- **Exclusión al reprogramar demostrada:** excluir la propia cita devuelve su hueco (12:00–14:00Z) y la otra cita sigue ocupando (14:00–14:30Z no reaparece); exclusión de cita desconocida no altera; exclusión inválida rechazada. Ocupación provisional: pendiente bloquea, cancelación provisional libera, conflicto excluye, aceptada+provisional sin duplicados tras reapertura.
- **Límites explícitos:** sin escrituras de Citas (CreateAppointment empieza en T08); la exclusión no se envía a la API; `C06` de Agenda sigue pendiente de revisión del usuario por decisión expresada.

## T07: Secuencias y dependencias de operaciones

**Status:** Done

**Description:** Implementar preparación común con base aceptada/local, versiones previstas, provisionales y Outbox atómico.

**Acceptance criteria:**
- [x] Provisional y Outbox se confirman o revierten juntos.
- [x] Las secuencias respetan ocho dependencias, sin duplicados, ciclos ni truncamiento.
- [x] Bases antiguas o bloqueadas no crean operación y el trabajo previo se conserva.

**Evidencia 2026-11-08 (T07):**

- `appointments/application.rs`: `EditBase` (`Accepted{version}`/`LocalOperation{operationId}`), `AppointmentState` (estado completo con nueve snapshots, camelCase para borradores), `Accepted`/`Provisional`/`OperationView`/`EntityRow` (misma fusión que Patients/Agenda), `NewAppointment` (referencias + intervalo + snapshots), `AppointmentMutation` con las seis acciones (`Create` con `Box<NewAppointment>` por tamaño de variante, clippy), `action()`, `LocalMutationResult`, puerto `AppointmentRepository` (`appointment_row`, `persist`) y `require_active` (acciones staff con cuenta activa; la autorización por acción la vuelve a validar API).
- `appointments/adapters/intents.rs`: `edit_base` (base aceptada exige fila sin provisional y misma versión; base local exige `can_depend` en operaciones previas), `validate_depends_on` (UUIDv7 canónico, existente en el ámbito, sin duplicados, ≤8 — el grafo sigue acíclico porque una operación nueva no puede referenciarse), `prepare` con payloads contractuales exactos (create con `responsibleContactId` solo cuando existe; confirm `{}`; reschedule `{startsAt,endsAt}`; motivo solo cuando existe), transiciones de dominio aplicadas en cada acción, `NO_CHANGES` al reprogramar al mismo intervalo, confirmación con `confirmedAt` siempre nulo localmente, y `confirmed()` (recibo aceptado + versión confirmada ≤ vista aceptada).
- `appointments/adapters/outbound.rs`: `appointment_rows` (aceptado + provisional + operaciones separados; provisional retira solo por recibo+vista), `current_access` (acceso activo y época igual), `SqliteAppointmentRepository::persist` — preparación fuera de la transacción, `record_operation` + `insert_local_change` + commit en una `IMMEDIATE`; sin SQL/HTTP expuestos.
- `appointments/test_support.rs` + `application_tests.rs` (4 pruebas): secuencia create→confirm→reschedule→cancel con bases locales encadenadas (versiones 0→3) y reapertura con provisional cancelado y 4 operaciones; bases aceptadas con provisional, bases locales falsas y operaciones en conflicto rechazan (`VERSION_CONFLICT`/`DEPENDENCY_FAILED`) sin crear intención ni Outbox; dependencias validadas (no canónica/inexistente/duplicada/>8 rechazadas, 8 válidas aceptadas, conflicto excluye); `NO_CHANGES` en reprogramación idéntica y transiciones inválidas en terminales sin crear operación.
- Convención del usuario aplicada: comentarios de código nuevos en inglés (pase de conversión sobre los archivos tocados en T01–T07; los comentarios del plan de Agenda se preservan).
- Verificación: `cargo test --workspace --locked` → **335 passed, 0 failed** (329 lib + 5 core_persistence + 1 bin/doc), 8 ignored preexistentes; `cargo fmt --all -- --check` y `cargo clippy --workspace --all-targets --locked -- -D warnings` OK; `git diff --check` OK.

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

**Status:** Done

**Description:** Implementar `CreateAppointment` con validación booking, snapshots, disponibilidad, dependencias y versión cero.

**Acceptance criteria:**
- [x] Adulto, menor y responsable cumplen las reglas a la fecha local de la cita.
- [x] Cita provisional, snapshots y Outbox se guardan atómicamente con `expectedVersion=0`.
- [x] Cobertura insuficiente o solapamiento conocido bloquea sin efectos parciales.

**Evidencia 2026-11-08 (T08):**

- `appointments/application.rs`: `CreateAppointmentInput` (ids + intervalo RFC 3339 de minuto) y validador puro `validate_new_appointment(account, patient, offer, input, evaluation)` — componible y probable sin base de datos: cuenta activa; paciente `active`+`complete` con nombre; `own_document` exige documento `declared`/`verified` y teléfono propio o responsable válido; `minor_without_document` exige nacimiento, menor de 18 a la fecha civil local de la cita (dominio) y responsable; responsable elegido debe ser vínculo activo marcado responsable, vigente en la fecha local (extremos inclusivos) y con teléfono (sin inferir representación/tutela); oferta correspondiente a profesional+servicio; `insufficient_coverage` → `INSUFFICIENT_COVERAGE`; intervalo contenido en un intervalo disponible → si no `SLOT_UNAVAILABLE`; `no_synced_window` permite trabajar con lo conocido (regla de Agenda); snapshots armados desde oferta (nombre, duración efectiva, profesional) + cuenta (lugar completo y zona).
- `appointments/adapters/outbound.rs`: `SqliteAppointmentRepository::create_appointment` compone los puertos existentes (mismo patrón que `local_availability` de Agenda): vista de cuenta aceptada → `PatientBookingQuery` → `CatalogBookingQuery` → ocupación leída primero como snapshot `KnownOccupancy` (sin compartir el `&mut` de la conexión) → `ScheduleAvailabilityQuery.local_availability` con duración real; valida; declara dependencias de las referencias provisionales (paciente y oferta, sin duplicados); `persist` atómico con `expectedVersion=0`. Permiso: acción en `allowedActions` del acceso vigente. Ningún fallo escribe nada: todo se valida antes de abrir la transacción.
- `appointments/create_tests.rs` (8 pruebas): matriz pura — adulto `declared`/`verified` con snapshots correctos; documento `invalidated`/ausente, sin teléfono ni responsable, ficha `draft` e inactivo rechazados; adulto sin teléfono con responsable válido aceptado; menor límite (cumple 18 el día de la cita, fecha local de la zona de la cuenta) rechazado y menor de 17 con responsable aceptado; responsable desconocido/vínculo no vigente (inicio posterior o fin previo a la fecha local)/sin teléfono rechazados; `insufficient_coverage` rechaza con código propio, `no_synced_window` permite, intervalo que no cabe o hueco ocupado → `SLOT_UNAVAILABLE`. Flujo integrado real: create guarda provisional + Outbox (`payload` contractual exacto, `confirmedAt` nulo, nueve snapshots), doble reserva del mismo hueco bloqueada por la ocupación provisional, contigüidad permitida, fuera de horario sin efectos parciales; paciente menor provisional encadena la operación de completitud como dependencia registrada en Outbox y `appointment_local`.
- Verificación: `cargo test --workspace --locked` → **343 passed, 0 failed** (337 lib + 5 core_persistence + 1 bin/doc), 8 ignored preexistentes; `cargo fmt --all -- --check` y `cargo clippy --workspace --all-targets --locked -- -D warnings` OK; `git diff --check` OK.

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

**Status:** Done

**Description:** Implementar `ConfirmAppointment` desde estado programado aceptado o provisional.

**Acceptance criteria:**
- [x] El payload es `{}` y la base/dependencia pertenece a la misma cita.
- [x] Solo `scheduled` pasa a `confirmed`; otra transición devuelve `INVALID_STATE_TRANSITION`.
- [x] No se fabrica `confirmedAt` local ni una segunda confirmación.

**Evidencia 2026-11-08 (T09):**

- La preparación de T07 fija el contrato de la acción; esta tarea lo cierra con pruebas dedicadas `appointments/confirm_tests.rs` (4 pruebas): confirm sobre base local encadena la operación create de la misma cita como dependencia, `expectedVersion=1`, payload exactamente `{}` en Outbox, borrador `confirmed` con `confirmedAt` nulo (solo el servidor asigna la hora de confirmación) e intervalo conservado; reapertura conserva el provisional. Base aceptada limpia (recibo aceptado + vista en versión 1) confirm sin dependencias. Rechazos sin efectos: base local de otra cita (`VERSION_CONFLICT` — la base/dependencia queda atada a la fila de su cita), segunda confirmación (`INVALID_STATE_TRANSITION`), base aceptada ajena/falsa (`VERSION_CONFLICT`), confirm sobre cancelado (`INVALID_STATE_TRANSITION`); conteo de Outbox intacto tras los rechazos.
- Verificación: `cargo test --workspace --locked` → **347 passed, 0 failed** (341 lib + 5 core_persistence + 1 bin/doc), 8 ignored preexistentes; fmt, clippy `-D warnings` y `git diff --check` OK.

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

**Status:** Done

**Description:** Implementar `RescheduleAppointment` con igual duración real, exclusión propia, snapshots y confirmación invalidada provisionalmente.

**Acceptance criteria:**
- [x] El nuevo intervalo conserva duración real 1–1440 y referencias/snapshots.
- [x] Disponibilidad excluye la cita actual y detecta otras ocupaciones/bloqueos.
- [x] `scheduled` o `confirmed` vuelve a `scheduled` y `confirmedAt` queda nulo provisionalmente.

**Evidencia 2026-11-08 (T10):**

- `adapters/intents.rs` (preparación): reprogramar exige igual duración real que la cita visible (cálculo en minutos con `Span::total`, tolerancia exacta) → `VALIDATION_ERROR`; el mismo intervalo sigue siendo `NO_CHANGES`.
- `adapters/outbound.rs`: `reschedule_appointment` — fila fusionada y transición resueltas antes de escribir; disponibilidad local con `exclude_appointment_id = propia cita` y duración solicitada = duración real; `insufficient_coverage` → `INSUFFICIENT_COVERAGE`; intervalo contenido en un intervalo disponible o `SLOT_UNAVAILABLE`; `persist` atómico. Permiso por acción en el acceso vigente.
- `reschedule_tests.rs` (3 pruebas): confirmada reprogramada vuelve a `scheduled` con `confirmedAt` nulo, payload exactamente `{startsAt, endsAt}`, snapshots/referencias conservados y reapertura durable; cambio de duración real y mismo intervalo rechazados sin operación; la ocupación de otra cita bloquea el nuevo hueco aun excluyendo la propia, y tras cancelarla la misma mudanza se permite — la ocupación provisional sigue al último borrador (hueco viejo libre, nuevo ocupado).
- Verificación: `cargo test --workspace --locked` → **350 passed, 0 failed** (344 lib + 5 core_persistence + 1 bin/doc), 8 ignored preexistentes; fmt, clippy `-D warnings` y `git diff --check` OK.

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

**Status:** Done

**Description:** Implementar `CancelAppointment` con motivo opcional, historia local separada y liberación de ocupación.

**Acceptance criteria:**
- [x] Solo `scheduled` o `confirmed` puede cancelarse y el motivo presente no está vacío.
- [x] La cancelación libera ocupación provisional sin borrar operaciones anteriores.
- [x] La historia local permanece separada de la aceptada y sobrevive a reapertura.

**Evidencia 2026-11-08 (T11):**

- La preparación de T07 fija la transición, el motivo opcional no vacío (`ChangeReason`, dominio T02) y el borrador terminal; `cancel_tests.rs` (3 pruebas): cancelación con motivo viaja en el payload (`{"changeReason": ...}`), el borrador conserva snapshots y referencias, y la ocupación provisional queda libre (la consulta del puerto devuelve vacío); cancelación sin motivo produce payload exactamente `{}`; motivo vacío rechazado en dominio; terminales rechazan otras transiciones (`INVALID_STATE_TRANSITION`); reapertura conserva el provisional cancelado y ambas operaciones. La historia local permanece en `appointment_local`/`sync_outbox`, separada de la vista aceptada (`appointment_history_view` la escribe solo el proyector, T15).
- Verificación: ver C02.

## T12: Registrar atención

**Status:** Done

**Description:** Implementar `CompleteAppointment` con reloj inyectado y motivo opcional.

**Acceptance criteria:**
- [x] Solo se completa cuando `clock.now >= startsAt` desde `scheduled` o `confirmed`.
- [x] El estado terminal y motivo se conservan sin alterar ocupación ni historia anterior.
- [x] Una llamada antes del inicio no crea operación ni modifica la cita.

**Evidencia 2026-11-08 (T12):**

- `adapters/outbound.rs`: `complete_appointment` — permiso por acción, fila fusionada, comparación `now >= startsAt` en milisegundos con el reloj inyectado (el servidor sigue siendo la autoridad temporal); fallo → `VALIDATION_ERROR` antes de cualquier transacción; `persist` atómico. `complete_tests.rs` (2 pruebas): antes del inicio no crea operación ni intención (conteos intactos); en el inicio alcanza `completed` con motivo en el payload, sigue ocupando según contrato y sobrevive a reapertura quedando terminal (una segunda llamada devuelve `INVALID_STATE_TRANSITION`).
- Verificación: ver C02.

## T13: Marcar ausencia

**Status:** Done

**Description:** Implementar `MarkAppointmentNoShow` con inicio+margen aceptado vigente y acción staff.

**Acceptance criteria:**
- [x] Solo se marca cuando `clock.now >= startsAt + noShowGraceMinutes` vigente.
- [x] La ausencia nunca es automática por falta de respuesta y requiere acción del personal.
- [x] El estado terminal conserva ocupación, motivo e historia.

**Evidencia 2026-11-08 (T13):**

- `adapters/outbound.rs`: `mark_no_show_appointment` — permiso por acción; el margen se lee de la **vista aceptada** de la cuenta (`account_view.policy`, vigente en el ámbito), nunca de preferencias o del dispositivo; comparación en milisegundos con reloj inyectado; fallo → `VALIDATION_ERROR` sin operación. `no_show_tests.rs` (3 pruebas): antes del inicio y dentro del margen (inicio+15 min − 1 s) rechazados sin crear operación; al vencer el margen alcanza `no_show` con motivo, sigue ocupando según contrato y sobrevive a reapertura; un cambio de política aceptada (margen 0) permite marcar exactamente al inicio — la política aceptada vigente gobierna el margen.
- Verificación: ver C02.

## C02: Ciclo de vida nativo

- [ ] Las seis acciones y secuencias offline/reapertura pasan con referencias provisionales y aceptadas.
- [x] `cargo test --workspace --locked` y `pnpm build:desktop` pasan.
- [ ] El usuario revisa estados, tiempos y disponibilidad antes de sincronización.
- [ ] `git diff --check` pasa.

**Evidencia 2026-11-08 (C02, pendiente de revisión del usuario):**

- `cargo test --workspace --locked`: **358 passed, 0 failed** (352 lib + 5 core_persistence + 1 bin/doc), 8 ignored preexistentes. Una ejecución intermedia falló con 2 tests de sincronización por temporización (el flake ya registrado por Agenda en su T21); la repetición pasó completa — se registra como flake no relacionado. Citas: 47/47 (dominio 8, repositorio 6, ocupación 3, aplicación 4, create 8, confirm 4, reschedule 3, cancel 3, complete 2, no-show 3).
- `cargo fmt --all -- --check` y `cargo clippy --workspace --all-targets --locked -- -D warnings`: OK. `git diff --check`: OK. `pnpm build:desktop`: OK (`target/debug/nightingale.exe`).
- Seis acciones verificadas offline con bases aceptadas y locales encadenadas, reapertura durable y rechazos sin efectos parciales (`NO_CHANGES`, `INVALID_STATE_TRANSITION`, `VERSION_CONFLICT`, `DEPENDENCY_FAILED`, `SLOT_UNAVAILABLE`, `INSUFFICIENT_COVERAGE`, `VALIDATION_ERROR`, `PERMISSION_DENIED`).
- Recorridos para revisión: `create_tests.rs` (matriz adulto/menor/responsable + flujo integrado), `confirm_tests.rs`, `reschedule_tests.rs`, `cancel_tests.rs`, `complete_tests.rs`, `no_show_tests.rs`.

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
- [x] `cargo test --workspace --locked` y `pnpm build:desktop` pasan.
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
