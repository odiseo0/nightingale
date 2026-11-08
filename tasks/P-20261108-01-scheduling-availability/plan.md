# P-20261108-01-scheduling-availability: Agenda y Disponibilidad

**Status:** Approved  
**Created:** 2026-11-08  
**Updated:** 2026-11-08 (T01–T04 ejecutadas; C01 pendiente de revisión del usuario)  
**Task list:** [todo.md](todo.md)

**Aprobación:** el usuario aprobó el diseño el 2026-11-08 y después autorizó expresamente materializarlo en `tasks/`. Ninguna tarea ha comenzado. El inicio de T01 requiere una instrucción de ejecución posterior.

## Contexto y resultado

Core Engine está cerrado; Account/Services está completado; Patients tiene implementación local y queda su verificación real T17, que no se reabre ni bloquea el trabajo nativo de Agenda. Los modelos remotos `schedule_rule`, `schedule_exception`, `schedule_block` y `appointment` ya existen en el adaptador de `sync`, pero no hay componente `scheduling`, tablas, casos de uso, proyección de agenda ni disponibilidad local.

El resultado previsto es una vertical completa con datos sintéticos:

- Propietario y asistente gestionan reglas semanales, excepciones y bloqueos según `allowedActions`; solo el propietario/profesional propio gestiona bloqueos globales.
- Las ediciones sucesivas funcionan sin red, sobreviven al reinicio y conservan IDs, dependencias, recibos, conflictos y rechazos.
- La aplicación calcula intervalos disponibles bajo demanda a partir de agenda, excepciones, bloqueos y ocupación conocida, sin persistir una segunda “disponibilidad” mutable.
- Una consulta local declara siempre su carácter provisional y su cobertura; nunca presenta como libre un intervalo cuya ocupación local sea desconocida.
- Una consulta remota explícita obtiene la vista vigente de `GET /availability`, sin convertirla en reserva ni sustituir el trabajo offline.
- React ofrece una UI mínima para agenda y disponibilidad mediante comandos Tauri específicos.
- Citas recibe puertos de disponibilidad y ocupación, sin adelantar sus casos de escritura, historia, Patient Hub ni recordatorios.

## Límites

No incluye crear/editar/cancelar citas, historia de citas, pacientes, cambios de catálogo o cuenta, Patient Hub, lista de espera, recordatorios, n8n, analítica, servidores, cambios en Nightingale API ni datos reales. Tampoco modifica el contrato importado, Master Plan, SRS, ADR cerrados ni migraciones `0001`–`0007`.

La disponibilidad local no prueba ausencia de conflictos con otros dispositivos o cuentas. La aceptación final de una reserva y de cambios incompatibles con citas pertenece a Nightingale API/PostgreSQL.

## Fuentes de verdad

- `AGENTS.md`, `tasks/README.md`, `CONTRIBUTING.md`.
- `docs/master-plan.md` §§6–9, 11–15, 19 y 23, especialmente §9.6.
- `docs/technical-architecture.md` §§7–14, 17 y 23.
- `docs/decisions/desktop-core-engine.md` y `docs/operations/core-next-tracks.md`.
- Contrato fijo `api-v1.1`: guía §§5.3–5.5, 6–10; `accepted-operations.schema.json`, `public-models.schema.json`, `capabilities.json`, `openapi.json` y manifiesto.
- Implementación existente: `services::application::CatalogQuery`, `account::application::AccountViewQuery`, `sync::application::{record_operation,CursorStore}`, `RemoteProjector`, `DesktopProjector`, modelos remotos y patrones IPC de Services/Patients.
- Jiff 0.2.38 ya resuelto transitivamente en `Cargo.lock`: documentación oficial <https://docs.rs/jiff/0.2.38/jiff/> y <https://docs.rs/jiff-static/0.2.38/jiff_static/>.

## Decisiones de arquitectura

### 1. Propietarios y datos

Nuevo componente `scheduling`: dominio, aplicación y adaptadores propios. No lee tablas de Account/Services/Patients/Sync ni contiene reglas de Citas.

Se reservan dos migraciones consecutivas:

