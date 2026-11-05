# C01: contratos y persistencia de Pacientes

Fecha: 2026-11-04. Estado: presentado para revisión; **C01 no aprobado**. T01, T02 y T03 completos, en ese orden. T04 no iniciado. Rama local: `codex/patients-core-c01`. Sin commits ni push.

## Cambios

- T01: el plan fija las once escrituras y siete consultas, claves, modelos, permisos, base aceptada u operación local previa, separación aceptado/provisional, errores y límite de ocho dependencias. Se cotejaron acciones/modelos con el bundle fijo y se revisaron draft, menor con responsable y secuencia de ediciones.
- T02: dominio puro para actividad, completitud, base de identificación, documentos, teléfono, fechas, vigencia y patch. Conserva documentos como texto y diferencia Keep/Clear/Set. Las entradas de documento no tienen campos de verificación. Reutiliza países del kernel. Chrono 0.4.45 pasa a dependencia directa sin funciones opcionales; el lockfile solo añade la referencia desde nightingale.
- T03: migración 0007_patients, disponible al inicio, añade cuatro tablas de vistas aceptadas y patient_local. Todas llevan cuenta/usuario/dispositivo; vínculo con clave compuesta; índices por estado, completitud y referencias. Intenciones sin cascadas ni claves foráneas desde vistas. Los registros locales conservan base, dependencias, payload y provisional, separados de recibos/época de permisos que pertenecen a sync.
- Adaptador SQLite interno al componente con lecturas acotadas y escrituras dentro de la transacción recibida. Las vistas no retroceden ante una versión igual o antigua. No se conecta aún a un proyector o caso de uso.
- Tres ajustes de pruebas existentes para la versión de esquema 7: comprobación del migrador, contador de migraciones en core_persistence y localSchemaVersion en activación. No se cambia el comportamiento de Identidad ni Sync.

## Verificaciones

Entorno: Windows, PowerShell, Rust/Cargo del proyecto. Datos ficticios y bases temporales. Los ejemplos de dominio y persistencia se ejercen desde pruebas Rust; no hay recorrido IPC de Pacientes en esta fase.

| Comando o cotejo | Resultado |
|---|---|
| `cargo test --workspace --locked components::sync::contract_tests` (T01) | 4 pruebas correctas; cotejo adicional de 11 acciones/modelos/offline y enlaces locales correcto |
| `cargo test --workspace --locked components::patients` (T02) | 8 pruebas reales correctas al terminar T02 |
| `cargo test --workspace --locked patients` (T03) | 18 correctas: 8 dominio, 8 almacenamiento y 2 migración |
| `cargo test --workspace --locked components::patients` (T03) | 16 correctas; las dos pruebas de migración pertenecen a platform::sqlite y no entran en este filtro |
| `cargo check --workspace --all-targets --locked` | Correcto en T02 y T03 |
| `cargo test --workspace --locked` (C01, fuera del aislamiento) | 174 unitarias correctas, 6 manuales ignoradas; 5 de integración y 1 de documentación correctas |
| `pnpm build:desktop` (fuera del aislamiento) | Correcto; genera target/debug/nightingale.exe |
| `cargo fmt --all -- --check` | Correcto |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Correcto |
| `git diff --check`, enlaces locales y cinco SHA-256 del manifest | Correctos |

La prueba de actualización abre una base de versión 6 con datos de dos cuentas, aplica 7 y reabre sin alterar esos datos. La prueba de fallo inyecta SQL inválido al final de 0007 y comprueba archivo idéntico byte por byte, versión 6 e historial intacto. Las pruebas de almacenamiento cubren aislamiento por cada parte del ámbito, reapertura de los cuatro tipos e intenciones, contacto compartido, versiones repetidas/antiguas, bases/dependencias, restricciones y rollback. Borrar vistas de un ámbito conserva sus intenciones y los datos de otro.

