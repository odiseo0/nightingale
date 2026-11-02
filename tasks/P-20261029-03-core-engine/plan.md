# P-20261029-03-core-engine: Motor transversal del escritorio

**Status:** Completed
**Created:** 2026-10-29  
**Updated:** 2026-11-02
**Task list:** [todo.md](todo.md)  
**Plan relacionado:** [P-20261029-02-component-foundation](../P-20261029-02-component-foundation/plan.md), completado; este plan no reabre su historial.

**Cierre (2026-11-02):** El usuario aprobó C02–C06 y la guía T19 con «Sí, aprobar y cerrar el plan», según el [informe de verificación](verification-2026-11-02.md). Se conservan la excepción de macOS/Linux y los límites de las pruebas temporales de capturas/journal. C01 ya estaba aprobado. T01–T19 y C01–C06 quedan completos dentro de ese alcance.

## Contexto y resultado

El proyecto ya tiene la base hexagonal, un kernel pequeño, migraciones SQLite, IPC tipado y un Outbox local. Todavía no puede autenticar una instalación, enviar esas operaciones, recuperar resultados, descargar cambios ni reconstruir un dispositivo. Esas piezas son el verdadero motor compartido que necesitan los siguientes casos de uso.

El usuario eligió limitar este plan al **motor transversal**, sin implementar Pacientes, Servicios, Agenda o Citas como funcionalidades de producto. Al terminar, el equipo debe disponer de un recorrido verificable de extremo a extremo:

```text
OIDC/PKCE → token seguro de dispositivo → acceso a cuenta → registro de instalación
→ operación local durable → envío idempotente → resultado/conflicto conservado
→ cambios incrementales/cursor → recuperación por captura → aviso SSE
→ estado visible mediante IPC, sin secretos ni SQL en React
```

La base quedará lista para desarrollo colaborativo con datos sintéticos. No se declarará lista para datos reales ni producción mientras cifrado de SQLite, distribución de claves, instaladores y endurecimiento de publicación sigan abiertos.

## Qué se considera «core»

| Pieza | Responsabilidad y límite |
|---|---|
| `shared_kernel` | IDs tipados, actor/cuenta/dispositivo, instantes, versiones y errores estables. Sin entidades de negocio, repositorios ni clientes HTTP. |
| `identity` | OIDC Authorization Code + PKCE S256, sesión, identidad activa, cuentas y última autorización válida. No decide reglas de Pacientes o Citas. |
| `devices` | Token opaco de API, estado de la instalación y registro en la cuenta. La credencial se guarda solo en el almacén seguro del SO. |
| `platform` | Configuración validada, SQLite/migraciones, almacén seguro y transporte HTTPS. No contiene reglas del producto. |
| `sync` | Outbox, dependencias, reintentos, resultados desconocidos, Inbox, cursor, captura, proyección remota aceptada, coordinador y SSE. |
| IPC y diagnóstico | Comandos específicos para acceso, sincronización manual y estado. React nunca recibe tokens, credenciales ni acceso SQL. |
| Contratos de colaboración | Propiedad de datos, puertos de proyección, fixtures, reservas de migraciones y puntos de integración explícitos. |

No se crearán un bus de eventos genérico, un repositorio universal, un motor de reglas arbitrarias, un framework de formularios ni un sistema visual completo. No se añadirán servidores, web apps, PostgreSQL o credenciales cloud a este repositorio.

## Source of Truth

- `AGENTS.md`: casos de uso en Rust, SQLite local sin HTTP, React mediante comandos específicos, operación offline y tres repositorios separados.
- `docs/master-plan.md`: §§6–9, 11–14, 19–24, 26–27; especialmente propiedad de componentes, Outbox/Inbox/cursor, errores, seguridad y pruebas.
- `docs/technical-architecture.md`: §§7–12, 17, 19–24 y 26; separación entre aceptación local y compartida.
- `docs/contracts/api-v1/nightingale-api.md`, `openapi.json`, `accepted-operations.schema.json`, `capabilities.json`, `manifest.json` y `sse.schema.json`, revisión fija `api-v1.1`. Los artefactos no se editan manualmente.
- Código vigente: `src-tauri/src/shared_kernel/`, `platform/sqlite/`, `components/sync/`, `components/storage/`, `components/diagnostics/`, `src/shared/api/` y las migraciones `0001`/`0002`.
- Convenciones heredadas de `P-20261029-02-component-foundation`: kernel deliberadamente pequeño, migraciones reservadas, un comando por caso de uso y DTO camelCase.

