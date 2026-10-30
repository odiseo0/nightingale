# Task List: P-20261029-03-core-engine

**Plan:** [plan.md](plan.md)  
**Updated:** 2026-10-29

El usuario aprobó este plan el 2026-10-29 tras la revisión en modo plan. Una tarea a la vez; las fases son secuenciales y cada checkpoint requiere revisión del usuario. Antes de cada cambio de código, aplicar `minimal-code` tras trazar el flujo real; para APIs de bibliotecas, aplicar `docs-first-framework-code` con documentación oficial acorde a la versión seleccionada.

Migraciones reservadas: `0003_identity_access.sql` (identity/devices) y `0004_sync_control.sql` (sync). Los tracks posteriores empiezan en `0005`. Integración propietaria: `src-tauri/src/lib.rs`, `src-tauri/build.rs`, `src-tauri/src/components/mod.rs`, `src-tauri/src/platform/sqlite/mod.rs`, `src-tauri/Cargo.toml`, `Cargo.lock` y `src-tauri/capabilities/default.json`.

## Fase 1: Contratos y persistencia del motor

## T01: Cerrar el ADR del motor y seleccionar dependencias

**Status:** Done

**Evidence (2026-10-29):** `docs/decisions/desktop-core-engine.md` fija propietarios, tablas, estados de identidad/Outbox, permisos, proyección, recuperación, configuración y concurrencia. `CONTRIBUTING.md` y `docs/operations/development.md` recogen puntos calientes y reglas de integración. Dependencias seleccionadas con documentación oficial versionada: `reqwest 0.12.28`, `openidconnect 4.0.1`, `keyring 4.2.0` y `sha2 0.10.9`; se conserva `tauri-plugin-opener 2.7.2`. `Cargo.lock` actualizado. `cargo check --workspace --locked` y `git diff --check` pasan. El usuario pidió detener la ejecución al terminar esta tarea; T02 no ha comenzado y la revisión pertenece a C01.

**Description:** Documentar límites, máquina de estados, propiedad de tablas, puertos de proyección y estrategia de configuración. Comparar HTTP/SSE, OIDC/PKCE y almacén seguro con documentación oficial y fijar versiones mínimas.

**Acceptance criteria:**
- [x] El ADR cubre respuesta perdida, revocación, captura y concurrencia.
- [x] La guía de colaboración identifica propietarios y puntos calientes.
- [x] Toda API no trivial seleccionada cita documentación acorde a su versión.

**Verification:**
- [x] Revisión de enlaces y coherencia con contrato v1.
- [x] `git diff --check`.
- [x] Revisión del usuario en C01.

**Dependencies:** None  
**Estimated scope:** M

## T02: Completar primitivas y envoltorio de operación

**Status:** Done

**Evidence (2026-10-29):** IDs tipados y contexto en `shared_kernel/id.rs`; `OperationEnvelope` camelCase sin contexto local, validación UUIDv7/dependencias/tamaño y SHA-256 en `sync/domain.rs`. Round-trip contra campos de OpenAPI, huella fija contrastada con SHA-256 de .NET y pruebas de cambios semánticos, claves anidadas, arrays y null. `cargo test --workspace --locked`: 31 pruebas; Clippy con `-D warnings`: correcto. Fuentes: https://docs.rs/uuid/1.28.0/uuid/struct.Uuid.html#method.try_parse, https://serde.rs/container-attrs.html, https://docs.rs/serde_json/1.0.154/serde_json/, https://docs.rs/sha2/0.10.9/sha2/#one-shot-api.

**Description:** Introducir solo los IDs/contexto/versiones necesarios y extender la operación con `dependsOn`, serialización contractual y huella semántica estable.

**Acceptance criteria:**
- [x] UUIDv7 canónicos y errores estables.
- [x] Round-trip camelCase conforme a ejemplos del contrato.
- [x] La misma intención produce la misma huella; cualquier cambio semántico la altera.

