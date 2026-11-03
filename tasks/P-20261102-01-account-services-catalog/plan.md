# P-20261102-01-account-services-catalog: Cuenta y catálogo de servicios

**Status:** In progress — C02 presentado el 2026-11-02; ejecución detenida por solicitud del usuario
**Created:** 2026-11-02
**Updated:** 2026-11-02
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

## T01 — Contratos para revisión C01 (2026-11-02)

**C01 aprobado:** el usuario revisó los contratos y autorizó continuar («Me parece bien, continua», 2026-11-02). Se mantienen C02–C04 como revisiones previas a la siguiente fase.

### Preparación y alcance confirmado

Se leyeron AGENTS.md, este plan y todo.md, el ADR del motor, el track de cuenta/catálogo en core-next-tracks.md y la guía API §§5.1, 5.3, 5.5, 8 y 10. Se contrastaron PMD §§9.2/9.5, las reglas de tareas, las capacidades y los esquemas del bundle fijo.

El código confirma `record_operation(tx, store, ids, clock, input)` y `SqliteOperationStore`, incluida la consulta de resultados por operación; `AccountAccess::allows_offline` cruza permiso, estado activo y capacidades. Los cuatro modelos de transporte y `RemoteState` existen. Los dos proyectores vacíos siguen en `identity/desktop.rs:434` y `:538`. `apply_group` ya deja pasar versiones iguales y omite versiones estrictamente menores. No se necesita cambiar esa regla de sync.

El usuario autorizó incluir `src-tauri/src/platform/sqlite/mod.rs`: su lista fija debe registrar 0005 y 0006. También autorizó corregir el recuento a **14 comandos: 6 de cuenta y 8 de catálogo**. Hay 10 acciones remotas: 3 de cuenta y 7 de catálogo. Las otras cuatro entradas son lecturas o preferencias locales. La autorización de ejecución no sustituye la revisión C01.

Estos contratos son la propuesta completa de T01; se congelan al aprobar C01. No hay código de producto en T01. No se usa la skill de planificación para rehacer el plan ya aprobado. `minimal-code` y `docs-first-framework-code` se aplicarán antes de los cambios de código que correspondan.

### Tipos y reglas comunes de IPC

Las siguientes formas describen JSON de IPC, no tipos de transporte de API. `Id` es string UUIDv7 canónico; `Version` es entero positivo; `Status` es `active | inactive`; `Duration` es entero 1–1440. `?` significa campo ausente permitido; `| null` significa nulo explícito permitido. Se rechazan campos extra. Los comandos con entrada reciben `{ request: <DTO> }`; los comandos sin entrada se invocan sin argumentos. Rust obtiene cuenta, actor, dispositivo y época desde la sesión seleccionada: React no los puede sustituir.

Las lecturas usan el último acceso activo de la sesión, sin HTTP. `list_catalog` no tiene acción de escritura en capabilities: no se inventa `ListCatalog` ni se llama a `allows_offline` con un nombre ajeno al catálogo. Las siete mutaciones sí comprueban su acción exacta con `allows_offline`; además, exigen propietario y profesional propio. Los tres cambios de cuenta exigen propietario, sesión y acceso activos, y permiso de la acción tanto antes como después del refresco de `/access`.

`CommandError = { code: string, message: string, operationId?: Id, correlationId?: Id }`. El componente traduce errores de aplicación y transporte a códigos estables y mensajes seguros; no expone estado remoto crudo, tokens, SQL ni detalles de servidor. Se reutilizan los códigos existentes (p. ej. `VALIDATION_ERROR`, `AUTHENTICATION_REQUIRED`, `PERMISSION_DENIED`, `TEMPORARY_FAILURE`, `SERVICE_UNAVAILABLE`, `VERSION_CONFLICT`). No se amplía shared_kernel para añadir errores específicos del track.

### DTO de los 14 comandos