## Decisiones de arquitectura

1. **Tres estados separados.** `sync` conserva una proyección del estado remoto aceptado; cada componente de negocio conservará sus entidades e intenciones locales; el Outbox conserva el estado de sincronización. Un rechazo, cambio de ventana o captura nueva no borra intenciones pendientes.
2. **La proyección remota no es un ORM.** Los elementos aceptados se almacenan como datos de transporte versionados dentro de `sync`, no como tablas PostgreSQL reflejadas ni como API genérica para React. Cada componente posterior implementa un proyector tipado hacia sus propias tablas y consultas.
3. **Operación inmutable.** ID, acción, clave, versión esperada, payload, dependencias e instante local permanecen estables durante reintentos y reinicios. Un cambio de intención requiere otro ID. Se guarda una huella canónica para detectar reutilización inconsistente.
4. **Contrato completo actual.** Se añade `dependsOn`, ausente en el Outbox actual. Cuenta, actor y dispositivo se obtienen de ruta/credencial; no se inventan campos en el cuerpo. Se respetan límites de 100 operaciones, 1 MiB por lote y 64 KiB por operación.
5. **HTTP fuera de las transacciones SQLite.** Ningún lock de conexión se mantiene durante red. Un coordinador single-flight evita envíos y descargas concurrentes; las funciones locales no esperan HTTP.
6. **Autoridad de permisos.** Se consume `allowedActions` y `permissionEpoch` de API junto con `capabilities.json`; no se duplican listas de 42/27 acciones en Rust. Offline solo se usa la última autorización válida; cambios administrativos exigen conexión. API sigue siendo la autoridad final.
7. **Recuperación conservadora.** Timeout o respuesta perdida deja resultado desconocido y recupera la misma operación. `conflict`/`rejected` conserva la intención para revisión. `CURSOR_EXPIRED`, cambio de época o captura inválida inicia otra captura, nunca un nuevo ID automático.
8. **Sin dependencias por rutina.** HTTP/SSE, OIDC/PKCE y almacén seguro se seleccionan en T01/T04 tras verificar documentación oficial de las versiones propuestas. Se prefiere el runtime Tauri existente y el mínimo número de crates directos; lockfiles actualizados junto al manifiesto.
9. **Integración central pequeña.** Los componentes exponen funciones de composición y proyectores. Un responsable de integración registra comandos/migraciones en los puntos calientes; los colaboradores no editan simultáneamente `lib.rs`, `build.rs` o capacidades por deducción.
10. **Desarrollo seguro, no producción.** Tokens OIDC/de API y refresh tokens nunca se guardan en SQLite, React o logs. Los ejemplos y pruebas usan datos ficticios. El cifrado de SQLite queda en un plan posterior obligatorio antes de datos reales.

## Datos, offline y excepciones cloud

- **Local sin red:** lectura del último acceso válido, consulta del estado/proyección ya descargada, registro durable de operaciones permitidas, reinicio con Outbox/cursor intactos y diagnóstico.
- **Requiere conexión:** OIDC, emisión/rotación/revocación de credenciales, registro de instalación, cambios de permisos/cuenta, aceptación remota, consulta de resultados, capturas, cambios y SSE.
- **Contrato remoto:** únicamente Nightingale API v1 por HTTPS. Patient Hub no participa en este plan.
- **Recuperación:** la captura y `baseCursor` se instalan atómicamente; cada página de cambios y `nextCursor` se confirman en la misma transacción; grupos vacíos pueden avanzar cursor; `remove` solo retira la vista remota, no intenciones locales.
- **Observabilidad:** correlación, duración, resultado, Outbox y antigüedad sin payloads personales, tokens ni secretos.

## Migraciones y puntos calientes reservados

- `0003_identity_access.sql`: identidad local, cuentas/dispositivo y última autorización válida; sin tokens.
- `0004_sync_control.sql`: dependencias/huella/resultados/reintentos del Outbox, Inbox, cursor, proyección remota aceptada y recuperación.
- Los planes posteriores reservarán desde `0005` antes de implementar.
- Integración propietaria: `src-tauri/src/lib.rs`, `src-tauri/build.rs`, `src-tauri/src/components/mod.rs`, `src-tauri/src/platform/sqlite/mod.rs`, `src-tauri/Cargo.toml`, `Cargo.lock` y `src-tauri/capabilities/default.json`.

