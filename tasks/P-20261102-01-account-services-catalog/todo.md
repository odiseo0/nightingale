# P-20261102-01-account-services-catalog — Tareas

**Plan:** [plan.md](plan.md)
**Status:** In progress — C02 presentado; ejecución detenida por solicitud del usuario
**Updated:** 2026-11-03

## T01: Cerrar los contratos del track

**Status:** Done — contratos aprobados en C01.

- [x] DTO por comando (camelCase) para los 14 comandos: 6 de cuenta y 8 de catálogo. Recuento corregido con autorización del usuario.
- [x] Tabla de las 10 acciones: acción, entityType, clave UUIDv7/compuesta, versión esperada, modo y modelo/payload del bundle. Congelación efectiva tras aprobar C01.
- [x] Propiedad y claves de las siete tablas; reservas 0005_account y 0006_services. service_local incluye las intenciones de perfil profesional sin añadir tablas.
- [x] Firmas públicas de AccountViewQuery y CatalogQuery; solo datos aceptados, sin SQL ni transporte API.
- [x] Integración asignada al ejecutor: lib.rs, build.rs, capabilities/default.json, components/mod.rs, platform/sqlite/mod.rs (autorizado) y ambos proyectores vacíos de identity/desktop.rs.

**Verificación:** contraste con el código vigente, capabilities y esquemas del bundle; comprobación de hashes, recuento de comandos, enlaces locales y git diff --check. Evidencia en plan.md §T01. Sin pruebas de producto en esta tarea documental.

**Resultado 2026-11-02:** `python -` (comprobación por stdin) pasó hashes 5/5, acciones/modelos 10/10, separación 7 offline / 3 online-only, override obligatorio, DTO 14/14, enlaces y reservas. `git diff --check`: código 0. `git diff --name-only`: solo plan.md y todo.md. Pendientes: revisión C01, T02–T13 y API/OIDC reales.

## C01: Contratos congelados

- [x] Revisión del usuario de DTO, acciones, tablas, puertos y puntos de integración: «Me parece bien, continua» (2026-11-02).
- [x] Sin implementación hasta esta aprobación. Contratos T01 congelados.

## T02: Esqueletos de componentes y migraciones

**Status:** Done

- [x] Crear `components/account/{domain,application,adapters/{inbound,outbound}}` + `mod.rs` y guía breve en docs/technical/components/account.md (responsabilidad, datos, API, puertos, invariantes; ubicación corregida durante revisión C02).
- [x] Crear `components/services/{domain,application,adapters/{inbound,outbound}}` + `mod.rs` y guía breve en docs/technical/components/services.md (ubicación corregida durante revisión C02).
- [x] `src-tauri/migrations/0005_account.sql` con `PRAGMA user_version = 5` y `schema_migrations`.
- [x] `src-tauri/migrations/0006_services.sql` con `PRAGMA user_version = 6` y `schema_migrations`.
- [x] Verificar apertura de base nueva y migración desde esquema 4 con pruebas del runner.
- [x] Registrar ambos módulos en `components/mod.rs`.

**Verificación T02:** `cargo fmt --all`; `cargo test --workspace --locked platform::sqlite::tests`: 17 pruebas correctas (incluye base nueva, esquema 4, reapertura, rollback 0006 y conservación de intenciones). `git diff --check`: código 0. rusqlite 0.40.2, SQLite incluida 3.53.2. Sin dependencias nuevas. Fuentes: https://docs.rs/rusqlite/0.40.2/rusqlite/struct.Connection.html#method.execute_batch y https://www.sqlite.org/lang_createtable.html#check_constraints.

## T03: Dominio y aplicación de `account`

**Status:** Done

- [x] Entidades/objetos de valor: `AccountView`, `AccountPolicy`, `AccountDetailsPatch`, `PreferenceKey`; validaciones (patch no vacío, `noShowGraceMinutes` 0–1440, zona IANA, país ISO alfa-2).
- [x] Casos `get_account_view`, `get_preferences`, `set_preference` (transacción local, sin Outbox).
- [x] Política: preferencia local no puede fijar política central obligatoria.
- [x] Pruebas de dominio sin base ni red; pruebas de aplicación con puertos falsos.