| Comando / caso Rust | Entrada DTO (contenido de `request`) | Salida DTO |
|---|---|---|
| `account_get_view` / `get_account_view` | Sin entrada | `AccountScreen` |
| `account_get_preferences` / `get_preferences` | Sin entrada | `Preferences` |
| `account_set_preference` / `set_preference` | `{ key: PreferenceKey, value: PreferenceValue }` | `Preferences` tras confirmar la transacción local |
| `account_update_details` / `update_account_details` | `{ expectedVersion: Version, patch: AccountDetailsPatch }` | `AccountMutationResult` |
| `account_update_policy` / `update_account_policy` | `{ expectedVersion: Version, noShowGraceMinutes: integer }` | `AccountMutationResult` |
| `account_set_status` / `set_account_status` | `{ expectedVersion: Version, status: Status }` | `AccountMutationResult` |
| `services_list_catalog` / `list_catalog` | Sin entrada | `CatalogScreen` |
| `services_create_service` / `create_service` | `{ professionalId: Id, name: string, defaultDurationMinutes: Duration }` | `LocalMutationResult` (Rust genera serviceId y expectedVersion=0) |
| `services_update_service` / `update_service` | `{ serviceId: Id, expectedVersion: Version, patch: ServicePatch }` | `LocalMutationResult` |
| `services_set_service_status` / `set_service_status` | `{ serviceId: Id, expectedVersion: Version, status: Status }` | `LocalMutationResult` |
| `services_offer_service` / `offer_service` | `{ professionalId: Id, serviceId: Id, durationMinutesOverride?: Duration | null }` | `LocalMutationResult` (expectedVersion=0) |
| `services_update_service_offer` / `update_service_offer` | `{ professionalId: Id, serviceId: Id, expectedVersion: Version, durationMinutesOverride: Duration | null }` | `LocalMutationResult` |
| `services_set_service_offer_status` / `set_service_offer_status` | `{ professionalId: Id, serviceId: Id, expectedVersion: Version, status: Status }` | `LocalMutationResult` |
| `services_update_professional_profile` / `update_professional_profile` | `{ professionalId: Id, expectedVersion: Version, publicName: string }` | `LocalMutationResult` |

`AccountDetailsPatch = { displayName?: string, timeZone?: string, locationName?: string, locationAddress?: string, locationCity?: string, locationState?: string, locationCountry?: string }`. Debe contener al menos un campo; los textos no pueden estar vacíos, la zona debe ser IANA y el país ISO alfa-2 asignado. Este formulario local no ofrece borrar con null los datos obligatorios de cuenta. No rellena campos omitidos al construir el payload. `ServicePatch = { name?: string, defaultDurationMinutes?: Duration }`, también no vacío, sin null para nombre o duración base. `UpdateServiceOffer` distingue campo ausente (error) de null (volver a base). Política admite 0–1440 enteros. Se rechazan fracciones y valores fuera del rango.

Preferencias iniciales permitidas: `theme` (`system | light | dark`) y `servicesShowInactive` (boolean). `PreferenceKey` es esa lista cerrada y `PreferenceValue` debe corresponder a la clave. `Preferences = { theme: ..., servicesShowInactive: boolean }`, con valores por defecto `system` y `true` si no hay fila. No se aceptan claves de zona, política, permisos o credenciales. Las preferencias pertenecen al usuario/dispositivo/cuenta local y no generan operaciones.

### DTO de lectura y resultados

Los siguientes tipos son propios del componente y mantienen nombres camelCase al serializar:

```text
AccountPolicy = { noShowGraceMinutes: integer }
AccountView = {
  accountId: Id, displayName: string, timeZone: string,
  locationName: string, locationAddress: string, locationCity: string,
  locationState: string, locationCountry: string,
  policy: AccountPolicy, status: Status, version: Version,
  updatedAt: string | null
}
AccountScreen = {
  account: AccountView | null, preferences: Preferences,
  canManage: boolean
}
ServiceView = {
  serviceId: Id, professionalId: Id, name: string,
  defaultDurationMinutes: Duration, status: Status, version: Version
}
ServiceOfferView = {
  professionalId: Id, serviceId: Id, name: string,
  defaultDurationMinutes: Duration, durationMinutesOverride: Duration | null,
  effectiveDurationMinutes: Duration, serviceStatus: Status,
  status: Status, version: Version, serviceVersion: Version
}
ProfessionalView = {
  professionalId: Id, userId: Id, publicName: string,
  status: Status, version: Version
}
LocalChange = {
  operationId: Id, action: string, expectedVersion: integer,
  syncStatus: pending | in_flight | unknown | retryable | accepted | conflict |
              rejected | review_required,
  code: string | null
}
CatalogRow<T, D> = {
  accepted: T | null, provisional: D | null,
  isProvisional: boolean, localChanges: LocalChange[]
}
CatalogScreen = {
  services: CatalogRow<ServiceView, ServiceDraft>[],
  offers: CatalogRow<ServiceOfferView, ServiceOfferDraft>[],
  professionals: CatalogRow<ProfessionalView, ProfessionalDraft>[],
  canManage: boolean
}
LocalMutationResult = {
  operationId: Id, entityType: service | service_offer | professional,
  entityKey: Id | { professionalId: Id, serviceId: Id }, syncStatus: pending
}
AccountMutationResult = {
  operationId: Id, status: accepted | conflict | rejected | retryable_failure,
  entityVersion: Version | null, code: string | null,
  refreshStatus: requested | failed | not_requested
}
```