| Migración | Tabla | Propietario y contenido |
|---|---|---|
| `0008_scheduling` | `schedule_rule_view` | Vista aceptada de reglas; solo proyector |
| `0008_scheduling` | `schedule_exception_view` | Vista aceptada de excepciones; solo proyector |
| `0008_scheduling` | `schedule_block_view` | Vista aceptada de bloqueos de cuenta/globales; motivo nullable por audiencia |
| `0008_scheduling` | `schedule_local` | Intenciones con operación, entidad, base, dependencias, payload y provisional |
| `0009_appointment_occupancy` | `appointment_occupancy_view` | Proyección mínima aceptada de ocupación, propiedad de `appointments` |

Todas las filas incluyen `account_id`, `user_id`, `device_id`. Las vistas no tienen cascadas hacia intenciones. `schedule_local` consulta estados mediante `SqliteOperationStore.operation_result`, sin duplicar la máquina de sincronización. No se crea una tabla de disponibilidad calculada.

`appointments` se crea únicamente con una proyección/consulta de ocupación, no como módulo vacío ni con casos de Citas. Define el puerto `ScheduleOccupancyQuery`; `scheduling` depende de ese contrato. La futura vertical de Citas reutilizará el puerto y añadirá ocupación provisional, sin obligar a Agenda a leer sus tablas.

### 2. Tiempo y zonas

- IDs UUIDv7 propios de Scheduling; no se amplía `shared_kernel`.
- Reglas y excepciones conservan fecha civil y minutos desde medianoche; inicio 0–1439, fin exclusivo 1–1440. Vigencias de reglas incluyen ambos extremos.
- Bloques y disponibilidad usan instantes RFC3339 con offset y se normalizan a UTC internamente. Todos los intervalos son `[start,end)`.
- La zona civil procede de `AccountViewQuery`; no se deduce del dispositivo ni del teléfono.
- Se selecciona Jiff 0.2.38 como dependencia directa, con `tzdb-bundle-always` para que Windows/macOS/Linux usen la misma base embebida y resultados reproducibles. No se añade `jiff-static` directamente ni macros. T01 confirmará features y versión tzdb, y comprobará que los 598 identificadores de `reference_data` resuelven.
- Horas civiles ambiguas o inexistentes se rechazan explícitamente; no se usa la resolución compatible predeterminada de Jiff para escoger una silenciosamente. `1440` se convierte desde el día civil siguiente.
- Chrono se conserva donde ya se usa; no se sustituye globalmente ni se actualizan otras dependencias.

### 3. Contrato de escrituras

Las nueve acciones son offline según `capabilities.json`; creación usa `expectedVersion=0`, edición una base aceptada positiva o la última operación local. Ausencia conserva, `null` solo retira donde el caso lo admite. Un cambio sin diferencia devuelve `NO_CHANGES`.

| Entidad | Acciones | Reglas principales |
|---|---|---|
| Regla | `CreateScheduleRule`, `UpdateScheduleRule`, `SetScheduleRuleStatus` | Día ISO 1–7; inicio<fin; `validUntil>=validFrom`; profesional y vigencia obligatorios |
| Excepción | `CreateScheduleException`, `UpdateScheduleException`, `SetScheduleExceptionStatus` | `closed` exige minutos nulos; `replacement` exige ambos e inicio<fin; una excepción efectiva por profesional/fecha |
| Bloqueo | `CreateScheduleBlock`, `UpdateScheduleBlock`, `SetScheduleBlockStatus` | Instantes inicio<fin y motivo de creación obligatorio; `accountId` actual o `null` global; profesional/ámbito no cambian por patch |

Payloads exactos:

- Regla: `{professionalId,isoWeekday,localStartMinute,localEndMinute,validFrom,validUntil?}`; patch con esos campos opcionales y `validUntil` nullable.
- Excepción: `{professionalId,localDate,kind,localStartMinute?,localEndMinute?}`; patch revalida el estado resultante y no permite nulos en campos requeridos por su tipo.
- Bloqueo: `{professionalId,accountId,startsAt,endsAt,reasonCode}`; patch solo `startsAt`, `endsAt`, `reasonCode`, sin retirar valores obligatorios.
- Estados: `{status:"active"|"inactive"}` con la acción `Set...Status` correspondiente.

Propietario y asistente escriben según acción efectiva en `allowedActions`. Un bloqueo global exige rol propietario y que `ProfessionalView.user_id` sea el usuario seleccionado. Un asistente puede recibir un bloqueo global con `reasonCode=null`; la UI no inventa su motivo.