## Orden de dependencias

```text
T01 → T02 → T03 → C01
T04 → T05 → T06 → T07 → C02
T08 → T09 → C03
T10 → T11 → T12 → C04
T13 → T14 → T15 → C05
T16 → T17 → T18 → T19 → C06

T04 depende de T01/T03; T05 depende de T02/T04; T08 depende de T07.
T10 depende de T03/T05; T13 depende de T09/T12.
Las fases se ejecutan secuencialmente; no se declara trabajo paralelo dentro de este plan.
```

## Tareas

### Fase 1: Contratos y persistencia del motor

#### T01: Cerrar el ADR del motor y seleccionar dependencias
- **Resultado:** documentar límites, máquina de estados, propiedad de tablas, puertos de proyección y estrategia de configuración; comparar las opciones HTTP/SSE/OIDC/secret store con documentación oficial y fijar versiones mínimas.
- **Aceptación:** el ADR cubre respuesta perdida, revocación, captura y concurrencia; la guía de colaboración identifica propietarios/puntos calientes; toda API no trivial seleccionada cita documentación acorde a su versión.
- **Verificación:** revisión de enlaces y coherencia con contrato v1; `git diff --check`; revisión del usuario en C01.
- **Archivos probables:** `docs/decisions/desktop-core-engine.md`, `docs/operations/development.md`, `CONTRIBUTING.md`, `src-tauri/Cargo.toml`, `Cargo.lock`.
- **Tamaño:** M.

#### T02: Completar primitivas y envoltorio de operación
- **Resultado:** introducir solo los IDs/contexto/versiones necesarios y extender el modelo de operación con `dependsOn`, serialización contractual y huella semántica estable.
- **Aceptación:** UUIDv7 canónicos y errores estables; round-trip camelCase conforme a ejemplos; misma intención produce la misma huella y cualquier cambio semántico la altera.
- **Verificación:** pruebas unitarias de dominio/serialización; `cargo test --workspace --locked`; `cargo clippy --workspace --all-targets --locked -- -D warnings`.
- **Archivos probables:** `shared_kernel/{mod.rs,id.rs,error.rs}`, `components/sync/{domain.rs,application.rs}` y pruebas en esos módulos.
- **Tamaño:** M.

#### T03: Persistir identidad y control de sincronización
- **Resultado:** aplicar `0003` y `0004`, evolucionar el Outbox existente sin perder operaciones y añadir repositorios para acceso, cursor, Inbox y proyección remota.
- **Aceptación:** migración desde `0002` conserva Outbox; reapertura no repite cambios; rollback deja archivo/historial intactos; no existen columnas de tokens.
- **Verificación:** pruebas temporales de migración/repositorios; `cargo test --workspace --locked`; inspección del esquema resultante.
- **Archivos probables:** dos migraciones nuevas, `platform/sqlite/mod.rs`, `components/identity/`, `components/sync/adapters/outbound.rs`.
- **Tamaño:** M.

### C01: Contratos congelados
- [ ] `pnpm quality` pasa y las bases `0002` migran sin pérdida.
- [ ] El usuario revisa estados, tablas, puertos y dependencias antes de abrir la fase de acceso.
- [ ] Quedan reservados `0003`/`0004` y los puntos de integración.

### Fase 2: Identidad, secretos y dispositivo

#### T04: Validar configuración y proteger credenciales
- **Resultado:** cargar URL de API/OIDC y parámetros por entorno con valores seguros por defecto; implementar el almacén seguro del SO y un fake para pruebas.
- **Aceptación:** configuración inválida falla antes de sincronizar; tokens solo se escriben mediante el puerto seguro; borrado/rotación no deja copias en SQLite ni logs.
- **Verificación:** pruebas de precedencia y almacén fake; prueba real del almacén del SO con credencial ficticia; `pnpm quality`.
- **Archivos probables:** `platform/config/`, `platform/secrets/`, `src-tauri/Cargo.toml`, `docs/operations/development.md`, ejemplo no sensible de configuración.
- **Tamaño:** M.