`ServiceDraft = { serviceId: Id, professionalId: Id, name: string, defaultDurationMinutes: Duration, status: Status }`; `ServiceOfferDraft = { professionalId: Id, serviceId: Id, durationMinutesOverride: Duration | null, status: Status }`; `ProfessionalDraft = { professionalId: Id, publicName: string }`. Ningún borrador asigna versión aceptada ni duración efectiva aceptada. Las filas se identifican por la clave de su valor aceptado o provisional. Si solo queda una intención tras remove, conserva ese borrador para mostrar la intención; no recrea la vista aceptada.

La lectura agrupa por entidad y aplica las intenciones en orden local estable (`created_at`, `operation_id`), conservando todos los recibos visibles en `localChanges`. No mezcla intenciones de otros actores/dispositivos. El borrador muestra los valores elegidos por el usuario; los puertos para Citas/Agenda solo devuelven valores aceptados. El estado de envío se obtiene del resultado público de `SqliteOperationStore`; no se duplica la máquina de sync en las tablas del componente. Un resultado accepted no basta para retirar el provisional hasta que la proyección confirme la versión correspondiente. Conflictos y rechazos conservan la intención para revisión y una corrección crea un nuevo ID, sin editar la operación anterior. `canManage` solo orienta la UI; Rust vuelve a autorizar cada escritura.

Los cambios online-only mantienen envelope e ID solo en memoria durante la invocación. `duplicate` se normaliza mediante `originalResult`, incluido rechazo/conflicto. Solo un resultado original accepted dispara `sync_now`. Si falla esa actualización, se devuelve accepted con `refreshStatus=failed`; no se repite la escritura con otro ID ni se muestra una cuenta local inventada. Tras agotar reintentos sin resultado, se devuelve error seguro con operationId para informar al usuario. El refresco de acceso previo no guarda una intención ni una entidad; el sync posterior sí puede escribir sus propios metadatos y proyecciones. No se interpreta «nada en SQLite» como prohibir la proyección aceptada solicitada por el propio plan.

### Acciones y payloads congelados para C01

La tabla «Contrato de sincronización usado» conserva las 10 acciones, claves, modos y versiones. La siguiente tabla completa su correspondencia exacta con `capabilities.payloadModels` y el payload enviado (sin datos de contexto):

| Acción | Modelo (nombre final del valor en capabilities) | Payload |
|---|---|---|
| `CreateService` | `CreateServicePayload` | `{ professionalId, name, defaultDurationMinutes }` |
| `UpdateService` | `ServicePatch` | campos presentes de `{ name?, defaultDurationMinutes? }` |
| `SetServiceStatus` | `ServiceStatusPayload` | `{ status }` |
| `OfferService` | `OfferServicePayload` | `{ durationMinutesOverride? }` |
| `UpdateServiceOffer` | `UpdateServiceOfferPayload` | `{ durationMinutesOverride }` (null permitido) |
| `SetServiceOfferStatus` | `ServiceStatusPayload` | `{ status }` |
| `UpdateProfessionalProfile` | `ProfessionalPayload` | `{ publicName }` |
| `UpdateAccountDetails` | `AccountDetailsPatch` | solo campos presentes del patch |
| `UpdateAccountPolicy` | `AccountPolicyPayload` | `{ noShowGraceMinutes }` |
| `SetAccountStatus` | `MembershipStatusPayload` | `{ status }` |

Las claves compuestas contienen exactamente los dos UUIDv7 canónicos. Los IDs nuevos de servicio/operación se generan en Rust con el generador existente. Las dependencias causales de una oferta sobre un servicio aún pendiente se calculan desde la intención local y se incluyen en el envelope existente; no se permite un expectedVersion local ficticio para encadenar cambios. Si ya existe una operación sin resolver de la misma entidad, el caso impide otra escritura dependiente de una versión aceptada desconocida y pide sincronizar o revisar primero. No se muta la huella ni se cambia el contenido de una operación ya guardada.

### Propiedad, claves y reserva de las siete tablas