Incidencias resueltas durante la verificación:
- El nuevo test de migración consultó inicialmente value_json en vez de value; corregido y repetido con éxito.
- La suite completa dentro del aislamiento falló en pruebas existentes con servidores locales y terminó con 0xc0000409. La repetición fuera del aislamiento mostró una expectativa antigua de esquema 6 en activación. Se ajustó a 7 y la suite completa pasó.
- Corepack no pudo obtener pnpm dentro del aislamiento por fallo DNS; el build fuera del aislamiento pasó sin cambiar versiones del proyecto.
- Cargo avisó de que copió archivos al no poder crear enlaces físicos de caché. No impidió las comprobaciones.

## Aplicación de las skills

minimal-code: se trazó el arranque SQLite (migrador único), almacenamiento y proyectores de Services, record_operation y ámbito de Identity. Se reutilizan el migrador, la transacción del llamador, OperationContext, errores comunes y países; no se crea otro motor de persistencia. El dominio no usa SQL, HTTP, Serde o Tauri. Los modelos de almacenamiento quedan en el adaptador y no se usan como entidad de dominio ni como DTO IPC.

docs-first-framework-code: versiones leídas en Cargo.toml/Cargo.lock (Chrono 0.4.45, rusqlite 0.40.2). Fuentes oficiales consultadas:
- [Chrono: parse_date con NaiveDate::parse_from_str](https://docs.rs/chrono/0.4.45/chrono/naive/struct.NaiveDate.html#method.parse_from_str): se comprueba además la forma exacta YYYY-MM-DD.
- [Chrono: parse_instant con DateTime::parse_from_rfc3339](https://docs.rs/chrono/0.4.45/chrono/struct.DateTime.html#method.parse_from_rfc3339): exige offset y compara instantes.
- [Chrono: funciones opcionales](https://docs.rs/chrono/0.4.45/chrono/#features): sin clock, now ni integración Serde nueva.
- [rusqlite: Transaction](https://docs.rs/rusqlite/0.40.2/rusqlite/struct.Transaction.html): el llamador confirma o revierte; el adaptador no abre otra transacción.
- [rusqlite: Statement::query_map](https://docs.rs/rusqlite/0.40.2/rusqlite/struct.Statement.html#method.query_map): parámetros enlazados y lectura de registros.
- [SQLite: restricciones](https://www.sqlite.org/lang_createtable.html#check_constraints), [índices](https://www.sqlite.org/lang_createindex.html), [JSON](https://www.sqlite.org/json1.html#the_json_valid_function) y [UPSERT](https://www.sqlite.org/lang_upsert.html): comprobaciones de forma, claves con ámbito y guarda de versión.

## Límites y revisión pendiente

Esta fase prepara contratos, reglas de formato y acceso al almacenamiento. No implementa registrar/editar/completar fichas como casos de uso, unión con Outbox, permisos de los casos, protección de completitud compartida, proyector, conciliación o comandos Tauri; corresponden a T04–T15. PatientQuery y DTO IPC están definidos en el plan, aún no como interfaces de código. No se abrió la aplicación contra la base habitual del usuario; la migración y reapertura se probaron con archivos temporales. No se acredita una prueba visual o IPC.

No se ejecutó API/OIDC real de Pacientes: corresponde a T17. Las pruebas con servidores locales de otras áreas son regresiones sintéticas. No se añaden React, datos clínicos, fusión, consentimiento, Hub o cambios en otros repositorios. Contrato importado, especificaciones y migraciones 0001–0006 intactos.

**Pendiente:** revisión expresa del usuario sobre interfaces, límites y migración. Detenerse aquí; no iniciar T04 ni marcar C01 aprobado.

## Revisión posterior

El usuario aprobó C01 el 2026-11-04: «Por favor, puedes continuar con el plan. C01 está bien». Se conserva arriba la evidencia de la entrega original. La ejecución continúa con T04–T06 hasta C02.
