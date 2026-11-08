# P-20261108-02-appointments-core: Citas y ciclo de vida administrativo

**Status:** Approved  
**Created:** 2026-11-08  
**Updated:** 2026-11-08  
**Task list:** [todo.md](todo.md)

**Aprobación:** el usuario aprobó el diseño el 2026-11-08 y después autorizó materializarlo en `tasks/`. Ninguna tarea ha comenzado. T01 se ejecutará después de `P-20261108-01-scheduling-availability/C06` y de una instrucción de inicio posterior.

## Qué vamos a hacer

Vamos a completar el componente `appointments` que Agenda inicia únicamente con ocupación de lectura. La entrega será una vertical nativa y usable con datos sintéticos:

1. Crear una cita seleccionando paciente, profesional, servicio, responsable cuando sea necesario e intervalo.
2. Consultar hoy, mañana y rangos locales con datos aceptados y provisionales.
3. Confirmar, reprogramar, cancelar, registrar atención y marcar ausencia mediante las seis acciones del contrato.
4. Conservar cada operación offline y sus dependencias, sobrevivir a reinicios y sincronizar sin duplicar efectos.
5. Proyectar cita e historia aceptadas, mantener la historia inmutable y reconciliar provisionales solo cuando cita e historia alcancen la versión del recibo.
6. Exponer comandos Tauri, cliente TypeScript y una UI mínima de reserva, agenda, detalle e historia.
7. Verificar concurrencia entre dispositivos, respuesta perdida, conflictos, permisos, ventanas y API real de desarrollo.

No se crea otro motor de agenda, otra autoridad de pacientes ni una implementación de las capacidades del Patient Hub.

## Resultado y reglas principales

Al terminar, propietario y asistente podrán ejecutar localmente las acciones permitidas por `allowedActions`:

| Acción | Entrada contractual | Estado previo | Resultado esperado |
|---|---|---|---|
| `CreateAppointment` | `patientId`, `professionalId`, `serviceId`, `startsAt`, `endsAt`, `responsibleContactId?` | No existe la cita | `scheduled`, versión 1 e historia `created` |
| `ConfirmAppointment` | `{}` y base | `scheduled` | `confirmed`, mismo intervalo; `confirmedAt` del servidor |
| `RescheduleAppointment` | Nuevo `startsAt`/`endsAt` y base | `scheduled` o `confirmed` | `scheduled`, igual duración real, `confirmedAt=null`; snapshots conservados |
| `CancelAppointment` | `changeReason?` y base | `scheduled` o `confirmed` | `cancelled`; libera ocupación y conserva historia |
| `CompleteAppointment` | `changeReason?` y base | `scheduled` o `confirmed`, inicio alcanzado | `completed`; sigue ocupando según contrato |
| `MarkAppointmentNoShow` | `changeReason?` y base | `scheduled` o `confirmed`, inicio+margen alcanzado | `no_show`; sigue ocupando según contrato |

Reglas transversales:

- `cancelled`, `completed` y `no_show` son terminales. Otra transición recibe `INVALID_STATE_TRANSITION`; repetir la misma operación recupera su recibo original, no crea otro efecto.
- Instantes RFC3339 con offset, precisión de minuto y duración real 1–1440. El intervalo es `[start,end)` y la duración de referencia del servicio puede ser distinta.
- No se cambian paciente, profesional, servicio, cuenta o responsable después de crear. No existe un patch libre de cita.
- La falta de respuesta no marca ausencia. Completar y ausencia usan reloj inyectado; ausencia además exige acción del personal y el margen aceptado vigente de la cuenta.
- El actor de estas acciones de escritorio es `staff`. Actores `patient`/`contact` se proyectan para futura compatibilidad, pero no habilitan acciones del Hub.
- Una cancelación libera disponibilidad; `scheduled`, `confirmed`, `completed` y `no_show` ocupan. Citas contiguas no se solapan.

## Límites

Fuera de alcance: Patient Hub y solicitudes de reprogramación, recordatorios, lista de espera, analítica, cambios de pacientes/contactos/catálogo/cuenta/agenda, edición de referencias de una cita existente, reapertura de estados terminales, datos clínicos, servidores, datos reales y cambios en otros repositorios.

`reschedule_requested` no es un estado de Citas. Reprogramar aquí es una acción del personal, no una solicitud del paciente.

No se edita el contrato `api-v1.1`, Master Plan, SRS, ADR cerrados, migraciones `0001`–`0009` ni el plan de Agenda.

