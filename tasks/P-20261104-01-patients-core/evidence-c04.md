# C04: sincronización de Pacientes

Fecha: 2026-11-05. C03 aprobado por el usuario (con correcciones T19–T21). T10–T12 completos, en orden. **C04 pendiente de revisión; no aprobado. T13 no iniciado.**

## Cambios

- T10: `PatientsProjector` implementa `RemoteProjector` para las cuatro entidades (`patient_profile`, `patient_identifier`, `contact_person`, `patient_contact` con clave compuesta). Traduce con los `save_*` existentes (upsert con guarda de versión: repeticiones y versiones antiguas no retroceden vistas), valida con las reglas de dominio (nombre, pareja telefónica, terna documental, vigencia, documento) y exige `accountId` del ámbito. `Remove` y `reset` borran solo vistas del ámbito; `patient_local` y otras cuentas no se tocan. Un fallo revierte el grupo y sus efectos de cursor/inbox en la transacción recibida.
- T11: los constructores de filas retiran un provisional solo cuando su operación tiene recibo `accepted` y la vista aceptada alcanzó la versión confirmada por el recibo (`entityVersion`). Un recibo sin descarga no retira nada. Conflictos, rechazos y resultados `unknown` conservan provisional, ID, contenido y dependencias; las ediciones sobre secuencias bloqueadas se rechazan con `DEPENDENCY_FAILED` y las correcciones siguen creando operaciones nuevas por caso específico.
- T12: `DesktopProjector` compone Account, Services y Patients con la misma transacción, sin cambiar reglas de propiedad; el proyector no toca Outbox ni hace HTTP. `patients::test_support` quedó `pub(crate)` para la prueba de integración.
- No se añade migración ni dependencia. Se conserva todo el trabajo local de C01–C03.

## Verificación

| Comprobación | Resultado |
|---|---|
| `cargo test --workspace --locked components::patients` al terminar T10 | 60 pruebas reales correctas |
| Mismo filtro al terminar T11 | 63 pruebas reales correctas |
| `cargo test --workspace --locked patients_integration_tests` (T12) | 1 prueba real de extremo a extremo correcta |
| Regresiones de sync (`cargo test --workspace --locked`) | Suite completa correcta: 222 unitarias, 5 de integración y 1 de documentación; 6 manuales ignoradas |
| `cargo check --workspace --all-targets --locked` | Correcto |
| `cargo fmt --all -- --check` | Correcto |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Correcto |
| `pnpm build:desktop` | Correcto; genera target/debug/nightingale.exe |
| `git diff --check` | Correcto |
| SHA-256 del manifest | Las cinco huellas del bundle coinciden |

Las pruebas nuevas de la fase: 4 de proyección (grupo completo repetido, versiones antiguas, remove/reset con conservación de intenciones y otra cuenta, reversión de grupo inválido con cursor/inbox en cero, rechazo de estado fuera de ámbito), 3 de conciliación (retirada solo con recibo+vista, conservación de secuencias rechazadas con bloqueo de dependientes, resultados `unknown` con edición distinta que conserva el ID original) y 1 de integración de extremo a extremo.

Recorrido de integración reproducible por nombre:

```powershell
cargo test --workspace --locked components::sync::patients_integration_tests -- --exact
```

## Recorrido de la prueba de integración (servidor sintético local)

1. Registro offline de un menor con responsable y completitud (4 operaciones Outbox + `patient_local`); reapertura duradera antes de HTTP.
2. Ciclo normal: envío, aceptación con versiones, descarga de las cuatro entidades. Vistas aceptadas alcanzadas; provisionales retirados; operaciones y recibos conservados; HTTP nunca retiene SQLite.
3. Edición de contacto desde base aceptada con respuesta remota `VERSION_CONFLICT`: la intención provisional se conserva con su código; la vista aceptada del paciente llega a v3 por descarga de otro dispositivo.
4. Descarga de un retiro remoto del contacto: vista aceptada borrada, intenciones intactas.
5. Captura nueva (`SnapshotMode::Restart`): vistas reconstruidas (contacto v2), trabajos locales y recibos conservados, cursor `fresh-base`.

Se usan datos ficticios y archivos temporales. No se cuentan ejecuciones con cero pruebas como evidencia.

## Fuentes de las APIs usadas

Se aplicaron minimal-code y docs-first-framework-code, con versiones de Cargo.lock: rusqlite 0.40.2, Serde 1.0.229, serde_json 1.0.154 y Chrono 0.4.45. Esta fase no introduce APIs nuevas: reutiliza los patrones ya citados en evidencias anteriores ([rusqlite transactions](https://docs.rs/rusqlite/0.40.2/rusqlite/struct.Transaction.html), [upsert con guarda de versión](https://www.sqlite.org/lang_upsert.html), [Serde derive](https://serde.rs/derive.html), [Chrono parse_from_rfc3339](https://docs.rs/chrono/0.4.45/chrono/struct.DateTime.html#method.parse_from_rfc3339)).

## Límites

La prueba de integración usa un servidor HTTP sintético (`TestApi`) con respuestas programadas; no acredita Nightingale API/OIDC real (eso corresponde a T17). Los recibos se simulan en `sync_outbox` con las mismas formas que escribe el despachador. Los estados de recibo `NOT_FOUND`/`RESULT_EXPIRED` (recuperación por GET) se ejercitan en las pruebas generales de sync, no aquí de nuevo.

`patient_local` conserva las filas de provisionales retirados (el modelo de Pacientes retira en lectura, a diferencia del borrado físico de `service_local`); las filas ocultas no reaparecen en las consultas y las operaciones/recibos permanecen en el Outbox de sync. El coste de acumulación se revisará si crece en cuentas grandes.

No hay comandos Tauri ni pantalla; eso corresponde a T13–T15. Contrato importado, especificaciones, React y migraciones previas intactos. No se abrió el ejecutable contra la base habitual del usuario. Sin commits ni push.

**Revisión pendiente:** validar que un resultado parcial o perdido no duplica efectos ni borra cambios antes de autorizar T13. La entrega de esta evidencia no aprueba C04.
