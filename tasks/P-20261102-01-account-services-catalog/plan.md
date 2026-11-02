# P-20261102-01-account-services-catalog: Cuenta y catálogo de servicios

**Status:** Draft — pendiente de aprobación del usuario
**Created:** 2026-11-02
**Task list:** [todo.md](todo.md)
**Plan relacionado:** [P-20261029-03-core-engine](../P-20261029-03-core-engine/plan.md) (motor transversal, en progreso; este plan lo consume, no lo reabre). Contratos iniciales: [core-next-tracks.md](../../docs/operations/core-next-tracks.md) §«Cuenta y catálogo de servicios».

## Contexto y resultado

El motor transversal (`identity`, `devices`, `platform`, `sync`) ya puede autenticar una instalación, registrar el dispositivo en la cuenta, guardar intenciones en el Outbox, enviarlas con idempotencia, aplicar recibos, descargar cambios/capturas y proyectar el estado remoto aceptado. El proyector compuesto actual es `()` (vacío): los 16 `entityType` del contrato se descartan al aplicarse.

Este plan implementa, de forma **absolutamente vertical**, los dos primeros componentes de negocio sobre ese motor:

1. **`account`** — cuenta profesional: vista aceptada de la cuenta, preferencias locales y cambios administrativos (datos, política, estado), que el contrato marca **online-only**.
2. **`services`** — catálogo del profesional: servicios, ofertas por cuenta (`professional_services`) y perfil profesional, con intenciones offline y vista aceptada proyectada.

Al terminar, el equipo debe poder, desde el escritorio y con datos sintéticos: ver los datos de la cuenta y su política, editarlos con conexión, mantener preferencias locales sin red, crear/editar/activar servicios y ofertas **sin conexión** conservando intenciones durables, verlas aparecer como aceptadas tras sincronizar, y observar conflictos/rechazos para revisión. La agenda de hoy y mañana de estos datos seguirá disponible tras reiniciar sin red.

**Qué NO es este plan:** Pacientes, Agenda/Disponibilidad, Citas, Patient Hub, recordatorios, lista de espera. No se implementa lógica de disponibilidad ni de reservas. No se tocan las migraciones `0001`–`0004` ni las reglas internas de `sync` (solo se registra el proyector compuesto en los puntos de integración existentes).

## Comprobaciones de arquitectura (tasks/README.md)

- **Propietarios y casos nativos:** `account` y `services`, en Rust, con dominio/aplicación/adaptadores por componente. React solo invoca comandos Tauri específicos.
- **Offline:** consulta de cuenta/catálogo/ofertas y preferencias funciona sin red con los últimos datos aceptados o creados localmente; creación/edición de servicios y ofertas guarda intención durable (misma transacción SQLite que el cambio local); cambios administrativos de cuenta exigen conexión y **no** se registran como trabajo offline (ADR del motor, §6).
- **Contrato de sincronización:** las 10 acciones usadas viajan por `POST /v1/accounts/{id}/sync/operations` con envoltorio existente (`OperationEnvelope`); el escritorio no añade campos al cuerpo (cuenta/actor/dispositivo salen de ruta/credencial). API revalida permisos vigentes, versiones y reglas compartidas.
- **Propietarios cloud:** Nightingale API acepta todo; el Hub no participa; sin servicios nuevos en este repositorio.
- **Comprobaciones enfocadas:** sin conexión (crear oferta → intención pendiente → reapertura conserva), servicio cloud caído (intención queda `retryable`; consultas locales siguen), recuperación (respuesta perdida → `unknown` → GET por ID; `VERSION_CONFLICT` conserva intención; `remove` retira vista sin tocar intenciones).
- **Contratos a fijar antes de paralelismo (C01):** DTO por comando, lista de acciones, propiedad de tablas, números de migración reservados (`0005`, `0006`), puertos públicos `AccountViewQuery` y `CatalogQuery`, y puntos calientes de integración (`lib.rs`, `build.rs`, `capabilities/default.json`, `components/mod.rs`, registro del proyector compuesto en `identity/desktop.rs`).

## Source of Truth

