# ADR: Motor transversal del escritorio

**Estado:** Propuesta cerrada en T01; pendiente de revisión del usuario en C01.  
**Fecha:** 2026-10-29  
**Plan:** [P-20261029-03-core-engine](../../tasks/P-20261029-03-core-engine/plan.md)  
**Contrato remoto:** Nightingale API v1, revisión fija `api-v1.1`.

## Contexto

La base ya ejecuta casos locales en Rust, guarda SQLite y registra un Outbox sin red. No existe todavía un recorrido para autenticar la instalación, enviar operaciones, recuperar resultados o reconstruir la vista remota. Los tracks de Pacientes, Servicios, Agenda y Citas no deben resolver esas necesidades por separado.

Esta decisión fija únicamente el motor transversal. No añade entidades de negocio, una API local genérica para React, acceso PostgreSQL ni servicios cloud al repositorio.

## Decisión

### 1. Propietarios y límites

| Componente | Posee | No posee |
|---|---|---|
| `shared_kernel` | IDs UUIDv7, contexto tipado, reloj y errores estables | Entidades, repositorios, HTTP o reglas de permisos |
| `identity` | OIDC/PKCE, identidad activa y última autorización válida por cuenta | Token opaco de API o reglas de Pacientes/Citas |
| `devices` | Instalación, activación y ciclo local de la credencial opaca | Autorización de objetos o aceptación remota |
| `platform` | Configuración, SQLite, almacén seguro y HTTPS | Estado de negocio o sincronización |
| `sync` | Outbox, Inbox, cursor, captura, proyección remota y coordinación | Tablas propias de futuros componentes |
| IPC/diagnóstico | Casos de uso concretos y estado visible | Tokens, SQL o acceso remoto genérico |

React invoca comandos específicos. Los casos locales terminan en SQLite sin esperar HTTP. La aceptación compartida pertenece siempre a Nightingale API.

### 2. Datos y migraciones reservadas

| Tabla | Propietario | Contenido |
|---|---|---|
| `identity_context` | identity | Identidad activa, usuario y cuenta seleccionada; sin tokens |
| `device_installation` | devices | ID y metadatos no secretos de la instalación |
| `identity_account_access` | identity | Última respuesta válida de acceso: rol, estado, época y acciones |
| `sync_outbox` | sync | Intención, alcance local, dependencias, huella, lease y resultado |
| `sync_inbox` | sync | Operaciones/grupos remotos ya aplicados |
| `sync_cursor` | sync | Cursor confirmado y época asociada por cuenta |
| `sync_remote_items` | sync | Vista remota aceptada como datos de transporte versionados |
| `sync_snapshot_recovery` | sync | Captura en preparación y `baseCursor` |
| `sync_snapshot_items` | sync | Elementos staging hasta instalar la captura |

`0003_identity_access.sql` crea las tres primeras tablas. `0004_sync_control.sql` evoluciona el Outbox y crea las tablas de sincronización. Los tracks posteriores reservan desde `0005`.

Las operaciones nuevas incluyen localmente cuenta, usuario, dispositivo y época de permiso para seleccionar ruta y credencial; esos datos no se añaden al cuerpo del contrato. Los registros heredados sin alcance o huella se conservan, pero no se envían hasta una asignación y verificación explícitas. Nunca se reetiquetan automáticamente al abrir otra cuenta.

### 3. Identidad y dispositivo

El acceso sigue este recorrido:

```text
signed_out → authenticating → device_activation_required
→ registration_required → active
                    ↘ revoked / inactive
```

- `GET /v1/auth/config` proporciona emisor, cliente público y audiencia. Se usa Authorization Code, PKCE S256, navegador externo y loopback `127.0.0.1` con puerto efímero.
- `state`, `nonce`, firma y claims del ID token se validan antes de aceptar la sesión. El ID token nunca sustituye el access token destinado a API.
- El access token OIDC se canjea por el token opaco de dispositivo. Las credenciales OIDC y de API viven solo en el almacén seguro del SO y tienen claves distintas.
- `registration_required` permite únicamente el propio `RegisterAccountDevice`. `revoked` o `inactive` detiene intercambio y descarga sin borrar Outbox.
- Logout elimina credenciales locales sin revocar remotamente el dispositivo. La revocación es una operación distinta.
- Offline es una condición de conectividad, no otro estado de identidad. La última autorización válida permite trabajo local acotado; no promete revocación instantánea sin conexión.

### 4. Outbox y respuesta perdida

