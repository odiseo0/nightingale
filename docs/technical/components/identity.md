# Identidad y acceso

Propietario de OIDC/PKCE, identidad seleccionada y último acceso válido por cuenta/usuario/dispositivo. `oidc.rs` valida el acceso del navegador externo; `AccountActivation` consulta API y activa la instalación. `repository.rs` posee `identity_context` e `identity_account_access`. Los tokens pertenecen al almacén seguro, nunca a SQLite.

`desktop.rs` compone los comandos de acceso/cancelación, cuentas, activación y logout con el coordinador. El perfil de la ventana es `desktop`; los tests usan perfiles propios. Logout cancela y espera el supervisor antes de retirar credenciales. No revoca el dispositivo remoto.

Offline: `cached_access` y `selected_access` permiten leer la última autorización. Un caso local debe usar `AccountAccess::allows_offline` y conservar el actor/época de la intención. No concede permisos por fecha local ni permite cambios administrativos offline. API vuelve a decidir al aceptar.

Puertos y adaptadores: `AccessStore` tiene implementación SQLite; `IdentityAPI`, `OidcProvider` y `SystemBrowser` adaptan API, OIDC y navegador; `SecretStore` protege las credenciales. No publica eventos de dominio propios. Los comandos IPC exponen solo identidad seleccionada, cuentas, acceso y metadatos de sincronización.

Pruebas: `cargo test --workspace --locked components::identity`. Las pruebas de navegador/API reales están ignoradas y documentadas en la [guía de desarrollo](../../operations/development.md). El [ADR aprobado](../../decisions/desktop-core-engine.md) fija estos límites.