**Verification:**
- [x] Pruebas unitarias de dominio y serialización.
- [x] `cargo test --workspace --locked`.
- [x] `cargo clippy --workspace --all-targets --locked -- -D warnings`.

**Dependencies:** T01  
**Estimated scope:** M

## T03: Persistir identidad y control de sincronización

**Status:** Done

**Evidence (2026-10-29):** `0003_identity_access.sql` y `0004_sync_control.sql` registradas con sus nombres y números reservados. Repositorios de contexto/acceso, metadatos de instalación, cursor, Inbox y proyección remota; puertos de sincronización en `sync/application.rs`. El Outbox guarda contexto, dependencias y huella; repetir un ID con otra intención o contexto produce `IDEMPOTENCY_CONFLICT`. Las filas heredadas conservan alcance/huella nulos y no pueden formar un envoltorio para envío hasta una asignación explícita. No se añadieron dependencias.

Pruebas de migración en `platform/sqlite/mod.rs`: una base 0002 con `pending`, `sent`, `failed` y `conflict` conserva todos los campos de intención, intentos, error y fecha. Solo transforma `sent → unknown` y `failed → retryable`, según el ADR. Una reapertura normal no escribe ni repite migraciones; una reapertura con `in_flight` recupera `unknown` sin alterar la intención. Un fallo inyectado después del SQL de 0004 revierte también 0003: archivo idéntico byte por byte, historial de dos migraciones y cuatro operaciones intactos; después permite migrar. Inspección del esquema creado: ocho tablas nuevas, ninguna columna de token/secreto/credencial y `PRAGMA integrity_check = ok`.

`src-tauri/tests/core_persistence.rs`: cinco pruebas sobre archivos temporales verifican acceso/contexto/instalación tras reapertura y revocación, Outbox con contexto/dependencias/huella, rechazo de ID reutilizado, Inbox repetida, aislamiento por cuenta/usuario/dispositivo/época, proyección sin retroceso de versión, cursor con página vacía y retirada de vista sin borrar intención. El fallo de una página revierte Inbox, proyección y cursor y deja el archivo y el historial intactos.