**Verificación T03:** `cargo fmt --all`; `cargo test --workspace --locked components::account`: 5 pruebas correctas de dominio/aplicación, sin base ni red. Validación por pertenencia a listas IANA tzdb 2026f (598 zonas/enlaces y 249 países), con origen y hash en [docs/technical/reference-data.md](../../docs/technical/reference-data.md) (traslado solicitado el 2026-11-03); sin dependencia nueva. Serde 1.0.229 para errores camelCase y campos opcionales: https://serde.rs/container-attrs.html#rename_all y https://serde.rs/field-attrs.html#skip_serializing_if.

## T04: Repositorios y proyector de `account`

**Status:** Done

- [x] `AccountProjectionRepository` (SQLite) para `account_view` escritura por proyector.
- [x] `AccountReadRepository`: lectura de vista + preferencias fusionadas.
- [x] `AccountProjector` implementa `RemoteProjector` para `EntityType::Account`; `reset` retira solo su ámbito; ignora otros tipos.
- [x] Pruebas: proyección idempotente, `remove` retira vista, `reset` por captura, aislamiento por cuenta.

**Verificación T04:** `cargo fmt --all`; `cargo test --workspace --locked components::account`: 8 pruebas correctas. Se cubren proyección repetida/vieja, remove, reset/reconstrucción, aislamiento, preferencias sin Outbox y rollback. Fuentes rusqlite 0.40.2: https://docs.rs/rusqlite/0.40.2/rusqlite/struct.Transaction.html y https://www.sqlite.org/lang_upsert.html.

## T05: Casos online-only de `account`

**Status:** Done

- [x] `update_account_details`, `update_account_policy`, `set_account_status`: exigen sesión + acceso `active` + rol propietario; refresco `/access` como prueba de conectividad; rechazo sin persistir si falla.
- [x] Envío directo con `SyncAPI::send_operations` y envelope en memoria (ID estable por invocación entre reintentos de transporte); tratamiento de `accepted` (disparar `sync_now`), `duplicate` (`originalResult`), `conflict`/`rejected`, y resultado desconocido.
- [x] Nada escrito en SQLite; documentar la semántica de repetición manual en la guía técnica de cuenta.
- [x] Pruebas con servidor HTTP sintético loopback: éxito, 401/403 (denegación local), timeout con reintento mismo ID, rechazo visible.

**Verificación T05:** `cargo fmt --all`; `cargo test --workspace --locked components::account`: 12 pruebas correctas al permitir HTTP loopback. La primera ejecución aislada falló al conectar, sin resultados acreditados para esos cuatro tests; la repetición autorizada pasó. Se verifica timeout con mismo envelope/ID, las tres acciones, 401/403, duplicado con conflicto/rechazo y ausencia de escritura local. El adaptador de escritorio llama sync_now solo tras accepted y conserva accepted si falla el refresco. Reutiliza la puerta de sesión y credential de identity (solo visibilidad crate, sin cambiar reglas sync).

## T06: IPC y UI mínima de `account`

**Status:** Done

**Alcance adicional autorizado:** el usuario permitió añadir src/App.tsx para montar features/settings (2026-11-02).

- [x] Comandos: `account_get_view`, `account_get_preferences`, `account_set_preference`, `account_update_details`, `account_update_policy`, `account_set_status`; permisos en `capabilities/default.json` y `build.rs`; registro en `lib.rs`.
- [x] `src/shared/api/account.ts` como única frontera TS; DTO sin tokens ni estado remoto crudo.
- [x] `features/settings`: vista, formularios (propietario), aviso «requiere conexión», preferencias.
- [x] Pruebas de permisos/redacción IPC (patrón T14/T15 del motor).

**Verificación T06:** `pnpm build`: correcto; `pnpm quality`: correcto tras corregir un aviso de Clippy (as_chunks) y dos expectativas heredadas del esquema 4. Resultado final de cargo test --workspace --locked dentro de quality: 135 unitarias, 5 de integración y 1 doctest correctos; 6 manuales ignoradas. Las 15 pruebas de account incluyen permisos IPC, tipos de entrada y errores sin datos remotos. `pnpm build:desktop`: correcto, ejecutable target/debug/nightingale.exe. `git diff --check`: código 0. Se conservan DTO C01 sin expectedAccountId por decisión del usuario.

## C02: Revisión vertical `account`

**Presentado:** 2026-11-02. Detención solicitada por el usuario: «Detente en el siguiente checkpoint». No se inicia T07.

- [ ] El usuario revisa el recorrido completo con datos sintéticos (con y sin conexión simulada).

**Tareas completadas:** T01–T06. Componentes y migraciones 0005/0006 registrados; cuenta con dominio, puertos, repositorio, proyector, tres cambios online-only, seis comandos y panel mínimo de settings.

