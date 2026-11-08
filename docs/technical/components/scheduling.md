# Agenda y disponibilidad

Responsabilidad: reglas semanales, excepciones por fecha y bloqueos del
profesional, más la disponibilidad calculada bajo demanda. No incluye el ciclo de
vida de las citas (crear, confirmar, reprogramar, cancelar, completar, ausencia),
la historia de citas ni el Patient Hub; `appointments` aporta únicamente la
proyección mínima de ocupación aceptada que Agenda necesita para restar.

Propietario y asistente escriben según `allowedActions` vigentes; un bloqueo
global (`accountId = null`) exige rol propietario y que el profesional sea el
propio (`professional_view.professional_user_id` igual al usuario de la sesión).
El motivo de un bloqueo global puede llegar oculto por audiencia y la UI no lo
inventa.

Datos: `schedule_rule_view`, `schedule_exception_view`, `schedule_block_view` y
`schedule_local`, migración `0008_scheduling`; la ocupación aceptada vive en
`appointment_occupancy_view` (migración `0009_appointment_occupancy`, propiedad
de `appointments`). Cada fila tiene ámbito de cuenta, usuario y dispositivo. Las
vistas aceptadas y las intenciones tienen distintos escritores; `remove`/`reset`
nunca borran intenciones. No existe una tabla de disponibilidad calculada: la
disponibilidad es una conclusión del motor puro (`availability.rs`).

Tiempo: reglas y excepciones usan fecha civil y minutos desde medianoche (inicio
0–1439, fin exclusivo 1–1440; `1440` es la medianoche del día civil siguiente).
Bloques y disponibilidad usan instantes RFC 3339 normalizados a UTC. Todos los
intervalos son `[start,end)`. La zona civil procede de `AccountViewQuery`; nunca
del dispositivo. Jiff 0.2.38 con `tzdb-bundle-always` (tzdb 2026e embebida, la
misma en Windows/macOS/Linux); las horas civiles ambiguas o inexistentes se
rechazan, no se resuelven.

Escrituras: `SchedulingRepository::persist` aplica las nueve acciones
contractuales (Create/Update/SetStatus × regla/excepción/bloqueo). Cada mutación
comprueba permiso, valida contra la ocupación conocida (SLOT_UNAVAILABLE si deja
una cita conocida fuera del horario o dentro de un bloqueo) y escribe su
intención con `record_operation` en la misma transacción IMMEDIATE: provisional y
Outbox se confirman o revierten juntos. La base de una edición es una versión
aceptada o la última operación local; sin diferencia devuelve NO_CHANGES; base
antigua, VERSION_CONFLICT; operaciones no resolventes, DEPENDENCY_FAILED; una
excepción activa por profesional y fecha, VALIDATION_ERROR.

Lecturas: `rule_rows_list`, `exception_rows_list` y `block_rows_list` fusionan
aceptado, provisional y operaciones con su estado. Un provisional se retira solo
con recibo aceptado y la vista aceptada en la versión confirmada; conflicto,
rechazo y resultado desconocido lo conservan. `ScheduleAvailabilityQuery`
(`local_availability`) compone cuenta (zona, versión), catálogo (oferta activa,
duración efectiva, versiones), agenda, ocupación del puerto `ScheduleOccupancyQuery`
y `CursorStore` para la cobertura. Salida siempre provisional, con
`within_synced_window` / `no_synced_window` / `insufficient_coverage`; una lista
vacía nunca significa «libre» por sí sola.

Consulta remota: `scheduling_check_remote_availability` usa
`GET /v1/accounts/{account}/availability` (máximo 31 días), refresca acceso,
valida la respuesta contra la petición y nunca escribe Outbox ni vistas. La
puerta de sesión (`SessionGate` sobre `identity_context` más la puerta de
`identity::desktop`) impide cambios de cuenta/logout durante la petición y ningún
candado SQLite se retiene durante el HTTP.

IPC: catorce comandos `scheduling_*` registrados en `lib.rs`, `build.rs` y
`capabilities/default.json`; DTO camelCase con `deny_unknown_fields` y base
inequívoca. React usa solo `src/shared/api/scheduling.ts`; la UI
(`src/features/scheduling/`) presenta estados y cobertura sin calcular
disponibilidad ni versiones.

Verificado: pruebas de dominio, persistencia, escrituras, disponibilidad local y
remota, proyección, conciliación, contrato e integración loopback de dos perfiles
(`cargo test --workspace --locked`, 305+ pruebas). Pendiente: prueba opt-in con
API/OIDC real de desarrollo (T22); los límites de Patients T17 y la evidencia
manual Windows se registran en su plan.