#### T05: Crear transporte HTTPS tipado para API v1
- **Resultado:** cliente Rust con Bearer, timeouts, límites, correlación, errores estructurados y DTO solo para rutas consumidas.
- **Aceptación:** códigos HTTP/de operación se traducen sin depender de `message`; URL y credenciales no aparecen en errores; ninguna llamada retiene el mutex SQLite.
- **Verificación:** pruebas contra servidor local de contrato para respuestas 200/401/403/410/413/422/429/503; `pnpm quality`.
- **Archivos probables:** `platform/http/`, `components/sync/adapters/outbound/api.rs`, `components/identity/adapters/outbound/api.rs`.
- **Tamaño:** M.

#### T06: Implementar acceso OIDC con PKCE
- **Resultado:** usar navegador externo, Authorization Code, PKCE S256, state/nonce y redirección local; canjear/renovar/cerrar sesión sin exponer tokens a React.
- **Aceptación:** state/nonce inválidos se rechazan; cancelar navegador conserva el estado anterior; refresh y logout no revocan por error el token de dispositivo.
- **Verificación:** pruebas con proveedor OIDC ficticio y navegador externo; flujo manual con cuenta de desarrollo; `pnpm quality`.
- **Archivos probables:** `components/identity/{domain.rs,application.rs}`, adaptadores OIDC/loopback, `src-tauri/src/lib.rs`.
- **Tamaño:** M.

#### T07: Activar instalación y autorización de cuenta
- **Resultado:** canjear el access token OIDC por token opaco de API, consultar `/v1/me`, cuentas y `/access`, y ejecutar `RegisterAccountDevice` solo cuando corresponda.
- **Aceptación:** token opaco se persiste una única vez de forma segura; `registration_required` conduce al registro propio; acceso revocado detiene intercambio sin borrar Outbox.
- **Verificación:** pruebas de estados de alta/acceso/revocación; activación manual sobre API de desarrollo; `pnpm quality`.
- **Archivos probables:** `components/devices/`, `components/identity/adapters/outbound/`, `components/sync/application.rs`, composición/IPC de acceso.
- **Tamaño:** M.

### C02: Instalación autorizada
- [ ] Transporte, acceso OIDC/PKCE y activación funcionan con datos de prueba.
- [ ] No hay secretos en archivos, IPC, SQLite o registros.
- [ ] El usuario revisa el recorrido de alta, logout y revocación.

### Fase 3: Envío y reconciliación de operaciones

#### T08: Despachar Outbox con dependencias y reintentos
- **Resultado:** seleccionar lotes causales válidos, marcar intentos de forma durable y reintentar con backoff/jitter sin alterar contenido ni ID.
- **Aceptación:** `dependsOn` respeta orden/máximo ocho; una operación independiente fallida no bloquea otras; reinicio durante envío recupera el mismo lote sin duplicar intención.
- **Verificación:** pruebas de planificación, leases expirados, límites y reloj controlado; prueba de reinicio; `pnpm quality`.
- **Archivos probables:** `components/sync/{domain.rs,application.rs}`, `adapters/outbound.rs`, `platform/sqlite/mod.rs`.
- **Tamaño:** M.

#### T09: Aplicar resultados y recuperar respuestas perdidas
- **Resultado:** reconciliar `accepted`, `duplicate`, `conflict`, `rejected`, `retryable_failure` y resultado desconocido mediante `GET /sync/operations/{id}`.
- **Aceptación:** duplicado no repite efecto; conflicto/rechazo conserva intención y estado remoto; 404/`RESULT_EXPIRED` nunca genera otro ID automáticamente.
- **Verificación:** matriz unitaria/de adaptador incluida pérdida después del commit; `pnpm quality`.
- **Archivos probables:** `components/sync/{domain.rs,application.rs}`, `adapters/{inbound.rs,outbound.rs}`, `src/shared/api/sync.ts` solo si cambia el DTO.
- **Tamaño:** M.

### C03: Outbound fiable
- [ ] Una misma operación sobrevive timeout, reinicio y duplicado sin doble efecto.
- [ ] Conflictos y rechazos quedan disponibles para revisión.
- [ ] El usuario revisa la demostración y los estados visibles.

### Fase 4: Entrada, cursor y recuperación