**Comandos y resultados finales:**

| Comando | Resultado |
|---|---|
| `cargo test --workspace --locked platform::sqlite::tests` | 17 correctas en T02: base nueva, migración desde 4, reapertura, rollback, aislamiento y conservación de intenciones. |
| `cargo test --workspace --locked components::account` | 12 correctas en T05; la suite final de quality incluye las 15 de account tras T06. HTTP sintético loopback autorizado; la ejecución aislada sin HTTP no se acredita. |
| `pnpm build` | TypeScript y Vite correctos. |
| `pnpm quality` | Build, formato, Clippy con -D warnings y suite workspace correctos: 135 unitarias + 5 integración + 1 doctest; 6 manuales ignoradas. |
| `pnpm build:desktop` | Correcto; target/debug/nightingale.exe, build debug sin instalador. |
| `git diff --check` | Correcto. |
| Comprobación Python por stdin | Hashes 5/5 del bundle intactos, enlaces README válidos, 598 nombres IANA y 249 países. Sin cambios de dependencias/lockfiles, migraciones 0001–0004, artefactos API ni reglas sync. |

**Decisiones dentro del diseño:** preferencias con transacción local sin Outbox; datos IANA incluidos sin una dependencia nueva; envelope de cuenta solo en memoria con ID estable durante dos intentos; duplicate usa originalResult; sync_now solo tras accepted y fallo de descarga no convierte accepted en fallo de escritura. Se reutilizan puerta y credencial de identity con visibilidad crate. Se actualizan únicamente dos expectativas de pruebas existentes: identity/activation_tests.rs informa localSchemaVersion=6 y tests/core_persistence.rs cuenta seis migraciones. No cambian reglas de esos componentes. App.tsx se añadió al alcance con autorización.

