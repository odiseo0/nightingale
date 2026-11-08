# Plan: consolidar los tests de componentes en `src/tests/`

**Status:** Approved
**Created:** 2026-11-08
**ID:** P-20261108-03-tests-consolidation

## Contexto

Hoy ~60 archivos `*_tests.rs` viven dentro de `src-tauri/src/components/<comp>/`, mezclados con el código de producto, más 4 `test_support.rs` y los fixtures JSON de sync. El usuario quiere una carpeta dedicada para navegar el proyecto más fácilmente.

**Decisión de ubicación (clave):** el árbol nuevo será `src-tauri/src/tests/` (dentro del crate lib, cableado con `#[cfg(test)] mod tests;` en `lib.rs`), **no** `src-tauri/tests/`. Razón: los tests actuales usan masivamente ítems `pub(crate)` (repositorios SQLite, preparación de intenciones, `platform::http::testing`, `platform::test_api`). En `src-tauri/tests/` (tests de integración externos) solo se ve la API `pub`; moverlos ahí obligaría a hacer pública la interna de cada componente, violando la regla del plan maestro de API pública mínima. Dentro del crate se conserva `pub(crate)` y el refactor no cambia ningún comportamiento.

## Inventario descubierto

- Declaraciones en `mod.rs` de cada componente con `#[cfg(test)] mod xxx_tests;`: appointments (16), patients (12), scheduling (14), services (1: booking_tests), sync (7: contract, 4 de integración, matrix, development).
- Declaraciones vía `#[path]` **dentro de archivos fuente** (estos tests usan ítems privados del módulo anfitrión con `use super::*;`):
  - account: `adapters/online.rs` → `online_tests.rs`; `adapters/outbound.rs` → `projection_tests.rs`.
  - appointments: `adapters/projector.rs` → `projection_tests.rs`, `history_projection_tests.rs`.
  - identity: `application.rs` → `activation_tests.rs`; `oidc.rs` → `oidc_tests.rs`.
  - patients: `adapters/inbound_contacts.rs` → `ipc_contact_tests.rs`; `adapters/inbound_profiles.rs` → `ipc_profile_tests.rs`; `adapters/projector.rs` → `projection_tests.rs`.
  - scheduling: `adapters/projector.rs` → `projection_tests.rs`.
  - services: `application.rs` → `application_tests.rs`; `adapters/inbound.rs` → `ipc_tests.rs`; `adapters/outbound.rs` → `repository_tests.rs`; `adapters/projector.rs` → `projection_tests.rs`.
  - sync: `coordinator.rs` → `coordinator_tests.rs`; `dispatch.rs` → `dispatch_tests.rs` (y `dispatch_tests.rs` declara `reconciliation_tests.rs` vía `#[path]`); `download.rs` → `download_tests.rs`; `remote.rs` → `remote_tests.rs`; `snapshot.rs` → `snapshot_tests.rs`.
- `test_support.rs` (appointments, patients, scheduling, services) es `#[cfg(test)] pub(crate)` y se comparte entre componentes (los tests de integración de sync usan los de services, scheduling, patients, appointments).
- `include_str!` con rutas relativas: `sync/contract_tests.rs` (docs/contracts a 4 niveles + `fixtures/`), `sync/download_tests.rs` y `remote_tests.rs` (`fixtures/remote-group.json`), `identity/oidc_tests.rs` (`../../../tests/fixtures/oidc-test-key.pem`, del `tests/` externo del crate).
- Se quedan en su sitio: tests inline `#[cfg(test)] mod tests` dentro de fuentes pequeñas (storage, platform/sqlite, platform/http, dominios), `platform/test_api.rs`, `platform::http::testing`, y el test externo `src-tauri/tests/core_persistence.rs`.
- Los tests de registro IPC (`account/adapters/inbound.rs`, `ipc_registration_tests` de scheduling/appointments) parsean el texto de `lib.rs`/`build.rs` por subcadenas de comandos; añadir `#[cfg(test)] mod tests;` a `lib.rs` es compatible.

## Enfoque

Refactor mecánico, sin cambios de comportamiento, guiado por el compilador (visibilidades privadas → `pub(crate)` donde un test las use).

## Cambios principales

### 1. Árbol nuevo