- [Master Plan](../../docs/master-plan.md): §9.2 (cuenta profesional), §9.5 (catálogo de servicios), §§6–7 (hexagonal y empaquetado), §11 (kernel y tiempo), §§13–15 (persistencia, sincronización, citas y disponibilidad), §22 (esqueleto UI), §23 (pruebas), §26 (calidad de componente).
- [Arquitectura técnica](../../docs/technical-architecture.md): §§7, 9–11, 13, 17.
- [ADR del motor transversal](../../docs/decisions/desktop-core-engine.md): tres estados separados, proyección remota no-ORM, operación inmutable, permisos, recuperación conservadora, colaboración e integración central.
- [Contrato API v1 fijo](../../docs/contracts/api-v1/nightingale-api.md) (`api-v1.1`): §5.1 (cuenta/profesional), §5.3 (servicio/oferta/agenda), §5.5 (proyecciones), §8 (catálogo de acciones, online-only, resultados), §10 (formatos y límites). Artefactos JSON del bundle **no se editan**.
- [core-next-tracks.md](../../docs/operations/core-next-tracks.md): sección «Cuenta y catálogo de servicios» y «Recorrido para añadir un caso».
- [Guía de desarrollo](../../docs/operations/development.md): convenciones de migración, calidad (`pnpm quality`), IPC y colaboración.

## Alcance vertical detallado

### Componente `account`

**Responsabilidad:** presentar la cuenta profesional activa y sus preferencias; canalizar los cambios administrativos que solo con conexión.

**Propiedad de datos (nuevas tablas, migración `0005`):**

| Tabla | Propietario | Contenido |
|---|---|---|
| `account_view` | `account` (proyector, entityType `account`) | `account_id`, `display_name`, `time_zone`, `location_name/address/city/state/country`, `no_show_grace_minutes`, `status`, `version`, `updated_at`. Solo la escribe el proyector. |
| `account_preferences` | `account` (local, **no sincronizada**) | `account_id`, `key`, `value` (JSON), `updated_at`. Preferencias de dispositivo permitidas. |

**Casos de uso públicos:**
- `get_account_view` — lectura offline de `account_view` + preferencias; DTO camelCase sin datos de otros ámbitos.
- `get_preferences` / `set_preference` — lectura/escritura local en su propia transacción; sin Outbox.
- `update_account_details` — **online-only** (propietario): patch no vacío de nombre, zona y/o lugar.
- `update_account_policy` — **online-only** (propietario): `noShowGraceMinutes` (0–1440).
- `set_account_status` — **online-only** (propietario): `active`/`inactive`. Expuesto por completitud contractual; UI mínima.

**Regla online-only (diseño):** estos tres casos **no escriben intención ni entidad local**. Flujo: exigen sesión activa y acceso `active`; refrescan `/access` como prueba de conectividad (si falla → se rechaza con error estable, sin persistir nada); construyen el `OperationEnvelope` en memoria (ID UUIDv7 nuevo por invocación, reutilizado en los reintentos de transporte de esa misma invocación); envían con `SyncAPI::send_operations`; y actúan según el resultado: `accepted` → disparan `sync_now` para traer la vista actualizada; `duplicate` → se resuelve con `originalResult`; `conflict`/`rejected` → se muestran al usuario; resultado desconocido tras reintentos → error visible pidiendo repetir o sincronizar (la vista remota manda; una repetición manual es una operación nueva). Nada se guarda en SQLite: el estado de cuenta solo cambia por proyección de cambios aceptados.

**Invariantes:** toda entidad operativa pertenece a la cuenta seleccionada; la vista nunca afirma aceptación sin recibo/cambio aceptado; las preferencias locales no sustituyen política central (una preferencia no puede fijar `noShowGraceMinutes` ni la zona).

**Eventos públicos:** ninguno nuevo en este track (los eventos de dominio quedan para cuando exista un bus; los recibos y cambios remotos no son eventos de negocio, igual que `sync`).

### Componente `services`

**Responsabilidad:** catálogo del profesional y sus ofertas en la cuenta; perfil profesional. Escrituras solo del **propietario**; el asistente recibe vista de lectura.

**Propiedad de datos (nuevas tablas, migración `0006`):**

