# P-20261029-02-component-foundation: Esqueleto Rust para trabajo paralelo

**Status:** Completed
**Created:** 2026-10-29
**Updated:** 2026-10-29
**Task list:** [todo.md](todo.md)

## Overview

Crear lo mínimo para trabajar varios componentes en paralelo (varias personas, un repo, mínimos conflictos de Git): estructura de componentes hexagonales, kernel compartido congelado, base de almacenamiento SQLite generalizada con convención de numeración de migraciones, convenciones de comunicación Rust↔Tauri, y el primer componente real: el registro local de operaciones (Outbox) que guarda los cambios del usuario para que el plan posterior de sincronización cloud solo tenga que enviarlos.

El modelo de datos de negocio (pacientes, citas, agenda…) queda para planes posteriores; la tabla `sync_outbox` pertenece al componente sync y se define aquí.

El usuario aprobó este plan tras revisarlo el 2026-10-29 (modo plan). Se ejecutará una tarea a la vez con `pnpm quality` verde tras cada una.

## Source of Truth

- PMD §6–8 (arquitectura hexagonal, empaquetado por componentes, estructura del repositorio), §9.9–9.11 (sincronización, Outbox), §11 (kernel compartido), §12.6 (errores estables), §19.2 (comandos Tauri).
- Contrato API v1 (`docs/contracts/api-v1/`): envoltorio de operación (UUIDv7, `contractVersion`, `action`, `entityKey`, `expectedVersion`, `payload`, `occurredAtLocal`).
- Código actual: `src-tauri/src/{lib.rs, sqlite.rs, diagnostics.rs}`, `src-tauri/migrations/0001_schema.sql`, `src-tauri/capabilities/default.json`, `src/diagnostics.ts`.
- `docs/operations/development.md` (convención de migraciones a documentar).

## Architecture Decisions

- Layout: `src-tauri/src/` se reorganiza en `shared_kernel/`, `platform/sqlite/`, `components/<nombre>/`; `lib.rs` es la composición (módulos + `manage` + `generate_handler!`). Sin carpetas vacías ceremoniales.
- Kernel congelado: solo `Clock`, `IdGenerator`, `NightingaleError` (códigos estables) y `Result<T>`.
- Migraciones: runner multi-migración estricto; convención `NNNN_<componente>_<slug>.sql`; cada plan declara su número en `todo.md` (este plan: `0002_sync_outbox`).
- IPC: un comando Tauri por caso de uso, DTOs camelCase, errores `{code,message}` estables, permiso explícito por comando en `capabilities/default.json`.
- Outbox local: guarda exactamente lo que el contrato v1 exige enviar; el cambio de negocio y su registro van en la misma transacción SQLite. Sin envío a la nube.
- Única dependencia nueva: `uuid` (feature `v7`) para UUIDv7 canónicos del contrato.

## Dependency Order

T01 (reestructura) → T02 (kernel) y T03 (SQLite) en paralelo lógico pero secuenciales en ejecución → C01 → T04 (IPC) → T05 (Outbox) → C02.

## Task List

### Phase 1: Esqueleto y almacenamiento
- [x] T01: Reestructurar `src-tauri/src` al layout de componentes
- [x] T02: Kernel compartido mínimo (`shared_kernel/`)
- [x] T03: Base SQLite generalizada + convención de migraciones

### C01: Después de T01–T03
- [x] Controles Cargo completos en verde (`cargo test`, `cargo clippy -D warnings`)
- [x] Convenciones de migraciones e IPC aceptadas en revisión

### Phase 2: IPC y Outbox
- [x] T04: Convenciones IPC Rust↔Tauri demostradas
- [x] T05: Componente sync: Outbox local (slice sin nube)

### C02: Revisión final
- [x] `cargo test` (26), `cargo clippy -D warnings`, `cargo build --workspace --locked` y `cargo fmt --check` en verde; `pnpm exec tsc` OK
- [x] Decisión de que los planes paralelos pueden reservar migraciones 0003+
- [x] `pnpm quality`/`pnpm tauri dev` omitidos por indicación del usuario (sin `index.html`); comprobaciones manuales pendientes de recuperar la entrada de Vite
- [x] Revisión del usuario completada (aceptación el 2026-10-29)

## Risks and Mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| Reestructurar rompe la base existente del usuario | Alto | T01 no cambia comportamiento ni `application_id`; tests intactos |
| `lib.rs` y `Cargo.toml` como puntos calientes | Medio | Convención declarada; cada plan declara sus líneas antes de ejecutar |
| Outbox anticipa semántica del sync cloud | Medio | Solo se guarda lo que el contrato v1 exige; sin lógica de envío |

## Open Questions

- None.

## Convenciones para planes paralelos (registradas aquí)

- Migración: nombre `NNNN_<componente>_<slug>.sql`, número = siguiente libre, reservado en el `todo.md` del plan antes de ejecutar.
- Comando Tauri: `snake_case`, un caso de uso por comando, DTO `camelCase`, permiso `allow-<comando>` en `capabilities/default.json`, cliente tipado propio en `src/shared/api/<componente>.ts`.
- Kernel (`shared_kernel/`): congelado; modificarlo requiere justificación en el plan.