```text
src-tauri/src/tests/
  mod.rs                      // pub(crate) mod components;
  components/
    mod.rs                    // pub(crate) mod <comp>; por componente con tests
    account/        online_tests.rs, projection_tests.rs, mod.rs
    appointments/   16 archivos *_tests.rs + test_support.rs + mod.rs
    identity/       activation_tests.rs, oidc_tests.rs + mod.rs
    patients/       12 archivos *_tests.rs + test_support.rs + mod.rs
    scheduling/     14 archivos *_tests.rs + test_support.rs + mod.rs
    services/       5 archivos *_tests.rs + test_support.rs + mod.rs
    sync/           13 archivos *_tests.rs + fixtures/ + mod.rs
```

- `lib.rs`: añadir `#[cfg(test)] mod tests;` (única línea; el resto intacto).
- Todo el árbol solo compila bajo `cfg(test)`; `test_support` sigue siendo `pub(crate)` dentro de su `mod.rs` (visible para los tests de sync que viven en la misma rama del árbol).

### 2. Limpieza de cableado viejo

- Quitar de cada `components/<comp>/mod.rs` todas las líneas `#[cfg(test)] mod *_tests;` y `#[cfg(test)] pub(crate) mod test_support;`.
- Quitar los `#[path = "..._tests.rs"]` de los archivos fuente y el bloque `#[cfg(test)] mod` asociado.
- Mover archivos con `git mv` (historia preservada), incluidos `sync/fixtures/`.

### 3. Reescritura de imports en los archivos movidos

- Tests declarados antes en `mod.rs` del componente: `use super::{adapters..., application..., domain...}` → `use crate::components::<comp>::{...}`. Las referencias `super::test_support::…` **se conservan sin cambios**, porque `test_support.rs` queda como módulo hermano en el nuevo directorio (`super::` ahora apunta a `tests::components::<comp>`).
- Tests declarados antes vía `#[path]` dentro de un fuente: `use super::*;` → `use crate::components::<comp>::<ruta>::*;` y las cadenas `super::super::…` → rutas `crate::` completas.
- Tests de integración de sync: `crate::components::<x>::test_support::…` → `crate::tests::components::<x>::test_support::…`.

### 4. Visibilidades (guiado por compilador)

Iterar `cargo check --workspace --all-targets --locked` y ampliar a `pub(crate)` los ítems privados que los tests movidos usaban vía `super::*` (p. ej. internos de `sync::coordinator/dispatch/download/remote/snapshot`, `identity::application/oidc`, adaptadores de account/patients/services). Sin tocar firmas ni lógica; ampliación crate-interna, sin efecto en la API pública ni en el binario de producción.

### 5. `include_str!` / `include_bytes!`

- `sync/contract_tests.rs`: `../../../../docs/contracts/...` → `../../../../../docs/contracts/...` (un nivel más).
- `identity/oidc_tests.rs`: `../../../tests/fixtures/oidc-test-key.pem` → `../../../../tests/fixtures/oidc-test-key.pem`.
- `fixtures/*.json` de sync: sin cambios (la carpeta se mueve junto a sus tests).

### 6. Documentación viva

- `docs/operations/development.md`: los filtros de `cargo test` cambian de `components::<comp>::...` a `tests::components::<comp>::...`; actualizar comandos y matriz de revisión.
- `docs/technical/components/*.md`: actualizar las rutas de archivos de test citadas en los recorridos de revisión.
- No se reescriben `tasks/*/todo.md` (historial cerrado).

## Verificación

- `cargo test --workspace --locked` → mismo total de tests en verde (381 passed, 0 failed, 9 ignored: 8 preexistentes + 1 opt-in).
- `cargo fmt --all -- --check` y `cargo clippy --workspace --all-targets --locked -- -D warnings`.
- `pnpm build:desktop` (sin cambios JS, control de regresión).
- `git diff --check` y `git status`: solo movimientos/ediciones esperados; sin artefactos.
- Comprobación manual ofrecida al usuario: `cargo test --workspace --locked tests::components::scheduling` como ejemplo del nuevo filtro.

## Supuestos

- Se elige `src-tauri/src/tests/` (no `src-tauri/tests/`) para preservar `pub(crate)`; la alternativa obligaría a publicar internas, contra el plan maestro.
- `test_support.rs` y `fixtures/` se mueven también (son solo de tests); el binario de producción no cambia.
- Los tests inline pequeños, `platform/test_api.rs`, `http::testing` y `tests/core_persistence.rs` externo permanecen donde están.
- Refactor puro: ningún cambio de lógica, contratos, migraciones ni DTO.