| Tabla | Propietario | Contenido |
|---|---|---|
| `service_catalog_view` | `services` (proyector, entityType `service`) | `service_id`, `professional_id`, `name`, `default_duration_minutes`, `status`, `version`. Solo el proyector escribe. |
| `service_offer_view` | `services` (proyector, entityType `service_offer`) | clave compuesta (`professional_id`, `service_id`), `name`, `duration_minutes_override`, `effective_duration_minutes`, `status`, `version`, **`service_version`**. Solo el proyector escribe. |
| `professional_view` | `services` (proyector, entityType `professional`) | `professional_id`, `public_name`, `status`, `version`, `user_id`. Solo el proyector escribe. |
| `service_local` | `services` | intención local: `service_id`, `operation_id`, `payload` (JSON), `sync_status` (`pending`/`accepted`/`conflict`/`rejected`), `created_at`, `updated_at`. Escrita por casos de uso junto al Outbox. |
| `service_offer_local` | `services` | intención local análoga con clave compuesta y `operation_id`. |

**Casos de uso públicos (todos validan acción contra `AccountAccess::allows_offline`, que ya cruza `allowedActions` con `capabilities.json`):**
- `list_catalog` — lectura offline fusionada: vista aceptada + intenciones locales no resueltas, con marca de provisional y estado de sincronización.
- `create_service` — `expectedVersion=0`, payload `professionalId`, `name`, `defaultDurationMinutes` (entero 1–1440).
- `update_service` — patch no vacío (`name` y/o `defaultDurationMinutes`), `expectedVersion` = versión conocida.
- `set_service_status` — `active`/`inactive`.
- `offer_service` — clave compuesta `{professionalId, serviceId}`, `expectedVersion=0`, `durationMinutesOverride` opcional (nulo → duración base).
- `update_service_offer` — exige `durationMinutesOverride` **presente** en el patch (admite `null` para volver a la base).
- `set_service_offer_status` — `active`/`inactive`.
- `update_professional_profile` — `publicName` (propietario).

Cada mutación offline: una sola transacción `IMMEDIATE` que escribe la intención local y llama a `sync::application::record_operation` con el mismo `tx`. La vista aceptada **solo** avanza por el proyector al descargar cambios/captura o por el recibo aceptado que resuelve la intención (marcando `service_local.sync_status='accepted'`; la fila local provisional se retira al confirmarse la vista aceptada, sin borrar la intención del Outbox, que conserva su recibo).

**Proyectores (`RemoteProjector` por entityType):**
- `AccountProjector` (`account`), `ServicesProjector` (`service`, `service_offer`, `professional`). Ignoran tipos que no poseen (composición por tupla/estructura delegante; el integrador los compone en `identity/desktop.rs`, reemplazando los `&()` actuales en las líneas ~434 y ~538).
- `project` escribe solo su vista con la transacción de la página; `reset` retira solo sus filas del ámbito al instalar captura. Nada de HTTP ni Outbox dentro del proyector.
- **Regla de versiones dobles de oferta:** un cambio de `service_offer` con `version` igual pero `serviceVersion` mayor **sí** se proyecta (igual no es menor); un `version` estrictamente menor no se reaplica. Caso de prueba obligatorio (contrato §5.3).
- `remove` retira filas de la vista aceptada y **nunca** toca `service_local`/`service_offer_local` ni el Outbox.

**Invariantes:** duración entera 1–1440 (local y remota); `effective = override ?? default` se recalcula solo desde estado aceptado; patch vacío rechazado localmente (evita `NO_CHANGES` innecesario); desactivar servicio u oferta no cancela ni altera citas (no hay citas aún; la regla queda documentada para el track de Citas); el catálogo de un asistente es de solo lectura (su `allowedActions` no incluye estas acciones y la UI lo refleja).

## Qué se reutiliza y cómo (mapa de reuso)

