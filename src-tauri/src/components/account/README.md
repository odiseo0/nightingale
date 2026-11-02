# Cuenta

Responsabilidad: vista aceptada de la cuenta seleccionada, preferencias de este
dispositivo y cambios administrativos con conexión. No posee miembros ni credenciales.

Datos: `account_view` y `account_preferences`, migración `0005_account`.
Cada fila tiene ámbito de cuenta, usuario y dispositivo. Solo el proyector escribe
la vista; las preferencias se guardan sin Outbox y no sustituyen la política central.

API de C01: get_account_view, get_preferences, set_preference,
update_account_details, update_account_policy y set_account_status.
El puerto público AccountViewQuery devuelve solo la vista y política aceptadas.

Los cambios administrativos exigen propietario y acceso activo, refrescan `/access`
y envían una operación en memoria. La cuenta solo cambia al proyectar el estado
aceptado. Un reintento de transporte conserva ID y contenido; una repetición manual
tras un resultado desconocido crea otra operación y exige revisar el estado remoto.

Comandos IPC: account_get_view, account_get_preferences, account_set_preference,
account_update_details, account_update_policy y account_set_status. Cada uno tiene
permiso Tauri explícito. React usa solo src/shared/api/account.ts. El panel está en
src/features/settings y se monta desde App.tsx con autorización del usuario.

Preferencias: theme (system/light/dark) y servicesShowInactive (boolean), con lista
cerrada de claves y valores. El componente guarda estas opciones de dispositivo;
no impone política clínica ni cambia permisos. Las listas de zonas y países tienen
origen, versión y hash en [validation-data.md](validation-data.md).

Errores: códigos estables y mensajes seguros; sin SQL, tokens ni estado de transporte.
El envío conserva el ID durante dos intentos. duplicate usa su resultado original;
conflict/rejected permanecen visibles. Solo accepted dispara sync_now; si falla la
descarga, se mantiene accepted con refreshStatus=failed. Se comparte la puerta de
identity para evitar cambio de sesión durante el refresco de acceso y el envío.

Límites para C02: el registro del proyector compuesto sigue reservado a T12. Hasta
entonces, las pruebas invocan AccountProjector con datos sintéticos; el ciclo normal
todavía usa el proyector vacío. No se acredita API/OIDC real ni revisión manual del
usuario. El usuario decidió mantener los DTO sin expectedAccountId: la sesión Rust
al ejecutar fija el destino; un formulario previo no tiene una guardia atómica que
lo vincule con la cuenta que mostraba.

Pruebas: `cargo test --workspace --locked components::account` (HTTP loopback para
los casos online), pruebas del runner, `pnpm quality` y `pnpm build:desktop`.
El contrato aprobado está en
[plan.md](../../../../tasks/P-20261102-01-account-services-catalog/plan.md).

Fuentes: [transacciones rusqlite 0.40.2](https://docs.rs/rusqlite/0.40.2/rusqlite/struct.Transaction.html),
[UPSERT SQLite](https://www.sqlite.org/lang_upsert.html),
[comandos Tauri 2](https://v2.tauri.app/develop/calling-rust/#async-commands),
[permisos por comando](https://v2.tauri.app/security/permissions/#app-manifest),
[invoke JS](https://v2.tauri.app/reference/javascript/api/namespacecore/#invoke),
[cleanup de efectos React 19.3](https://react.dev/reference/react/useEffect#fetching-data-with-effects).
Versiones fijadas: Tauri/API JS 2.12.4, React 19.3.1, Serde 1.0.229 y
rusqlite 0.40.2 (SQLite incluida 3.53.2). No se añaden dependencias.
