# Task List: P-20261029-03-core-engine

**Plan:** [plan.md](plan.md)  
**Updated:** 2026-11-02

**Cierre aprobado (2026-11-02):** Tras las comprobaciones contra la API local y la entrega del [informe de verificación](verification-2026-11-02.md), el usuario respondió «Sí, aprobar y cerrar el plan» a la aceptación de C02–C06 y T19. C01 ya estaba aprobado. T01–T19 y C01–C06 quedan completos, con la excepción de macOS/Linux y los límites temporales de capturas/journal que recoge el informe. Se conserva el plan independiente de Cuenta/Servicios.

**Continuación hasta la entrega (2026-10-30):** El usuario pidió «Finalicemos con el plan, ya aprobaré los checkpoints luego». Autoriza implementar T14–T19 en orden y continuar con las comprobaciones manuales y revisiones de C02–C06 pendientes. No acredita esas comprobaciones ni aprueba checkpoints. T13 queda implementada con verificación local; la prueba API real permanece pendiente sin bloquear esta continuación. El cierre de implementación se distinguirá de la aceptación del plan.

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
- [x] Pruebas con proveedor OIDC ficticio y navegador externo.
- [x] Flujo manual con cuenta de desarrollo.
- [x] `pnpm quality`.

**Dependencies:** T05  
**Estimated scope:** M

## T07: Activar instalación y autorización de cuenta

**Status:** Done

**Evidence (2026-10-30):** `devices/activation.rs` guarda un marcador de emisión antes de pedir la credencial, conserva el localizador en el almacén seguro y escribe el token una sola vez bajo una clave con entorno, perfil, identidad y dispositivo. Reutiliza esa credencial entre activaciones; respuesta perdida o fallo de escritura exige recuperación explícita y no otra emisión automática. La protección de activación impide altas y logout simultáneos del mismo perfil dentro del proceso; T13 conserva la coordinación completa.

`identity/application.rs` verifica `/me`, recorre cuentas paginadas y guarda acceso por cuenta/usuario/dispositivo. Solo `registration_required` crea el registro propio con versión cero, payload vacío y sin dependencias. El Outbox guarda la intención y marca el intento `unknown` antes de HTTP. Reintentar tras reapertura conserva ID, cuerpo y huella; un duplicado aplica su resultado original, incluido rechazo. Después de aceptación vuelve a consultar `/access`. Acceso revocado/inactivo y HTTP 401/403 deshabilitan permisos afectados sin borrar intenciones. El acceso offline usa `allowedActions` y el catálogo contractual; logout retira credenciales locales sin revocación remota. Las peticiones no mantienen el mutex SQLite.

Nueve pruebas automáticas de `identity/activation_tests.rs` pasan: alta/reutilización/reapertura/logout, respuesta perdida y duplicado aceptado, acceso revocado/inactivo, HTTP 401/403, emisión perdida/fallo del almacén, duplicado rechazado sin nuevo ID, identidad/alcance incorrectos, paginación/cuenta ausente y logout durante activación. Comprueban también conservación del Outbox y ausencia de los tokens ficticios en los bytes de SQLite. Tras los cambios del usuario se completó el renombrado de siete referencias a `CredentialKind::APIDevice`/`APIDeviceState`; no cambia servicios, claves persistidas ni decisiones.

Verificación final sobre el código actual: `pnpm quality` termina con código 0; incluye build, `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings` y `cargo test --workspace --locked` (59 unitarias, 5 de integración y un doctest correctos; 3 pruebas ignoradas). Entre las pruebas pasan migración desde 0002 sin pérdida, reaperturas y rollback de migraciones/páginas. Se usó pnpm 11.17.0 instalado y ejecución fuera del aislamiento para HTTP loopback. Las tres ignoradas requieren almacén nativo o navegador/API reales; Windows se probó por separado en T04, sin repetirlo aquí. `git diff --check` correcto.

**Verificación manual diferida:** Se conserva la autorización del usuario de revisar C02 luego. `manual_c02_activation_and_local_logout` y los pasos de `docs/operations/development.md` quedan listos; no se ejecutaron contra API/OIDC de desarrollo. macOS/Linux y la revisión del usuario siguen pendientes. T08 no ha comenzado.