## Fuentes de verdad

- `AGENTS.md`, `tasks/README.md`, `CONTRIBUTING.md`.
- `docs/master-plan.md` §§9.4–9.7, 10–15, 19 y 23.
- `docs/technical-architecture.md` §§7–14, 17, 20 y 23, especialmente estados §11 y flujos §12.
- `docs/decisions/transactional-sync-three-projects.md`, `docs/decisions/desktop-core-engine.md` y `docs/operations/core-next-tracks.md`.
- Contrato fijo: guía §§5.2–5.5, 6–10; `CreateAppointmentPayload`, `AppointmentInterval`, `AppointmentReasonPayload`, `EmptyAppointmentPayload`, `AppointmentResponse`, `AppointmentHistoryResponse`, `AppointmentHistoryProjection`, capabilities y OpenAPI.
- `tasks/P-20261108-01-scheduling-availability/plan.md`: disponibilidad, cobertura, ocupación y composición de proyectores.
- Código actual: `PatientQuery`, `CatalogQuery`, `AccountViewQuery`, modelos remotos de cita/historia, Outbox, `record_operation`, `RemoteProjector`, `DesktopProjector` y patrones de Patients/Services.

## Arquitectura y contratos

### 1. Propietario y persistencia

Se reserva `0010_appointments`, después de las migraciones de Agenda:

| Tabla | Contenido y escritor |
|---|---|
| `appointment_view` | Cita aceptada, estado, intervalo, referencias, responsable, confirmación, versión y nueve snapshots; solo proyector |
| `appointment_history_view` | Una fila por cita/versión aceptada con acción, actor, motivo, instante, intervalo, estado y snapshots; solo proyector |
| `appointment_local` | Intención, acción, base, dependencias, payload, provisional y orden local; casos de uso |
| `appointment_occupancy_view` | Proyección de lectura creada por Agenda; se conserva y se extiende su consulta con provisionales |

Todas las filas llevan ámbito cuenta/usuario/dispositivo, con índices por profesional/intervalo, paciente, servicio y estado. No hay cascadas desde vistas hacia intenciones. No se guarda historia local como si fuera aceptada: la secuencia local se deriva de operaciones y recibos.

Los nueve snapshots se capturan al crear:

1. Nombre del servicio.
2. Duración de referencia ofrecida.
3. Nombre del profesional.
4. Nombre del lugar.
5. Dirección del lugar.
6. País del lugar.
7. Estado/región del lugar.
8. Ciudad del lugar.
9. Zona horaria de la cuenta.

El servidor vuelve a obtenerlos al aceptar. El cliente no los envía en el payload y nunca los recalcula al reprogramar o proyectar historia.

### 2. Lecturas de reserva aceptadas y provisionales

Los puertos actuales `PatientQuery` y `CatalogQuery` devuelven solo datos aceptados. Para permitir registrar paciente/servicio y después citar durante la misma desconexión se añaden contratos específicos, sin cambiar los puertos existentes:

- `PatientBookingQuery`: ficha, documentos y responsables aceptados o provisionales utilizables, con teléfono, vigencia y referencia de versión/operación. Se implementa dentro de Patients usando sus lecturas existentes.
- `CatalogBookingQuery`: servicio, oferta y profesional aceptados o provisionales utilizables, con nombre y duración efectiva para snapshot y referencia de versión/operación. Se implementa dentro de Services.
- `AccountViewQuery`: sigue siendo solo aceptado para zona, lugar y margen.
- `ScheduleAvailabilityQuery`: se añade `excludeAppointmentId?: AppointmentId` para que reprogramar no reste la propia cita. Es una ampliación aditiva del contrato de Agenda.
- `ScheduleOccupancyQuery`: su implementación en Appointments combina vista aceptada y operaciones locales, de modo que una nueva cita pendiente también bloquee localmente.

Las referencias provisionales solo se usan cuando su operación puede encadenarse. Un conflicto/rechazo impide usarlas; un resultado desconocido exige dependencia del original, nunca reemplazarlo. Toda nueva operación declara sus dependencias de paciente/contacto/servicio/oferta/agenda y de la acción previa de cita, hasta ocho, sin duplicados ni ciclos.

### 3. Validación de una nueva cita

Antes de guardar `CreateAppointment`:

- Cuenta activa y acción permitida; paciente `active` y `complete` en el estado elegido.
- `own_document`: nombre, documento propio `declared` o `verified`, y teléfono propio o responsable válido con teléfono.
- `minor_without_document`: nombre, nacimiento, responsable válido con teléfono y edad menor de 18 en la fecha local de la cita. El 29 de febrero cumple el 1 de marzo en años no bisiestos.
- Un responsable exige vínculo activo, `isResponsibleContact=true`, vigencia en `startsAt` y teléfono. No se exige ni se inventa verificación de representación/tutela.
- Profesional, servicio y oferta activos. `responsibleContactId` se exige cuando la base del paciente necesita ese apoyo.
- Inicio<fin, minutos exactos, duración real 1–1440 y disponibilidad local que contenga el intervalo completo con esa duración real.

La cobertura sigue la regla de Agenda: dentro de ventana se puede guardar como provisional; sin captura se puede trabajar solo con datos locales conocidos y advertencia; cobertura insuficiente devuelve `INSUFFICIENT_COVERAGE` y no crea intención. La consulta remota de Agenda es informativa y no se usa para saltarse esta protección.

La creación válida guarda cita provisional y `record_operation` en una transacción `IMMEDIATE`, con `expectedVersion=0`. La disponibilidad local nunca garantiza la reserva compartida: API vuelve a validar permisos, referencias, versiones y solapamiento global.

### 4. Transiciones y reprogramación

Toda acción posterior usa `Accepted { version }` o `LocalOperation { operationId }` como base. Las versiones previstas de una secuencia son provisionales y cada paso depende del anterior.

- Confirmar no cambia intervalo ni fabrica `confirmedAt` local.
- Reprogramar exige el nuevo intervalo con la misma duración real, disponibilidad excluyendo `excludeAppointmentId`, conserva snapshots y deja confirmación nula hasta aceptación.
- Cancelar libera la ocupación provisional y admite `changeReason` opcional no vacío si está presente.
- Completar exige `clock.now >= startsAt` y conserva ocupación.
- Ausencia exige `clock.now >= startsAt + noShowGraceMinutes` de la cuenta aceptada vigente y conserva ocupación.

Los cambios que API rechace o marque en conflicto conservan ID, contenido, base y dependencias. No se rebasa, ajusta versión ni repite con otro ID automáticamente.

### 5. Proyección e historia

`AppointmentsProjector` procesa `appointment`; `AppointmentHistoryProjector` procesa `appointment_history`. Ambos validan ámbito, claves, estados, tiempos y versiones y participan en `DesktopProjector` con la transacción recibida.

La proyección de historia:

- Sustituye el conjunto aceptado de esa cita solo con una versión agregada posterior.
- Exige entradas desde versión 1, sin huecos ni duplicados, acciones coherentes y última entrada consistente con la versión agregada.
- Conserva actor, motivo, confirmación, intervalo y snapshots de cada versión; no reescribe entradas antiguas ni recalcula snapshots.
- Si cita e historia llegan juntas, valida que la última entrada coincida con el estado actual de cita. Si llegan por separado, mantiene visible la falta de historia sin inventarla.

`remove` retira cita, historia y ocupación aceptadas cuando salen de ventana; nunca borra intenciones. Un estado remoto nuevo se proyecta normalmente, pero la intención antigua permanece visible para revisión. Una cancelación remota no se sobrescribe con una operación local antigua.

Un provisional se resuelve solo cuando existe recibo aceptado y **cita e historia** alcanzan la versión confirmada. Un recibo solo no retira nada.

### 6. Consultas, IPC y UI

Consultas locales sin HTTP:

- `appointments_list_appointments`: `[from,until)` hasta 366 días, intersección de intervalos, filtros profesional/servicio/paciente/estado y paginación 1–500.
- `appointments_get_appointment`: aceptado, provisional y operaciones con estados.
- `appointments_get_history`: historia aceptada con `afterVersion` opcional y operaciones locales separadas.
- `appointments_get_booking_context`: cuenta, zona/margen, pacientes, responsables, ofertas y profesionales utilizables, marcando provisionales y dependencias.

Escrituras: `appointments_create_appointment`, `appointments_confirm_appointment`, `appointments_reschedule_appointment`, `appointments_cancel_appointment`, `appointments_complete_appointment`, `appointments_mark_no_show`.

Serán diez comandos con DTO camelCase, campos extra rechazados, permiso explícito y ámbito derivado de sesión. React solo usa `src/shared/api/appointments.ts`.

UI mínima:

- Reserva: contexto aceptado/provisional, selección de referencias, responsable condicional, intervalo y disponibilidad/cobertura; no crea si faltan requisitos.
- Agenda: hoy y mañana calculados con la zona de la cuenta, rango personalizado, estados de negocio y sincronización.
- Detalle: datos y snapshots, acciones permitidas por estado, reprogramación con duración conservada y conflictos enlazados a revisión.
- Historia: versiones aceptadas inmutables y operaciones locales claramente separadas; nunca presenta hora local como confirmación del servidor.

Puertos públicos para evoluciones posteriores: `AppointmentQuery`, `AppointmentHistoryQuery` y la implementación ampliada de `ScheduleOccupancyQuery`. No se añade bus ni eventos de dominio; recibos y cambios no sustituyen `AppointmentCreated` u otros eventos futuros.

## Dependencia de ejecución

La redacción/aprobación puede hacerse ahora. La implementación de T01 empieza después de C06 de Agenda, porque reutiliza sus tablas, puertos, cobertura y proyectores. Patients T17 no se reabre ni bloquea el diseño o dominio nativo; su verificación real no se da por heredada y la prueba real de Citas registra sus propios resultados.

Secuencial, una tarea en progreso, sin trabajo paralelo. Un único integrador registra `lib.rs`, `build.rs`, `components/mod.rs`, `platform/sqlite/mod.rs`, `components/projector.rs`, manifiestos/lockfile y `capabilities/default.json`.

```text
T01–T06 → C01
T07–T13 → C02
T14–T16 → C03
T17–T19 → C04
T20–T21 → C05
T22–T25 → C06
```

## Task List

El detalle verificable de cada tarea está en [todo.md](todo.md).

### Fase 1: contratos, dominio y composición
- [ ] T01: Congelar contratos de Citas.
- [ ] T02: Dominio de cita y tiempo.
- [ ] T03: Persistencia de Citas.
- [ ] T04: Puerto PatientBookingQuery.
- [ ] T05: Puerto CatalogBookingQuery.
- [ ] T06: Exclusión y ocupación provisional.

### C01: Base contractual y de datos
- [ ] Dominio, migración y puertos booking/disponibilidad pasan junto con workspace y `pnpm build:desktop`.
- [ ] Usuario revisa snapshots, provisionales, exclusión y cobertura.
- [ ] No hay escrituras de Citas ni cambios en contratos anteriores.

### Fase 2: ciclo de vida offline
- [ ] T07: Secuencias y dependencias de operaciones.
- [ ] T08: Crear cita.
- [ ] T09: Confirmar cita.
- [ ] T10: Reprogramar cita.
- [ ] T11: Cancelar cita.
- [ ] T12: Registrar atención.
- [ ] T13: Marcar ausencia.

### C02: Ciclo de vida nativo
- [ ] Las seis acciones y secuencias offline/reapertura pasan con referencias provisionales y aceptadas.
- [ ] `cargo test --workspace --locked` y `pnpm build:desktop` pasan.
- [ ] Usuario revisa estados, tiempos y disponibilidad antes de sincronización.

### Fase 3: proyección y conciliación
- [ ] T14: Proyector de cita.
- [ ] T15: Proyector de historia.
- [ ] T16: Conciliación de provisionales.

### C03: Sincronización e historia
- [ ] Cita, historia y ocupación se proyectan y reconcilian sin duplicados ni pérdida de intenciones.
- [ ] Workspace y `pnpm build:desktop` pasan.
- [ ] Usuario revisa historia, cancelación remota y retiradas de ventana.

### Fase 4: IPC y registro
- [ ] T17: Comandos de escritura.
- [ ] T18: Comandos de consulta.
- [ ] T19: Registro y cliente TypeScript.

### C04: Interfaz nativa
- [ ] Diez comandos, DTO y permisos pasan con sesiones válidas y denegadas.
- [ ] `pnpm quality` y `pnpm build:desktop` pasan.
- [ ] Usuario revisa IPC antes de UI.

### Fase 5: interfaz de usuario
- [ ] T20: Reserva y acciones.
- [ ] T21: Agenda, detalle e historia.

### C05: Vertical usable
- [ ] Reserva, acciones, agenda e historia funcionan con datos sintéticos y estados claros.
- [ ] `pnpm quality`, `pnpm build:desktop` y revisión manual Windows pasan.
- [ ] Usuario revisa la vertical antes de pruebas finales.

