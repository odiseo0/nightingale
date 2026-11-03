# Plataforma

Configuración validada, transporte HTTP/SSE, almacén seguro del SO, SQLite y registros de metadatos. Este módulo no decide reglas de pacientes o citas ni acepta cambios compartidos.

`APIConfig::from_environment` se lee al iniciar una operación remota. Sin URL válida siguen disponibles la apertura local y las consultas de diagnóstico. Release exige HTTPS; desarrollo admite HTTP solo en loopback. El transporte no sigue redirects ni reintenta emisión de tokens por su cuenta.

`sqlite::open` aplica migraciones y recupera reservas de un proceso anterior. No abras otra conexión de arranque mientras el coordinador envía. No compartas el archivo entre dispositivos, lo borres ante un error o edites una migración ya aplicada.

`observability::Event` solo admite metadatos tipados. Escribe JSON en stderr; un fallo del destino no altera operaciones. No amplíes el tipo con texto remoto, cuerpos, URLs, cursores o secretos. La hora del último intercambio que muestra la ventana pertenece a esta ejecución; Outbox, resultados y cursor sí se reconstruyen desde SQLite.

Los adaptadores disponibles son `APIConfig`, `APITransport`, `NativeSecretStore`/`MemorySecretStore`, la apertura SQLite y `observability::{emit,write}`. No posee entidades ni publica eventos de dominio; `observability::Event` es telemetría, no un evento de negocio.

Pruebas: `cargo test --workspace --locked platform`. [Requisitos y verificación manual](../../operations/development.md).