Todas las tablas incluyen `account_id`, `user_id`, `device_id` como ámbito, igual que el motor. La época acompaña a las operaciones mediante su contexto, no cambia la identidad de una preferencia. Las consultas y resets siempre filtran las tres columnas. No hay borrado en cascada desde vistas a intenciones.

| Migración reservada | Tabla / propietario | Clave dentro del ámbito y escritura |
|---|---|---|
| `0005_account.sql` | `account_view` / account | Una fila por ámbito; solo AccountProjector. Datos de AccountView, con política como columna no_show_grace_minutes. |
| `0005_account.sql` | `account_preferences` / account | `key`; caso set_preference en transacción local. JSON validado según clave, updated_at local. |
| `0006_services.sql` | `service_catalog_view` / services | `service_id`; solo ServicesProjector. Columnas de ServiceView. |
| `0006_services.sql` | `service_offer_view` / services | `(professional_id, service_id)`; solo ServicesProjector. Incluye ambas versiones, duración base y service_status para que la vista sea completa incluso sin una fila de catálogo visible. |
| `0006_services.sql` | `professional_view` / services | `professional_id`; solo ServicesProjector. Columnas de ProfessionalView. |
| `0006_services.sql` | `service_local` / services | `operation_id`; intenciones de service o professional, diferenciadas con entity_type. professional_id obligatorio; service_id nulo solo para UpdateProfessionalProfile. action, expected_version, payload, created_at, updated_at y estado derivado del recibo. |
| `0006_services.sql` | `service_offer_local` / services | `operation_id`, más professional_id y service_id; action, expected_version, payload y fechas. Caso offline escribe junto con Outbox en la misma transacción IMMEDIATE. |

La precisión de `service_local` evita una octava tabla y cubre la séptima mutación (perfil profesional), que no tiene serviceId. Las intenciones conservan su payload completo por operación y un `draft` JSON con los valores locales resultantes, para poder mostrar la intención incluso tras remove de la vista base; ese draft no se envía como payload de un patch. El Outbox conserva la operación y el recibo originales. La lectura/resolución de intenciones pertenece a services, nunca al proyector ni a una modificación de sync. Reset solo retira account_view o las tres vistas de services, según el proyector; remove retira solo la vista de su clave. La proyección de oferta acepta version igual con serviceVersion mayor y nunca retrocede version ni serviceVersion cuando la version propia sea igual.

Ambos SQL añaden su fila a schema_migrations y fijan PRAGMA user_version a 5 o 6. El integrador registra la lista en platform/sqlite/mod.rs y prueba base nueva y actualización desde 4. No cambia SQL 0001–0004, application_id, reglas del runner ni dependencias.

### Firmas públicas para Agenda y Citas

Firmas de contrato Rust (aún sin implementación): los IDs ServiceId y ProfessionalId son valores propios de services, validados con canonical_uuid_v7; no se añaden al kernel. AccountView/Policy y las tres vistas de catálogo son modelos propios, sin serde remoto, SQL o framework. Los adaptadores encapsulan conexión y autorización; el consumidor pasa el contexto autorizado. `Result` expresa errores estables de aplicación; `None` significa que no hay vista aceptada visible en ese ámbito.

```rust
// account::application
pub trait AccountViewQuery {
    fn account_view(&self, scope: &OperationContext) -> Result<Option<AccountView>>;
    fn account_policy(&self, scope: &OperationContext) -> Result<Option<AccountPolicy>>;
}

// services::application
pub trait CatalogQuery {
    fn list_services(&self, scope: &OperationContext) -> Result<Vec<ServiceView>>;
    fn service(&self, scope: &OperationContext, service: &ServiceId)
        -> Result<Option<ServiceView>>;
    fn list_offers(&self, scope: &OperationContext) -> Result<Vec<ServiceOfferView>>;
    fn offer(&self, scope: &OperationContext, professional: &ProfessionalId,
        service: &ServiceId) -> Result<Option<ServiceOfferView>>;
    fn list_professionals(&self, scope: &OperationContext) -> Result<Vec<ProfessionalView>>;
    fn professional(&self, scope: &OperationContext, professional: &ProfessionalId)
        -> Result<Option<ProfessionalView>>;
}
```