### Fase 6: contrato, integración y aceptación
- [ ] T22: Fixtures del consumidor.
- [ ] T23: Integración loopback de dos perfiles.
- [ ] T24: Guías y calidad.
- [ ] T25: API/OIDC real de desarrollo.

### C06: Aceptación de Citas
- [ ] Suite, contrato, integración loopback, calidad y build de escritorio pasan.
- [ ] API/OIDC real y recorrido UI tienen evidencia; límites no ejecutados permanecen explícitos.
- [ ] Usuario revisa y acepta el cierre. Si T25 no pasa, el plan permanece abierto.

## Matriz de verificación obligatoria

- Dominio: seis transiciones, terminales, nueva operación repetida inválida, minutos, duración real distinta de referencia y actor staff.
- Pacientes: adulto con documento declarado/verified, documento invalidated, teléfono propio/responsable, menor al límite de 18, 29 de febrero, responsable inactivo/no vigente/sin teléfono y provisionales encadenables.
- Snapshots: nueve datos presentes; cambios posteriores de cuenta/catálogo/profesional no reescriben cita ni historia; reprogramar conserva snapshots y duración real.
- Disponibilidad: crear dentro/fuera de horario, bloqueo, ocupación, contigüidad, cobertura insuficiente, reprogramación sin autobloqueo y cancelación que libera.
- Tiempo: zona de cuenta para fecha local/edad/hoy/mañana, inicio alcanzado y margen vigente de ausencia con reloj controlado.
- Offline: dependencias hasta ocho, rollback, secuencias create→confirm→reschedule→cancel, base antigua, operación desconocida y corrección explícita con nuevo ID.
- Sincronización: duplicado, aceptación parcial, respuesta perdida, `SLOT_UNAVAILABLE`, `VERSION_CONFLICT`, cancelación remota, cursor/captura, remove/reset y revocación.
- Historia: inicio en 1, sin huecos, última entrada=estado actual, actores staff/patient/contact proyectados y llegada separada de cita.
- Seguridad: aislamiento por cuenta/usuario/dispositivo, DTO sin ámbito/secrets, logs sin datos personales y sin credenciales SQL.
- Plataformas: Windows con evidencia manual; macOS/Linux solo si se ejecutan. Patients T17 no se hereda como evidencia.

## Riesgos y mitigaciones

| Riesgo | Impacto | Mitigación |
|---|---|---|
| Reserva duplicada entre dispositivos | Alto | Operación inmutable, idempotencia y prueba concurrente real/loopback |
| Autobloqueo al reprogramar | Alto | `excludeAppointmentId` y prueba de mismo intervalo/otra cita |
| Datos provisionales incompatibles | Alto | Puertos booking con referencias/dependencias y rechazo de estados no encadenables |
| Falsa disponibilidad por ventana | Alto | `INSUFFICIENT_COVERAGE`; no guardar cuando falta evidencia local |
| Historia reescrita o incompleta | Alto | Validación agregada consecutiva y resolución recibo+cita+historia |
| Snapshots divergentes | Alto | Captura al crear y prohibición de recalcular historia |
| Confirmación/ausencia con reloj incorrecto | Alto | Reloj inyectado, servidor como autoridad y margen aceptado vigente |
| Conflicto con cancelación remota | Alto | Proyección remota y conservación de intención sin resurrección local |
| Alcance mezclado con Hub | Medio | Solo acciones staff nativas; Hub y solicitudes quedan fuera |
| Entorno real no disponible | Medio | Fakes/loopback completos; T25/C06 abiertos sin afirmar verificación |

## Supuestos

- Se incluye UI mínima porque se solicita una vertical completa, sin sistema visual nuevo.
- Las ampliaciones `PatientBookingQuery`, `CatalogBookingQuery` y `excludeAppointmentId` son aditivas y no cambian los contratos existentes de consulta aceptada.
- `0010_appointments` queda reservado al materializar; una colisión exige revisión previa.
- API permanece en `api-v1.1`; incompatibilidades vuelven al repositorio propietario y detienen el punto afectado.
- La implementación comienza después de Agenda C06; la redacción/aprobación no inicia código ni modifica Agenda.
- Solo datos sintéticos. Cifrado SQLite, claves, publicación y producción permanecen fuera.

## Preguntas abiertas

Ninguna decisión funcional pendiente. No se inventan estados, acciones de Hub, cambio de referencias de una cita existente ni reabertura de terminales.