Antes de guardar un cambio se valida contra ocupación local conocida. Si deja una cita conocida fuera del horario o dentro de un bloqueo, devuelve `SLOT_UNAVAILABLE` y no crea intención. La falta de datos remotos no se interpreta como prueba de seguridad; API revalida siempre.

### 4. Disponibilidad local

Puerto público `ScheduleAvailabilityQuery`, sin HTTP ni SQL hacia consumidores. Entrada: profesional, servicio, `[from,until)` de hasta 31 días y duración opcional 1–1440. Salida: zona, duración de referencia/solicitada, versiones de oferta/catálogo/cuenta y agenda usada, intervalos UTC, origen local, carácter provisional y cobertura.

Algoritmo puro y determinista:

1. Exige cuenta, profesional, oferta y servicio aceptados activos; la duración usa `durationMinutesOverride ?? defaultDurationMinutes`, salvo override de la consulta.
2. Combina reglas aceptadas y provisionales visibles, aplica vigencia y une solapamientos.
3. Una excepción activa reemplaza todo el horario de esa fecha; `closed` produce día vacío. No se duplican intersecciones.
4. Convierte cada intervalo civil a UTC con la zona aceptada, intersecta con la consulta y resta bloqueos activos de la cuenta y globales.
5. Resta ocupación `scheduled`, `confirmed`, `completed` y `no_show`; `cancelled` no ocupa. Citas contiguas no se tratan como solapadas.
6. Une resultados contiguos/solapados y conserva intervalos capaces de contener la duración solicitada. No inventa una cuadrícula fija ni reserva un inicio.

`CursorStore` aporta la ventana de ocupación descargada:

- Dentro de ventana: resultado `within_synced_window`, todavía provisional por posibles cambios no recibidos y ocupación global desconocida.
- Sin captura: resultado `no_synced_window`, marcado como solo local; puede servir para trabajo inicial offline, con advertencia explícita.
- Consulta parcial/total fuera de ventana: `insufficient_coverage`; no devuelve una lista vacía con significado de “sin huecos”. La UI distingue cobertura desconocida de ausencia de disponibilidad y ofrece la consulta remota.

La futura ocupación provisional de Citas entrará por el mismo puerto y también se restará localmente.

### 5. Consulta remota separada

`scheduling_check_remote_availability` usa `GET /v1/accounts/{account}/availability` con los mismos parámetros y máximo 31 días. Refresca acceso, no genera Outbox ni escribe una vista aceptada, valida IDs/duraciones/intervalos de respuesta y nunca mantiene el mutex SQLite durante HTTP.

El resultado remoto se presenta como consulta vigente de API, no como reserva ni dato offline. Un error conserva la vista local y devuelve código seguro. Se reutiliza la puerta de sesión de `identity::desktop` para impedir cambio de cuenta/logout durante la petición.

### 6. Proyección, conciliación e IPC

`SchedulingProjector` traduce los tres tipos de agenda; `AppointmentOccupancyProjector` traduce `appointment`. Ambos validan ámbito, claves, tiempos, estados y versión, ignoran tipos ajenos, no hacen HTTP ni tocan intenciones. `remove` retira solo vistas; `reset` solo su ámbito. `DesktopProjector` los compone con Account/Services/Patients en la misma transacción.

Un provisional se retira solo con recibo aceptado y vista que alcanza la versión confirmada. Conflictos/rechazos/resultados desconocidos conservan operación y dependencias. Una corrección usa otra operación explícita.

Se prevén 14 comandos con DTO camelCase, `deny_unknown_fields`, permiso Tauri explícito y ámbito derivado de sesión:

- Nueve comandos `scheduling_create/update/set_*` para reglas, excepciones y bloqueos.
- `scheduling_list_rules`, `scheduling_list_exceptions`, `scheduling_list_blocks`.
- `scheduling_get_local_availability`, `scheduling_check_remote_availability`.

Las lecturas fusionan `accepted`, `provisional` y operaciones con su estado; los puertos para Citas no exponen provisionales de catálogo/cuenta ni almacenamiento. React usa solo `src/shared/api/scheduling.ts`.

### 7. UI mínima

Dos secciones accesibles, sin sistema visual nuevo:

- Agenda: reglas semanales, excepciones `closed/replacement`, bloqueos de cuenta y, cuando proceda, globales; filtros, estados, provisionales y enlaces a revisión de sincronización.
- Disponibilidad: selección de profesional/servicio/fechas/duración, zona visible, intervalos en hora local y UTC opcional, estados `provisional`, `sin huecos`, `cobertura insuficiente`, `consulta remota fallida` y botón de comprobación API.

No se habilitan bloqueos globales por rol solo en React: Rust vuelve a autorizar. La UI no calcula disponibilidad ni versiones.

## Contratos públicos para Citas

- `ScheduleAvailabilityQuery`: evalúa disponibilidad local y devuelve intervalos, base versionada y cobertura.
- `ScheduleOccupancyQuery`: devuelve ocupación aceptada por profesional/intervalo; admite evolución a provisional.
- `ScheduleRuleView`, `ScheduleExceptionView`, `ScheduleBlockView`, `AvailabilityEvaluation`, `OccupancyInterval` y errores estables permanecen libres de Serde remoto, SQL, HTTP y Tauri en dominio/aplicación pública.
- No se añaden eventos de dominio ni bus; recibos y páginas no son eventos `ScheduleRuleChanged`/`ScheduleBlock*`.

## Orden y checkpoints

Secuencial, una tarea en progreso; sin trabajo paralelo. Un único integrador registra `lib.rs`, `build.rs`, `components/mod.rs`, `platform/sqlite/mod.rs`, `components/projector.rs`, manifiestos/lockfile y `capabilities/default.json`. Cada checkpoint detiene la fase siguiente hasta revisión del usuario.

```text
T01–T04 → C01
T05–T08 → C02
T09–T11 → C03
T12–T13 → C04
T14–T18 → C05
T19–T22 → C06
```

## Task List

El detalle verificable de cada tarea está en [todo.md](todo.md).

### Fase 1: contratos y persistencia
- [x] T01: Congelar contratos y tiempo.
- [x] T02: Dominio de agenda.
- [x] T03: Persistencia de Scheduling.
- [x] T04: Ocupación aceptada de Citas.

### C01: Contratos y persistencia
- [x] Pruebas de dominio/migración/repositorios y `cargo test --workspace --locked` pasan.
- [x] Contratos, zonas, tablas, puertos y puntos calientes revisados por el usuario.
- [x] No hay implementación de Citas ni cambios en contratos anteriores.

### Fase 2: escrituras offline
- [ ] T05: Intenciones y ediciones sucesivas.
- [ ] T06: Reglas semanales.
- [ ] T07: Excepciones.
- [ ] T08: Bloqueos de cuenta y globales.

### C02: Escrituras nativas
- [x] Las nueve acciones guardan offline, reabren y respetan permisos/versiones/dependencias.
- [x] Suite workspace y `pnpm build:desktop` pasan; conflictos conocidos no crean efectos parciales.
- [x] El usuario revisa reglas, excepciones y bloqueos antes de disponibilidad.

### Fase 3: disponibilidad
- [x] T09: Motor puro de intervalos.
- [x] T10: Consulta local con cobertura.
- [x] T11: Consulta remota API.

### C03: Disponibilidad nativa y remota
- [ ] Reglas/excepciones/bloqueos/ocupación/DST/cobertura tienen matriz verde.
- [ ] Consulta remota funciona loopback sin convertir respuesta en reserva o dato offline.
- [ ] Usuario revisa semántica y mensajes de disponibilidad.

### Fase 4: proyección y conciliación
- [ ] T12: Proyectores de agenda y ocupación.
- [ ] T13: Conciliación de provisionales.

### C04: Sincronización completa del componente
- [ ] Proyección, captura, remove, recibos parciales y reapertura conservan trabajo.
- [ ] `cargo test --workspace --locked` y `pnpm build:desktop` pasan.
- [ ] Usuario revisa conciliación antes de IPC/UI.

### Fase 5: IPC e interfaz
- [ ] T14: Comandos de escritura.
- [ ] T15: Comandos de consulta.
- [ ] T16: Registro y cliente TypeScript.
- [ ] T17: UI de Agenda.
- [ ] T18: UI de Disponibilidad.