El contenido semántico de una operación es inmutable: ID, versión de contrato, acción, clave, versión esperada, payload, dependencias y `occurredAtLocal`. La huella es SHA-256 hexadecimal de su serialización JSON canónica UTF-8. `serde_json` sin `preserve_order` ordena las claves de objetos; los structs fijan los campos del envoltorio y no se añaden espacios. Reutilizar un ID con otra huella es un error local y no una nueva intención.

```text
pending ──lease──> in_flight ──accepted──> accepted
   ↑                  │  ├────duplicate──> resultado original
   │                  │  ├────conflict───> conflict
   │                  │  ├────rejected───> rejected
   │                  │  └─retryable_failure→ retryable
   │                  └────transporte────→ unknown
   └────────backoff──────────────retryable       │
                                      consulta/reintento mismo ID
```

Al reabrir, `in_flight` pasa a `unknown`: un proceso anterior pudo haber enviado antes de terminar. `sent` heredado migra a `unknown` y `failed` a `retryable`. Un duplicado no es otro efecto ni necesariamente un éxito: se reconcilia con su `originalResult`.

Timeout, desconexión o respuesta incompleta dejan resultado desconocido. La recuperación consulta `GET /sync/operations/{id}` o repite exactamente la misma operación. 404 y `RESULT_EXPIRED` exigen revisión y jamás generan otro ID automáticamente. Conflictos y rechazos conservan la intención para corrección mediante una operación nueva y explícita.

Un lease no prueba que no hubo envío. Los reintentos usan el mismo contenido, backoff y jitter; las dependencias se limitan a ocho y respetan orden causal. Los lotes respetan 100 operaciones, 1 MiB total y 64 KiB por operación. Una operación independiente fallida no bloquea las demás.

### 5. Entrada, capturas y proyección

La proyección remota no es un ORM ni una copia del esquema PostgreSQL. Guarda `entityType`, clave compuesta o UUID, estado de transporte y versiones dentro de `sync`. Cada track implementará un proyector tipado que recibe un cambio y una transacción SQLite para actualizar exclusivamente sus tablas.

Cada grupo remoto y su Inbox se confirman atómicamente. Cada página y `nextCursor` se confirman juntos; grupos vacíos pueden avanzar. Repetir una operación/grupo no vuelve a proyectar su efecto. `remove` retira la vista remota, nunca intenciones locales.

Una captura se descarga en staging y solo reemplaza la vista actual junto a `baseCursor` en una transacción. `CURSOR_EXPIRED`, cambio de época, captura vencida o cambio de alcance inicia recuperación. Interrumpir una captura no mezcla generaciones ni borra Outbox/conflictos.

SSE solo despierta al coordinador. Un ID de evento no sustituye el cursor; `access_revoked` detiene el intercambio. El coordinador es single-flight por instalación/cuenta y jamás mantiene el mutex o una transacción SQLite durante HTTP/SSE.

### 6. Permisos

La autorización local usa la última respuesta `active` con `allowedActions` y `permissionEpoch`. `capabilities.json` aporta acciones online-only y límites; Rust no duplica las listas de acciones por rol.

Antes de enviar se refresca acceso. Si cambia la época se detiene el ciclo y se recupera la vista con el alcance vigente. API vuelve a validar identidad, dispositivo, permiso, versión y reglas al aceptar. La fecha del cliente no prueba que una operación fuera anterior a una revocación. Las acciones administrativas online-only no se registran como trabajo offline.

### 7. Configuración y secretos

`NIGHTINGALE_API_BASE_URL` es la única configuración remota obligatoria y no tiene destino cloud implícito. Emisor, cliente y audiencia se descubren por `/v1/auth/config`; no se duplican en variables. Nombre de dispositivo, versión del cliente y esquema se derivan del SO/build, con override local solo si una prueba lo necesita.

Release exige HTTPS. Los tests y builds de desarrollo admiten HTTP únicamente para loopback, sin una opción general para desactivar TLS. No se leen `.env` automáticamente ni se guardan secretos en configuración, SQLite, IPC o logs.

El almacén seguro usa Windows Credential Manager, macOS Keychain y Secret Service en Linux. Las claves incluyen servicio, entorno, identidad/cuenta y dispositivo para evitar colisiones entre perfiles de prueba. Los valores secretos no implementan `Debug` en tipos propios ni aparecen en errores.

### 8. Dependencias seleccionadas

Las versiones declaradas son mínimas compatibles; `Cargo.lock` fija la resolución exacta. No se actualizan las dependencias existentes como parte de esta selección.