Fuentes de la implementación: [transacciones rusqlite 0.40.2](https://docs.rs/rusqlite/0.40.2/rusqlite/struct.Connection.html#method.transaction_with_behavior), [keyring 4.2.0 v1](https://docs.rs/keyring/4.2.0/keyring/v1/index.html), [etiqueta interna Serde 1.0.229](https://serde.rs/enum-representations.html#internally-tagged) y contrato API fijo `api-v1.1`. La corrección final solo completa nombres locales; no introduce APIs de biblioteca.

**Description:** Canjear access token OIDC por token opaco de API, consultar identidad/cuentas/acceso y ejecutar `RegisterAccountDevice` solo cuando corresponda.

**Acceptance criteria:**
- [x] Token opaco se persiste una única vez de forma segura.
- [x] `registration_required` conduce al registro propio.
- [x] Acceso revocado detiene intercambio sin borrar Outbox.

**Verification:**
- [x] Pruebas de alta, acceso y revocación.
- [x] Activación manual sobre API de desarrollo.
- [x] `pnpm quality`.

**Dependencies:** T06  
**Estimated scope:** M

## C02: Instalación autorizada

**Estado:** Aprobado por el usuario el 2026-11-02. OIDC real, activación y logout con almacén nativo Windows pasan; la matriz real verifica denegación y revocación SSE. Evidencia y límites aceptados en [verification-2026-11-02.md](verification-2026-11-02.md). Se conserva debajo el historial de continuación autorizada.

**Continuación autorizada (2026-10-30):** El usuario indicó «Continuar con T08 y dejar C02 pendiente». Se abre T08 sin marcar esta revisión ni sus comprobaciones manuales como realizadas.

**Límites de verificación:** El usuario autorizó continuar con macOS Keychain y Linux Secret Service pendientes. Los recorridos API/OIDC de Windows se comprobaron el 2026-11-02; el usuario aprobó el checkpoint con esa excepción.

- [x] Transporte, acceso OIDC/PKCE y activación funcionan con datos de prueba.
- [x] No hay secretos en archivos, IPC, SQLite o registros.
- [x] El usuario revisa alta, logout y revocación.

## Fase 3: Envío y reconciliación de operaciones

## T08: Despachar Outbox con dependencias y reintentos

**Status:** Done

**Evidence (2026-10-30):** `sync/dispatch.rs` refresca acceso antes de cada lote, valida el alcance de la credencial y detiene envío ante acceso denegado o cambio de época. `SqliteOperationStore::lease_batch` reserva bajo transacción `IMMEDIATE`, conserva contexto/ID/cuerpo/huella y selecciona dependencias aceptadas del mismo actor/cuenta o anteriores en el lote. Excluye registro propio de T07, filas heredadas, otra época/alcance, acciones sin permiso y contenido inválido. Respeta ocho dependencias, 100 operaciones, 64 KiB por operación y 1 MiB de cuerpo UTF-8. Un bloqueo causal no impide operaciones independientes.

Cada reserva guarda `in_flight`, intento y fechas antes de HTTP. Reserva de 60 s, timeout de transporte de 30 s y espera exponencial con variación, de 0,5–1 s inicialmente hasta 150–300 s. Fallo de petición conserva resultado desconocido; reapertura o vencimiento conserva intención y calendario. Un intento tardío no puede liberar otro más reciente. Se reutiliza el generador existente para variar esperas, sin otra dependencia. No hay migraciones nuevas ni cambios de las reservadas. Las respuestas recibidas se entregan con la reserva a T09: todavía no se reconcilian ni se aplica proyección remota.

Diez pruebas de `sync/dispatch_tests.rs` pasan: orden causal con antecesor posterior, referencias ausentes/rechazadas/ajenas/ciclos, límite de ocho dependencias, rollback de todos los intentos al fallar una reserva, dos conexiones y reserva vencida, respuesta tardía, reinicio con mismos envoltorios/huellas, espera controlada y tope sin desbordamiento, límites de cantidad/UTF-8, aislamiento/época/fecha/registro propio, respuesta perdida y reintento HTTP con cuerpo idéntico, ausencia de mutex SQLite durante HTTP, 401/403 antes y después del POST y fallos 422/503 sin pérdida de IDs. El servidor ficticio existente se trasladó a `platform/test_api.rs` para reutilizarlo; las nueve pruebas automáticas de T07 también pasan.

`pnpm quality` termina con código 0 sobre el código final: build, `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo test --workspace --locked` (69 unitarias + 5 de integración + un doctest; 3 ignoradas). Pasa también `git diff --check`. HTTP loopback se probó fuera del aislamiento. Contrato, Master Plan, lockfile y migraciones sin cambios. C02 sigue pendiente; T09 no ha comenzado. Límites de selección, recuperación y entrega a T09 documentados en `docs/operations/development.md`.

Fuentes: [rusqlite 0.40.2, transacción con comportamiento explícito](https://docs.rs/rusqlite/0.40.2/rusqlite/struct.Connection.html#method.transaction_with_behavior), [uuid 1.28.0, generación v7](https://docs.rs/uuid/1.28.0/uuid/struct.Uuid.html#method.new_v7), fuente oficial instalada de `serde_json 1.0.154/src/ser.rs::to_vec` y contrato fijo `api-v1.1`, apartados 8 y 10.

**Description:** Seleccionar lotes causales válidos, marcar intentos durablemente y reintentar con backoff/jitter sin alterar contenido ni ID.

**Acceptance criteria:**
- [x] `dependsOn` respeta orden y máximo ocho.
- [x] Una operación independiente fallida no bloquea otras.
- [x] Reinicio durante envío recupera el mismo lote sin duplicar intención.

**Verification:**
- [x] Pruebas de planificación, leases, límites y reloj controlado.
- [x] Prueba de reinicio.
- [x] `pnpm quality`.

**Dependencies:** T07  
**Estimated scope:** M

## T09: Aplicar resultados y recuperar respuestas perdidas

**Status:** Done

**Evidence (2026-10-30):** `OperationResult::original` normaliza los cuatro resultados originales y sus duplicados; exige identidad del recibo original. Los campos contractuales obligatorios que admiten null ya no aceptan ausencia. `SqliteOperationStore::reconcile` confirma todos los recibos del lote en una transacción tras comprobar IDs, alcance, envoltorios, huellas, intentos y reservas. Un recibo repetido no vuelve a escribir; una respuesta tardía no altera otro intento ni un resultado terminal. `dispatch` integra esa conciliación antes de devolver éxito. `conflict`/`rejected` conservan intención y estado remoto en `result`; `retryable_failure` guarda `retryable` con espera. No se aplica proyección ni otra escritura de negocio.

`OutboxDispatcher::recover` refresca acceso y reserva una consulta GET del ID original. Reutiliza reservas, contador y espera de T08 sin mantener SQLite durante HTTP; el contador incluye consultas de recuperación. 404 o `RESULT_EXPIRED` deja `unknown` con marca persistente para revisión: ni envío ni consulta automática vuelve a seleccionarlo, y nunca se crea otro ID. `operation_result` permite leer intención, recibo, estado y código de revisión localmente. Un fallo temporal deja incertidumbre y espera; 401/403 conserva las reglas de denegación de T08.

Diez pruebas nuevas en `sync/reconciliation_tests.rs`, más las diez de T08, pasan. Matriz de originales y duplicados aceptados/conflictivos/rechazados/temporales; comparación tras reapertura de intención, huella y estado remoto; replay sin escrituras y archivo intacto; lote incompleto/duplicado/ajeno; rollback al fallar el segundo recibo; respuesta tardía y resultado terminal; recibo temporal repetido en otro intento; duplicado sin original válido; campos obligatorios ausentes; recuperación GET de todos los estados y fallo 503. Dos demostraciones HTTP simulan confirmación remota seguida de pérdida de respuesta y reinicio: GET recupera aceptación sin otro POST; un timeout HTTP real con plazo breve conduce a repetir el POST con cuerpo/IDs idénticos y resolver duplicados sin otro efecto. Se reutiliza el constructor del transporte con visibilidad interna para esa prueba. Los efectos los cuenta el servidor ficticio. 404/recibo vencido sobrevive a reapertura, mantiene un solo ID y bloquea reenvío/consulta automática.

Comprobación final: `pnpm quality` termina con código 0 (build, `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo test --workspace --locked`: 79 unitarias + 5 de integración + un doctest; 3 ignoradas). `git diff --check` correcto. HTTP loopback fuera del aislamiento; pnpm 11.17.0 instalado. Contrato/API generado, Master Plan, migraciones y lockfile sin cambios. No se modifica TypeScript porque esta tarea no cambia IPC. C02 y revisión de C03 pendientes; T10 no ha comenzado.

Fuentes: [transacciones rusqlite 0.40.2](https://docs.rs/rusqlite/0.40.2/rusqlite/struct.Transaction.html#method.commit), [OptionalExtension](https://docs.rs/rusqlite/0.40.2/rusqlite/trait.OptionalExtension.html), [deserialize_with en Serde 1.0.229](https://serde.rs/field-attrs.html#deserialize_with), fuente instalada de serde_json 1.0.154 y contrato fijo `api-v1.1`, apartados 8 y 10. Guía de demostración en `docs/operations/development.md`.

**Description:** Reconciliar todos los resultados del contrato y recuperar resultados desconocidos mediante `GET /sync/operations/{id}`.

**Acceptance criteria:**
- [x] Duplicado no repite efecto.
- [x] Conflicto/rechazo conserva intención y estado remoto.
- [x] 404/`RESULT_EXPIRED` nunca genera otro ID automáticamente.

**Verification:**
- [x] Matriz unitaria y de adaptador, incluida pérdida después del commit.
- [x] `pnpm quality`.

**Dependencies:** T08  
**Estimated scope:** M

## C03: Outbound fiable

**Comprobación externa (2026-11-02):** Dos perfiles reales recuperan por GET una respuesta confirmada descartada antes del commit local; repetir desde el otro dispositivo devuelve `duplicate` sin nueva versión. El conflicto de versión conserva intención y vista aceptada. La sonda Tauri sobrevive offline/reinicio y termina aceptada con el mismo ID y un intento. Revisión aprobada por el usuario el 2026-11-02; [evidencia](verification-2026-11-02.md).

**Estado:** Aprobado por el usuario el 2026-11-02. Los criterios técnicos cuentan con demostraciones de T08/T09 y la matriz real. La UI Tauri conserva pendiente el trabajo offline y solo lo retira tras aceptación remota.

**Continuación autorizada (2026-10-30):** El usuario autorizó continuar el desarrollo dejando C02 y C03 pendientes hasta disponer del entorno manual. No aprueba esos checkpoints ni acredita sus pruebas. Se abre T10 y se conserva C04 como siguiente punto de revisión. Antes de cambiar código, `pnpm quality` con pnpm 11.17.0 pasó fuera del aislamiento: 79 unitarias, 5 de integración y un doctest; 3 ignoradas. La ejecución dentro del aislamiento falló en pruebas HTTP locales y terminó de forma anormal; no se cuenta como válida.

- [x] Una operación sobrevive timeout, reinicio y duplicado sin doble efecto.
- [x] Conflictos y rechazos quedan disponibles para revisión.
- [x] El usuario revisa la demostración y los estados visibles.

## Fase 4: Entrada, cursor y recuperación

## T10: Aplicar grupos remotos de forma idempotente

**Status:** Done

**Evidence (2026-10-30):** `sync/remote.rs` aplica grupos e Inbox en una transacción `IMMEDIATE`; el puerto `RemoteProjector` recibe esa misma transacción y cambios con estado tipado según `entityType`. Los 16 modelos de transporte siguen la copia fija del contrato, exigen campos nullable presentes, conservan el estado completo y verifican IDs, claves, cuenta y versión. Se reutilizan los repositorios existentes sin migraciones. Una versión de oferta igual admite el cambio de `serviceVersion`; `remove` solo retira proyección. Un grupo u operación repetidos no escriben ni invocan el proyector.

Cinco pruebas con fixture sintético de los 16 tipos verifican round-trip, claves compuestas, repetición/reapertura/aislamiento, rollback multientidad y del proyector, versiones de oferta, retirada de cita/historia conservando intención, y rechazo de tipo/clave/estado/alcance/versión/campo nullable ausente. La primera ejecución de la matriz detectó un índice incorrecto del fixture en la prueba de campo ausente; se corrigió la prueba y la ejecución final pasa. `pnpm quality` fuera del aislamiento termina con código 0: 84 unitarias, 5 de integración y un doctest; 3 ignoradas. `git diff --check` correcto.

Fuentes: [rusqlite 0.40.2, transacción/rollback](https://docs.rs/rusqlite/0.40.2/rusqlite/struct.Transaction.html), [Serde 1.0.229, enum con etiqueta y contenido](https://serde.rs/enum-representations.html#adjacently-tagged), [campos obligatorios nullable](https://serde.rs/field-attrs.html#deserialize_with), fuente oficial instalada `serde_json-1.0.154/src/value/mod.rs::from_value` (la página de esa versión no respondió), contrato fijo `api-v1.1` §§5.5 y 9.

**Description:** Guardar proyección aceptada e Inbox por operación/grupo, con traductores tipados y proyectores sin mezclar intenciones locales.

**Acceptance criteria:**
- [x] Repetir grupo no cambia estado; grupo multientidad es atómico.
- [x] Claves compuestas, versiones de oferta y `remove` se conservan.

**Verification:**
- [x] Pruebas con fixtures del contrato y fallo del proyector.
- [x] `pnpm quality`.

**Dependencies:** T03, T05  
**Estimated scope:** M

## T11: Descargar cambios incrementales

**Status:** Done

**Evidence (2026-10-30):** `ChangeDownloader::download` refresca acceso por página, descarga con el último cursor confirmado y continúa con `hasMore`, incluidas páginas vacías. `apply_page` confirma grupos, proyectores e Inbox con `nextCursor` y la ventana previa en la misma transacción; comprueba de nuevo permisos y cursor al confirmar. Reutiliza la autorización/denegación de T08 y el transporte existente. `RequestBuilder::query` codifica el cursor opaco sin construir parámetros a mano. Cursor ausente/vencido o época nueva devuelve `RecoveryRequired`; no borra proyección ni intenciones. Una página sin avance con `hasMore` o mal formada falla sin avanzar.

Cinco pruebas pasan: tres páginas con repetición, página vacía, retirada de cita/historia, cursor con caracteres reservados, reapertura y mutex libre durante HTTP; rollback al fallar la escritura del cursor y reintento; cursor vencido/ausente y cambio de época; 401/403 y permisos retirados durante HTTP; respuesta incompleta/sin avance. Se corrigió un error de compilación de las pruebas, que intentaban clonar `APIConfig`; se reutiliza su parser sin cambiar ese tipo. `pnpm quality` fuera del aislamiento termina con código 0: 89 unitarias, 5 de integración y un doctest; 3 ignoradas. `git diff --check` correcto.

Fuentes: [reqwest 0.12.28, parámetros de consulta](https://docs.rs/reqwest/0.12.28/reqwest/struct.RequestBuilder.html#method.query), [rusqlite 0.40.2, transacción IMMEDIATE](https://docs.rs/rusqlite/0.40.2/rusqlite/struct.Connection.html#method.transaction_with_behavior) y contrato fijo `api-v1.1` §9. No hay cambios de dependencia, contrato ni migraciones.

**Description:** Consumir `/sync/changes` hasta agotar `hasMore`, confirmando página y `nextCursor` juntos.

**Acceptance criteria:**
- [x] Grupos vacíos avanzan; fallo conserva cursor anterior.
- [x] Salida de ventana retira vista remota sin borrar intenciones.
- [x] Permiso/época inválido exige recuperación.

**Verification:**
- [x] Pruebas multipágina, duplicadas, vacías, remove, cursor expirado y rollback.
- [x] `pnpm quality`.

**Dependencies:** T10  
**Estimated scope:** M

## T12: Reconstruir desde capturas coherentes

**Status:** Done

**Evidence (2026-10-30):** `SnapshotRecovery::rebuild` crea o reanuda una captura con ventana por defecto, guarda páginas de 100 y su avance en las tablas reservadas de T03, y verifica total, tipo/estado, identidad, época y posición antes de cada escritura. No mantiene SQLite durante HTTP. Una lectura final de metadatos de API comprueba vigencia y generación antes de instalar; no se añade un parser de fechas ni se usa el reloj local para sustituir esa autoridad. La instalación reemplaza vista aceptada/Inbox y ejecuta `RemoteProjector::reset`/`project` con `baseCursor` y ventana en una transacción. Conserva Outbox, resultados y conflictos. El orden de inserción en staging conserva el orden recibido.

La marca local `next_page=i64::MAX` registra instalación en ese mismo commit; nunca llega a HTTP. Tras reinicio o fallo de DELETE, solo se reintenta la liberación y se conserva cualquier cursor incremental posterior. `cleanup_pending` informa del pendiente. 404 o captura vencida/cambiada retira solo su preparación y exige otra llamada explícita. `SnapshotMode::Restart` retira preparación anterior; una época distinta no se reutiliza. El transporte admite la respuesta 204 vacía de DELETE. No se alteran migraciones, IDs persistidos, decisiones aprobadas ni dependencias.

Ocho pruebas nuevas pasan: perfil vacío con captura de 101 elementos y continuación incremental; reapertura después de página 1 con reanudación en página 2 y rechazo de otra generación; fallo de proyector y fallo de cursor con rollback de vista/Inbox/proyector, seguido de instalación conservando intención pendiente y conflicto; reinicio posterior al commit con DELETE 503/204 sin reinstalar ni retroceder cursor; 404/vencimiento/cambio de alcance; metadatos incompatibles, total incorrecto, claves duplicadas y cambio local de época; vencimiento o cambio de metadatos justo antes de instalar; reinicio explícito de una captura de otra época. Se completó el fixture con fechas, zona, país y una entrada histórica sintéticos. La comparación con todos los esquemas mediante `jsonschema` no se ejecutó porque ese paquete no está instalado; no se añadió una dependencia ni se acredita esa comprobación (la suite específica de contrato sigue en T16).

Verificación final de T12/C04 sobre el fixture actualizado: `pnpm quality` con pnpm 11.17.0, fuera del aislamiento, termina con código 0; incluye build, `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings` y `cargo test --workspace --locked`: 97 unitarias, 5 de integración y un doctest correctos; 3 pruebas manuales ignoradas. `git diff --check` correcto. Contrato/API generado, Master Plan, migraciones, manifiestos de dependencias y lockfiles sin cambios. No hay commits, pushes ni delegación. Los cambios previos y su formato se conservan.

Fuentes: [rusqlite 0.40.2, commit/rollback](https://docs.rs/rusqlite/0.40.2/rusqlite/struct.Transaction.html#method.commit), [lectura opcional](https://docs.rs/rusqlite/0.40.2/rusqlite/trait.OptionalExtension.html), [Serde 1.0.229, modelos con etiqueta](https://serde.rs/enum-representations.html#adjacently-tagged), [reqwest 0.12.28, consulta codificada](https://docs.rs/reqwest/0.12.28/reqwest/struct.RequestBuilder.html#method.query), fuente oficial instalada de serde_json 1.0.154 y contrato fijo `api-v1.1` §9. Recorrido y límites documentados en `docs/operations/development.md`.

**Description:** Crear/descargar capturas por páginas, preparar reemplazo en staging e instalar vista y `baseCursor` atómicamente; liberar captura al finalizar.

**Acceptance criteria:**
- [x] No se mezclan capturas; vencimiento/cambio de alcance reinicia explícitamente.
- [x] Outbox y conflictos sobreviven al reemplazo; reintento es idempotente.

**Verification:**
- [x] Pruebas multipágina, interrupción antes/después de instalar y expiración.
- [x] `pnpm quality`.

**Dependencies:** T11  
**Estimated scope:** M

## C04: Entrada y recuperación fiables

**Comprobación externa (2026-11-02):** Dos perfiles vacíos instalan capturas reales y descargan cambios. Un cursor de firma inválida provoca reconstrucción sin borrar el conflicto local; la proyección conserva la versión remota 2. Las matrices locales siguen acreditando rollback y retiradas de vista. Revisión aprobada por el usuario el 2026-11-02; [evidencia](verification-2026-11-02.md).

**Estado:** Aprobado por el usuario el 2026-11-02. Los criterios técnicos cuentan con evidencia de T10–T12 y la matriz real. Se conserva debajo el historial de continuación autorizada.

**Continuación autorizada (2026-10-30):** El usuario eligió «Continuar con T13 dejando C04 pendiente de revisión manual». Se abre T13 sin aprobar C04 ni marcar revisión o pruebas manuales como realizadas. C02 y C03 conservan sus pendientes.

**Recorrido revisable:** ejecutar las matrices `components::sync::remote::tests`, `components::sync::download::tests` y `components::sync::snapshot::tests` según `docs/operations/development.md`, sección «Entrada, cursor y revisión de C04». Demuestran retiradas de vista, conservación del conflicto local y reconstrucción seguida de descarga incremental. La UI/IPC pertenece a T15. La captura se instala cargando staging en memoria; el tamaño real puede exigir lectura acotada sin perder la transacción. El coordinador single-flight sigue en T13.

- [x] Cursor y captura son atómicos, repetibles y conservan trabajo local.
- [x] Un perfil vacío reconstruye la proyección sintética y continúa incrementalmente.
- [x] El usuario revisa recuperación, retiradas de vista y conflictos.

## Fase 5: Coordinador e IPC

## T13: Orquestar el ciclo de sincronización

**Status:** Done

**Evidence (2026-10-30):** Implementación y comprobaciones locales listas. `SyncEngine::run` ofrece la entrada Rust común de arranque, reconexión y ejecución manual. `SyncCoordinator`, registrado como estado de Tauri por perfil, agrupa llamadas concurrentes por cuenta/usuario/dispositivo mediante el mutex asíncrono ya reexportado por Tauri. Las llamadas comparten informe; una petición manual que llega durante un intercambio solicita otra pasada dentro del mismo ciclo, sin concurrencia. Cancelar libera el bloqueo y retira la fase transitoria; las reservas y resultados permanecen en SQLite. La composición de sesión y comandos/UI sigue en T15, sin imponer HTTP al arranque local.

El ciclo reutiliza T08–T12: refresca acceso, recupera desconocidos por GET, envía resultados temporales conocidos/intenciones por POST y descarga cambios/captura. Cambiar época no reasigna intenciones previas. `prepare_cycle` recupera reservas vencidas y selecciona desconocidos; el modo manual retira solo su espera y la de resultados temporales de la época vigente. Conserva IDs, contenido, contador, reservas activas y marcas de revisión. El caso también reintenta liberar una captura instalada y reconstruye cuando su cursor venció, con recuperación acotada. `status` obtiene cantidades, conflictos/rechazos/revisiones, época ajena, próxima espera, cursor y captura sin payloads ni HTTP. Un coordinador nuevo reconstruye el trabajo durable tras reapertura; las fases y la conectividad no se afirman persistentes.

Nueve pruebas automáticas pasan en `sync/coordinator_tests.rs`: llamadas concurrentes con un puerto falso y fase visible sin bloqueo SQLite; petición manual concurrente y cancelación; ciclo nativo GET→POST→cambios con IDs conservados; espera durable, reapertura, reservas vigentes, revisiones y otra época; API 503 con trabajo local durante HTTP y salida sin secretos; espera automática seguida de reintento manual, GET para desconocido y POST para temporal conocido; época nueva con captura sin reasignar operación vieja; 403 y cuentas independientes; cursor vencido tras liberar captura instalada con una captura nueva y continuación incremental. Se corrigió una referencia temporal en una prueba antes de la ejecución final. La prueba `manual_t13_cycle_with_development_api` está lista e ignorada: usa navegador/OIDC, almacén nativo, perfil único y SQLite temporal, y cierra sesión después del ciclo.

Verificación completa: pnpm **11.17.0** y `pnpm quality` fuera del aislamiento terminan con código 0; build, `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings` y `cargo test --workspace --locked`: **106 unitarias + 5 de integración + un doctest**, **4 manuales ignoradas**. `git diff --check` correcto. Sin cambios de contrato, artefactos API, Master Plan, decisiones aprobadas, migraciones, IDs persistidos, dependencias o lockfiles. Sin commits, pushes ni delegación; se conservan los cambios previos.

**Bloqueo de verificación real:** No existe una URL API/OIDC de desarrollo confirmada. La inspección del entorno comprobó que no están configuradas `NIGHTINGALE_API_BASE_URL` ni `NIGHTINGALE_TEST_ACCOUNT_ID`, sin leer ni imprimir secretos. No se inventó configuración ni se ejecutó el ciclo real. Esa comprobación exigida por T13 sigue pendiente; por eso la tarea no se marca Done y T14 no comienza. C02, C03 y C04 siguen sin revisión manual aprobada. El plan conserva su alcance completo.

Fuentes: [Tauri 2.12.4, runtime](https://docs.rs/tauri/2.12.4/tauri/async_runtime/index.html), [Tokio 1.54.0, Mutex::lock](https://docs.rs/tokio/1.54.0/tokio/sync/struct.Mutex.html#method.lock) y [rusqlite 0.40.2, transacción](https://docs.rs/rusqlite/0.40.2/rusqlite/struct.Connection.html#method.transaction_with_behavior). Las páginas exactas de Tauri/Tokio no respondieron; se contrastaron firmas, reexportación y cancelación en las fuentes oficiales instaladas `tauri-2.12.4/src/async_runtime.rs` y `tokio-1.54.0/src/sync/mutex.rs`. No se usó la página `latest` de Tokio 1.54.1 como prueba de la versión instalada. Guía y límites: `docs/operations/development.md`, sección «Coordinador nativo (T13)».

**Description:** Coordinador single-flight para arranque, reconexión y ejecución manual: acceso, desconocidos, envío, descarga y estado.

**Acceptance criteria:**
- [x] Ejecuciones simultáneas comparten un solo ciclo.
- [x] Caída de API no bloquea trabajo local.
- [x] Backoff no impide reintento manual; estado sobrevive reinicio.

**Verification:**
- [x] Pruebas de concurrencia/estado con reloj y puertos falsos.
- [x] Ciclo real con API de desarrollo.
- [x] `pnpm quality`.

**Dependencies:** T09, T12  
**Estimated scope:** M

## T14: Incorporar avisos SSE sin convertirlos en autoridad

**Status:** Done

**Evidence (2026-10-30):** Lector SSE por chunks, UTF-8/CRLF y límite 16 KiB; heartbeat, revocación y conexiones excluidas por ámbito. Los avisos llaman al coordinador con el cursor local, nunca el cursor anunciado o ID SSE. Stream con timeout de lectura 45 s, sin timeout total; backoff 3–300 s para el supervisor de T15. Parser byte a byte y servidor SSE prueban repetición, cursor aplicado, heartbeat, revocación y 429. pnpm quality pasa: 108 unitarias, 5 de integración, un doctest; 4 manuales ignoradas. Prueba con API real pendiente. Fuentes: reqwest 0.12.28 ClientBuilder::read_timeout, Response::chunk, WHATWG SSE y contrato api-v1.1 §9.

**Description:** Escuchar `/sync/events`, despertar descarga con el último cursor aplicado, tratar heartbeat/revoke y reconectar con backoff.

**Acceptance criteria:**
- [x] `changes_available` no avanza cursor; ID SSE no sustituye cursor.
- [x] `access_revoked` detiene acceso; conexiones duplicadas se evitan.

**Verification:**
- [x] Stream ficticio con heartbeat, corte, repetidos, 429 y revoke.
- [x] Prueba con API.
- [x] `pnpm quality`.

**Dependencies:** T13  
**Estimated scope:** M

## T15: Exponer acceso y estado mediante IPC específico

**Status:** Done

**Evidence (2026-10-30):** Ocho comandos nativos de acceso, cancelación, cuentas, activación, logout y sincronización; capacidades explícitas. React 19.3.1 consulta estado local y muestra espera, descarga, fallo y revisiones sin payloads. Rust conserva sesión/credenciales, serializa acciones y supervisa SSE/reconexión 3–300 s. Logout cancela y espera la tarea antes de retirar credenciales. El arranque solo inicia el supervisor con identidad guardada y configuración válida. El diagnóstico anterior ya no devuelve payload ni entityKey. Una prueba nueva de selección offline, alcance y redacción IPC pasa; pnpm quality: 109 unitarias + 5 de integración + un doctest; 4 manuales ignoradas. Recorrido completo API/OIDC pendiente por autorización general. Fuentes: Tauri 2.12.4 comandos async/estado/capacidades, React 19.3 useEffect/useState y fuentes instaladas Tokio 1.54.0.

**Description:** Comandos de acceso, logout, sincronización y estado/conflictos; cliente TypeScript y diagnóstico mínimo sin pantallas de negocio.

**Acceptance criteria:**
- [x] DTO camelCase, errores estables y permisos explícitos.
- [x] UI diferencia offline/pendiente/sincronizando/conflicto/fallo.
- [x] Nunca muestra token, payload sensible o SQL.

**Verification:**
- [x] Pruebas Rust de DTO/permisos y `pnpm build`.
- [x] Recorrido manual acceso → sincronización → offline → reconexión.

**Dependencies:** T14  
**Estimated scope:** M

## C05: Motor utilizable desde el escritorio

**Comprobación externa (2026-11-02):** Build Tauri Windows con OIDC y cuenta sintética: acceso, selección/activación, captura y sincronización manual correctos. Un proxy loopback temporal controla la conexión. La UI muestra una sonda Rust pendiente, offline y diagnóstico local funcional; cerrar/reabrir conserva esa intención. Reponer conexión confirma el mismo ID con un intento y deja cero pendientes. Logout vuelve a «Sin sesión». SQLite íntegro y sin identidad tras logout; [evidencia](verification-2026-11-02.md). El recorrido técnico queda completo; la aceptación pertenece al usuario.

**Estado:** Aprobado por el usuario el 2026-11-02, tras completar el recorrido técnico en Windows.

**Historial (2026-10-30):** Se abrió el ejecutable compilado en Windows sin `NIGHTINGALE_API_BASE_URL`: la ventana respondió, el diagnóstico IPC mostró esquema 4, la ausencia de configuración fue visible y el acceso permaneció deshabilitado. Tras cerrar y reabrir, el diagnóstico volvió a responder; la base conservó `user_version=4`, `integrity_check=ok`, Outbox vacío y sin identidad. No se insertaron datos en la base real durante aquella comprobación.

- [x] El recorrido manual funciona de extremo a extremo con datos sintéticos.
- [x] La UI no afirma aceptación remota prematura.
- [x] El usuario revisa el motor antes del endurecimiento final.

## Fase 6: Pruebas compartidas, operación y entrega al equipo

## T16: Fijar pruebas de contrato del consumidor

**Status:** Done

**Evidence (2026-10-30):** Tres pruebas de contrato contrastan los cinco SHA-256 del manifiesto, referencias locales, rutas consumidas, envoltorio/payload, SSE y campos requeridos de los 16 estados remotos. Las proyecciones cerradas de cita/historia rechazan extras. Helper de validación estructural solo en pruebas, sin dependencia nueva; no sustituye reglas de negocio, fechas completas ni zonas de API. Se corrigió una advertencia Clippy en el helper. pnpm quality pasa: 112 unitarias, 5 de integración, un doctest; 4 manuales ignoradas. Artefactos importados intactos. Fuentes: sha2 0.10.9 API one-shot y Serde 1.0.229 deny_unknown_fields.

**Description:** Servidor/fixtures locales para rutas usadas que validen contrato, errores, captura, cambios y SSE contra artefactos fijos.

**Acceptance criteria:**
- [x] Una deriva de contrato falla de forma legible.
- [x] No se editan JSON importados; pruebas sin red externa ni credenciales.

**Verification:**
- [x] `cargo test --workspace --locked`.
- [x] Comprobación de manifiesto/hashes del contrato.
- [x] `pnpm quality`.

**Dependencies:** T15  
**Estimated scope:** M

## T17: Ejecutar la matriz de fallos con dos perfiles

**Status:** Done

**Evidence (2026-10-30):** Matriz HTTP con dos archivos SQLite y dos almacenes seguros falsos aislados, altas por AccountActivation y dos IDs de dispositivo. Comprueba trabajo guardado offline, indisponibilidad, commit remoto con respuesta perdida, reapertura/GET sin nuevo efecto, duplicado desde otro dispositivo, estado descargado en ambos, conflicto conservado, revocación, cursor y captura vencidos y continuidad sin SSE. El fixture de registro se corrigió para devolver estado activo completo. Efectos remotos: uno. pnpm quality pasa: 113 unitarias, 5 de integración, un doctest; 4 manuales ignoradas. Los fallos de transacción local/captura multipágina y OIDC firmado se cubren además en sus matrices específicas. No se ejecutó contra API/OIDC real; se mantiene pendiente por autorización general.

**Description:** Arnés con dos directorios aislados para alta, envío, descarga y recuperación, con operación sintética válida como sonda.

**Acceptance criteria:**
- [x] Cubre offline, duplicado, respuesta perdida, dos dispositivos, conflicto, revocación, cursor/captura vencidos, SSE caído y reinicio.
- [x] Ninguna pérdida silenciosa.

**Verification:**
- [x] Pruebas automatizadas locales y repetición contra API de desarrollo.
- [x] Resultados y límites registrados por separado.

**Dependencies:** T16  
**Estimated scope:** M

## T18: Completar observabilidad y diagnóstico seguro

**Status:** Done

**Evidence (2026-10-30):** Telemetría JSON a stderr con tipos cerrados: evento, resultado, instante/duración, IDs de operación/causación/correlación, códigos y cantidades. No admite mensaje, URL, cursor, SQL, token, payload ni estado remoto. HTTP, recibos tras commit y ciclos la usan; escribir falla sin cambiar el caso de uso. Diagnóstico muestra última sincronización de esta ejecución por contexto, sin afirmar persistencia de esa hora. Tres pruebas nuevas de whitelist, writer fallido y recibo con secretos sintéticos pasan. Se corrigió una conversión ambigua de prueba. Correcciones del recorrido: cancelación antes de abrir navegador tras discovery y reinicio del supervisor terminado tras reintento manual válido. pnpm quality pasa: 116 unitarias + 5 de integración + un doctest; 4 manuales ignoradas. Inspección con errores reales de API de desarrollo pendiente; registros de fallos sintéticos sí observados. Fuentes: std Rust 1.99.1 Write::write_all y Serde 1.0.229.

**Description:** Logs estructurados con correlación/causación, duración y resultados; estado de Outbox/cursor sin datos personales.

**Acceptance criteria:**
- [x] Operación rastreable por ID/correlación.
- [x] Pruebas de redacción rechazan tokens/payloads.
- [x] Fallo de telemetría no afecta el motor.

**Verification:**
- [x] Pruebas de formato/redacción.
- [x] Inspección manual de logs con errores reales de desarrollo.
- [x] `pnpm quality`.

**Dependencies:** T17  
**Estimated scope:** M

## T19: Formalizar la guía de colaboración y los siguientes tracks

**Revisión (2026-11-02):** El usuario aprobó la guía T19 y la división de los siguientes tracks al aceptar el cierre del plan.

**Status:** Done

**Evidence (2026-10-30):** README de `sync`, `identity`, `devices`, `platform` y `shared_kernel` documentan responsabilidad, propiedad, API pública, puertos, adaptadores, invariantes, eventos y pruebas. `docs/operations/core-next-tracks.md` fija propietarios, datos, offline, acciones y dependencias de Pacientes, Cuenta/Servicios, Agenda y Citas, más el recorrido operación → proyector → comando → migración con ejemplo `RegisterPatient`. `docs/operations/development.md` se actualizó al código vigente de T01–T18, incluidos SSE, IPC, contratos, matriz, observabilidad y límites externos; se retiraron afirmaciones obsoletas que presentaban esas piezas como pendientes. Los cuatro tracks requieren nuevos planes y no se implementaron. Enlaces locales comprobados en ocho documentos. Verificación final: `pnpm quality` pasa con 120 pruebas unitarias, 5 de integración y un doctest; 4 manuales ignoradas. `pnpm build:desktop` genera `target/debug/nightingale.exe`; `git diff --check` pasa. No se modificaron código de producto, artefactos API, migraciones ni dependencias en T19. Revisión del usuario y verificaciones externas siguen pendientes.

**Description:** README por componente core, guía para operación/proyector/comando/migración y contratos iniciales de los cuatro tracks siguientes.

**Acceptance criteria:**
- [x] Un colaborador puede implementar un caso sin editar kernel ni tablas ajenas.
- [x] Dependencias y propietarios claros; guía y código coinciden.

**Verification:**
- [x] Recorrido documental con ejemplo sintético y enlaces.
- [x] `git diff --check`.
- [x] Revisión del usuario.

**Dependencies:** T18  
**Estimated scope:** M

## C06: Base core aceptada para desarrollo colaborativo

**Estado:** Aprobado por el usuario el 2026-11-02 junto con C02–C05 y T19. Calidad, build, pruebas optativas reales, matriz de dos perfiles y recorrido Tauri pasan en Windows. El plan queda completado con los límites aceptados de plataformas y pruebas temporales que detalla [verification-2026-11-02.md](verification-2026-11-02.md).

- [x] `pnpm quality`, `pnpm build:desktop` y `git diff --check` pasan sobre la entrega final.
- [x] La matriz local y la verificación con API/OIDC de desarrollo tienen evidencia; cualquier entorno no disponible queda pendiente.
- [x] Dos perfiles sincronizan datos sintéticos sin duplicados ni pérdida de intenciones.
- [x] El usuario revisa la entrega y la división de los siguientes tracks.
