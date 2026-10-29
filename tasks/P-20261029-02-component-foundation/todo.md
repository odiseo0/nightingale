# Task List: P-20261029-02-component-foundation

**Plan:** [plan.md](plan.md)
**Updated:** 2026-10-29

El usuario aprobó este plan el 2026-10-29 tras la revisión en modo plan. Una tarea a la vez; `pnpm quality` verde tras cada una. Antes de cada cambio de código, aplicar `minimal-code` tras trazar el flujo real; para APIs de bibliotecas, `docs-first-framework-code` con documentación oficial acorde a la versión instalada.

Migración reservada por este plan: `0002_sync_outbox.sql` (componente sync). Puntos calientes tocados: `src-tauri/src/lib.rs`, `src-tauri/Cargo.toml`, `src-tauri/capabilities/default.json` (declarados aquí por convención).

## T01: Reestructurar `src-tauri/src` al layout de componentes

**Status:** Done

**Evidence (2026-10-29):** `platform/sqlite/mod.rs` y `components/diagnostics/mod.rs` creados con `mod.rs` raíz; `lib.rs` compone y conserva el comando `get_diagnostics`. `cargo test --workspace --locked`: 7 passed; `cargo clippy --workspace --all-targets --locked -- -D warnings` limpio; `pnpm build` correcto. Nota: `pnpm build` estaba roto antes de esta tarea porque `index.html` fue eliminado en el commit `78c54fe` ("design prototype"); se restauró tal cual estaba en `632f565` para desbloquear la verificación (defecto preexistente, no causado por esta tarea).

**Description:** Crear `platform/sqlite/` y `components/diagnostics/`; mover `sqlite.rs` y `diagnostics.rs` (con sus tests intactos) a esos módulos; `lib.rs` queda como composición con la misma funcionalidad de antes. Sin carpetas vacías ni cambios de comportamiento.

**Acceptance criteria:**
- [ ] `src-tauri/src` contiene `platform/sqlite/mod.rs` y `components/diagnostics/mod.rs`; los módulos raíz los exponen.
- [ ] `lib.rs` compone módulos y registra el mismo comando `get_diagnostics` de antes.
- [ ] Los tests existentes de SQLite pasan sin cambios de comportamiento.

**Verification:**
- [ ] Focused tests: `cargo test --workspace --locked`
- [ ] Build or broader check: `cargo clippy --workspace --all-targets --locked -- -D warnings`; `pnpm build`
- [ ] Manual check: `pnpm tauri dev` abre y el diagnóstico muestra igual que antes.

**Dependencies:** None

**Files likely touched:**
- `src-tauri/src/lib.rs`
- `src-tauri/src/platform/sqlite/mod.rs` (movido desde `src-tauri/src/sqlite.rs`)
- `src-tauri/src/components/diagnostics/mod.rs` (movido desde `src-tauri/src/diagnostics.rs`)
- `src-tauri/src/platform/mod.rs`, `src-tauri/src/components/mod.rs` (nuevos)

**Estimated scope:** S

## T02: Kernel compartido mínimo (`shared_kernel/`)

**Status:** Done