| Uso | Selección | Motivo y límite |
|---|---|---|
| API HTTPS | `reqwest 0.12.28`, `default-features=false`, `json`, `rustls-tls` | Compatible con el cliente OIDC; timeouts, JSON y TLS sin OpenSSL del sistema |
| SSE | `reqwest::Response::chunk` | Un solo cliente HTTP; parser incremental propio sin crate SSE ni `futures-util` |
| OIDC | `openidconnect 4.0.1`, `reqwest`, `rustls-tls` | Discovery, PKCE S256, state/nonce y validación de ID token; su OAuth2 5.0.0 usa reqwest 0.12 |
| Almacén seguro | `keyring 4.2.0`, feature `v1` | Almacén nativo en los tres sistemas; Linux requiere Secret Service |
| Huella | `sha2 0.10.9` | SHA-256 probado; ya existe transitivamente, se declara para uso directo |
| Navegador | `tauri-plugin-opener 2.7.2` existente | Abre el navegador externo; no requiere otro plugin |

Reqwest `0.13.5` puede permanecer dentro de Tauri; no se fuerza una unificación incompatible con `openidconnect 4.0.1`. No se añaden `tokio`, `url`, `rand`, `base64`, `dotenvy`, un crate SSE ni un servidor HTTP: Tauri aporta el runtime async, reqwest expone `Url`, OIDC genera los secretos del protocolo, la configuración usa `std::env` y el callback usa `std::net` en un hilo acotado.

## Colaboración

Cada track consume puertos y comandos públicos, nunca tablas de otro componente. Antes de trabajo paralelo se fijan DTO, acción, propiedad de tablas, proyector, migración y comando IPC. El integrador del plan es el único que registra composición, migraciones y permisos en `lib.rs`, `build.rs`, `components/mod.rs`, `platform/sqlite/mod.rs`, manifiestos/lockfile y `capabilities/default.json`.

Los cambios de `shared_kernel` requieren justificación en el plan. Los artefactos de `docs/contracts/api-v1/` no se editan localmente; una incompatibilidad vuelve al repositorio propietario del contrato.

## Alternativas consideradas

- **OAuth/OIDC manual:** evita un crate, pero traslada discovery, validación JWT, nonce y renovación a código de seguridad propio. Se descarta.
- **Reqwest 0.13 para toda la app:** exigiría un adaptador HTTP personalizado para OIDC. Se elige 0.12 en el código de aplicación.
- **Crate SSE independiente:** duplica configuración HTTP/TLS para una lectura incremental que reqwest ya cubre.
- **Archivo cifrado propio o Tauri Store para tokens:** no equivale al almacén de credenciales del SO y exigiría gestionar otra clave. Se descarta.
- **Puerto OIDC fijo y servidor adicional:** aumenta colisiones y dependencias; se usa loopback efímero con biblioteca estándar.

## Consecuencias

- Los siguientes tracks no necesitan diseñar autenticación, reintentos, cursores o capturas, pero sí sus propios proyectores y reglas.
- Los registros heredados ambiguos quedan preservados y bloqueados para envío hasta revisión, no descartados ni corregidos por suposición.
- Linux necesita un Secret Service disponible en la sesión; la prueba real de los tres almacenes queda en T04, no se acredita por esta selección.
- Esta base es para datos sintéticos. Cifrado SQLite, distribución de claves, instaladores y publicación siguen fuera del plan.

## Fuentes oficiales consultadas

- [openidconnect 4.0.1: Authorization Code + PKCE y API async](https://docs.rs/openidconnect/4.0.1/openidconnect/#getting-started-authorization-code-grant-w-pkce) y [seguridad del cliente HTTP](https://docs.rs/openidconnect/4.0.1/openidconnect/#security-warning).
- [oauth2 5.0.0: manifiesto oficial con reqwest 0.12](https://docs.rs/crate/oauth2/5.0.0/source/Cargo.toml).
- [reqwest 0.12.28: lectura incremental con `Response::chunk`](https://docs.rs/reqwest/0.12.28/reqwest/struct.Response.html#method.chunk).
- [keyring 4.2.0: módulo `v1` y almacenes por sistema](https://docs.rs/keyring/4.2.0/keyring/v1/index.html).
- [Tauri Opener 2.7.2: apertura de URL mediante aplicación predeterminada](https://docs.rs/tauri-plugin-opener/2.7.2/tauri_plugin_opener/).
- [RFC 8252 §6 y §7.3: navegador externo, PKCE y loopback efímero](https://www.rfc-editor.org/rfc/rfc8252.html#section-7.3).
- [sha2 0.10.9: API SHA-256](https://docs.rs/sha2/0.10.9/sha2/).