Comprobaciones finales: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo test --workspace --locked` (34 unitarias + 5 de integración), `pnpm quality` y `git diff --check`: correctas. Se usó pnpm 11.17.0 ya instalado en `C:\Users\Pedro\AppData\Local\pnpm\bin`, ante el fallo de descarga del shim Corepack; solo se ajustó PATH para el proceso. Cargo avisó que copió la caché incremental al no poder crear enlaces duros; todas las comprobaciones terminaron con código 0.

Fuentes oficiales: [transacciones de rusqlite 0.40.2](https://docs.rs/rusqlite/0.40.2/rusqlite/struct.Transaction.html), [lecturas opcionales](https://docs.rs/rusqlite/0.40.2/rusqlite/trait.OptionalExtension.html), [reconstrucción de tablas SQLite](https://www.sqlite.org/lang_altertable.html#making_other_kinds_of_table_schema_changes) y [UPSERT](https://www.sqlite.org/lang_upsert.html). Para el orden canónico se contrastó además `serde_json 1.0.154/src/map.rs` instalado: usa `BTreeMap` sin `preserve_order`; `cargo tree -e features -i serde_json --locked` confirma que esa feature no está activa en serde_json.

**Description:** Aplicar `0003` y `0004`, evolucionar el Outbox sin perder operaciones y añadir repositorios para acceso, cursor, Inbox y proyección remota.

**Acceptance criteria:**
- [x] Migración desde `0002` conserva Outbox.
- [x] Reapertura no repite cambios; rollback deja archivo e historial intactos.
- [x] No existen columnas de tokens.

**Verification:**
- [x] Pruebas temporales de migración y repositorios.
- [x] `cargo test --workspace --locked`.
- [x] Inspección del esquema resultante.

**Dependencies:** T02  
**Estimated scope:** M

## C01: Contratos congelados

**Estado:** Aprobado por el usuario el 2026-10-29. El usuario indicó que añadió pequeños cambios de formato; se conservan. T04 no ha comenzado.

- [x] `pnpm quality` pasa y las bases `0002` migran sin pérdida.
- [x] El usuario revisa estados, tablas, puertos y dependencias antes de abrir la fase de acceso.
- [x] Quedan reservados `0003`/`0004` y los puntos de integración.

## Fase 2: Identidad, secretos y dispositivo

## T04: Validar configuración y proteger credenciales

**Status:** Done

**Evidence (2026-10-29):** `platform/config.rs` lee solo la URL explícita de API cuando se requiere trabajo remoto; sin carga de `.env`, destinos por defecto ni parámetros OIDC duplicados. Valida HTTPS y HTTP loopback solo en desarrollo, rechaza credenciales/query/fragmento y deriva versión/plataforma del build/SO. `platform/secrets.rs` aporta puerto seguro, adaptador nativo keyring 4.2.0 v1 y fake. Las claves separan API/perfil/issuer-subject/cuenta/dispositivo/tipo; `Secret` no implementa Debug ni Serialize y los errores no copian datos del SO. Pruebas de validación, aislamiento, rotación, borrado idempotente y redacción correctas. Prueba real de Windows con clave ficticia única: lectura, reemplazo y borrado correctos fuera del aislamiento; el intento aislado no podía acceder al almacén. `pnpm quality`: build, fmt, Clippy, 44 pruebas Rust y un doctest compile-fail correctos; la prueba nativa permanece ignorada por defecto y se ejecutó por separado.

**Excepción aprobada por el usuario (2026-10-29):** «Continuar; dejar macOS/Linux pendientes». La prueba real de Keychain y Secret Service queda como límite explícito de C02; no se afirma verificación de esos sistemas. Guía y comandos en `docs/operations/development.md`. Fuentes: [reqwest Url 0.12.28](https://docs.rs/reqwest/0.12.28/reqwest/struct.Url.html), [keyring v1 4.2.0](https://docs.rs/keyring/4.2.0/keyring/v1/index.html), [Entry](https://docs.rs/keyring/4.2.0/keyring/v1/struct.Entry.html) y [errores](https://docs.rs/keyring/4.2.0/keyring/v1/enum.Error.html).

**Description:** Cargar URL de API/OIDC y parámetros por entorno con valores seguros por defecto; implementar almacén seguro del SO y fake de pruebas.

**Acceptance criteria:**
- [x] Configuración inválida falla antes de sincronizar.
- [x] Tokens solo se escriben mediante el puerto seguro.
- [x] Borrado y rotación no dejan copias en SQLite ni logs.

**Verification:**
- [x] Pruebas de precedencia y almacén fake.
- [x] Prueba real del almacén del SO con credencial ficticia (Windows; macOS/Linux pendientes por excepción aprobada).
- [x] `pnpm quality`.

**Dependencies:** T01, T03  
**Estimated scope:** M

## T05: Crear transporte HTTPS tipado para API v1

**Status:** Done

**Evidence (2026-10-29):** `platform/http.rs` usa reqwest 0.12.28 con TLS, 10 s de conexión, 30 s totales, sin redirecciones ni reintentos implícitos. Peticiones de hasta 1 MiB y respuestas de hasta 8 MiB, contando chunks aunque no exista Content-Length. Authorization sensible y errores sin URL, mensajes remotos, detalles ni errores crudos de red. Correlación UUIDv7 y códigos tipados; código desconocido produce Unknown. Adaptadores tipados `identity/api.rs` y `sync/adapters/api.rs` para configuración, emisión, identidad, cuentas/paginación, acceso, lote y consulta de operación; no poseen conexiones SQLite. Validan límites del lote y operaciones, IDs de respuesta y originalResult de duplicados. HTTP 200 conserva estados individuales.

Siete pruebas con servidor loopback cubren 200/201/401/403/404/410/413/422/429/503, Bearer y cuerpo contractual, paginación, correlación, errores por código, respuesta perdida/incompleta, duplicado con rechazo original, JSON inválido, HTML, redirección, exceso declarado/chunked, timeout y conexión fallida. Pasan fuera del aislamiento, que bloquea conexiones loopback. `pnpm quality` fuera del aislamiento: build, fmt, Clippy, 51 pruebas Rust y un doctest correctos; prueba nativa ignorada por defecto (ejecutada aparte en T04).

Fuentes: [ClientBuilder 0.12.28](https://docs.rs/reqwest/0.12.28/reqwest/struct.ClientBuilder.html), [Policy::none](https://docs.rs/reqwest/0.12.28/reqwest/redirect/struct.Policy.html#method.none), [retry](https://docs.rs/reqwest/0.12.28/reqwest/retry/index.html) y [Response::chunk](https://docs.rs/reqwest/0.12.28/reqwest/struct.Response.html#method.chunk). Artefactos de API sin cambios.

**Description:** Cliente Rust con Bearer, timeouts, límites, correlación, errores estructurados y DTO solo para rutas consumidas.

**Acceptance criteria:**
- [x] Códigos HTTP y de operación se traducen sin depender de `message`.
- [x] URL y credenciales no aparecen en errores.
- [x] Ninguna llamada retiene el mutex SQLite.

**Verification:**
- [x] Pruebas contra servidor local para 200/401/403/410/413/422/429/503.
- [x] `pnpm quality`.

**Dependencies:** T02, T04  
**Estimated scope:** M

## T06: Implementar acceso OIDC con PKCE

**Status:** Done

**Evidence (2026-10-29):** `identity/oidc.rs` implementa discovery mediante el transporte limitado de T05, cliente público Authorization Code + PKCE S256, state/nonce nuevos, navegador del SO con Opener 2.7.2 y recepción `127.0.0.1` con puerto efímero. La espera corre en spawn_blocking, admite cancelación y vence a los cinco minutos. Valida camino/Host, parámetros únicos y emisor de callback cuando existe. openidconnect 4.0.1 valida firma, emisor, audiencia del cliente y vencimiento; se verifican nonce, tipo Bearer y at_hash. En refresh se admite nonce ausente; si aparece debe coincidir, y no se permite cambiar identidad. OidcSession usa solo SecretStore para guardar/cargar credenciales; devuelve StorageUnavailable si falla y trata de restaurar las entradas previas. Logout borra solo claves locales explícitas y no tiene cliente HTTP de revocación.

Cuatro pruebas automáticas en `identity/oidc_tests.rs` pasan con proveedor ficticio loopback, navegador simulado y clave RSA generada solo para pruebas: verifican PKCE contra el code_verifier enviado, login firmado, guardado/reapertura, renovación sin ID token, nonce de renovación opcional, identidad estable, tipo Bearer, cierre local, state inválido, cancelación y conservación de la sesión anterior. Rechazan nonce/audiencia/emisor/vencimiento/firma/hash alterados y nonce inicial ausente. Un fallo de almacén entre escrituras produce error y restaura la sesión previa en el caso probado; no se afirma una transacción entre claves del SO. Se corrigió el modo de sockets aceptados en Windows también en el servidor de prueba compartido. No se añadieron dependencias ni se modificaron artefactos del contrato.

Verificación final local: `pnpm quality` fuera del aislamiento pasa (build, fmt, Clippy, 50 pruebas unitarias + 5 de integración + un doctest); dos pruebas se omiten por defecto: almacén nativo, ejecutada con éxito por separado en Windows en T04, y navegador real, aún pendiente. `git diff --check` correcto. Fuentes: [openidconnect 4.0.1](https://docs.rs/openidconnect/4.0.1/openidconnect/#getting-started-authorization-code-grant-w-pkce), [IdToken::claims](https://docs.rs/openidconnect/4.0.1/openidconnect/struct.IdToken.html#method.claims), [AsyncHttpClient](https://docs.rs/openidconnect/4.0.1/openidconnect/trait.AsyncHttpClient.html), [refresh OIDC](https://openid.net/specs/openid-connect-core-1_0.html#RefreshTokenResponse), [Opener 2.7.2](https://docs.rs/tauri-plugin-opener/2.7.2/tauri_plugin_opener/fn.open_url.html) y fuentes oficiales instaladas de esas versiones y de std 1.99.1.

**Verificación manual diferida por el usuario (2026-10-29):** «Comencemos con T07, yo revisaré manualmente C02 luego». La implementación y las pruebas locales de T06 están completas; el recorrido con navegador externo y cuenta de desarrollo queda pendiente para C02 y no bloquea T07. `manual_oidc_login_in_external_browser` queda lista para ese entorno; no imprime ni guarda tokens. C02 no está revisado. macOS/Linux siguen pendientes por la excepción aprobada en T04.

**Description:** Navegador externo, Authorization Code, PKCE S256, state/nonce y redirección local; canjear, renovar y cerrar sesión sin exponer tokens a React.

**Acceptance criteria:**
- [x] State/nonce inválidos se rechazan.
- [x] Cancelar navegador conserva el estado anterior.
- [x] Refresh y logout no revocan por error el token de dispositivo.

**Verification:**
- [ ] Pruebas con proveedor OIDC ficticio y navegador externo.
- [ ] Flujo manual con cuenta de desarrollo.
- [x] `pnpm quality`.

**Dependencies:** T05  
**Estimated scope:** M

## T07: Activar instalación y autorización de cuenta

**Status:** In progress

**Description:** Canjear access token OIDC por token opaco de API, consultar identidad/cuentas/acceso y ejecutar `RegisterAccountDevice` solo cuando corresponda.

**Acceptance criteria:**
- [ ] Token opaco se persiste una única vez de forma segura.
- [ ] `registration_required` conduce al registro propio.
- [ ] Acceso revocado detiene intercambio sin borrar Outbox.

**Verification:**
- [ ] Pruebas de alta, acceso y revocación.
- [ ] Activación manual sobre API de desarrollo.
- [ ] `pnpm quality`.

**Dependencies:** T06  
**Estimated scope:** M

## C02: Instalación autorizada

**Límites de verificación:** El usuario autorizó continuar con macOS Keychain y Linux Secret Service pendientes. El 2026-10-29 pidió ejecutar T07 y dejó para su revisión manual de C02 los recorridos reales de navegador, acceso y activación. Esa revisión sigue pendiente; no se afirman pruebas reales con API/OIDC de desarrollo.

- [ ] Transporte, acceso OIDC/PKCE y activación funcionan con datos de prueba.
- [ ] No hay secretos en archivos, IPC, SQLite o registros.
- [ ] El usuario revisa alta, logout y revocación.

## Fase 3: Envío y reconciliación de operaciones

## T08: Despachar Outbox con dependencias y reintentos

**Status:** Todo

**Description:** Seleccionar lotes causales válidos, marcar intentos durablemente y reintentar con backoff/jitter sin alterar contenido ni ID.

**Acceptance criteria:**
- [ ] `dependsOn` respeta orden y máximo ocho.
- [ ] Una operación independiente fallida no bloquea otras.
- [ ] Reinicio durante envío recupera el mismo lote sin duplicar intención.

**Verification:**
- [ ] Pruebas de planificación, leases, límites y reloj controlado.
- [ ] Prueba de reinicio.
- [ ] `pnpm quality`.

**Dependencies:** T07  
**Estimated scope:** M

## T09: Aplicar resultados y recuperar respuestas perdidas

**Status:** Todo

**Description:** Reconciliar todos los resultados del contrato y recuperar resultados desconocidos mediante `GET /sync/operations/{id}`.

**Acceptance criteria:**
- [ ] Duplicado no repite efecto.
- [ ] Conflicto/rechazo conserva intención y estado remoto.
- [ ] 404/`RESULT_EXPIRED` nunca genera otro ID automáticamente.

**Verification:**
- [ ] Matriz unitaria y de adaptador, incluida pérdida después del commit.
- [ ] `pnpm quality`.

**Dependencies:** T08  
**Estimated scope:** M

## C03: Outbound fiable

- [ ] Una operación sobrevive timeout, reinicio y duplicado sin doble efecto.
- [ ] Conflictos y rechazos quedan disponibles para revisión.
- [ ] El usuario revisa la demostración y los estados visibles.

## Fase 4: Entrada, cursor y recuperación

## T10: Aplicar grupos remotos de forma idempotente

**Status:** Todo

**Description:** Guardar proyección aceptada e Inbox por operación/grupo, con traductores tipados y proyectores sin mezclar intenciones locales.

**Acceptance criteria:**
- [ ] Repetir grupo no cambia estado; grupo multientidad es atómico.
- [ ] Claves compuestas, versiones de oferta y `remove` se conservan.

**Verification:**
- [ ] Pruebas con fixtures del contrato y fallo del proyector.
- [ ] `pnpm quality`.

**Dependencies:** T03, T05  
**Estimated scope:** M

## T11: Descargar cambios incrementales

**Status:** Todo

**Description:** Consumir `/sync/changes` hasta agotar `hasMore`, confirmando página y `nextCursor` juntos.

**Acceptance criteria:**
- [ ] Grupos vacíos avanzan; fallo conserva cursor anterior.
- [ ] Salida de ventana retira vista remota sin borrar intenciones.
- [ ] Permiso/época inválido exige recuperación.

**Verification:**
- [ ] Pruebas multipágina, duplicadas, vacías, remove, cursor expirado y rollback.
- [ ] `pnpm quality`.

**Dependencies:** T10  
**Estimated scope:** M

## T12: Reconstruir desde capturas coherentes

**Status:** Todo

**Description:** Crear/descargar capturas por páginas, preparar reemplazo en staging e instalar vista y `baseCursor` atómicamente; liberar captura al finalizar.

**Acceptance criteria:**
- [ ] No se mezclan capturas; vencimiento/cambio de alcance reinicia explícitamente.
- [ ] Outbox y conflictos sobreviven al reemplazo; reintento es idempotente.

**Verification:**
- [ ] Pruebas multipágina, interrupción antes/después de instalar y expiración.
- [ ] `pnpm quality`.

**Dependencies:** T11  
**Estimated scope:** M

## C04: Entrada y recuperación fiables

- [ ] Cursor y captura son atómicos, repetibles y conservan trabajo local.
- [ ] Un perfil vacío reconstruye la proyección sintética y continúa incrementalmente.
- [ ] El usuario revisa recuperación, retiradas de vista y conflictos.

## Fase 5: Coordinador e IPC

## T13: Orquestar el ciclo de sincronización

**Status:** Todo

**Description:** Coordinador single-flight para arranque, reconexión y ejecución manual: acceso, desconocidos, envío, descarga y estado.

**Acceptance criteria:**
- [ ] Ejecuciones simultáneas comparten un solo ciclo.
- [ ] Caída de API no bloquea trabajo local.
- [ ] Backoff no impide reintento manual; estado sobrevive reinicio.

**Verification:**
- [ ] Pruebas de concurrencia/estado con reloj y puertos falsos.
- [ ] Ciclo real con API de desarrollo.
- [ ] `pnpm quality`.

**Dependencies:** T09, T12  
**Estimated scope:** M

## T14: Incorporar avisos SSE sin convertirlos en autoridad

**Status:** Todo

**Description:** Escuchar `/sync/events`, despertar descarga con el último cursor aplicado, tratar heartbeat/revoke y reconectar con backoff.

**Acceptance criteria:**
- [ ] `changes_available` no avanza cursor; ID SSE no sustituye cursor.
- [ ] `access_revoked` detiene acceso; conexiones duplicadas se evitan.

**Verification:**
- [ ] Stream ficticio con heartbeat, corte, repetidos, 429 y revoke.
- [ ] Prueba con API.
- [ ] `pnpm quality`.

**Dependencies:** T13  
**Estimated scope:** M

## T15: Exponer acceso y estado mediante IPC específico

**Status:** Todo

**Description:** Comandos de acceso, logout, sincronización y estado/conflictos; cliente TypeScript y diagnóstico mínimo sin pantallas de negocio.

**Acceptance criteria:**
- [ ] DTO camelCase, errores estables y permisos explícitos.
- [ ] UI diferencia offline/pendiente/sincronizando/conflicto/fallo.
- [ ] Nunca muestra token, payload sensible o SQL.

**Verification:**
- [ ] Pruebas Rust de DTO/permisos y `pnpm build`.
- [ ] Recorrido manual acceso → sincronización → offline → reconexión.

**Dependencies:** T14  
**Estimated scope:** M

## C05: Motor utilizable desde el escritorio

- [ ] El recorrido manual funciona de extremo a extremo con datos sintéticos.
- [ ] La UI no afirma aceptación remota prematura.
- [ ] El usuario revisa el motor antes del endurecimiento final.

## Fase 6: Pruebas compartidas, operación y entrega al equipo

## T16: Fijar pruebas de contrato del consumidor

**Status:** Todo

**Description:** Servidor/fixtures locales para rutas usadas que validen contrato, errores, captura, cambios y SSE contra artefactos fijos.

**Acceptance criteria:**
- [ ] Una deriva de contrato falla de forma legible.
- [ ] No se editan JSON importados; pruebas sin red externa ni credenciales.

**Verification:**
- [ ] `cargo test --workspace --locked`.
- [ ] Comprobación de manifiesto/hashes del contrato.
- [ ] `pnpm quality`.

**Dependencies:** T15  
**Estimated scope:** M

## T17: Ejecutar la matriz de fallos con dos perfiles

**Status:** Todo

**Description:** Arnés con dos directorios aislados para alta, envío, descarga y recuperación, con operación sintética válida como sonda.

**Acceptance criteria:**
- [ ] Cubre offline, duplicado, respuesta perdida, dos dispositivos, conflicto, revocación, cursor/captura vencidos, SSE caído y reinicio.
- [ ] Ninguna pérdida silenciosa.

**Verification:**
- [ ] Pruebas automatizadas locales y repetición contra API de desarrollo.
- [ ] Resultados y límites registrados por separado.

**Dependencies:** T16  
**Estimated scope:** M

## T18: Completar observabilidad y diagnóstico seguro

**Status:** Todo

**Description:** Logs estructurados con correlación/causación, duración y resultados; estado de Outbox/cursor sin datos personales.

**Acceptance criteria:**
- [ ] Operación rastreable por ID/correlación.
- [ ] Pruebas de redacción rechazan tokens/payloads.
- [ ] Fallo de telemetría no afecta el motor.

**Verification:**
- [ ] Pruebas de formato/redacción.
- [ ] Inspección manual de logs con errores reales de desarrollo.
- [ ] `pnpm quality`.

**Dependencies:** T17  
**Estimated scope:** M

## T19: Formalizar la guía de colaboración y los siguientes tracks

**Status:** Todo

**Description:** README por componente core, guía para operación/proyector/comando/migración y contratos iniciales de los cuatro tracks siguientes.

**Acceptance criteria:**
- [ ] Un colaborador puede implementar un caso sin editar kernel ni tablas ajenas.
- [ ] Dependencias y propietarios claros; guía y código coinciden.

**Verification:**
- [ ] Recorrido documental con ejemplo sintético y enlaces.
- [ ] `git diff --check`.
- [ ] Revisión del usuario.

**Dependencies:** T18  
**Estimated scope:** M

## C06: Base core aceptada para desarrollo colaborativo

- [ ] `pnpm quality`, `pnpm build:desktop` y `git diff --check` pasan sobre la entrega final.
- [ ] La matriz local y la verificación con API/OIDC de desarrollo tienen evidencia; cualquier entorno no disponible queda pendiente.
- [ ] Dos perfiles sincronizan datos sintéticos sin duplicados ni pérdida de intenciones.
- [ ] El usuario revisa la entrega y la división de los siguientes tracks.