| Pieza existente | Ubicación | Uso en este plan |
|---|---|---|
| `record_operation` | `components/sync/application.rs` | Llamar dentro de la misma transacción que la escritura de `service_local`/`service_offer_local` (patrón ya usado en `identity/application.rs` para `RegisterAccountDevice`). |
| `SqliteOperationStore` | `components/sync/adapters/outbound.rs` | Implementación `OperationStore`; consulta de revisiones (`ReviewSummary`) ya disponible para la UI de conflictos. |
| `OperationEnvelope`, huella, `CONTRACT_VERSION` | `components/sync/domain.rs` | Envoltorio y serialización canónica sin cambios; los payloads nuevos son `serde_json::Value` validados por el dominio. |
| `RemoteProjector`, `RemoteChange`, `EntityType` | `components/sync/remote.rs` | `EntityType` ya incluye `Account`, `Professional`, `Service`, `ServiceOffer`. Se implementa el trait por componente; el proyector vacío `()` queda sustituido por el compuesto. |
| Modelos de transporte (solo frontera de adaptador) | `components/sync/adapters/remote_models.rs` | `AccountResponse`, `ProfessionalResponse`, `ServiceResponse`, `ServiceOfferResponse` y `RemoteState` ya están validados contra el bundle. Son **esquemas remotos del API, no entidades de negocio**: solo se usan dentro de la frontera del adaptador (el proyector los recibe y los traduce a los modelos de persistencia/lectura propios de `account`/`services`). Los componentes definen sus propias entidades de dominio y DTO de IPC; **no se reutilizan como dominio ni se exponen a React**. Cero modelos de transporte nuevos. |
| `AccountAccess::allows_offline` | `components/identity/domain.rs` | Autorización local de cada acción (offline + online-only ya cruzados con `capabilities.json` del bundle). |
| `IdentityAPI::access` refresh | `components/identity/api.rs` | Prueba de conectividad y permiso vigente antes de casos online-only. |
| `SyncAPI::send_operations` | `components/sync/adapters/api.rs` | Envío directo de casos online-only y, vía Outbox, de las intenciones de catálogo. |
| `SyncCoordinator` / `sync_now` / `DesktopSession` | `components/sync/coordinator.rs`, `identity/desktop.rs` | Tras aceptar un caso online-only se dispara `sync_now` para traer la vista; el ciclo existente procesa el Outbox de catálogo sin cambios. |
| `SqliteAccessStore`, `SqliteInstallationStore` | `components/identity/repository.rs`, `components/devices` | Contexto de identidad y alcance (cuenta/usuario/dispositivo) para ámbito de tablas y comandos. |
| `platform::sqlite` (runner, `open`) | `src-tauri/src/platform/sqlite/` | Convención de migraciones `NNNN_componente_slug.sql` con `PRAGMA user_version`; reservar `0005`/`0006` en C01. |
| `platform::observability::Event` | `src-tauri/src/platform/observability.rs` | Metadatos de caso de uso (sin payloads ni datos personales). |
| Errores estables y códigos | `shared_kernel::error`, `http::{APICode, APIFailure}` | Conversión dominio→aplicación→IPC con códigos del contrato (`VALIDATION_ERROR`, `PERMISSION_DENIED`, `VERSION_CONFLICT`, `DEPENDENCY_UNAVAILABLE`…). |
| Patrón IPC + TS | `identity/desktop.rs`, `components/sync/adapters/inbound.rs`, `src/shared/api/*.ts`, `capabilities/default.json`, `build.rs`, `lib.rs` | Un comando por caso de uso, DTO camelCase, permiso explícito por comando; nuevos clientes `account.ts` y `services.ts`. |
| Patrones de prueba | `components/sync/*_tests.rs`, `identity/activation_tests.rs` | Fixtures SQLite temporales, reloj e IDs controlados, servidor HTTP sintético loopback, pruebas de contrato sobre el bundle fijo. |

## Niveles de dependencia (quién dependerá de esto)