Estas consultas no hacen HTTP, no devuelven borradores, no usan DTO de transporte ni exponen repositorios. Devuelven también estados inactivos para que el consumidor aplique sus reglas explícitas. Las listas usan orden por ID estable. AccountView aporta zona, lugar, política y versión; ServiceOfferView aporta nombre y duración aceptados y ambas versiones; ProfessionalView aporta nombre aceptado. Las futuras citas tomarán sus snapshots desde estos valores, sin recalcularlos desde intenciones.

### Propietario único de integración y comprobación T01

El agente ejecutor de este plan registra: `components/mod.rs` y `platform/sqlite/mod.rs` en T02; `build.rs`, `capabilities/default.json` y `lib.rs` en T06/T10; composición de AccountProjector y ServicesProjector en los dos puntos de `identity/desktop.rs` en T12. Se usará una estructura delegante si el motor no implementa tuplas; no se cambia sync para añadir esa comodidad. No hay tareas paralelas autorizadas en esta ejecución.

T01 se verifica con lectura del código y contraste de las diez acciones/modelos con capabilities y esquemas, hashes del manifiesto, recuento de 14 comandos, enlaces locales y `git diff --check`. No necesita compilar ni ejecutar casos de producto porque solo cambia plan.md y todo.md. Las pruebas de consumidor ejecutables quedan en T11. API/OIDC reales, aprobación C01 y T02–T13 siguen pendientes.

**Resultado 2026-11-02:** comprobación Python por stdin (`python -`, biblioteca estándar) en verde: cinco hashes SHA-256 del manifiesto, diez acciones/modelos, siete acciones offline y tres online-only, obligatoriedad de durationMinutesOverride, catorce DTO de comando únicos (6+8), destinos de enlaces y disponibilidad de migraciones 0005/0006. `git diff --check` terminó con código 0; Git solo avisó de su conversión LF/CRLF configurada. `git diff --name-only` confirmó únicamente este plan y todo.md. No se ejecutaron cargo/pnpm en T01 porque no hubo cambios de producto.

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

**Casos de uso públicos (las siete mutaciones validan su acción contra `AccountAccess::allows_offline`; la lectura valida acceso local activo, como fija T01):**
- `list_catalog` — lectura offline fusionada: vista aceptada + intenciones locales no resueltas, con marca de provisional y estado de sincronización.
- `create_service` — `expectedVersion=0`, payload `professionalId`, `name`, `defaultDurationMinutes` (entero 1–1440).
- `update_service` — patch no vacío (`name` y/o `defaultDurationMinutes`), `expectedVersion` = versión conocida.
- `set_service_status` — `active`/`inactive`.
- `offer_service` — clave compuesta `{professionalId, serviceId}`, `expectedVersion=0`, `durationMinutesOverride` opcional (nulo → duración base).
- `update_service_offer` — exige `durationMinutesOverride` **presente** en el patch (admite `null` para volver a la base).
- `set_service_offer_status` — `active`/`inactive`.
- `update_professional_profile` — `publicName` (propietario).

Cada mutación offline: una sola transacción `IMMEDIATE` que escribe la intención local y llama a `sync::application::record_operation` con el mismo `tx`. La vista aceptada **solo** avanza por el proyector al descargar cambios/captura. La lectura/resolución consulta el recibo para marcar la intención accepted; retira el provisional solo al confirmarse la versión en la vista, sin borrar la operación del Outbox. El proyector no escribe intenciones ni Outbox.

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
- **T02 — Componentes y migraciones:** esqueletos `account`/`services` (domain/application/adapters), `mod.rs`, guías breves en docs/technical/components (ubicación solicitada por el usuario; PMD §27.2), migraciones `0005_account.sql` y `0006_services.sql`, runner verificado.
- **T03–T06 — Vertical `account`:** dominio y aplicación (vista, preferencias, validaciones de patch y rangos); repositorios y proyector `account`; casos online-only con envío directo y disparo de `sync_now`; IPC + `account.ts` + UI settings. **C02: revisión vertical account.**
- **T07–T10 — Vertical `services`:** dominio y aplicación (8 casos, validación 1–1440, patch no vacío, `allows_offline`); repositorios e intenciones locales con `record_operation` en la misma transacción; proyectores con regla de versiones dobles; IPC + `services.ts` + UI catálogo. **C03: revisión vertical services.**
- **T11 — Contratos de consumidor:** fixtures contra el bundle fijo (huellas del manifiesto, presencia de las 10 acciones, correspondencia acción↔payload según `capabilities.json`, modelos de estado ya existentes). Sin editar artefactos.
- **T12 — Integración y sincronización de punta a punta:** proyector compuesto registrado en `identity/desktop.rs`; escenarios sintéticos con servidor loopback: crear servicio offline → dispatch → `accepted` → proyección y resolución de la intención; `VERSION_CONFLICT` conserva intención; `remove` retira vista; reapertura conserva todo; oferta con `serviceVersion` mayor y `version` igual se proyecta.
- **T13 — Calidad y guías:** `pnpm quality`, `pnpm build:desktop`, `git diff --check`, guías finales por componente en docs/technical/components. **C04: aceptación del track.**

