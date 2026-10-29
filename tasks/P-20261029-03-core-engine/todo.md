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
- [ ] Revisión del usuario en C01.

**Dependencies:** None  
**Estimated scope:** M

## T02: Completar primitivas y envoltorio de operación

**Status:** Todo

**Description:** Introducir solo los IDs/contexto/versiones necesarios y extender la operación con `dependsOn`, serialización contractual y huella semántica estable.

**Acceptance criteria:**
- [ ] UUIDv7 canónicos y errores estables.
- [ ] Round-trip camelCase conforme a ejemplos del contrato.
- [ ] La misma intención produce la misma huella; cualquier cambio semántico la altera.

**Verification:**
- [ ] Pruebas unitarias de dominio y serialización.
- [ ] `cargo test --workspace --locked`.
- [ ] `cargo clippy --workspace --all-targets --locked -- -D warnings`.

**Dependencies:** T01  
**Estimated scope:** M

## T03: Persistir identidad y control de sincronización

**Status:** Todo

**Description:** Aplicar `0003` y `0004`, evolucionar el Outbox sin perder operaciones y añadir repositorios para acceso, cursor, Inbox y proyección remota.

**Acceptance criteria:**
- [ ] Migración desde `0002` conserva Outbox.
- [ ] Reapertura no repite cambios; rollback deja archivo e historial intactos.
- [ ] No existen columnas de tokens.

**Verification:**
- [ ] Pruebas temporales de migración y repositorios.
- [ ] `cargo test --workspace --locked`.
- [ ] Inspección del esquema resultante.

**Dependencies:** T02  
**Estimated scope:** M

## C01: Contratos congelados

- [ ] `pnpm quality` pasa y las bases `0002` migran sin pérdida.
- [ ] El usuario revisa estados, tablas, puertos y dependencias antes de abrir la fase de acceso.
- [ ] Quedan reservados `0003`/`0004` y los puntos de integración.

## Fase 2: Identidad, secretos y dispositivo

## T04: Validar configuración y proteger credenciales

**Status:** Todo

**Description:** Cargar URL de API/OIDC y parámetros por entorno con valores seguros por defecto; implementar almacén seguro del SO y fake de pruebas.

**Acceptance criteria:**
- [ ] Configuración inválida falla antes de sincronizar.
- [ ] Tokens solo se escriben mediante el puerto seguro.
- [ ] Borrado y rotación no dejan copias en SQLite ni logs.

**Verification:**
- [ ] Pruebas de precedencia y almacén fake.
- [ ] Prueba real del almacén del SO con credencial ficticia.
- [ ] `pnpm quality`.

**Dependencies:** T01, T03  
**Estimated scope:** M

## T05: Crear transporte HTTPS tipado para API v1

**Status:** Todo

**Description:** Cliente Rust con Bearer, timeouts, límites, correlación, errores estructurados y DTO solo para rutas consumidas.

**Acceptance criteria:**
- [ ] Códigos HTTP y de operación se traducen sin depender de `message`.
- [ ] URL y credenciales no aparecen en errores.
- [ ] Ninguna llamada retiene el mutex SQLite.

**Verification:**
- [ ] Pruebas contra servidor local para 200/401/403/410/413/422/429/503.
- [ ] `pnpm quality`.

**Dependencies:** T02, T04  
**Estimated scope:** M

## T06: Implementar acceso OIDC con PKCE

**Status:** Todo

**Description:** Navegador externo, Authorization Code, PKCE S256, state/nonce y redirección local; canjear, renovar y cerrar sesión sin exponer tokens a React.

**Acceptance criteria:**
- [ ] State/nonce inválidos se rechazan.
- [ ] Cancelar navegador conserva el estado anterior.
- [ ] Refresh y logout no revocan por error el token de dispositivo.

**Verification:**
- [ ] Pruebas con proveedor OIDC ficticio y navegador externo.
- [ ] Flujo manual con cuenta de desarrollo.
- [ ] `pnpm quality`.

**Dependencies:** T05  
**Estimated scope:** M

## T07: Activar instalación y autorización de cuenta

**Status:** Todo

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
