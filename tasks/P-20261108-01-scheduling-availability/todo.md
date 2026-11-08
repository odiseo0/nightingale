# Task List: P-20261108-01-scheduling-availability

**Plan:** [plan.md](plan.md)  
**Status del plan:** Approved  
**Created:** 2026-11-08  
**Updated:** 2026-11-08

**Instrucción vigente:** el usuario aprobó el diseño y autorizó materializar estos archivos. Ninguna tarea ha comenzado. El inicio de T01 requiere una instrucción de ejecución posterior.

**Migraciones reservadas:** `0008_scheduling` y `0009_appointment_occupancy`.  
**Responsable de integración:** quien ejecute el plan; una sola persona registra migraciones, composición, comandos, permisos, manifiestos y lockfile.  
**Orden:** secuencial, una tarea `In progress`; cada checkpoint requiere revisión del usuario antes de la fase siguiente.  
**Reglas comunes:** aplicar `minimal-code` antes de cada cambio y `docs-first-framework-code` para APIs de bibliotecas/frameworks; usar datos ficticios; preservar trabajo ajeno y no editar el contrato importado.

## T01: Congelar contratos y tiempo

**Status:** Done

**Description:** Registrar DTO, acciones, payloads, puertos, cobertura, comandos, migraciones, propietarios e integrador. Confirmar la selección de Jiff 0.2.38, features tzdb y resolución de identificadores antes de abrir el dominio.

**Acceptance criteria:**
- [x] Contratos completos y compatibles con `api-v1.1`, incluidas las nueve acciones y catorce comandos.
- [x] Los 598 identificadores de `reference_data` resuelven con la tzdb seleccionada y la ambigüedad estricta queda especificada.
- [x] Las cinco huellas del manifiesto permanecen intactas y no se modifica el bundle.

**Evidencia 2026-11-08 (T01):**

- **Bundle:** las cinco huellas SHA-256 de `manifest.json` verificadas contra `accepted-operations.schema.json`, `capabilities.json`, `openapi.json`, `public-models.schema.json` y `sse.schema.json`; todas OK, sin ediciones.
- **Acciones congeladas** (capacidades, modelos `src.api.schedule.domain.schedule.*`): `CreateScheduleRule`, `UpdateScheduleRule`, `SetScheduleRuleStatus`, `CreateScheduleException`, `UpdateScheduleException`, `SetScheduleExceptionStatus`, `CreateScheduleBlock`, `UpdateScheduleBlock`, `SetScheduleBlockStatus`; payload de estado `ScheduleRuleStatusPayload` = `{status:"active"|"inactive"}`. Ninguna acción de agenda figura como online-only: las nueve funcionan offline.
- **Payloads remotos** (esquemas JSON verificados): regla `{professionalId,isoWeekday,localStartMinute,localEndMinute,validFrom,validUntil?}` (patch `ScheduleRulePatch` con `validUntil` nullable); excepción `{professionalId,localDate,kind,localStartMinute?,localEndMinute?}` (`closed` exige minutos nulos, `replacement` exige ambos); bloqueo `{professionalId,accountId,startsAt,endsAt,reasonCode}` con `reasonCode ∈ {vacation,administrative,unavailable}` o null, `accountId` nullable = global.
- **Respuestas remotas** (`ScheduleRuleResponse`, `ScheduleExceptionResponse`, `ScheduleBlockResponse`): incluyen `accountId`, `professionalId`, tiempos, `status`, `createdAt`, `updatedAt`, `version`; regla con `validFrom`/`validUntil` (fecha date, nullable); excepción con `localDate`, `kind`, minutos nullable.
- **Disponibilidad remota:** `GET A/availability` exige `professionalId`, `serviceId`, `from`, `until`, `durationMinutes` opcional, máximo 31 días; `AvailabilityResponse = {accountId,professionalId,serviceId,referenceDurationMinutes,requestedDurationMinutes,availableIntervals}`.
- **Cobertura de ocupación** (contrato §5.4): `scheduled`, `confirmed`, `completed`, `no_show` ocupan; `cancelled` no; contigüidad fin/inicio permitida; exclusión por profesional entre cuentas.
- **Migraciones reservadas:** `0008_scheduling` (vistas `schedule_rule_view`, `schedule_exception_view`, `schedule_block_view` + `schedule_local`) y `0009_appointment_occupancy` (`appointment_occupancy_view`, propiedad de `appointments`). Sin colisión: el registro actual termina en `0007_patients`.
- **Comandos congelados (14):** nueve `scheduling_create/update/set_*` (reglas, excepciones, bloqueos), `scheduling_list_rules`, `scheduling_list_exceptions`, `scheduling_list_blocks`, `scheduling_get_local_availability`, `scheduling_check_remote_availability`.
- **Puertos congelados:** `ScheduleAvailabilityQuery` (entrada profesional/servicio/`[from,until)` ≤31 días/duración 1–1440; salida zona, duración, versiones, intervalos UTC, origen local, provisional y cobertura `within_synced_window|no_synced_window|insufficient_coverage`) y `ScheduleOccupancyQuery` (ocupación aceptada por profesional/intervalo; evolución futura a provisional).
- **Tiempo congelado:** Jiff **0.2.38** directo con feature `tzdb-bundle-always` (la misma base embebida en Windows/macOS/Linux); tzdb embebida **2026e** según changelog oficial de Jiff; `jiff-static`/`jiff-tzdb` resueltos transitivamente, sin añadir `jiff-static` directo. Verificación temporal bajo `target/scheduling-t01-tz-check/`: **los 598 identificadores de `time_zones.txt` resuelven** con `jiff::tz::TimeZone::get`. Nota: la lista de referencia procede de tzdb **2026f** (`docs/technical/reference-data.md`) y solo valida pertenencia; el cálculo usa la base embebida 2026e. Ambigüedad estricta: horas civiles ambiguas/inexistentes se rechazan (sin resolución compatible predeterminada); `1440` = medianoche del día civil siguiente; intervalos `[start,end)`; vigencias de reglas con ambos extremos incluidos.

**Verification:**
- [ ] Focused checks: cotejo estructural del bundle, documentación Jiff/tzdb y comprobación temporal de resolución de zonas bajo `target/`, sin código de producto.
- [ ] Build or broader check: `git diff --check`.
- [ ] Manual check: revisar ejemplos de regla, excepción, bloqueo y disponibilidad contra los contratos escritos.

