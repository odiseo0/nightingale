# Sincronización

Propietario de Outbox, Inbox, cursor, captura en preparación y vista remota aceptada. No posee tablas de Pacientes, Servicios, Agenda o Citas. Las migraciones `0003`/`0004` ya existen; un track nuevo reserva otro número mediante su plan.

Entradas nativas:

- `record_operation(tx, store, ids, clock, input)` conserva intención y contexto en la misma transacción que la escritura local del componente. ID y contenido no cambian al reintentar.
- `SyncEngine::run` coordina acceso, resultados desconocidos, envío y descarga/captura. `listen_once` usa SSE solo como aviso. Un coordinador por perfil agrupa los ciclos por cuenta/usuario/dispositivo.
- `RemoteProjector::project` recibe un `RemoteChange` tipado y la transacción de la página. Escribe solo la vista aceptada del componente. `reset` retira esa vista del ámbito al sustituir captura; no borra intenciones locales ni hace HTTP.
- `SyncCoordinator::status` y `SqliteOperationStore::reviews` leen metadatos sin descargar payloads a React. El estado remoto completo del conflicto sigue en el repositorio Rust, para casos de revisión específicos.

El estado de la intención y el estado remoto son distintos. `accepted` de un recibo no mueve cursor ni sustituye la proyección. `remove` solo retira la vista. Repetir un grupo no proyecta otra vez. La captura y `baseCursor`, o la página y `nextCursor`, se confirman juntos.

`unknown` primero consulta su ID; un resultado temporal conocido puede reintentar POST. 404 y `RESULT_EXPIRED` requieren revisión. El modo manual retira una espera, sin cambiar ID, huella, contador o reservas vigentes. Una época nueva no reasigna operaciones antiguas.

Puertos y adaptadores: `OperationStore`, `CursorStore`, `InboxStore` y `RemoteProjectionStore` tienen implementaciones SQLite; `APITransport`/`SyncAPI` adaptan HTTPS y SSE; `RemoteProjector` pertenece a cada componente consumidor. No publica eventos de dominio propios: los recibos y avisos de transporte no sustituyen eventos aceptados de negocio.

Pruebas: `cargo test --workspace --locked components::sync`. `contract_tests` comprueba el bundle fijo y las formas consumidas; `matrix_tests` usa dos perfiles aislados y servidor sintético. No acreditan API de desarrollo. Límites de memoria, recuperación y despliegue: [desarrollo](../../operations/development.md). Contratos para los siguientes equipos: [tracks](../../operations/core-next-tracks.md).