#### T10: Aplicar grupos remotos de forma idempotente
- **Resultado:** guardar proyección aceptada e Inbox por operación/grupo, con traductores tipados y notificación a proyectores sin mezclar intenciones locales.
- **Aceptación:** repetir grupo no cambia estado; grupo multientidad es atómico; claves compuestas, versiones de oferta y `remove` se conservan correctamente.
- **Verificación:** pruebas con fixtures del contrato y fallo del proyector; `pnpm quality`.
- **Archivos probables:** `components/sync/{domain.rs,application.rs}`, `adapters/outbound.rs`, puerto `RemoteProjector` y proyector de prueba.
- **Tamaño:** M.

#### T11: Descargar cambios incrementales
- **Resultado:** consumir `/sync/changes` hasta agotar `hasMore`, confirmando página y `nextCursor` juntos.
- **Aceptación:** grupos vacíos avanzan; fallo conserva cursor anterior; salida de ventana retira vista remota sin borrar intenciones; permiso/época inválido exige recuperación.
- **Verificación:** pruebas multipágina, duplicadas, vacías, remove, cursor expirado y rollback; `pnpm quality`.
- **Archivos probables:** `components/sync/application.rs`, `adapters/outbound/{api.rs,mod.rs}`, repositorio de cursor/proyección.
- **Tamaño:** M.

#### T12: Reconstruir desde capturas coherentes
- **Resultado:** crear/descargar capturas por páginas, preparar reemplazo en staging e instalar vista y `baseCursor` en una transacción; liberar captura al finalizar.
- **Aceptación:** no se mezclan capturas; vencimiento/cambio de alcance reinicia explícitamente; Outbox y conflictos sobreviven al reemplazo; reintento es idempotente.
- **Verificación:** pruebas de captura multipágina, interrupción antes/después de instalar y expiración; `pnpm quality`.
- **Archivos probables:** `components/sync/{application.rs,domain.rs}`, adaptadores API/SQLite, migración solo si una carencia probada lo exige.
- **Tamaño:** M.

### C04: Entrada y recuperación fiables
- [ ] Cursor y captura son atómicos, repetibles y conservan trabajo local.
- [ ] Un perfil local vacío reconstruye la proyección remota sintética y continúa incrementalmente.
- [ ] El usuario revisa recuperación, retiradas de vista y conflictos.

### Fase 5: Coordinador e IPC

#### T13: Orquestar el ciclo de sincronización
- **Resultado:** coordinador single-flight para arranque, reconexión y ejecución manual: refrescar acceso, recuperar desconocidos, enviar, descargar y publicar estado.
- **Aceptación:** ejecuciones simultáneas comparten un solo ciclo; caída de API no bloquea trabajo local; backoff no impide reintento manual y el estado sobrevive reinicio.
- **Verificación:** pruebas de concurrencia/estado con reloj y puertos falsos; ciclo real con API de desarrollo; `pnpm quality`.
- **Archivos probables:** `components/sync/application.rs`, `components/sync/coordinator.rs`, `src-tauri/src/lib.rs`.
- **Tamaño:** M.

#### T14: Incorporar avisos SSE sin convertirlos en autoridad
- **Resultado:** escuchar `/sync/events`, despertar descarga con el último cursor aplicado, tratar heartbeat/revoke y reconectar con backoff.
- **Aceptación:** `changes_available` no avanza cursor por sí solo; ID SSE no sustituye cursor; `access_revoked` detiene acceso; conexiones duplicadas se evitan.
- **Verificación:** stream ficticio con heartbeat, corte, eventos repetidos, 429 y revoke; prueba con API; `pnpm quality`.
- **Archivos probables:** `components/sync/adapters/outbound/sse.rs`, `components/sync/coordinator.rs`, `src-tauri/src/lib.rs`.
- **Tamaño:** M.

#### T15: Exponer acceso y estado mediante IPC específico
- **Resultado:** comandos para iniciar/completar acceso, cerrar sesión, sincronizar y consultar estado/conflictos; cliente TypeScript y diagnóstico mínimo, sin pantallas de negocio.
- **Aceptación:** DTO camelCase/errores estables y permisos explícitos; UI diferencia offline/pendiente/sincronizando/conflicto/fallo; nunca muestra token, payload sensible o SQL.
- **Verificación:** pruebas Rust de DTO/permisos y `pnpm build`; recorrido manual acceso → sincronización → offline → reconexión.
- **Archivos probables:** `components/identity/adapters/inbound.rs`, `components/sync/adapters/inbound.rs`, `src/shared/api/{identity,sync}.ts`, `src/App.tsx`, `capabilities/default.json`/`build.rs` mediante integración.
- **Tamaño:** M.