**Evidence (2026-10-29):** `shared_kernel/` con `Clock`/`SystemClock`, `IdGenerator`/`UuidV7Generator` (UUIDv7 vía `uuid` feature `v7`; patrón verificado contra documentación oficial: https://docs.rs/uuid/latest/uuid/struct.Uuid.html#method.now_v7 — garantiza orden de creación por proceso), `NightingaleError` con códigos estables serializables a `{code,message}` camelCase y `Result<T>`. `Cargo.toml` añade `uuid` (única dependencia nueva) y `Cargo.lock` actualizado. El módulo se expone como `pub` en el lib para que los tracks paralelos lo consuman. `cargo test --workspace --locked`: 12 passed; `cargo clippy --workspace --all-targets --locked -- -D warnings` limpio. Nota: el usuario indicó omitir `pnpm quality` mientras no haya trabajo de UI.

**Description:** Crear `shared_kernel/` con puertos `Clock` (adaptador `SystemClock`), `IdGenerator` (adaptador UUIDv7 vía crate `uuid` con feature `v7`), `NightingaleError` con códigos estables serializables a `{code,message}` camelCase, y `Result<T>` común. Añadir `uuid` a `src-tauri/Cargo.toml` (única dependencia nueva).

**Acceptance criteria:**
- [ ] `shared_kernel` expone `Clock`, `SystemClock`, `IdGenerator`, `UuidV7Generator`, `NightingaleError` (códigos `VALIDATION_ERROR`, `NOT_FOUND`, `VERSION_CONFLICT`, `TEMPORARY_FAILURE`, `INTERNAL_ERROR`) y `Result<T>`.
- [ ] `Cargo.toml` declara `uuid` con feature `v7` y `Cargo.lock` actualizado.
- [ ] Ningún otro módulo depende aún del kernel (se conecta en T04/T05).

**Verification:**
- [ ] Focused tests: `cargo test --workspace --locked` (tests unitarios de errores serializados y de UUIDv7 distintos/ordenables)
- [ ] Build or broader check: `cargo clippy --workspace --all-targets --locked -- -D warnings`
- [ ] Manual check: lectura del módulo confirma que no filtra dependencias de frameworks.

**Dependencies:** T01

**Files likely touched:**
- `src-tauri/src/shared_kernel/mod.rs`, `clock.rs`, `id.rs`, `error.rs`
- `src-tauri/src/lib.rs` (declarar módulo)
- `src-tauri/Cargo.toml`, `Cargo.lock`

**Estimated scope:** S

## T03: Base SQLite generalizada + convención de migraciones

**Status:** Done

**Evidence (2026-10-29):** runner multi-migración con lista ordenada embebida; validación estricta preservada y ampliada (rechaza historial con hueco y migración que olvida `user_version`); `in_transaction` con commit/rollback probado. Tests: 16 passed (incluye orden de aplicación, hueco, rollback y transacciones). `cargo clippy --workspace --all-targets --locked -- -D warnings` limpio (los módulos `platform` y `shared_kernel` se exponen `pub` en el lib para los tracks paralelos). Convención `NNNN_<componente>_<slug>.sql` y reserva por plan documentadas en `docs/operations/development.md`. Dos correcciones de borrow (`Drop` del fixture y SQL temporal en test) resueltas con lifetimes parametrizados en `Migration<'a>`.

**Description:** Generalizar el runner de migraciones a una lista ordenada de migraciones embebidas preservando las garantías estrictas actuales (historial exacto, versión futura rechazada, archivo intacto ante fallo, rollback en transacción); añadir helper `in_transaction` para cambio + Outbox atómicos; documentar la convención de numeración en `docs/operations/development.md`.

**Acceptance criteria:**
- [ ] `MIGRATIONS` lista versionada con `0001_schema`; abrir base nueva aplica en orden y abrir existente valida el historial.
- [ ] Base con `user_version` futura o historial con hueco se rechaza sin modificar el archivo.
- [ ] `in_transaction` aplica y confirma, y revierte ante error del closure.

**Verification:**
- [ ] Focused tests: `cargo test --workspace --locked` (tests del runner: orden, hueco, rollback, suite anterior verde)
- [ ] Build or broader check: `cargo clippy --workspace --all-targets --locked -- -D warnings`
- [ ] Manual check: `docs/operations/development.md` describe la convención `NNNN_<componente>_<slug>.sql` y la reserva por plan.

**Dependencies:** T01

**Files likely touched:**
- `src-tauri/src/platform/sqlite/mod.rs`
- `src-tauri/migrations/0001_schema.sql` (sin cambios de contenido)
- `docs/operations/development.md`

**Estimated scope:** M

## C01: Después de T01–T03

- [ ] `pnpm quality` completo en verde.
- [ ] Estructura inspeccionable y convenciones de migraciones e IPC aceptadas en revisión del usuario.

## T04: Convenciones IPC Rust↔Tauri demostradas

**Status:** Done

**Evidence (2026-10-29):** comando `storage_get_status` con `Result<Dto, NightingaleError>` y mapeo a `{code,message}` estable (tests de estado vacío y lock envenenado); permiso `allow-storage-get-status` generado declarando el comando en `build.rs` (los permisos de comando se autogeneran; `src-tauri/permissions/autogenerated/` está en `.gitignore`). Frontend: `src/shared/api/{diagnostics,storage}.ts` con tipos; `src/diagnostics.ts` eliminado; `App.tsx` muestra versión de esquema e identificador. Verificación: `cargo test --workspace --locked` 18 passed; `cargo clippy --workspace --all-targets --locked -- -D warnings` limpio; `pnpm exec tsc` OK (`pnpm build` omitido por indicación del usuario: sin `index.html` no hay entrada de Vite).

**Description:** Establecer y demostrar la convención IPC: comandos devuelven `Result<Dto, NightingaleError>` (mapeo a `{code,message}` estable), DTOs camelCase, permiso explícito por comando. Aplicarla al componente diagnostics y añadir `storage_get_status` (versión de esquema y `application_id`, sin exponer rutas). Frontend: cliente tipado en `src/shared/api/` reemplazando `src/diagnostics.ts`.

**Acceptance criteria:**
- [ ] `storage_get_status` devuelve `{schemaVersion, applicationId}` y errores con código estable.
- [ ] `capabilities/default.json` concede `allow-storage-get-status` y conserva `allow-get-diagnostics`.
- [ ] Frontend consume comandos solo vía `src/shared/api/` con tipos; `src/diagnostics.ts` desaparece.

**Verification:**
- [ ] Focused tests: `cargo test --workspace --locked` (mapeo de errores del comando)
- [ ] Build or broader check: `pnpm build`; `cargo clippy --workspace --all-targets --locked -- -D warnings`
- [ ] Manual check: en `pnpm tauri dev` se ven diagnóstico y estado de almacenamiento.

**Dependencies:** T02, T03

**Files likely touched:**
- `src-tauri/src/components/diagnostics/mod.rs`
- `src-tauri/src/components/storage/mod.rs` (nuevo)
- `src-tauri/src/lib.rs` (módulos + comando)
- `src-tauri/capabilities/default.json`
- `src/shared/api/diagnostics.ts`, `src/shared/api/storage.ts`
- `src/App.tsx` (ajustar import)

**Estimated scope:** M

## T05: Componente sync: Outbox local (slice sin nube)

**Status:** Done

**Evidence (2026-10-29):** migración `0002_sync_outbox.sql` (tabla + índice, registrada en `MIGRATIONS`); dominio con `NewOperation::validate` (action/entityType no vacíos, `expected_version >= 0`, `entity_key` UUID u objeto, `payload` objeto) y `OperationStatus`; aplicación `record_operation` que exige `&Transaction` (mismo test demuestra rollback del negocio con el outbox); adaptador SQLite `SqliteOperationStore`; comando `sync_list_pending` (DTO camelCase completo) con permiso autogenerado vía `build.rs`. Frontend: `src/shared/api/sync.ts` y contador visible en `App.tsx`. Tres tests SQLite históricos actualizados al mundo multi-migración (conteo dinámico, versión futura `len+1`, historial inválido acotado a version 1). Verificación: `cargo test --workspace --locked` 26 passed; `cargo clippy --workspace --all-targets --locked -- -D warnings` limpio; `pnpm exec tsc` OK; `cargo build --workspace --locked` y `cargo fmt --check` correctos. Comprobaciones manuales en `pnpm tauri dev` pendientes: requieren `index.html`, que el usuario retiró mientras no hay trabajo de UI.

**Description:** Crear `components/sync/` con dominio (`OperationRecord`, estados `pending/sent/failed/conflict`, invariantes), aplicación (`record_operation` dentro de la transacción del cambio), adaptador SQLite (`sync_outbox`, migración reservada `0002_sync_outbox.sql`), comando de solo lectura `sync_list_pending`, permiso y cliente frontend con contador visible en la pantalla de diagnóstico.

**Acceptance criteria:**
- [ ] Migración `0002_sync_outbox.sql` aplica sobre la base existente y el historial valida (`user_version=2`).
- [ ] `record_operation` exige transacción y persiste el registro; `sync_list_pending` lista operaciones `pending`.
- [ ] Comando, permiso `allow-sync-list-pending` y cliente frontend funcionan; el contador es visible y sobrevive reinicio.

**Verification:**
- [ ] Focused tests: `cargo test --workspace --locked` (repositorio: insert/list; dominio: invariantes)
- [ ] Build or broader check: `pnpm quality` completo
- [ ] Manual check: abrir la app, ver contador 0; registrar una operación de prueba solo si existe una vía de negocio (si no, contador 0 visible basta); reiniciar y conservar.

**Dependencies:** T04

**Files likely touched:**
- `src-tauri/migrations/0002_sync_outbox.sql`
- `src-tauri/src/platform/sqlite/mod.rs` (añadir a `MIGRATIONS`)
- `src-tauri/src/components/sync/{mod.rs, domain.rs, application.rs, adapters/outbound.rs}`
- `src-tauri/src/lib.rs` (módulo + comando)
- `src-tauri/capabilities/default.json`
- `src/shared/api/sync.ts`
- `src/App.tsx` (contador)

**Estimated scope:** M

## C02: Revisión final

- [ ] `pnpm quality` verde y `pnpm build:desktop` correcto.
- [ ] Decisión explícita: los planes paralelos pueden reservar migraciones 0003+ y consumir el kernel sin modificarlo.
- [ ] Revisión del usuario completada.