**Fuentes de patrones:** rusqlite 0.40.2 (https://docs.rs/rusqlite/0.40.2/rusqlite/struct.Transaction.html), SQLite UPSERT (https://www.sqlite.org/lang_upsert.html), Serde 1.0.229 (https://serde.rs/container-attrs.html y https://serde.rs/field-attrs.html), Tauri/API JS 2.12.4 (https://v2.tauri.app/develop/calling-rust/#async-commands, https://v2.tauri.app/security/permissions/#app-manifest y https://v2.tauri.app/reference/javascript/api/namespacecore/#invoke), React 19.3.1 (https://react.dev/reference/react/useEffect#fetching-data-with-effects y https://react.dev/reference/react/useRef). Versiones obtenidas de lockfiles; el código oficial instalado de Tauri confirma el estado y runtime usados. No se añaden dependencias.

**Pendientes y límites:** revisión manual del usuario C02; T07–T13; composición y registro de proyectores en el ciclo y SSE de identity, reservados a T12 (siguen ambos &()). Las pruebas invocan AccountProjector con datos sintéticos; no se acredita un recorrido UI conectado de principio a fin ni API/OIDC de desarrollo. Siguen seis pruebas manuales ignoradas. El usuario rechazó expectedAccountId: el formulario previo no queda ligado atómicamente a la cuenta que mostraba; la sesión seleccionada en Rust al ejecutar determina el destino.


## T07: Dominio y aplicación de `services`

- [ ] Objeto de valor `DurationMinutes` (entero 1–1440); `ServicePatch`/`ServiceOfferPatch` no vacíos; `UpdateServiceOffer` exige `durationMinutesOverride` presente (admite `null`).
- [ ] 8 casos de uso: lectura con acceso local activo y 7 mutaciones con validación de acción vía `AccountAccess::allows_offline`; clave compuesta `{professionalId, serviceId}` validada como UUIDv7 canónicos.
- [ ] Cálculo de duración efectiva = override ?? default solo desde estado aceptado.
- [ ] Pruebas de dominio: rangos, patches, ausencia vs nulo, claves compuestas, edición de intención local previa (nueva versión de la intención, mismo ID de operación solo si el contenido no cambió — huella).

## T08: Repositorios e intenciones locales de `services`

- [ ] Escritura de `service_local`/`service_offer_local` + `record_operation` en la **misma** transacción `IMMEDIATE` (rollback conjunto).
- [ ] Resolución de intenciones desde recibos (vía consulta de resultados por `operation_id`): `accepted` marca y retira provisional al confirmar vista; `conflict`/`rejected` quedan para revisión.
- [ ] Lectura fusionada aceptada + provisionales con marca de estado de sincronización.
- [ ] Pruebas: sin red, reapertura conserva, rollback, resolución por recibo.

## T09: Proyectores de `services`

- [ ] `ServicesProjector` para `Service`, `ServiceOffer`, `Professional` (misma estructura, ámbito compartido); ignora otros tipos.
- [ ] Regla de versiones dobles: `version` igual con `serviceVersion` mayor sí se proyecta; `version` menor no.
- [ ] `reset` retira solo sus tres vistas; `remove` retira vista sin tocar intenciones.
- [ ] Pruebas: idempotencia, grupos repetidos, remove, reset de captura, regla de versiones.

## T10: IPC y UI mínima de `services`

- [ ] Comandos: `services_list_catalog`, `services_create_service`, `services_update_service`, `services_set_service_status`, `services_offer_service`, `services_update_service_offer`, `services_set_service_offer_status`, `services_update_professional_profile`; permisos, `build.rs`, `lib.rs`.
- [ ] `src/shared/api/services.ts`; UI `features/services` con marca de provisional y estados de sincronización; solo lectura para asistentes.
- [ ] Revisión de conflictos enlazada al `ReviewSummary` existente.
- [ ] Pruebas de permisos/redacción IPC.

## C03: Revisión vertical `services`

- [ ] El usuario revisa: alta offline → sincronización → aceptado; conflicto visible; asistente sin escritura.

## T11: Pruebas de contrato del consumidor

- [ ] Fixtures contra el bundle fijo: hashes del manifiesto, presencia de las 10 acciones, correspondencia acción↔modelo según `capabilities.json`, claves compuestas.
- [ ] Los modelos de estado usados ya existen en `remote_models.rs`; verificar que no hay deriva respecto al bundle.
- [ ] Nunca editar artefactos de `docs/contracts/api-v1/`.

## T12: Integración y punta a punta

- [ ] Proyector compuesto `(AccountProjector, ServicesProjector)` registrado en `identity/desktop.rs` (sustituye `&()` en ciclo y SSE).
- [ ] Escenarios sintéticos con servidor loopback y dos perfiles donde aplique: crear servicio offline → dispatch → `accepted` → proyección; `VERSION_CONFLICT` conserva intención; `remove` retira vista; reapertura conserva; oferta con `serviceVersion` mayor proyecta; captura nueva reconstruye vistas conservando intenciones.
- [ ] Confirmar que casos online-only no dejan intenciones en Outbox.

## T13: Calidad, guías y cierre

- [ ] Guías finales de `account` y `services` en docs/technical/components (PMD §26/§27.2; ubicación solicitada por el usuario).
- [ ] `cargo test --workspace --locked`, `pnpm quality`, `pnpm build:desktop`, `git diff --check` en verde.
- [ ] Actualizar [core-next-tracks.md](../../docs/operations/core-next-tracks.md) si el recorrido real difiere de la referencia (solo con aprobación del usuario).

## C04: Aceptación del track

- [ ] Revisión del usuario de la evidencia local y decisión de aprobación.
- [ ] Pendientes externos (API/OIDC de desarrollo) quedan explícitos, no acreditados.

## Ajuste de organización durante revisión C02 — 2026-11-03

- [x] Listas de países y zonas en shared_kernel/reference_data, con acceso puro compartido; account usa sus funciones sin duplicar las listas.
- [x] Guías de los componentes, núcleo y plataforma en docs/technical/components; origen y hash de las listas en docs/technical/reference-data.md.
- [x] Índice, enlaces y referencias del track actualizados. Formato del usuario en application.rs y adapters/outbound.rs preservado.
- [x] Comprobaciones del traslado y evidencia final: `cargo test --workspace --locked components::account` (15 correctas, con HTTP loopback); `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo fmt --all -- --check` y `git diff --check` correctos. Comparación de contenido contra HEAD (normalizando LF/CRLF): listas idénticas, 249 países y 598 zonas. Enlaces relativos válidos en las guías trasladadas, índice, guías operativas y track; ningún Markdown permanece en src-tauri/src. Hashes del bundle API intactos. Los cambios de formato del usuario siguen limitados a 23 inserciones/2 borrados en outbound.rs y una línea en application.rs, como al empezar. No se repiten build web/escritorio: no cambian TS, configuración ni recursos de UI.
- [ ] Aprobación C02 del usuario después de revisar el ajuste; T07–T10 hasta C03 requieren esa aprobación.