- **Nivel 0 — existente:** `shared_kernel`, `platform`, `identity`, `devices`, `sync`, `diagnostics`, `storage`.
- **Nivel 1 — este plan:** `account`, `services`. (El track de **Pacientes** también es nivel 1 e independiente; puede trabajarse en paralelo una vez fijados en C01 los contratos compartidos.)
- **Nivel 2 — dependen de este plan por sus puertos públicos:**
  - **`scheduling` (Agenda):** consumirá `services::application::CatalogQuery` (servicio, profesional, duración de referencia efectiva, `version` + `serviceVersion`) para calcular disponibilidad; jamás lee tablas de `services`. Su contrato de ocupación con Citas se definirá en su propio plan.
  - **`appointments` (Citas):** consumirá `CatalogQuery` (nombre de servicio, duración de referencia ofertada, nombre profesional para los snapshots acordados) y `account::application::AccountViewQuery` (zona horaria, `noShowGraceMinutes`, lugar) en el momento de crear/reprogramar; más el puerto de Pacientes. Las nueve referencias snapshot de la cita se toman de estos puertos, no se recalculan del catálogo actual (contrato §5.4).
- **Nivel 3 — dependen transitivamente:** lista de espera (Citas + Pacientes), indicadores, y los futuros consumidores cloud. El Patient Hub no consume estos componentes: actúa vía Nightingale API.

**Puertos públicos que este plan fija (y congela para niveles 2+):**
- `account::application::AccountViewQuery` → `AccountView` (datos de cuenta y política de lectura) y `AccountPolicy` (`noShowGraceMinutes`).
- `services::application::CatalogQuery` → listado y consulta de ofertas con duración efectiva y ambas versiones, servicios y perfil profesional.
Ningún consumidor futuro importa repositorios ni tablas de estos componentes (PMD §7.2).

## Datos, offline y excepciones cloud

- Ventana local: lo que el dispositivo haya aceptado o creado; la captura inicial del motor ya trae `account`, `professional`, `service`, `service_offer` dentro de su ventana. La consulta funciona tras reinicio sin red.
- Sin red: catálogo/ofertas se consultan y mutan como intenciones durables; administración de cuenta se rechaza con mensaje claro (requiere conexión).
- API caída: intenciones quedan `pending`/`retryable`; vistas locales siguen; casos online-only fallan sin persistir.
- Recuperación: respuesta perdida → `unknown` → recuperación por GET del propio ID (motor existente); `CURSOR_EXPIRED`/época nueva → captura nueva, vistas se reconstruyen por `reset`+`project`, intenciones se conservan.
- Permisos: cada intención lleva la `permission_epoch` del contexto (motor); al reconectar, API aplica permisos vigentes; si el asistente perdió una acción, la intención se rechaza sin efecto y queda para revisión.

## Contrato de sincronización usado (api-v1.1)

| Caso | Acción | entityType | Clave | Versión esperada | Modo |
|---|---|---|---|---|---|
| Crear servicio | `CreateService` | `service` | UUIDv7 servicio | 0 | offline |
| Editar servicio | `UpdateService` | `service` | UUIDv7 servicio | >0 conocida | offline |
| Estado de servicio | `SetServiceStatus` | `service` | UUIDv7 servicio | >0 conocida | offline |
| Ofertar | `OfferService` | `service_offer` | `{professionalId, serviceId}` | 0 | offline |
| Editar oferta | `UpdateServiceOffer` | `service_offer` | `{professionalId, serviceId}` | >0 conocida | offline |
| Estado de oferta | `SetServiceOfferStatus` | `service_offer` | `{professionalId, serviceId}` | >0 conocida | offline |
| Nombre profesional | `UpdateProfessionalProfile` | `professional` | UUIDv7 profesional | >0 conocida | offline |
| Datos de cuenta | `UpdateAccountDetails` | `account` | UUIDv7 cuenta | >0 conocida | **online-only** |
| Política de cuenta | `UpdateAccountPolicy` | `account` | UUIDv7 cuenta | >0 conocida | **online-only** |
| Estado de cuenta | `SetAccountStatus` | `account` | UUIDv7 cuenta | >0 conocida | **online-only** |

Formatos y límites del contrato §10: duraciones enteras 1–1440; JSON camelCase; ausencia ≠ nulo (los patches solo envían campos elegidos; `durationMinutesOverride` se envía siempre en `UpdateServiceOffer`, con `null` explícito cuando se retira); lotes y tamaños los impone el motor existente.

## Esqueleto de interfaz (mínimo, PMD §22)