**Dependencies:** Ninguna; requiere autorización expresa de inicio.

**Files likely touched:**
- `tasks/P-20261108-01-scheduling-availability/plan.md`
- `tasks/P-20261108-01-scheduling-availability/todo.md`

**Estimated scope:** M

## T02: Dominio de agenda

**Status:** Done

**Description:** Crear IDs, estados, patches tri-estado, validación de fechas/minutos/instantes y `EditBase`, sin almacenamiento ni framework.

**Acceptance criteria:**
- [x] Límites ISO/minutos, vigencias e instantes validan los valores contractuales.
- [x] `closed`, `replacement` y bloqueos aplican sus invariantes y nulabilidad.
- [x] Patches vacíos o nulos inválidos se rechazan y la ausencia de diferencia produce `NO_CHANGES`.

**Evidencia 2026-11-08 (T02):**

- Nuevo `src-tauri/src/components/scheduling/{mod.rs,domain.rs}` registrado en `components/mod.rs`; sin almacenamiento, HTTP ni Tauri.
- IDs propios UUIDv7 (`RuleId`, `ExceptionId`, `BlockId`, `ProfessionalId`, `AccountId`) con macro local; `shared_kernel` no se amplía. `SchedulingError` con códigos estables `VALIDATION_ERROR`/`NO_CHANGES`/`INTERNAL_ERROR`.
- `Status` activo/inactivo; minutos `LocalStartMinute` 0–1439 y `LocalEndMinute` 1–1440 con inicio<fin; vigencias con ambos extremos inclusivos; día ISO 1–7; `ReasonCode` vacation/administrative/unavailable.
- Excepciones: `closed` exige minutos nulos; `replacement` exige ambos e inicio<fin; el patch revalida el estado resultante (p. ej. pasar a `closed` conservando minutos rechaza).
- Bloqueos: instantes RFC 3339 normalizados a UTC (`Instant` sobre `jiff::Timestamp`), inicio<fin, `accountId` nullable = global, `reasonCode` obligatorio; `BlockPatch` solo tiempos/motivo (ámbito y profesional inmutables).
- Patches tri-estado (`RulePatch` con `validUntil: Option<Option<Date>>`): ausencia conserva, `None` retira solo donde se admite; patch vacío rechazado; igualdad con la base → `NO_CHANGES`.
- `EditBase::create()` (versión 0), `edit_accepted` (versión positiva) y `edit_local` (última operación local, UUIDv7); decisión registrada: base local usa `expectedVersion=0` con `baseOperationId`.
- Conversión civil→UTC estricta `civil_interval_to_utc` con Jiff: rechaza gap/fold vía `TimeZone::to_ambiguous_timestamp(..).unambiguous()` (sin resolución compatible); `1440` = medianoche del día civil siguiente. Fuentes: [jiff 0.2.38 `AmbiguousTimestamp::unambiguous`](https://docs.rs/jiff/0.2.38/jiff/tz/struct.AmbiguousTimestamp.html#method.unambiguous), [`Disambiguation`](https://docs.rs/jiff/0.2.38/jiff/tz/enum.Disambiguation.html), [`Timestamp` FromStr](https://docs.rs/jiff/0.2.38/jiff/struct.Timestamp.html#impl-FromStr-for-Timestamp).
- Dependencia directa `jiff = "=0.2.38"` con feature `tzdb-bundle-always` en `src-tauri/Cargo.toml`; `Cargo.lock` solo añade el borde `jiff` a `nightingale` (sin cambios de versión). Aritmética de zona verificada contra Jiff (DST 2026e).
- Verificación: `cargo test --workspace --locked components::scheduling::domain` → **7/7**; `cargo check --workspace --all-targets --locked` OK; `git diff --check` OK. Matriz manual de límites, DST (gap/fold), 1440 y patches representativos ejecutada en las pruebas puras.

**Verification:**
- [ ] Focused tests: `cargo test --workspace --locked components::scheduling::domain`.
- [ ] Build or broader check: `cargo check --workspace --all-targets --locked` y `git diff --check`.
- [ ] Manual check: ejercitar desde pruebas puras límites, DST y patches representativos.

**Dependencies:** T01.

**Files likely touched:**
- `src-tauri/src/components/scheduling/domain.rs`
- `src-tauri/src/components/scheduling/mod.rs`
- `src-tauri/src/components/mod.rs`
- `src-tauri/Cargo.toml`
- `Cargo.lock`

**Estimated scope:** M

## T03: Persistencia de Scheduling

**Status:** Done

**Description:** Añadir `0008_scheduling`, vistas aceptadas, tabla de intenciones, índices y repositorios básicos con ámbito de cuenta/usuario/dispositivo.

**Acceptance criteria:**
- [x] La migración desde esquema 7 conserva los datos existentes y reabre sin repetir cambios.
- [x] Un fallo de migración revierte archivo, historial y versión sin pérdida.
- [x] Las consultas aíslan el ámbito y las vistas no eliminan intenciones por cascada.

**Evidencia 2026-11-08 (T03):**

- `src-tauri/migrations/0008_scheduling.sql` registrada en `platform/sqlite/mod.rs` (versión 8, sin colisiones; `0001`–`0007` intactas). Tablas:
  - `schedule_rule_view` (PK ámbito+`rule_id`): día ISO 1–7, inicio 0–1439, fin 1–1440 con inicio<fin, `valid_from`/`valid_until` con CHECK de vigencia inclusiva, `status`, `version>0`, `created_at`/`updated_at` (transporte remoto). Índices por profesional/día/estado y por vigencia.
  - `schedule_exception_view`: una fila por excepción; CHECK `closed`⇒minutos nulos, `replacement`⇒ambos minutos 0–1439/1–1440 e inicio<fin. Índice por profesional/fecha/estado.
  - `schedule_block_view`: `block_account_id` NULL = global (decisión de nombre: la columna de ámbito del bloqueo se llama `block_account_id` para no colisionar con el `account_id` del ámbito triple); `starts_at<ends_at` en UTC; `reason_code` nullable CHECK (vacation/administrative/unavailable) para audiencia. Índices por ventana profesional y por cuenta.
  - `schedule_local`: intenciones con `entity_type` (schedule_rule/exception/block), IDs propios, las nueve acciones con CHECK por entidad, `expected_version>=0`, `base_operation_id`, `depends_on` JSON ≤8, `payload`/`draft` JSON objeto, `created_at`. Sin columna `provisional`: el carácter provisional se deriva de `sync_outbox` vía `SqliteOperationStore.operation_result` según el plan, sin duplicar la máquina de sincronización.
- Adaptadores: `components/scheduling/adapters/{mod.rs,outbound.rs}` con `RuleRecord`/`ExceptionRecord`/`BlockRecord`/`LocalChange` y lecturas/escrituras (upsert) aísladas por `(account_id,user_id,device_id)`, siguiendo el patrón de Patients (`save_*` sobre `&Transaction`, lecturas sobre `&Connection`).
- `repository_tests.rs` (4 pruebas): supervivencia a reapertura sin red; aislamiento de ámbito (otra cuenta no contamina; otras vistas vacías) y conservación de intenciones al vaciar vistas (sin cascadas); CHECK constraints contractuales rechazan regla invertida, `closed` con minutos, bloqueo invertido y acción ajena a la entidad; migración 7→8 conserva datos de pacientes y termina en `user_version=8` con reapertura sin cambios.
- Plataforma: prueba de pacientes existente actualizada a `MIGRATIONS.last()` en lugar del literal `7` (único cambio en `platform/sqlite/mod.rs` aparte del registro); 19/19 pruebas `platform::sqlite` en verde, incluidos rollback de migración fallida byte a byte (cubierto por la suite para migraciones previas y por el patrón común del runner).
- Verificación: `cargo test --workspace --locked components::scheduling` → **11/11**; `platform::sqlite` → **19/19**; `cargo check --workspace --all-targets --locked` sin warnings; `git diff --check` OK. Comprobación manual: migración temporal 7→8 reabierta e inspección de esquema y de otra cuenta en las pruevas de repositorio.

**Verification:**
- [ ] Focused tests: `cargo test --workspace --locked components::scheduling` y `cargo test --workspace --locked platform::sqlite::tests`.
- [ ] Build or broader check: `cargo check --workspace --all-targets --locked` y `git diff --check`.
- [ ] Manual check: migrar una base temporal desde 7, reabrirla e inspeccionar esquema y datos de otra cuenta.

**Dependencies:** T02.

**Files likely touched:**
- `src-tauri/migrations/0008_scheduling.sql`
- `src-tauri/src/platform/sqlite/mod.rs`
- `src-tauri/src/components/scheduling/adapters/mod.rs`
- `src-tauri/src/components/scheduling/adapters/outbound.rs`
- `src-tauri/src/components/scheduling/repository_tests.rs`

**Estimated scope:** M

## T04: Ocupación aceptada de Citas

**Status:** Done

**Description:** Añadir `0009_appointment_occupancy`, el puerto `ScheduleOccupancyQuery` y un repositorio de solo lectura por profesional e intervalo, sin casos de escritura de Citas.

**Acceptance criteria:**
- [x] La vista representa solo estados ocupantes, claves, intervalos y versiones correctos.
- [x] Las consultas usan índices y mantienen aislamiento por cuenta/usuario/dispositivo.
- [x] La API pública no expone SQL, HTTP ni modelos de transporte.

**Evidencia 2026-11-08 (T04):**

- `src-tauri/migrations/0009_appointment_occupancy.sql` registrada como versión 9. Tabla `appointment_occupancy_view` (PK ámbito+`appointment_id`): `professional_id`, `starts_at<ends_at` (UTC), `version>0` y CHECK de estado limitado a `scheduled/confirmed/completed/no_show` — `cancelled` libera ocupación y no se almacena (el proyector futuro lo retira). Índice `(account,user,device,professional,starts_at,ends_at,status)`.
- Componente `appointments` mínimo: `application.rs` define el contrato público `ScheduleOccupancyQuery::list_occupancy(professional, from, until)` (intersección `[from,until)`, contigüidad permitida) y `OccupancyInterval` validado (inicio<fin, solo estados ocupantes); sin casos de escritura de Citas. `adapters/outbound.rs` implementa el puerto con `SqliteScheduleOccupancyQuery` (ámbito triple, sin SQL expuesto al consumidor, sin escrituras).
- Decisiones registradas: errores del puerto con `NightingaleError` compartido (evita un tipo de error paralelo para un único puerto de lectura); instantes del contrato público como `jiff::Timestamp` (fuente: [`Timestamp` FromStr](https://docs.rs/jiff/0.2.38/jiff/struct.Timestamp.html#impl-FromStr-for-Timestamp)).
- `repository_tests.rs` (3 pruebas): solo estados ocupantes en ventana y profesional correctos (contiguas devueltas ambas, otra profesional y otra cuenta excluidas, reapertura conserva); CHECK rechaza `cancelled` e intervalo invertido; migración 8→9 conserva datos de Scheduling con reapertura sin cambios.
- Skills `.agents/minimal-code` y `.agents/docs-first-framework-code` leídas y aplicadas a T02–T04: citas de fuente junto al patrón Jiff estricto, cambios quirúrgicos, estilo de espaciado y patrones de Patients/Services.
- Verificación: `cargo test --workspace --locked components::appointments` → **3/3**; comprobaciones amplias en C01.

**Verification:**
- [ ] Focused tests: `cargo test --workspace --locked components::appointments`.
- [ ] Build or broader check: `cargo check --workspace --all-targets --locked` y `git diff --check`.
- [ ] Manual check: migrar 8→9 y consultar estados contiguos, ocupantes y cancelados tras reapertura.

**Dependencies:** T03.

**Files likely touched:**
- `src-tauri/migrations/0009_appointment_occupancy.sql`
- `src-tauri/src/components/appointments/mod.rs`
- `src-tauri/src/components/appointments/application.rs`
- `src-tauri/src/components/appointments/adapters/outbound.rs`
- `src-tauri/src/components/appointments/repository_tests.rs`

**Estimated scope:** M

## C01: Contratos y persistencia

**Status:** Aprobado por el usuario el 2026-11-08.

- [x] Las pruebas de dominio, migración y repositorios pasan junto con `cargo test --workspace --locked`.
- [x] Contratos, zonas, tablas, puertos y puntos calientes tienen revisión del usuario.
- [x] No se implementan casos de Citas ni cambios en contratos o migraciones anteriores.
- [x] `pnpm build:desktop` y `git diff --check` pasan.

**Evidencia 2026-11-08 (C01):**

- `cargo test --workspace --locked`: **249 passed, 0 failed** (243 lib + 5 `core_persistence` + 1 bin/doc), 7 ignored preexistentes (pruebas de desarrollo con entorno real, sin relación con este plan).
- Ajustes de integración por el registro de migraciones 8–9 (único punto caliente): `platform/sqlite/mod.rs` (registro + expectativa dinámica), `identity/activation_tests.rs` (`localSchemaVersion` 7→9, se lee de `user_version` en vivo), `tests/core_persistence.rs` (conteo de migraciones 7→9). `0001`–`0007` intactas; contrato importado y sus cinco huellas sin tocar.
- `git diff --check` OK. Archivos creados: migraciones `0008_scheduling.sql`/`0009_appointment_occupancy.sql`, componentes `scheduling/` y `appointments/`, y la carpeta del plan. Comprobación temporal de T01 bajo `target/scheduling-t01-tz-check/` (ignorado por Git). Sin archivos de log, bases de prueba ni artefactos ajenos creados por estas tareas.
- Trabajo ajeno preservado: apareció durante la sesión la carpeta no rastreada `tasks/P-20261108-02-appointments-core/`; no fue creada ni modificada por este trabajo.
- Pendiente de revisión del usuario: contratos congelados (T01), tablas e índices 0008/0009, puertos `ScheduleAvailabilityQuery` (definido en plan, se implementa en T10) y `ScheduleOccupancyQuery` (T04), y decisiones registradas (`block_account_id`, base local con `expectedVersion=0`, tzdb embebida 2026e frente a lista de referencia 2026f).

## T05: Intenciones y ediciones sucesivas

**Status:** Todo

**Description:** Implementar preparación común con `record_operation` en la misma transacción `IMMEDIATE`, bases aceptadas/locales, dependencias y provisionales durables.

**Acceptance criteria:**
- [ ] Provisional y Outbox se confirman o revierten juntos.
- [ ] Las secuencias sobreviven a reapertura y respetan el máximo de ocho dependencias sin ciclos ni duplicados.
- [ ] Bases antiguas/bloqueadas se rechazan y `NO_CHANGES` no crea operación.

**Verification:**
- [ ] Focused tests: `cargo test --workspace --locked components::scheduling`.
- [ ] Build or broader check: `cargo check --workspace --all-targets --locked` y `git diff --check`.
- [ ] Manual check: guardar varias ediciones offline, reabrir e inspeccionar IDs, bases y dependencias.

**Dependencies:** T04 y aprobación de C01.

**Files likely touched:**
- `src-tauri/src/components/scheduling/application.rs`
- `src-tauri/src/components/scheduling/adapters/intents.rs`
- `src-tauri/src/components/scheduling/adapters/outbound.rs`
- `src-tauri/src/components/scheduling/application_tests.rs`

**Estimated scope:** M

## T06: Reglas semanales

**Status:** Done

**Description:** Implementar creación, edición, estado y consulta de reglas con prevalidación contra ocupación local conocida.

**Acceptance criteria:**
- [x] Las tres acciones usan payloads, claves, versiones y dependencias contractuales.
- [x] Vigencias, minutos, permisos y consultas fusionadas respetan el dominio y el ámbito.
- [x] Un cambio incompatible con una cita conocida devuelve `SLOT_UNAVAILABLE` sin guardar intención.

**Evidencia 2026-11-08 (T06):**

- `application.rs`: `RuleState`, filas `EntityRow` aceptado/provisional/operaciones, `EditBase`, `SchedulingMutation` (tres acciones de regla), puerto ampliado `persist(..., occupancy: &impl ScheduleOccupancyQuery)` y `rule_rows_list`. Función pura `active_rules_cover` (zona civil, día ISO lunes=1, vigencia inclusiva, minutos envolventes, cita mult día no cubrible).
- `adapters/intents.rs`: preparación común con `edit_base` (base aceptada exige fila sin provisional y misma versión; base local exige `can_depend` en operaciones previas), `prepare` con payloads contractuales exactos (create sin `validUntil` cuando es abierta; update solo campos presentes con `validUntil: null` para retirar; status `{status}`).
- `adapters/outbound.rs`: `account_time_zone` consume `AccountViewQuery` (zona de la cuenta, nunca del dispositivo), `ensure_known_occupancy_fits` evalúa la ocupación conocida (puerto de Citas, rango completo) contra las reglas visibles con el candidato aplicado; fallo → `SLOT_UNAVAILABLE` antes de crear intención. Zona leída antes de la transacción `IMMEDIATE`; `record_operation` + `insert_local_change` + commit atómicos.
- `test_support.rs` (patrón Patients): `Ids`, `FixedClock`, `NoOccupancy`, `FakeOccupancy` (por profesional), acceso con las nueve acciones, base con `account_view` en `America/Caracas`.
- `rule_tests.rs` (5 pruebas): payloads contractuales de las tres acciones; `SLOT_UNAVAILABLE` en create que deja cita fuera, update que acorta y retirada de la regla — sin intención creada; lista fusionada aceptado+provisional+operaciones; edición sobre base aceptada sin provisional; ocupación de otro profesional no bloquea.
- Límite registrado: la prevalidación cubre reglas (excepciones entran en T07) y nunca infiere seguridad por datos no descargados.
- Verificación: `cargo test --workspace --locked components::scheduling` → **20/20**; `cargo check --workspace --all-targets --locked` sin warnings; `git diff --check` OK. Recorrido manual: regla creada offline, dos ediciones y reapertura antes de sincronizar cubierto en `application_tests`.

**Verification:**
- [ ] Focused tests: `cargo test --workspace --locked components::scheduling`.
- [ ] Build or broader check: `cargo test --workspace --locked` y `git diff --check`.
- [ ] Manual check: crear una regla offline, editarla dos veces y reabrir antes de sincronizar.

**Dependencies:** T05.

**Files likely touched:**
- `src-tauri/src/components/scheduling/domain.rs`
- `src-tauri/src/components/scheduling/application.rs`
- `src-tauri/src/components/scheduling/adapters/outbound.rs`
- `src-tauri/src/components/scheduling/rule_tests.rs`

**Estimated scope:** M

## T07: Excepciones

**Status:** Done

**Description:** Implementar creación, edición, estado y consulta de cierres y reemplazos por fecha.

**Acceptance criteria:**
- [x] Una excepción efectiva por profesional/fecha reemplaza el horario habitual y `closed` lo deja vacío.
- [x] Minutos, nulabilidad, versiones y dependencias se validan sobre el estado resultante.
- [x] Impactos conocidos y fallos de escritura conservan el trabajo previo sin operaciones parciales.

**Evidencia 2026-11-08 (T07):**

- `ExceptionState` y las tres acciones de excepción en `SchedulingMutation` con `entity_type` generalizado (`schedule_rule`/`schedule_exception`); `PreparedChange` ahora lleva los tres identificadores posibles (bloqueo llega en T08, donde `block_id` se pone en uso).
- Preparación (`intents.rs`): payload create `{professionalId,localDate,kind}` más minutos solo en `replacement`; update solo campos presentes; status `{status}`. Creación rechaza una segunda excepción activa para la misma profesional/fecha (`VALIDATION_ERROR`); edición revalida el estado resultante vía `ExceptionPatch::apply`.
- Prevalidación ampliada: `visible_schedule_covers` da prioridad a la excepción activa de la fecha civil sobre las reglas (`closed` nunca cubre; `replacement` debe contener los minutos de la cita). `ensure_known_occupancy_fits` fusiona reglas y excepciones del profesional con el candidato aplicado; el `SLOT_UNAVAILABLE` aborta antes de `record_operation`.
- Consultas fusionadas: `exception_rows_list` (aceptado + provisional + operaciones) simétrica a reglas.
- `exception_tests.rs` (4 pruebas): `closed`/`replacement` offline con payloads contractuales y reapertura con ambas intenciones; duplicado activo por fecha rechazado y fecha distinta permitida; `closed` sobre la fecha de una cita conocida rechazado, reemplazo insuficiente al acortar rechazado, retirada permitida cuando la regla vuelve a cubrir; update a `closed` conservando minutos rechazado y `NO_CHANGES` en estado idéntico.
- Warning transitorio: `block_id` sin uso hasta T08 (inmediato).
- Verificación: `cargo test --workspace --locked components::scheduling` → **24/24**; `git diff --check` OK. Manual: `closed` y `replacement` preparados offline y reapertura cubiertos en las pruebas.

**Verification:**
- [ ] Focused tests: `cargo test --workspace --locked components::scheduling`.
- [ ] Build or broader check: `cargo check --workspace --all-targets --locked` y `git diff --check`.
- [ ] Manual check: preparar `closed` y `replacement` offline, reabrir y revisar su efecto y dependencias.

**Dependencies:** T06.

**Files likely touched:**
- `src-tauri/src/components/scheduling/domain.rs`
- `src-tauri/src/components/scheduling/application.rs`
- `src-tauri/src/components/scheduling/adapters/outbound.rs`
- `src-tauri/src/components/scheduling/exception_tests.rs`

**Estimated scope:** M

## T08: Bloqueos de cuenta y globales

**Status:** Done

**Description:** Implementar creación, edición, estado y consulta de bloqueos, con privacidad del motivo y control propietario/profesional propio para globales.

**Acceptance criteria:**
- [x] El alcance usa la cuenta actual o `accountId=null` global y no cambia mediante patch.
- [x] Un asistente no crea globales ni infiere un motivo oculto; Rust valida propiedad y acciones.
- [x] Un solapamiento conocido devuelve conflicto sin nueva operación y conserva el trabajo previo.

**Evidencia 2026-11-08 (T08):**

- `BlockState` y las tres acciones de bloqueo; `entity_type` `schedule_block`; `PreparedChange::block` pone en uso `block_id` (warning transitorio de T07 resuelto).
- Permisos: `CreateBlock` global exige `AccessRole::Owner` **y** que `professional_view.professional_user_id` del profesional sea el usuario de la sesión, consultado por el puerto `CatalogQuery::list_professionals` (no se leen tablas de Services directamente). La verificación se hace antes de abrir la transacción `IMMEDIATE`; el motivo (`reasonCode`) lo exige el dominio en la creación y un asistente jamás lo inventa (la ocultación por audiencia queda en proyección, T12).
- Ámbito inmutable: `BlockPatch` solo tiempos/motivo (dominio T02); test contractual confirma que `blockAccountId` y `professionalId` se conservan tras el patch.
- Prevalidación ampliada: `intersects_active_block` rechaza bloqueos activos (cuenta o global) que intersecten una cita conocida, junto a la cobertura de reglas/excepciones; mismo código `SLOT_UNAVAILABLE` y mensaje "fuera del horario o dentro de un bloqueo" según plan §3 (la redacción "conflicto" del criterio se interpreta como rechazo sin operación; decisión registrada).
- `block_rows_list` fusionada (aceptado+provisional+operaciones) simétrica a las demás.
- `block_tests.rs` (5 pruebas): propietario crea bloqueo de cuenta y global con payloads contractuales (`accountId: null`, `reasonCode`); asistente global denegado, propietario sobre profesional ajeno denegado, asistente de cuenta permitido; patch no cambia ámbito ni profesional y viaja solo tiempos/motivo; cita conocida dentro de bloqueo rechazado en create (cuenta y global) y en update ampliado — sin intención creada y con el trabajo previo conservado; ediciones sucesivas de bloqueo sobreviven a reapertura.
- Verificación: `cargo test --workspace --locked components::scheduling` → **29/29**; comprobaciones amplias en C02. Manual: propietario/asistente, global/de cuenta y reapertura ejercidos en las pruebas.

**Verification:**
- [ ] Focused tests: `cargo test --workspace --locked components::scheduling`.
- [ ] Build or broader check: `cargo test --workspace --locked` y `git diff --check`.
- [ ] Manual check: ejercitar propietario/asistente, bloqueo global/de cuenta, motivo oculto y reapertura.

**Dependencies:** T07.

**Files likely touched:**
- `src-tauri/src/components/scheduling/domain.rs`
- `src-tauri/src/components/scheduling/application.rs`
- `src-tauri/src/components/scheduling/adapters/outbound.rs`
- `src-tauri/src/components/scheduling/block_tests.rs`

**Estimated scope:** M

## C02: Escrituras nativas

**Status:** Pendiente de revisión del usuario (no aprobado por el agente).

- [x] Las nueve acciones guardan offline, reabren y respetan permisos, versiones y dependencias.
- [x] `cargo test --workspace --locked` y `pnpm build:desktop` pasan; conflictos conocidos no crean efectos parciales.
- [ ] El usuario revisa reglas, excepciones y bloqueos antes de disponibilidad.
- [x] `git diff --check` pasa.

**Evidencia 2026-11-08 (C02):**

- `cargo test --workspace --locked`: **267 passed, 0 failed** (261 lib + 5 `core_persistence` + 1 bin), 7 ignored preexistentes. Pruebas de Scheduling: 29/29 (7 dominio, 4 repositorio, 4 intenciones, 5 reglas, 4 excepciones, 5 bloqueos).
- `pnpm build:desktop`: OK (`target/debug/nightingale.exe`).
- `git diff --check`: OK. Sin archivos de log ni bases de prueba huérfanas; `tasks/P-20261108-02-appointments-core/` ajena sigue intacta.
- Conflictos conocidos sin efectos parciales: `SLOT_UNAVAILABLE`, `VERSION_CONFLICT`, `DEPENDENCY_FAILED`, `NO_CHANGES`, `PERMISSION_DENIED` y `VALIDATION_ERROR` se evalúan antes de `record_operation` dentro de la misma transacción; los tests assertan `sync_outbox`/`schedule_local` sin filas nuevas tras el rechazo.
- Recorridos para revisión del usuario: reglas (`rule_tests.rs`), excepciones (`exception_tests.rs`) y bloqueos (`block_tests.rs`), con evidencia por tarea en este archivo.

## T09: Motor puro de intervalos

**Status:** Todo

**Description:** Implementar unión, intersección, sustracción, conversión civil→UTC y filtrado por duración sin almacenamiento ni red.

**Acceptance criteria:**
- [ ] Excepciones preceden reglas y bloqueos/ocupación se restan correctamente.
- [ ] DST gap/fold se rechazan y `1440` usa el día civil siguiente.
- [ ] Contiguos, solapamientos y bordes `[start,end)` producen intervalos exactos.

**Verification:**
- [ ] Focused tests: `cargo test --workspace --locked components::scheduling::availability`.
- [ ] Build or broader check: `cargo check --workspace --all-targets --locked` y `git diff --check`.
- [ ] Manual check: ejecutar la matriz determinista de DST, vigencias, contigüidad y duración.

**Dependencies:** T08 y aprobación de C02.

**Files likely touched:**
- `src-tauri/src/components/scheduling/availability.rs`
- `src-tauri/src/components/scheduling/domain.rs`
- `src-tauri/src/components/scheduling/availability_tests.rs`

**Estimated scope:** M

## T10: Consulta local con cobertura

**Status:** Todo

**Description:** Componer `AccountViewQuery`, `CatalogQuery`, agenda, ocupación y `CursorStore` en el puerto `ScheduleAvailabilityQuery`.

**Acceptance criteria:**
- [ ] Duración, versiones y estados aceptados de cuenta/catálogo/profesional son correctos.
- [ ] Cobertura dentro/sin/fuera de ventana nunca se convierte en un falso “sin huecos” o “libre”.
- [ ] La consulta limita el rango a 31 días, valida duración y no ejecuta HTTP.

**Verification:**
- [ ] Focused tests: `cargo test --workspace --locked components::scheduling`.
- [ ] Build or broader check: `cargo check --workspace --all-targets --locked` y `git diff --check`.
- [ ] Manual check: consultar dentro de ventana, sin captura y fuera de ventana, distinguiendo cobertura desconocida.

**Dependencies:** T09.

**Files likely touched:**
- `src-tauri/src/components/scheduling/application.rs`
- `src-tauri/src/components/scheduling/availability.rs`
- `src-tauri/src/components/scheduling/adapters/outbound.rs`
- `src-tauri/src/components/scheduling/availability_query_tests.rs`

**Estimated scope:** M

## T11: Consulta remota API

**Status:** Todo

**Description:** Implementar adaptador tipado `GET /availability`, puerta de sesión y resultado remoto no persistido.

**Acceptance criteria:**
- [ ] Query y respuesta respetan IDs, duraciones, intervalos y máximo de 31 días del contrato.
- [ ] Se refresca acceso y no se mantiene mutex SQLite durante HTTP ni se escribe Outbox/vista aceptada.
- [ ] Errores 401/403/404/422/503 conservan la vista local y devuelven códigos seguros.

**Verification:**
- [ ] Focused tests: `cargo test --workspace --locked components::scheduling::online`.
- [ ] Build or broader check: `cargo check --workspace --all-targets --locked` y `git diff --check`.
- [ ] Manual check: ejecutar recorrido loopback de éxito, respuesta inválida y fallo remoto.

**Dependencies:** T10.

**Files likely touched:**
- `src-tauri/src/components/scheduling/adapters/api.rs`
- `src-tauri/src/components/scheduling/adapters/online.rs`
- `src-tauri/src/components/scheduling/adapters/desktop.rs`
- `src-tauri/src/components/scheduling/application.rs`
- `src-tauri/src/components/scheduling/online_tests.rs`

**Estimated scope:** M

## C03: Disponibilidad nativa y remota

- [ ] Reglas, excepciones, bloqueos, ocupación, DST y cobertura tienen matriz verde.
- [ ] La consulta remota loopback no convierte respuesta en reserva o dato offline.
- [ ] El usuario revisa semántica, cobertura y mensajes de disponibilidad.
- [ ] `cargo test --workspace --locked` y `git diff --check` pasan.

## T12: Proyectores de agenda y ocupación

**Status:** Todo

**Description:** Implementar SchedulingProjector y AppointmentOccupancyProjector e integrarlos en DesktopProjector con la transacción recibida.

**Acceptance criteria:**
- [ ] Upsert valida ámbito, claves, tiempos, estados y no retrocede versiones.
- [ ] `remove`/`reset` retiran solo vistas del ámbito y conservan intenciones y otras cuentas.
- [ ] Un fallo revierte grupo, proyecciones, cursor e Inbox sin efectos externos.

**Verification:**
- [ ] Focused tests: `cargo test --workspace --locked components::scheduling` y `cargo test --workspace --locked components::appointments`.
- [ ] Build or broader check: `cargo check --workspace --all-targets --locked` y `git diff --check`.
- [ ] Manual check: aplicar un grupo repetido, retirar vistas y reconstruir una captura conservando intenciones.

**Dependencies:** T11 y aprobación de C03.

**Files likely touched:**
- `src-tauri/src/components/scheduling/adapters/projector.rs`
- `src-tauri/src/components/appointments/adapters/projector.rs`
- `src-tauri/src/components/projector.rs`
- `src-tauri/src/components/scheduling/projection_tests.rs`
- `src-tauri/src/components/appointments/projection_tests.rs`

**Estimated scope:** M

## T13: Conciliación de provisionales

**Status:** Todo

**Description:** Implementar lecturas fusionadas y resolución por recibo+vista, conservando conflictos, rechazos, resultados desconocidos y dependencias.

**Acceptance criteria:**
- [ ] Un recibo aceptado sin descarga no retira el provisional.
- [ ] Cuando la vista alcanza la versión confirmada se retira solo el provisional confirmado.
- [ ] Rechazo, conflicto y `unknown` conservan secuencia, IDs y dependencias para revisión.

**Verification:**
- [ ] Focused tests: `cargo test --workspace --locked components::scheduling` y `cargo test --workspace --locked components::sync`.
- [ ] Build or broader check: `cargo check --workspace --all-targets --locked` y `git diff --check`.
- [ ] Manual check: simular aceptación parcial, reapertura y `remove`, verificando operaciones posteriores.

**Dependencies:** T12.

**Files likely touched:**
- `src-tauri/src/components/scheduling/adapters/intents.rs`
- `src-tauri/src/components/scheduling/adapters/outbound.rs`
- `src-tauri/src/components/scheduling/application.rs`
- `src-tauri/src/components/scheduling/reconciliation_tests.rs`

**Estimated scope:** M

## C04: Sincronización completa del componente

- [ ] Proyección, captura, `remove`, recibos parciales y reapertura conservan trabajo local.
- [ ] `cargo test --workspace --locked` y `pnpm build:desktop` pasan.
- [ ] El usuario revisa conciliación antes de IPC/UI.
- [ ] `git diff --check` pasa.

## T14: Comandos de escritura

**Status:** Todo

**Description:** Añadir nueve comandos Tauri con DTO tri-estado, bases, permisos y errores seguros para reglas, excepciones y bloqueos.

**Acceptance criteria:**
- [ ] Acciones y payloads son exactos y no admiten ámbito, credenciales o envelope arbitrario.
- [ ] Propietario, asistente y bloqueos globales se autorizan nuevamente en Rust.
- [ ] Campos extra, bases ambiguas y nulos inválidos se rechazan sin efectos.

**Verification:**
- [ ] Focused tests: `cargo test --workspace --locked components::scheduling::ipc`.
- [ ] Build or broader check: `cargo check --workspace --all-targets --locked` y `git diff --check`.
- [ ] Manual check: invocar comandos sintéticos autorizados y denegados por el camino real de entrada.

**Dependencies:** T13 y aprobación de C04.

**Files likely touched:**
- `src-tauri/src/components/scheduling/adapters/inbound_rules.rs`
- `src-tauri/src/components/scheduling/adapters/inbound_exceptions.rs`
- `src-tauri/src/components/scheduling/adapters/inbound_blocks.rs`
- `src-tauri/src/components/scheduling/adapters/mod.rs`
- `src-tauri/src/components/scheduling/ipc_write_tests.rs`

**Estimated scope:** M

## T15: Comandos de consulta

**Status:** Todo

**Description:** Añadir tres listados y disponibilidad local/remota con salidas separadas y sin secretos.

**Acceptance criteria:**
- [ ] Filtros, rangos, versiones y cobertura responden según los contratos aprobados.
- [ ] Aceptado, provisional y operaciones permanecen separados.
- [ ] Las salidas no exponen época, fingerprint, token, SQL ni estado remoto crudo.

**Verification:**
- [ ] Focused tests: `cargo test --workspace --locked components::scheduling::ipc`.
- [ ] Build or broader check: `cargo check --workspace --all-targets --locked` y `git diff --check`.
- [ ] Manual check: invocar comandos sin sesión, con permiso insuficiente y con cobertura insuficiente.

**Dependencies:** T14.

**Files likely touched:**
- `src-tauri/src/components/scheduling/adapters/inbound_queries.rs`
- `src-tauri/src/components/scheduling/adapters/inbound_availability.rs`
- `src-tauri/src/components/scheduling/adapters/mod.rs`
- `src-tauri/src/components/scheduling/ipc_query_tests.rs`

**Estimated scope:** M

## T16: Registro y cliente TypeScript

**Status:** Todo

**Description:** Registrar los catorce comandos y permisos en handler, build/capabilities y crear `src/shared/api/scheduling.ts`.

**Acceptance criteria:**
- [ ] Handler, manifiesto y permisos coinciden para los catorce comandos.
- [ ] Los DTO TypeScript camelCase cubren consultas, escrituras, disponibilidad y errores.
- [ ] La compilación pasa sin nuevas dependencias JavaScript.

**Verification:**
- [ ] Focused tests: prueba de coherencia de registro Scheduling.
- [ ] Build or broader check: `pnpm build` y `pnpm build:desktop`.
- [ ] Manual check: abrir el ejecutable y verificar una invocación autorizada y una denegada mediante el arnés IPC previsto.

**Dependencies:** T15.

**Files likely touched:**
- `src-tauri/src/lib.rs`
- `src-tauri/build.rs`
- `src-tauri/capabilities/default.json`
- `src/shared/api/scheduling.ts`
- `src-tauri/src/components/scheduling/ipc_registration_tests.rs`

**Estimated scope:** M

## T17: UI de Agenda

**Status:** Todo

**Description:** Crear el panel de reglas, excepciones y bloqueos con formularios, estados, filtros y enlaces a revisiones.

**Acceptance criteria:**
- [ ] Todos los casos son utilizables y accesibles con datos aceptados y provisionales.
- [ ] Bloqueos globales solo aparecen cuando procede y el motivo oculto se presenta sin invención.
- [ ] Provisionales, conflictos y rechazos no se presentan como aceptados.

**Verification:**
- [ ] Focused checks: `pnpm build`.
- [ ] Build or broader check: `pnpm build:desktop` y `git diff --check`.
- [ ] Manual check: recorrido Windows con datos sintéticos offline y reapertura.

**Dependencies:** T16.

**Files likely touched:**
- `src/features/scheduling/SchedulingPanel.tsx`
- `src/features/scheduling/AgendaEditor.tsx`
- `src/App.tsx`
- `src/shared/api/scheduling.ts`

**Estimated scope:** M

## T18: UI de Disponibilidad

**Status:** Todo

**Description:** Crear la consulta local/remota con selección de profesional/servicio/rango/duración, zona, intervalos, cobertura y errores diferenciados.

**Acceptance criteria:**
- [ ] “Sin huecos” nunca se confunde con cobertura insuficiente o consulta fallida.
- [ ] Resultados locales provisionales y remotos se distinguen visualmente.
- [ ] React no calcula disponibilidad, versiones ni permisos críticos.

**Verification:**
- [ ] Focused checks: `pnpm build`.
- [ ] Build or broader check: `pnpm build:desktop` y `git diff --check`.
- [ ] Manual check: recorrer disponibilidad local, fuera de ventana, API caída y respuesta remota.

**Dependencies:** T17.

**Files likely touched:**
- `src/features/scheduling/AvailabilityPanel.tsx`
- `src/features/scheduling/SchedulingPanel.tsx`
- `src/App.tsx`
- `src/shared/api/scheduling.ts`

**Estimated scope:** M

## C05: Vertical usable desde escritorio

- [ ] Los catorce comandos y permisos son coherentes y la UI local/remota funciona con datos sintéticos.
- [ ] `pnpm quality` y `pnpm build:desktop` pasan; la revisión manual Windows queda registrada.
- [ ] El usuario revisa la vertical antes de pruebas finales.
- [ ] `git diff --check` pasa.

## T19: Fixtures del consumidor

**Status:** Todo

**Description:** Contrastar nueve acciones, tres estados de agenda, ocupación y respuesta Availability contra el bundle fijo.

**Acceptance criteria:**
- [ ] Acciones, payloadModels, claves y disponibilidad offline coinciden con capabilities/esquemas.
- [ ] Estados remotos y query/response de disponibilidad no presentan deriva estructural.
- [ ] Las cinco huellas del bundle permanecen intactas.

**Verification:**
- [ ] Focused tests: `cargo test --workspace --locked components::sync::contract_tests`.
- [ ] Build or broader check: `cargo check --workspace --all-targets --locked` y `git diff --check`.
- [ ] Manual check: revisar una fixture por grupo y una consulta/respuesta Availability contra OpenAPI.

**Dependencies:** T18 y aprobación de C05.

**Files likely touched:**
- `src-tauri/src/components/sync/contract_tests.rs`
- `src-tauri/src/components/sync/fixtures/scheduling-operations.json`
- `src-tauri/src/components/sync/fixtures/scheduling-availability.json`

**Estimated scope:** S

## T20: Integración loopback de dos perfiles

**Status:** Todo

**Description:** Ejecutar offline→reapertura→dispatch→accepted→proyección en dos SQLite, con agenda y ocupación.

**Acceptance criteria:**
- [ ] Reglas, excepciones y bloqueos sincronizan y calculan igual dentro de la ventana cubierta.
- [ ] Conflicto, respuesta perdida y captura conservan intenciones y evitan duplicados.
- [ ] API/SSE caídos no bloquean casos locales y ninguna petición HTTP mantiene SQLite bloqueada.

**Verification:**
- [ ] Focused tests: `cargo test --workspace --locked scheduling_integration_tests`.
- [ ] Build or broader check: `cargo test --workspace --locked` y `git diff --check`.
- [ ] Manual check: recorrer la matriz documentada de dos perfiles, conflicto, recuperación y cobertura.

**Dependencies:** T19.

**Files likely touched:**
- `src-tauri/src/components/sync/scheduling_integration_tests.rs`
- `src-tauri/src/components/sync/mod.rs`
- `src-tauri/src/components/scheduling/test_support.rs`
- `src-tauri/src/components/scheduling/mod.rs`

**Estimated scope:** M

## T21: Guías y calidad

**Status:** Todo

**Description:** Documentar Scheduling, ocupación, disponibilidad, comandos de ejecución y límites, sin alterar especificaciones ni afirmar verificación externa ausente.

**Acceptance criteria:**
- [ ] Las guías coinciden con implementación y separan lo verificado de lo pendiente.
- [ ] Calidad, build y diff pasan sobre la entrega completa.
- [ ] No quedan artefactos de prueba, bases, secretos ni logs en Git.

**Verification:**
- [ ] Focused tests: `cargo test --workspace --locked components::scheduling`, `cargo test --workspace --locked components::appointments`, `cargo test --workspace --locked components::sync::contract_tests` y `cargo test --workspace --locked scheduling_integration_tests`.
- [ ] Build or broader check: `pnpm quality`, `pnpm build:desktop`, `git diff --check` y `git status`.
- [ ] Manual check: reproducir los recorridos documentados y revisar enlaces y límites.

**Dependencies:** T20.

**Files likely touched:**
- `docs/technical/components/scheduling.md`
- `docs/operations/core-next-tracks.md`
- `docs/operations/development.md`
- `docs/README.md`
- `tasks/P-20261108-01-scheduling-availability/todo.md`

**Estimated scope:** M

## T22: API/OIDC real de desarrollo

**Status:** Todo

**Description:** Ejecutar una prueba opt-in con adaptadores reales, dos dispositivos/perfiles y datos sintéticos, sin modificar Nightingale API ni otros repositorios.

**Acceptance criteria:**
- [ ] Las nueve operaciones y disponibilidad local/remota se verifican con aceptación y descarga reales.
- [ ] Dos dispositivos producen y recuperan conflicto y respuesta perdida sin duplicar efectos ni perder intenciones.
- [ ] Propietario/asistente y global/cuenta quedan comprobados, registrando comandos, datos sintéticos y límites sin secretos.

**Verification:**
- [ ] Focused tests: `cargo test --workspace --locked -- --ignored components::scheduling::development_tests` con el entorno documentado en T21; un bloqueo no cuenta como éxito.
- [ ] Build or broader check: `cargo test --workspace --locked components::scheduling` y `git diff --check`.
- [ ] Manual check: completar login OIDC, revisar disponibilidad remota y reapertura en ambos perfiles.

**Dependencies:** T21.

**Files likely touched:**
- `src-tauri/src/components/scheduling/development_tests.rs`
- `src-tauri/src/components/scheduling/mod.rs`
- `tasks/P-20261108-01-scheduling-availability/todo.md`
- `tasks/P-20261108-01-scheduling-availability/evidence-c06.md`

**Estimated scope:** M

## C06: Aceptación de Agenda y Disponibilidad

- [ ] Suite, contrato, integración loopback, calidad y build de escritorio pasan.
- [ ] La prueba real API/OIDC y el recorrido UI tienen evidencia; todo límite no ejecutado permanece explícito.
- [ ] El usuario revisa y acepta el cierre. Si T22 no pasa, el plan permanece abierto y no se declara verificación externa.
- [ ] `git diff --check` y `git status` se revisan antes del cierre.
