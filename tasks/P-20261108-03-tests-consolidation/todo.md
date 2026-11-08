# Task List: P-20261108-03-tests-consolidation

**Plan:** [plan.md](plan.md)
**Status del plan:** Approved
**Created:** 2026-11-08
**Updated:** 2026-11-08

**Instrucción vigente:** el usuario aprobó el diseño en modo plan y ordenó registrarlo en `tasks/`. Refactor puro: mover tests de componentes a `src-tauri/src/tests/components/`, sin cambios de comportamiento.

**Responsable de integración:** quien ejecute el plan; una sola persona mueve archivos, limpia declaraciones y registra `lib.rs`.
**Orden:** secuencial, una tarea `In progress`; el checkpoint requiere revisión del usuario antes del cierre.
**Reglas comunes:** aplicar `minimal-code` antes de cada cambio; usar `git mv` para preservar historia; no tocar lógica, contratos, migraciones ni DTO; preservar trabajo ajeno.

## T01: Árbol nuevo y movimiento de archivos

**Status:** Done

**Description:** Crear `src-tauri/src/tests/` con sus `mod.rs`, mover con `git mv` todos los `*_tests.rs`, los cuatro `test_support.rs` y `sync/fixtures/` desde `src/components/<comp>/`.

**Acceptance criteria:**
- [x] Todos los archivos de test de componentes viven bajo `src/tests/components/<comp>/`.
- [x] `git status` muestra renombres (R), no borrados+creados.
- [x] Ningún `*_tests.rs` permanece en `src/components/`.

**Verification:**
- [x] `git status --short` y recuento de archivos movidos (account 2, appointments 17, identity 2, patients 13, scheduling 15, services 6, sync 14 con fixtures).

**Dependencies:** Ninguna.

**Files likely touched:**
- `src-tauri/src/tests/**` (nuevo)
- `src-tauri/src/components/**/*_tests.rs`, `test_support.rs`, `sync/fixtures/` (movidos)

**Estimated scope:** M

## T02: Cableado nuevo y limpieza del viejo

**Status:** Done

**Description:** Añadir `#[cfg(test)] mod tests;` a `lib.rs`, declarar los módulos en los `mod.rs` del árbol nuevo y quitar las declaraciones viejas (líneas `#[cfg(test)] mod *_tests;` y `#[path = "..."]`) de los `mod.rs` de componentes y de los archivos fuente.

**Acceptance criteria:**
- [x] El árbol nuevo compila como módulos bajo `cfg(test)`.
- [x] Ningún `components/<comp>/mod.rs` ni fuente declara tests locales.
- [x] Los tests inline en fuentes pequeñas (storage, platform, dominios) siguen intactos.

**Verification:**
- [x] `cargo check --workspace --all-targets --locked` (fallará en imports hasta T03; aquí solo se exige que la estructura de módulos resuelva).

**Dependencies:** T01.

**Files likely touched:**
- `src-tauri/src/lib.rs`
- `src-tauri/src/tests/mod.rs`, `src-tauri/src/tests/components/mod.rs`, `src-tauri/src/tests/components/*/mod.rs`
- `src-tauri/src/components/{account,appointments,identity,patients,scheduling,services,sync}/mod.rs`
- `src-tauri/src/components/**` (fuentes con `#[path]`)

**Estimated scope:** M

## T03: Reescritura de imports

**Status:** Done

**Description:** Reescribir `use super::…` en los archivos movidos: rutas `crate::components::<comp>::…` para lo que antes era el módulo del componente o del fuente anfitrión; conservar `super::test_support::…` (resuelve en el nuevo árbol); actualizar referencias cruzadas de sync a `crate::tests::components::<x>::test_support`.

**Acceptance criteria:**
- [x] Ningún `use super::` referencia un módulo de producto.
- [x] Los `super::test_support::…` se conservan tal cual.

**Verification:**
- [x] `cargo check --workspace --all-targets --locked` → los únicos errores restantes son de visibilidad privada (T04).

**Dependencies:** T02.

**Files likely touched:**
- `src-tauri/src/tests/components/**/*.rs`

**Estimated scope:** M

## T04: Visibilidades crate-internas

**Status:** Done

**Description:** Ampliar a `pub(crate)` los ítems privados que los tests movidos usaban vía `super::*`, guiado por `cargo check`. Sin cambios de firmas ni lógica.

**Acceptance criteria:**
- [x] `cargo check --workspace --all-targets --locked` sin errores.
- [x] Ningún ítem queda `pub` (solo `pub(crate)`); la API pública de componentes no cambia.

**Verification:**
- [x] `cargo check --workspace --all-targets --locked`.

**Dependencies:** T03.

**Files likely touched:**
- `src-tauri/src/components/**` (solo atributos de visibilidad)

**Estimated scope:** M

## T05: Rutas de `include_str!` / `include_bytes!`

**Status:** Done

**Description:** Corregir las rutas relativas de los includes en tests movidos: contrato API en `sync/contract_tests.rs`, PEM OIDC en `identity/oidc_tests.rs`.

**Acceptance criteria:**
- [x] `cargo test --workspace --locked components::sync::contract_tests` y `identity::oidc` (vía nuevos filtros `tests::components::…`) compilan y corren.

**Verification:**
- [x] `cargo test --workspace --locked tests::components::sync::contract_tests` y `tests::components::identity`.

**Dependencies:** T04.

**Files likely touched:**
- `src-tauri/src/tests/components/sync/contract_tests.rs`
- `src-tauri/src/tests/components/identity/oidc_tests.rs`

**Estimated scope:** S

## T06: Verificación completa

**Status:** Done

**Description:** Ejecutar la batería completa sobre la entrega y confirmar paridad de tests.

**Acceptance criteria:**
- [x] `cargo test --workspace --locked` → 381 passed, 0 failed, 9 ignored (8 preexistentes + 1 opt-in).
- [x] `cargo fmt --all -- --check` y `cargo clippy --workspace --all-targets --locked -- -D warnings` pasan.
- [x] `pnpm build:desktop` y `git diff --check` pasan.
- [x] `git status` sin artefactos ajenos.

**Verification:**
- [x] Comandos anteriores sobre la entrega completa.

**Dependencies:** T05.

**Files likely touched:**
- Ninguno (verificación).

**Estimated scope:** S

## T07: Documentación viva

**Status:** Done

**Description:** Actualizar `docs/operations/development.md` (filtros de `cargo test` y matriz de revisión) y las rutas de archivos de test citadas en `docs/technical/components/*.md`.

**Acceptance criteria:**
- [x] Los comandos documentados ejecutan con los nuevos filtros `tests::components::…`.
- [x] Las guías referencian las nuevas rutas de archivos.

**Verification:**
- [x] Ejecutar un comando documentado de ejemplo (`cargo test --workspace --locked tests::components::scheduling`).
- [x] `git diff --check`.

**Dependencies:** T06.

**Files likely touched:**
- `docs/operations/development.md`
- `docs/technical/components/*.md`

**Estimated scope:** S

## C01: Revisión del usuario

**Status:** Todo

- [ ] Suite, calidad, build y diff pasan sobre la entrega completa.
- [ ] El usuario revisa la nueva estructura (`src/tests/components/`) y los recorridos de navegación.
- [ ] `git diff --check` y `git status` se revisan antes del cierre.

**Dependencies:** T07.