### C05: Motor utilizable desde el escritorio
- [ ] El recorrido manual funciona de extremo a extremo con datos sintéticos.
- [ ] La UI comunica resultado sin afirmar aceptación remota prematura.
- [ ] El usuario revisa el motor antes del endurecimiento final.

### Fase 6: Pruebas compartidas, operación y entrega al equipo

#### T16: Fijar pruebas de contrato del consumidor
- **Resultado:** servidor/fixtures locales para las rutas usadas que validen nombres, ausencia frente a nulo, campos extra, acciones, errores, captura, cambios y SSE contra los artefactos fijos.
- **Aceptación:** una deriva de contrato falla de forma legible; no se editan los JSON importados; las pruebas no requieren red externa ni credenciales.
- **Verificación:** `cargo test --workspace --locked`; comprobación de manifiesto/hashes del contrato; `pnpm quality`.
- **Archivos probables:** módulos `#[cfg(test)]` de adaptadores y fixtures sintéticos bajo `src-tauri/`, sin nuevo servidor de producto.
- **Tamaño:** M.

#### T17: Ejecutar la matriz de fallos con dos perfiles
- **Resultado:** arnés de integración con dos directorios de datos aislados para alta, envío, descarga y recuperación, incluida una operación sintética válida usada solo como sonda del motor.
- **Aceptación:** cubre offline, duplicado, respuesta perdida, dos dispositivos, versión en conflicto, revocación, cursor/captura vencidos, SSE caído y reinicio durante escritura; ninguna pérdida silenciosa.
- **Verificación:** pruebas automatizadas locales y repetición contra Nightingale API de desarrollo; resultados y límites registrados por separado.
- **Archivos probables:** pruebas de `components/sync/`, `platform/sqlite/`, fixtures de contrato y `docs/operations/development.md`.
- **Tamaño:** M.

#### T18: Completar observabilidad y diagnóstico seguro
- **Resultado:** logs estructurados con correlación/causación, duración y resultados; estado de Outbox, cursor y última sincronización sin datos personales.
- **Aceptación:** una operación puede rastrearse por ID/correlación; pruebas de redacción rechazan tokens/payloads; fallo de telemetría no afecta el motor.
- **Verificación:** pruebas de formato/redacción; inspección manual de logs con errores reales de desarrollo; `pnpm quality`.
- **Archivos probables:** `platform/observability/`, `components/diagnostics/`, `components/sync/coordinator.rs`.
- **Tamaño:** M.

#### T19: Formalizar la guía de colaboración y los siguientes tracks
- **Resultado:** README breve por componente core, guía para añadir una operación/proyector/comando/migración y contratos iniciales de los cuatro tracks siguientes.
- **Aceptación:** un colaborador puede implementar un caso sin editar kernel ni tablas ajenas; dependencias y propietarios quedan claros; guía y código coinciden.
- **Verificación:** recorrido documental con un ejemplo sintético, enlaces y `git diff --check`; revisión del usuario.
- **Archivos probables:** README de componentes core, `CONTRIBUTING.md`, `docs/operations/development.md`, `tasks/README.md` solo si la convención existente es insuficiente.
- **Tamaño:** M.

### C06: Base core aceptada para desarrollo colaborativo
- [ ] `pnpm quality`, `pnpm build:desktop` y `git diff --check` pasan sobre la entrega final.
- [ ] La matriz local y la verificación con API/OIDC de desarrollo tienen evidencia; cualquier entorno no disponible queda explícitamente pendiente y impide declarar esa verificación, no se sustituye por una suposición.
- [ ] Dos perfiles sincronizan datos sintéticos sin duplicados ni pérdida de intenciones.
- [ ] El usuario revisa la entrega y la división de los siguientes tracks.

## Características inmediatamente posteriores

Estos tracks **no se implementan** en este plan. Sus contratos públicos se preparan en T19 para comenzar sin rediseñar el motor.

