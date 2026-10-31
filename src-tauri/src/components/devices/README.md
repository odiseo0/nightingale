# Dispositivos

Propietario de `device_installation` y de la credencial opaca de instalación en el almacén seguro del SO. `DeviceCredentials::ensure` reutiliza la credencial o la emite una sola vez. Guarda el estado de emisión antes de HTTP: una respuesta perdida exige recuperación explícita, no otra emisión automática.

La clave segura distingue entorno, perfil, identidad, clase de secreto y dispositivo. OIDC y API usan entradas distintas. `forget` es una salida local explícita; no revoca en API ni cambia el alcance de operaciones guardadas con otro dispositivo. El registro de cuenta usa `RegisterAccountDevice` a través de `AccountActivation`.

La API pública es `DeviceCredentials::{ensure,load,forget}` y el resultado `DeviceCredential` para composición Rust. Usa los puertos `SecretStore` y `APITransport`, con adaptadores del almacén del SO e `IdentityAPI`; no publica eventos de dominio propios.

No añadas tokens a DTO, Debug, SQLite, fixtures reales o logs. Pruebas de activación: `cargo test --workspace --locked components::identity::application::tests`. Almacenes nativos: [desarrollo](../../../../docs/operations/development.md); macOS/Linux todavía necesitan prueba real.