## Verificación

```powershell
cargo test --workspace --locked components::account
cargo test --workspace --locked components::services
cargo test --workspace --locked components::sync::contract_tests
pnpm quality
pnpm build:desktop
git diff --check
```

Las pruebas usan datos sintéticos, relojes e IDs controlados y HTTP loopback; no acreditan API/OIDC de desarrollo ni revisión de usuario (C02–C04 siguen ese estándar).

## Decisiones abiertas registradas (no bloquean el track)

1. La repetición manual de un caso online-only con resultado desconocido genera una operación nueva (semántica online-only: no hay intención durable que recuperar). Aceptado por coherencia con el ADR; se documenta en docs/technical/components/account.md.
2. `service_local`/`service_offer_local` conservan el payload completo de la intención para permitir edición/reintento y revisión de conflictos; la vista aceptada sigue en las tablas `_view`. Si un track posterior añade un bus de eventos, la resolución de intenciones se puede simplificar sin cambiar contratos públicos.
3. El alta administrativa de cuenta (crear la consulta) queda fuera: es un script administrativo del servidor (contrato §6).
4. **Propiedad futura de los modelos de transporte (decisión del usuario, 2026-11-02):** a medio plazo los modelos de `sync/adapters/remote_models.rs` deberían ser reemplazados por los modelos que definan los componentes propietarios (p. ej. `account`/`services`). Esa refactorización se aplaza explícitamente hasta el final, cuando los componentes estén estables; ahora solo se respeta la frontera: transporte en el adaptador, dominio propio en el componente.

## Integración UI autorizada durante T06

El usuario autorizó incluir src/App.tsx para montar el panel de features/settings. El código vigente no tiene rutas ni un contenedor de features. El resto del panel permanece en features/settings y consume solo src/shared/api/account.ts.

## Decisión de revisión durante T06: cuenta del formulario

Se propuso expectedAccountId como guardia local en los tres comandos online para rechazar un formulario conservado tras cambiar de cuenta. El usuario decidió mantener los DTO C01 sin ese campo (2026-11-02). Se conserva el contrato: la sesión Rust seleccionada al ejecutar determina la cuenta. La puerta de identity mantiene ese ámbito durante refresh/envío, pero no vincula de forma atómica el formulario React previo con la selección. Dos cuentas pueden tener la misma expectedVersion. Este límite queda explícito para C02; no se afirma que exista una guardia de cuenta esperada.

## Estado al presentar C02

T01–T06 verificadas. pnpm quality, pnpm build:desktop y git diff --check correctos. El detalle de comandos, decisiones, fuentes y límites está en todo.md §C02. El usuario pidió detenerse en el siguiente checkpoint; no se inicia T07. La revisión C02 y API/OIDC reales no se acreditan. La composición del proyector sigue reservada a T12.

## Ajustes solicitados durante revisión C02 (2026-11-03)

El usuario pidió un lugar común para countries.txt/time_zones.txt y documentación técnica fuera de los componentes. Las listas pasan sin cambios a src-tauri/src/shared_kernel/reference_data; el módulo ofrece is_country_code e is_time_zone. Justificación del cambio en shared_kernel: son datos de referencia compartidos y funciones puras de pertenencia, sin entidades de negocio, estado, almacenamiento o framework; account conserva sus reglas de patch y llama esas funciones. No se añade una dependencia ni se cambian DTO.

Las guías de account, services, identity, devices, sync, platform y shared_kernel pasan a docs/technical/components. El origen/hash de las listas pasa a docs/technical/reference-data.md. Esta petición sustituye la ubicación README dentro de cada componente prevista en T02/T13 para este trabajo; no cambia el contenido de la especificación general. Se actualizan enlaces e índice. Los dos archivos de account con formato cambiado por el usuario (application.rs y adapters/outbound.rs) se conservan sin modificar.

C02 sigue pendiente de aprobación expresa del usuario tras revisar este ajuste. No se inicia T07 ni se acredita C03.
