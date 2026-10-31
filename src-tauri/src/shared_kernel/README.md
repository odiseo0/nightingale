# Kernel compartido

UUIDv7 canónicos, IDs de cuenta/usuario/dispositivo/operación, `OperationContext`, puertos de reloj/IDs y errores estables. No contiene entidades de negocio, SQL, HTTP, Tauri o repositorios universales.

Usa el reloj y el generador inyectados en los casos locales. Una fecha no sustituye una versión ni prueba permiso. Las referencias de transporte de entidades remotas permanecen en `sync`, sin añadir todos sus tipos al kernel.

Los adaptadores disponibles son `SystemClock` y `UuidV7Generator`; los casos reciben sus puertos `Clock` e `IdGenerator`. No contiene eventos, entidades ni tablas propias.

Pruebas: `cargo test --workspace --locked shared_kernel`. Un cambio de este límite requiere justificación y revisión en el plan del componente.
