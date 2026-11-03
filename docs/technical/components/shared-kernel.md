# Kernel compartido

UUIDv7 canónicos, IDs de cuenta/usuario/dispositivo/operación, `OperationContext`, puertos de reloj/IDs y errores estables. No contiene entidades de negocio, SQL, HTTP, Tauri o repositorios universales.

`reference_data` reúne las listas compartidas de países y zonas horarias y ofrece
`is_country_code` e `is_time_zone`. Son consultas puras sobre datos incluidos al
compilar. El [origen y la actualización](../reference-data.md) se documentan fuera
del código. Cada componente conserva las reglas de su caso de uso.

Usa el reloj y el generador inyectados en los casos locales. Una fecha no sustituye una versión ni prueba permiso. Las referencias de transporte de entidades remotas permanecen en `sync`, sin añadir todos sus tipos al kernel.

Los adaptadores disponibles son `SystemClock` y `UuidV7Generator`; los casos reciben sus puertos `Clock` e `IdGenerator`. No contiene eventos, entidades ni tablas propias.

Pruebas: `cargo test --workspace --locked shared_kernel`. Un cambio de este límite requiere justificación y revisión en el plan del componente.
