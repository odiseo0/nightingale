# Citas y historia

Responsabilidad: ciclo de vida administrativo de una reserva (crear, confirmar,
reprogramar, cancelar, registrar atención, marcar ausencia) con su historia
aceptada y sus intenciones locales. No incluye el Patient Hub ni sus solicitudes,
recordatorios, lista de espera ni datos clínicos. Agenda sigue siendo la
propietaria de reglas, excepciones, bloqueos y del cálculo de disponibilidad;
Pacientes y Servicios son la única autoridad de fichas y catálogo.

El actor de las acciones del escritorio es `staff`; los actores `patient` y
`contact` se proyectan en la historia solo como datos aceptados. Propietario y
asistente escriben según `allowedActions` vigentes; cada comando vuelve a
comprobar la acción en Rust y la API la revalida al aceptar.

Datos: `appointment_view`, `appointment_history_view` y `appointment_local`,
migración `0010_appointments`; la ocupación aceptada sigue en
`appointment_occupancy_view` (`0009_appointment_occupancy`). Cada fila tiene
ámbito de cuenta, usuario y dispositivo; vistas e intenciones tienen distintos
escritores y `remove`/`reset` nunca borran intenciones. La historia aceptada es
agregada: un agregado reemplaza al conjunto de su cita solo cuando es
estrictamente mayor, y las entradas son inmutables (desde versión 1, sin huecos
ni duplicados).

Tiempo: instantes RFC 3339 con precisión de minuto, normalizados a UTC; duración
real 1–1440 minutos e intervalos `[start,end)`. Reprogramar exige la misma
duración real. La edad del menor, la vigencia del responsable y «hoy/mañana» de
la UI usan la fecha civil de la zona de la cuenta (`AccountViewQuery`), nunca la
del dispositivo. Completar exige `clock.now >= startsAt`; marcar ausencia exige
`clock.now >= startsAt + noShowGraceMinutes` de la política aceptada vigente, con
reloj inyectado: la falta de respuesta nunca marca ausencia y el servidor sigue
siendo la autoridad temporal.

Escrituras: `AppointmentRepository::persist` aplica las seis acciones
contractuales con payload exacto (create con `responsibleContactId` opcional,
confirm con `{}`, reschedule con `{startsAt,endsAt}`, motivo opcional no vacío
en cancel/complete/no-show). `create_appointment` compone los puertos públicos
existentes —`AccountViewQuery` (zona, lugar, margen), `PatientBookingQuery`,
`CatalogBookingQuery` y `ScheduleAvailabilityQuery`— sin leer tablas ajenas:
valida ficha completa/activa, documento declarado o verificado del adulto, menor
de 18 a la fecha local de la cita con responsable vigente y con teléfono, oferta
activa e intervalo contenido en la disponibilidad local. La cobertura es
explícita: `insufficient_coverage` bloquea con `INSUFFICIENT_COVERAGE` y sin
efectos; `no_synced_window` permite trabajar con lo conocido; la validación
local nunca prueba la reserva compartida. Las referencias provisionales
encadenables se declaran como dependencias (hasta ocho, sin duplicados). Todo se
valida antes de abrir la transacción `IMMEDIATE` que confirma provisional y
Outbox juntos.

Reprogramar (`reschedule_appointment`) consulta la disponibilidad con
`exclude_appointment_id`: la propia cita deja de restarse (sin autobloqueo al
mudarse al mismo hueco) y el resto de la ocupación —aceptada y provisional—
sigue bloqueando. La ocupación provisional se deriva del último borrador local
por cita: pendiente bloquea, cancelación provisional libera, y operaciones en
conflicto o rechazo no anclan nada.

Lecturas: `appointment_rows` fusiona aceptado, provisional y operaciones
separadas. Un provisional se resuelve solo con recibo aceptado y cita e historia
aceptadas en la versión confirmada; recibo solo, conflicto, rechazo o
desconocido lo conservan. Una cancelación remota prevalece: la intención local
antigua permanece visible para revisión pero nunca sobrescribe la vista
aceptada. Los comandos `appointments_*` devuelven esas filas separadas y nunca
época, huella, token, SQL ni estado remoto crudo.

Comandos: diez comandos Tauri (`lib.rs`, `build.rs` y
`capabilities/default.json`); DTOs `deny_unknown_fields` camelCase con base
exactamente aceptada o local; la UI (`src/features/appointments`) solo presenta
lo que Rust devuelve: provisionales y conflictos marcados, historia aceptada
separada de operaciones locales, y la hora local jamás como confirmación del
servidor.

Verificación: `cargo test --workspace --locked components::appointments`,
`cargo test --workspace --locked appointments_integration_tests` y
`cargo test --workspace --locked components::sync::contract_tests`; prueba real
opt-in de desarrollo en `development_tests` (mismo entorno que Agenda). La
reserva concurrente real y una respuesta HTTP perdida real quedan como límites
documentados cuando el entorno no permite forzarlas.