- `features/settings`: vista de cuenta (nombre, zona, lugar, política, estado), formulario de edición habilitado solo con sesión activa y rol propietario, mensaje «requiere conexión» cuando corresponda; preferencias locales.
- `features/services`: lista fusionada (aceptadas + provisionales con marca), alta de servicio, edición, activar/desactivar, ofertar en la cuenta, ajuste de duración de oferta (con opción «usar duración base» = `null`), nombre profesional. Lectura para asistentes sin controles de escritura.
- Centro de sincronización existente: las revisiones de catálogo aparecen con el mecanismo `ReviewSummary` actual.
- Nada de decisiones visuales definitivas; componentes simples de Tailwind.

## Tareas y checkpoints

Orden detallado con estados en [todo.md](todo.md). Resumen:

- **T01 — Cerrar contratos del track:** DTO por comando, tabla de acciones/claves/versión, propiedad de tablas, reserva de migraciones `0005`/`0006`, firmas de `AccountViewQuery` y `CatalogQuery`, puntos calientes de integración. **C01: Contratos congelados.**
- **T02 — Componentes y migraciones:** esqueletos `account`/`services` (domain/application/adapters), `mod.rs`, READMEs breves (PMD §27.2), migraciones `0005_account.sql` y `0006_services.sql`, runner verificado.
- **T03–T06 — Vertical `account`:** dominio y aplicación (vista, preferencias, validaciones de patch y rangos); repositorios y proyector `account`; casos online-only con envío directo y disparo de `sync_now`; IPC + `account.ts` + UI settings. **C02: revisión vertical account.**
- **T07–T10 — Vertical `services`:** dominio y aplicación (8 casos, validación 1–1440, patch no vacío, `allows_offline`); repositorios e intenciones locales con `record_operation` en la misma transacción; proyectores con regla de versiones dobles; IPC + `services.ts` + UI catálogo. **C03: revisión vertical services.**
- **T11 — Contratos de consumidor:** fixtures contra el bundle fijo (huellas del manifiesto, presencia de las 10 acciones, correspondencia acción↔payload según `capabilities.json`, modelos de estado ya existentes). Sin editar artefactos.
- **T12 — Integración y sincronización de punta a punta:** proyector compuesto registrado en `identity/desktop.rs`; escenarios sintéticos con servidor loopback: crear servicio offline → dispatch → `accepted` → proyección y resolución de la intención; `VERSION_CONFLICT` conserva intención; `remove` retira vista; reapertura conserva todo; oferta con `serviceVersion` mayor y `version` igual se proyecta.
- **T13 — Calidad y guías:** `pnpm quality`, `pnpm build:desktop`, `git diff --check`, READMEs finales por componente. **C04: aceptación del track.**

## Verificación

```powershell
cargo test --workspace --locked components::account components::services
cargo test --workspace --locked components::sync::contract_tests
pnpm quality
pnpm build:desktop
git diff --check
```

Las pruebas usan datos sintéticos, relojes e IDs controlados y HTTP loopback; no acreditan API/OIDC de desarrollo ni revisión de usuario (C02–C04 siguen ese estándar).

## Decisiones abiertas registradas (no bloquean el track)

1. La repetición manual de un caso online-only con resultado desconocido genera una operación nueva (semántica online-only: no hay intención durable que recuperar). Aceptado por coherencia con el ADR; se documenta en el README de `account`.
2. `service_local`/`service_offer_local` conservan el payload completo de la intención para permitir edición/reintento y revisión de conflictos; la vista aceptada sigue en las tablas `_view`. Si un track posterior añade un bus de eventos, la resolución de intenciones se puede simplificar sin cambiar contratos públicos.
3. El alta administrativa de cuenta (crear la consulta) queda fuera: es un script administrativo del servidor (contrato §6).
4. **Propiedad futura de los modelos de transporte (decisión del usuario, 2026-11-02):** a medio plazo los modelos de `sync/adapters/remote_models.rs` deberían ser reemplazados por los modelos que definan los componentes propietarios (p. ej. `account`/`services`). Esa refactorización se aplaza explícitamente hasta el final, cuando los componentes estén estables; ahora solo se respeta la frontera: transporte en el adaptador, dominio propio en el componente.
