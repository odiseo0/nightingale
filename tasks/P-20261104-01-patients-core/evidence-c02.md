# C02: recorrido local de fichas

Fecha: 2026-11-04. C01 aprobado por el usuario. T04–T06 completos, en orden. **C02 pendiente de revisión; no aprobado. T07 no iniciado.**

## Cambios

- T04: registro de fichas draft, consulta y búsqueda por ámbito. Guardar provisional y operación Outbox comparte una transacción SQLite. Las lecturas separan estado aceptado, provisional y operaciones. Nombres iguales crean identidades distintas.
- T05: edición de perfil y actividad con base aceptada o última operación local. Cada cambio conserva ID, contenido y dependencias. Una base antigua devuelve VERSION_CONFLICT; un cambio igual devuelve NO_CHANGES sin crear operación. El orden de las versiones previstas no depende del reloj del equipo.
- T06: documentos iniciales o posteriores, siempre declarados; consulta de documentos y búsqueda por su número sin perder ceros o puntuación. Completar exige los requisitos de own_document o minor_without_document. Se consultan responsables ya aceptados de la misma cuenta, con contacto/vínculo activos, teléfono y vigencia. La edad a fecha de cita queda fuera.
- Una edición que quite requisitos a una ficha completa devuelve COMPLETENESS_CONFLICT con su referencia. Hay que pasarla expresamente a draft. Añadir después un documento propio conserva PatientId.
- Una operación con resultado desconocido conserva ID y contenido. Se puede guardar otra edición distinta que dependa de ella, sin atribuir aceptación a ninguna versión prevista. Conflictos, rechazos y operaciones de otra época requieren revisión.
- PatientQuery entrega fichas y documentos aceptados a consumidores Rust, sin exponer almacenamiento o HTTP. El resto del puerto se ampliará con los casos de contactos de la fase siguiente.

Se reutilizan permisos de Identidad, OperationContext, record_operation, reloj e IDs inyectados, dominio y tablas de C01. No se añade migración ni dependencia en esta fase. Se conserva todo el trabajo local de C01.

## Verificación

| Comprobación | Resultado |
|---|---|
| `cargo test --workspace --locked components::patients` al terminar T04 | 22 pruebas reales correctas |
| Mismo filtro al terminar T05 | 28 pruebas reales correctas |
| Mismo filtro tras el ajuste final de T06 | 37 pruebas reales correctas |
| `cargo check --workspace --all-targets --locked` | Correcto por tarea y en el estado final |
| `cargo fmt --all -- --check` | Correcto |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Correcto |
| `cargo test --workspace --locked` final, fuera del aislamiento | 195 unitarias, 5 de integración y 1 de documentación correctas; 6 manuales ignoradas |
| `pnpm build:desktop` final | Correcto; genera target/debug/nightingale.exe |
| `git diff --check` y enlaces locales | Correctos |
| SHA-256 del manifest | Las cinco huellas del bundle coinciden |

Las 37 pruebas centradas incluyen 21 nuevas en T04–T06, además de las 16 de C01 en el componente. Las dos pruebas de migración pertenecen a platform::sqlite y también pasan en la suite completa. No se cuentan ejecuciones con cero pruebas como evidencia.

Recorridos ejecutados dentro de la suite y reproducibles por nombre:

```powershell
cargo test --workspace --locked components::patients::application_tests::register_edit_document_complete_and_reopen_preserve_atomic_sequence -- --exact
cargo test --workspace --locked components::patients::application_tests::successive_profile_edits_keep_bases_payloads_and_status_after_reopen -- --exact
cargo test --workspace --locked components::patients::application_tests::uncertain_original_allows_distinct_dependent_edits_without_replacing_it -- --exact
```

Se usan datos ficticios y archivos temporales. Las pruebas fuerzan fallos al escribir patient_local u Outbox y comprueban la reversión; también cubren permisos, cuenta/usuario/dispositivo, reapertura, ausencia/null y dependencias de documentos al completar. Las pruebas de nombres iguales no fusionan registros.

Durante la revisión se corrigió una restricción de más sobre resultados desconocidos: ahora una edición nueva puede depender del original incierto y conserva su identidad. Se añadió una prueba y se repitieron las verificaciones. Clippy también señaló cuatro clones innecesarios en aserciones; se retiraron. Los avisos de Cargo sobre enlaces físicos de caché no impidieron las verificaciones.

## Fuentes de las APIs usadas

Se aplicaron minimal-code y docs-first-framework-code, con versiones de Cargo.lock: rusqlite 0.40.2, Serde 1.0.229, serde_json 1.0.154 y Chrono 0.4.45.

- [rusqlite: transaction_with_behavior](https://docs.rs/rusqlite/0.40.2/rusqlite/struct.Connection.html#method.transaction_with_behavior) y [Transaction](https://docs.rs/rusqlite/0.40.2/rusqlite/struct.Transaction.html): reservar el bloqueo antes de comprobar permisos/preparar la escritura; confirmar ficha e intención juntas o revertir ambas.
- [Serde: derive](https://serde.rs/derive.html) y [atributos del contenedor](https://serde.rs/container-attrs.html): modelos de lectura/estado local con nombres camelCase, separados de los valores puros del dominio. Los DTO IPC quedan para T13–T14.
- [Chrono: from_timestamp_millis](https://docs.rs/chrono/0.4.45/chrono/struct.DateTime.html#method.from_timestamp_millis) y [parse_from_rfc3339](https://docs.rs/chrono/0.4.45/chrono/struct.DateTime.html#method.parse_from_rfc3339): comparar vigencia con un instante inyectado, sin deducir zona ni consultar el reloj desde las reglas.

## Límites

No hay comandos Tauri o pantalla de Pacientes todavía. No se ejecutó API/OIDC real de Pacientes ni se acredita sincronización: proyector, conciliación y composición corresponden a T10–T12; la prueba remota corresponde a T17. Las pruebas de otros componentes con servidores locales son sintéticas.

Crear o editar contactos/vínculos y proteger completitud frente a sus cambios corresponde a T07–T09. Esta fase solo lee responsables aceptados ya presentes. El retiro de provisionales tras recibo/proyección y la presentación completa de dependencias bloqueadas queda para T11. Las búsquedas combinan las filas del ámbito antes de paginar; no se ha medido su coste con cuentas grandes.

Contrato importado, especificaciones, React y migraciones previas intactos. No se abrió el ejecutable contra la base habitual del usuario ni se usaron datos reales. Sin commits ni push.

**Revisión pendiente:** validar el recorrido local y estos límites antes de autorizar T07. La entrega de esta evidencia no aprueba C02.

## Revisión posterior

El usuario aprobó C02 el 2026-11-05: «Apruebo C02». El estado pendiente descrito arriba corresponde a la entrega original. T07–T09 quedan como siguiente fase; no se iniciaron en esta sesión. C03 no está aprobado.