### C05: Vertical usable desde escritorio
- [ ] Los 14 comandos y permisos son coherentes; UI local/remota funciona con datos sintéticos.
- [ ] `pnpm quality` y `pnpm build:desktop` pasan; revisión manual Windows registrada.
- [ ] Usuario revisa la vertical antes de pruebas finales.

### Fase 6: contrato, integración y aceptación
- [ ] T19: Fixtures del consumidor.
- [ ] T20: Integración loopback de dos perfiles.
- [ ] T21: Guías y calidad.
- [ ] T22: API/OIDC real de desarrollo.

### C06: Aceptación de Agenda y Disponibilidad
- [ ] Suite, contrato, integración loopback, calidad y build de escritorio pasan.
- [ ] Prueba real API/OIDC y recorrido UI tienen evidencia; límites no ejecutados permanecen explícitos.
- [ ] Usuario revisa y acepta el cierre. Si T22 no pasa, el plan permanece abierto y no se declara verificación externa.

## Verificación transversal obligatoria

- Escrituras: rollback local+Outbox, ediciones sucesivas, base antigua, `NO_CHANGES`, ocho dependencias, permisos, época nueva y corrección con nuevo ID.
- Tiempo: inicio/fin, vigencia inclusiva, intervalo exclusivo, unión de reglas, excepción cerrada/reemplazo, DST gap/fold, 1440, consulta de 31 días y cambio de zona sin mover instantes.
- Ocupación: cuatro estados ocupantes, cancelación, contigüidad, bloqueo global/cuenta y cita fuera de ventana.
- Cobertura: dentro/sin/fuera de ventana; nunca confundir desconocido con libre o sin huecos.
- Recuperación: duplicado, respuesta perdida, aceptación parcial, `VERSION_CONFLICT`, `SLOT_UNAVAILABLE`, cursor/captura, remove/reset y revocación.
- Seguridad: aislamiento triple, global solo propietario/profesional, motivo global limitado, DTO sin ámbito/secrets y logs sin payloads.
- Plataformas: evidencia manual Windows; macOS/Linux solo se afirman si se ejecutan allí. No se repiten ni se dan por heredadas las pruebas pendientes de Patients.

## Riesgos y mitigaciones

| Riesgo | Impacto | Mitigación |
|---|---|---|
| Falsa disponibilidad por ocupación fuera de ventana | Alto | Cobertura explícita, estado insuficiente y opción remota; nunca inferir “libre” |
| Divergencia civil/UTC o DST | Alto | Jiff tzdb embebida, rechazo de ambigüedad y matriz determinista |
| Agenda sin ocupación real | Alto | Puerto de Citas y proyección mínima aceptada desde el inicio; no adapter vacío |
| Cambiar agenda sobre citas existentes | Alto | Prevalidación local conocida y aceptación final API; conservar intención ante conflicto |
| Duplicar efectos o mezclar provisional/aceptado | Alto | Outbox inmutable, recibo+proyección y pruebas parciales/reapertura |
| Fuga de motivo global o ámbito | Alto | Modelo nullable, control Rust y pruebas propietario/asistente/cuenta |
| Deriva de tzdb entre sistemas | Medio | `tzdb-bundle-always`, verificación de 598 nombres y registro de versión |
| Sobrealcance de Citas | Medio | Solo ocupación de lectura; casos de Citas quedan para otro plan |
| Entorno real no disponible | Medio | Fakes/loopback completos; T22/C06 abiertos sin sustituir evidencia |

## Supuestos

- Se incluye UI mínima porque se pidió un enfoque completo; no se introduce un sistema visual definitivo.
- Se acepta una proyección de ocupación de Citas como parte necesaria de Disponibilidad, no una implementación anticipada del ciclo de Citas.
- `0008` y `0009` quedan reservados al materializar el plan; cualquier colisión exige revisar el plan antes de código.
- API mantiene `api-v1.1`; cualquier incompatibilidad vuelve al repositorio propietario y detiene el punto afectado.
- No hay tareas paralelas. Se preservan Patients T17 y todos los límites anteriores.
- Solo datos sintéticos. Cifrado SQLite, claves, publicación, producción y pruebas macOS/Linux requieren evidencia/planes propios.

## Preguntas abiertas

Ninguna decisión funcional pendiente para este plan. Las políticas de duración fija/variable de UI no se cierran: la entrega trabaja con intervalos y duración 1–1440, sin imponer bloques fijos.