| Orden | Track | Consume del core | Datos/proyección propios | Dependencia |
|---|---|---|---|---|
| 1 | Pacientes y contactos | actor/permiso, Outbox, acciones, snapshot/changes, conflictos | fichas, documentos, contactos y vínculos por cuenta | Core; puede compartir fase con Servicios tras fijar contratos |
| 1 | Cuenta y catálogo de servicios | acceso propietario/asistente, operaciones online/offline y proyección | preferencias, servicios y ofertas | Core; contratos antes de Agenda |
| 2 | Agenda y disponibilidad | IDs/tiempo/versiones, cambios remotos y Outbox | reglas, excepciones, bloqueos y disponibilidad derivada | Catálogo; coordina con Pacientes sin leer sus tablas |
| 3 | Citas | todos los anteriores, dependencias causales, versiones y conflictos | citas, historia y vista local de hoy/mañana | Pacientes + Servicios + Agenda |
| Posterior | Patient Hub y recordatorios | aceptación remota y vistas versionadas | adaptadores específicos, sin duplicar autoridad | Plan separado de etapa 2 |
| Posterior | Lista de espera y analítica | eventos/estado aceptado | modelos propios | Plan separado de etapa 3 |

Regla de paralelismo: antes de abrir tracks simultáneos se congelan DTO, acciones, propiedad de tablas, proyectores, comandos IPC y reservas de migración. Un integrador conecta los puntos calientes; cada colaborador trabaja dentro de su componente.

## Verificación final de la entrega

1. **Sin servicios:** guardado y reinicio offline, última autorización válida, Outbox/cursor persistentes, ausencia total de HTTP en casos locales.
2. **Contrato local:** pruebas de dominio, aplicación, SQLite, HTTP/SSE y OIDC con puertos falsos; `pnpm quality` completo.
3. **Escritorio real:** `pnpm build:desktop` y recorrido Tauri con acceso, sincronización manual, errores y reconexión.
4. **API de desarrollo:** dos instalaciones aisladas; operación sintética A visible en B; mismo ID tras respuesta perdida; conflicto de versión; revocación; recuperación por captura; caída SSE.
5. **Seguridad:** búsqueda/revisión de secretos en base, IPC, logs y Git; solo datos ficticios.
6. **Colaboración:** lectura de la guía y comprobación de que cada siguiente track tiene dueño, puertos, datos, offline, conflictos y punto de integración definidos.

## Riesgos y mitigaciones

| Riesgo | Impacto | Mitigación |
|---|---|---|
| Convertir el core en infraestructura genérica sobredimensionada | Alto | Proyección remota interna, puertos específicos y cuatro tracks concretos como única justificación; sin bus/repositorio universal. |
| Duplicar efectos tras timeout o reinicio | Alto | Contenido inmutable, huella, recuperación por ID y pruebas de pérdida posterior al commit. |
| Mezclar estado remoto con intenciones locales | Alto | Tablas y transacciones separadas; capturas/conflictos nunca borran Outbox. |
| Filtrar tokens o datos personales | Alto | Secret store del SO, IPC mínimo, redacción probada y datos sintéticos. |
| Elegir una librería incompatible con Tauri/plataformas | Medio | Selección docs-first en T01/T04, prueba Windows y diseño sin filtrar la librería al dominio. |
| Bloquear al equipo por falta de API/OIDC de desarrollo | Alto | Contratos y fakes para todo el trabajo local; credenciales/entorno se preparan antes de C02/C06; no se afirma verificación externa ausente. |
| Conflictos Git en composición y migraciones | Medio | Reservas en planes, contratos previos y un integrador para puntos calientes. |
| Declarar el core «seguro para producción» prematuramente | Alto | Alcance explícito de desarrollo; cifrado local y publicación quedan como planes obligatorios posteriores. |

## Supuestos y preguntas abiertas

- El usuario aprobó explícitamente el alcance **motor transversal**, sin primera vertical de negocio.
- Nightingale API y su configuración OIDC de desarrollo estarán disponibles para las verificaciones externas; las URLs/credenciales se suministrarán fuera de Git.
- No se cambia el contrato API v1 ni se edita su copia generada. Cualquier incompatibilidad real se registra y se resuelve en el repositorio propietario antes de continuar.
- No se reabre el plan completado `P-20261029-02`; este plan lo continúa.
- Las versiones exactas de nuevas librerías se fijan en T01/T04 mediante documentación oficial; no se presupone una API no verificada.
- Cifrado de SQLite, soporte de instaladores, CI y publicación no forman parte de este plan.
- No hay preguntas funcionales abiertas para aprobar este plan. Al aprobarlo, se materializará como `tasks/P-20261029-03-core-engine/plan.md` y `tasks/P-20261029-03-core-engine/todo.md`, manteniendo IDs, orden y checkpoints.
