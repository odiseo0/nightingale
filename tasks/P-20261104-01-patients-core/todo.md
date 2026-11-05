# Task List: P-20261104-01-patients-core

**Plan:** [plan.md](plan.md)
**Status del plan:** In progress
**Updated:** 2026-11-05

## Instrucción vigente

Aprobación del usuario el 2026-11-04: «Lo veo bien, apruebo el plan. No comiences el desarrollo».

El usuario autorizó T01–T03 el 2026-11-04 y sustituyó la prohibición anterior. C01 fue aprobado por el usuario el 2026-11-04: «Por favor, puedes continuar con el plan. C01 está bien». C02 fue aprobado el 2026-11-05: «Apruebo C02». T01–T09 están completos; C03 se entrega para revisión con su [evidencia](evidence-c03.md). No se inicia T10 hasta la aprobación del usuario. La revisión de C03 del 2026-11-05 (ejes Estándares y Spec) detectó tres defectos confirmados en T07–T09; se añaden las correcciones T19–T21 a este mismo plan (ver «Correcciones de C03» en el plan). C03 sigue pendiente de aprobación.

**Reserva de migración:** 0007_patients; verificada libre al inicio, creada y registrada en T03.
**Responsable de integración:** quien ejecute este plan; un único responsable para migraciones, composición, comandos y permisos.
**Orden:** secuencial; cada checkpoint requiere revisión del usuario antes de la fase siguiente.
**Reglas comunes:** preservar trabajo ajeno; aplicar minimal-code antes de código y docs-first-framework-code para APIs de bibliotecas/frameworks. Usar datos ficticios.

Los filtros nuevos indicados como `Not available yet` se fijarán en la tarea indicada y deberán seleccionar pruebas reales antes de acreditar un resultado. La evidencia ejecutada figura en T01–T06; las fases posteriores siguen pendientes.

## T01: Contratos del componente

**Status:** Done

**Description:** Concretar en el plan los contratos internos de las once escrituras y siete consultas, sin implementar código ni alterar el bundle de API. Habilita dominio, almacenamiento e IPC con una misma definición.

**Acceptance criteria:**
- [x] Las once acciones, sus claves, versiones, payloads y permisos corresponden al contrato fijo.
- [x] Los DTO separan aceptado/provisional y definen versión aceptada o última operación local como base de edición.
- [x] Los contratos fijan consultas, errores y dependencias sin añadir funciones fuera del alcance aprobado.

**Verification:**
- [x] Focused tests: `cargo test --workspace --locked components::sync::contract_tests`; cotejo de esquemas y capacidades con los contratos documentados.
- [x] Build or broader check: `git diff --check` y revisión de enlaces del plan.
- [x] Manual check: Revisar una ficha draft, un menor con responsable y una secuencia de ediciones contra los contratos escritos.

**Dependencies:** Ninguna; requiere autorización de inicio.

**Files likely touched:**
- `tasks/P-20261104-01-patients-core/plan.md`
- `tasks/P-20261104-01-patients-core/todo.md`

**Estimated scope:** S

**Evidencia T01 (2026-11-04):** cuatro pruebas reales de sync::contract_tests correctas; cotejo de once acciones/modelos/offline y enlaces locales correcto; ejemplos manuales en el plan; git diff --check correcto. No acredita API real.

## T02: Dominio de fichas y formatos

**Status:** Done

**Description:** Crear las reglas puras de estados y formatos que usan los casos de Pacientes. Reutilizar Chrono 0.4.45 para fechas con funciones mínimas y sin actualizar otras dependencias.

**Acceptance criteria:**
- [x] El dominio valida estados, documentos, pares telefónicos y fechas sin red ni almacenamiento.
- [x] Las ediciones distinguen ausencia/null y no permiten atribuir verificación al cliente.
- [x] Las pruebas cubren datos válidos e inválidos y conservan el texto significativo de documentos.

**Verification:**
- [x] Focused tests: `cargo test --workspace --locked components::patients`, con casos concretos de esta tarea y ejecución no vacía.
- [x] Build or broader check: `cargo check --workspace --all-targets --locked` y `git diff --check`.
- [x] Manual check: Ejecutar ejemplos válidos e inválidos de ficha/documento/teléfono desde las pruebas puras.

**Dependencies:** T01.

**Files likely touched:**
- `src-tauri/src/components/patients/domain.rs`
- `src-tauri/src/components/patients/mod.rs`
- `src-tauri/src/components/mod.rs`
- `src-tauri/Cargo.toml`
- `Cargo.lock`

**Estimated scope:** M

**Evidencia T02 (2026-11-04):** 8 pruebas reales de components::patients::domain::tests correctas (estados, documentos, contactos, teléfonos, fechas, vigencia, draft y patch). cargo check --workspace --all-targets --locked y git diff --check correctos. Los ejemplos manuales se ejecutaron en estas pruebas puras. Chrono 0.4.45 pasa a dependencia directa sin funciones opcionales; Cargo.lock solo añade la arista desde nightingale, sin cambiar versiones. Fuentes: [fecha](https://docs.rs/chrono/0.4.45/chrono/naive/struct.NaiveDate.html#method.parse_from_str), [instante](https://docs.rs/chrono/0.4.45/chrono/struct.DateTime.html#method.parse_from_rfc3339), [funciones opcionales](https://docs.rs/chrono/0.4.45/chrono/#features).

## T03: Persistencia local

**Status:** Done

**Description:** Crear las tablas e índices del componente y su acceso con ámbito, conservando las migraciones previas. Habilita los casos nativos y la persistencia durable de intenciones.

**Acceptance criteria:**
- [x] 0007_patients añade cuatro vistas aceptadas y una tabla de cambios locales con el ámbito previsto.
- [x] La migración desde esquema 6 conserva datos y un fallo revierte sin pérdida.
- [x] Las consultas no cruzan cuenta/usuario/dispositivo y las intenciones no dependen de cascadas sobre vistas.

**Verification:**
- [x] Focused tests: `cargo test --workspace --locked components::patients`, con casos concretos de esta tarea y ejecución no vacía.
- [x] Build or broader check: `cargo check --workspace --all-targets --locked` y `git diff --check`.
- [x] Manual check: Abrir una base temporal de versión 6, migrar, reabrir y comprobar datos de otra cuenta.

**Dependencies:** T02.

**Files likely touched:**
- `src-tauri/migrations/0007_patients.sql`
- `src-tauri/src/platform/sqlite/mod.rs`
- `src-tauri/src/components/patients/adapters/outbound.rs`
- `src-tauri/src/components/patients/adapters/mod.rs`
- `src-tauri/src/components/patients/repository_tests.rs`

**Estimated scope:** M

**Evidencia T03 (2026-11-04):** `cargo test --workspace --locked patients`: 18 pruebas reales correctas (8 de dominio, 8 de almacenamiento, 2 de migración). `cargo test --workspace --locked components::patients`: 16 pruebas reales correctas; el filtro no incluye las dos de platform::sqlite. `cargo check --workspace --all-targets --locked` y `git diff --check` correctos. La prueba `patients_upgrade_six_keeps_existing_data_and_reopens` abre esquema 6, migra y reabre con datos de dos cuentas; `patients_failed_seventh_migration_restores_six_byte_for_byte` verifica reversión del archivo, historial y versión. La primera ejecución detectó un nombre de columna incorrecto en el nuevo test; se corrigió y ambas pruebas pasaron. Almacenamiento cubre aislamiento triple, reapertura, documentos como texto, contacto compartido, secuencias, restricciones, rollback y conservación de intenciones al retirar vistas. No implementa los casos de T04 ni acredita API real.

## C01: Revisión de contratos y persistencia

Después de T01–T03:
- [x] Revisar interfaces, límites y migración antes de ampliar casos de uso.
- [x] Pasan las pruebas centradas de la fase y `cargo test --workspace --locked`.
- [x] `pnpm build:desktop` y `git diff --check` correctos.
- [x] Evidencia registrada con límites explícitos; revisión del usuario completa.

**Estado C01:** aprobado por el usuario el 2026-11-04. [Evidencia, fuentes y límites](evidence-c01.md). La nueva instrucción autoriza T04–T06, sin aprobar C02.

## T04: Registro y consulta de pacientes

**Status:** Done

**Description:** Implementar el recorrido local de registro draft y consulta/búsqueda de fichas. Es el primer caso nativo usable sin API.

**Acceptance criteria:**
- [x] Registrar guarda ficha provisional e intención Outbox de forma atómica.
- [x] Las consultas y búsquedas devuelven solo datos del ámbito local y separan aceptado/provisional.
- [x] Registro y lectura sobreviven al cierre y reapertura sin conexión.

**Verification:**
- [x] Focused tests: `cargo test --workspace --locked components::patients`, con casos concretos de esta tarea y ejecución no vacía.
- [x] Build or broader check: `cargo check --workspace --all-targets --locked` y `git diff --check`.
- [x] Manual check: Registrar una ficha con datos mínimos sin red, buscarla, cerrar y reabrir la base temporal.

**Dependencies:** T03 y revisión del usuario en C01.

**Files likely touched:**
- `src-tauri/src/components/patients/application.rs`
- `src-tauri/src/components/patients/adapters/outbound.rs`
- `src-tauri/src/components/patients/application_tests.rs`
- `src-tauri/src/components/patients/test_support.rs`
- `src-tauri/src/components/patients/mod.rs`

**Estimated scope:** M

**Evidencia T04:** 22 pruebas reales de `components::patients` correctas (6 nuevas de casos locales). Registro/búsqueda/reapertura, draft vacío, nombres repetidos, separación aceptado/provisional, triple ámbito, permisos vigentes y rollback ante fallo en cualquiera de las dos escrituras. `cargo check --workspace --all-targets --locked` y `git diff --check` correctos. Recorrido manual reproducido desde Rust en `register_search_read_and_reopen_work_without_http`. Sin API real.

## T05: Ediciones sucesivas

**Status:** Done

**Description:** Permitir varias ediciones de datos o actividad durante la misma desconexión, con identidad propia por operación y dependencias durables.

**Acceptance criteria:**
- [x] Las ediciones parten de versión aceptada o de la última operación local de la misma entidad y detectan bases antiguas.
- [x] La secuencia conserva contenido/IDs y dependencias tras reabrir; sus versiones previstas no se presentan como aceptadas.
- [x] Un cambio sin diferencia no crea otra operación y la actividad no altera citas.

**Verification:**
- [x] Focused tests: `cargo test --workspace --locked components::patients`, con casos concretos de esta tarea y ejecución no vacía.
- [x] Build or broader check: `cargo check --workspace --all-targets --locked` y `git diff --check`.
- [x] Manual check: Crear y editar varias veces una ficha offline; inspeccionar el orden y las bases tras reabrir.

**Dependencies:** T04.

**Files likely touched:**
- `src-tauri/src/components/patients/application.rs`
- `src-tauri/src/components/patients/domain.rs`
- `src-tauri/src/components/patients/adapters/outbound.rs`
- `src-tauri/src/components/patients/adapters/intents.rs`
- `src-tauri/src/components/patients/application_tests.rs`

**Estimated scope:** M

**Evidencia T05:** 28 pruebas reales de `components::patients` correctas (6 nuevas). Secuencia registro → edición → actividad conservada tras reapertura, incluso con tiempos locales en orden inverso; versiones previstas 0/1/2, payloads e IDs intactos. Bases aceptadas/locales antiguas se rechazan; ausencia/null se conservan; NO_CHANGES no crea operación; conflictos, rechazos y época nueva conservan el trabajo y exigen revisión. La revisión de T06 permite ediciones distintas dependientes de un resultado desconocido, sin reemplazar la operación original. `cargo check --workspace --all-targets --locked` y `git diff --check` correctos. Sin API real ni cambios sobre citas.

## T06: Documentos y completitud

**Status:** Done

**Description:** Implementar AddPatientIdentifier y el cambio explícito de completitud con las reglas de ficha del contrato. Habilita pacientes completos sin crear citas.

**Acceptance criteria:**
- [x] Añadir documentos conserva PatientId y solo crea documentos declarados.
- [x] Completar exige los datos contractuales; guardar draft conserva datos incompletos permitidos.
- [x] Completar después de ediciones/documentos locales conserva las dependencias y la atomicidad.

**Verification:**
- [x] Focused tests: `cargo test --workspace --locked components::patients`, con casos concretos de esta tarea y ejecución no vacía.
- [x] Build or broader check: `cargo check --workspace --all-targets --locked` y `git diff --check`.
- [x] Manual check: Completar una ficha con documento propio y verificar que una ficha sin sus requisitos sigue draft.

**Dependencies:** T05.

**Files likely touched:**
- `src-tauri/src/components/patients/domain.rs`
- `src-tauri/src/components/patients/application.rs`
- `src-tauri/src/components/patients/adapters/outbound.rs`
- `src-tauri/src/components/patients/application_tests.rs`
- `src-tauri/src/components/patients/repository_tests.rs`

**Estimated scope:** M

**Evidencia T06 (2026-11-04):** 37 pruebas reales de `components::patients` correctas; la suite completa final también las ejecuta. Incluyen registro → edición → documento → completitud → reapertura, documentos iniciales y posteriores declarados, requisitos faltantes, responsables aceptados activos/vigentes, identidad del menor, documento rechazado/invalidado, permisos y rollback. Una edición distinta puede depender de un resultado desconocido sin cambiar la operación original; su versión sigue provisional. `cargo check --workspace --all-targets --locked`, formato y Clippy correctos tras ese ajuste. La prueba `completing_without_requirements_keeps_the_draft_and_prior_operations` confirma que el caso inválido conserva draft. [Evidencia de C02](evidence-c02.md). Sin API real.

## C02: Revisión del recorrido de fichas

Después de T04–T06:
- [x] Demostrar registro → edición → documento → completitud sin red y tras reinicio.
- [x] Pasan las pruebas centradas de la fase y `cargo test --workspace --locked`.
- [x] `pnpm build:desktop` y `git diff --check` correctos.
- [x] Evidencia registrada con límites explícitos; revisión del usuario completa.

**Estado C02:** aprobado por el usuario el 2026-11-05: «Apruebo C02». T07 sigue Todo para la siguiente sesión. [Resultados y límites](evidence-c02.md).

## T07: Contactos

**Status:** Done

**Description:** Implementar creación, consulta, edición y actividad de contactos administrativos de la cuenta, reutilizables entre pacientes.

**Acceptance criteria:**
- [x] Los casos guardan contactos e intenciones sin red con control de permisos y versión.
- [x] Los campos de contacto respetan el contrato, incluidos documento y pareja telefónica.
- [x] Un contacto puede servir a varios pacientes sin crear otra identidad ni consultar otra cuenta.

**Verification:**
- [x] Focused tests: `cargo test --workspace --locked components::patients`, con casos concretos de esta tarea y ejecución no vacía.
- [x] Build or broader check: `cargo check --workspace --all-targets --locked` y `git diff --check`.
- [x] Manual check: Crear un contacto, editarlo offline y recuperar sus datos tras reabrir.

**Dependencies:** T06 y revisión del usuario en C02.

**Files likely touched:**
- `src-tauri/src/components/patients/domain.rs`
- `src-tauri/src/components/patients/application.rs`
- `src-tauri/src/components/patients/adapters/outbound_contacts.rs`
- `src-tauri/src/components/patients/contact_tests.rs`
- `src-tauri/src/components/patients/adapters/mod.rs`

**Estimated scope:** M

**Evidencia T07 (2026-11-05):** 44 pruebas reales de `components::patients` correctas (2 nuevas de dominio y 5 de casos de contacto). Creación/búsqueda/consulta/reapertura sin red; identidades distintas con nombre y teléfono iguales; edición y actividad sucesivas con bases, dependencias y versiones previstas; NO_CHANGES sin operación; pareja telefónica y terna documental exigidas; null no retira el nombre; base aceptada/antigua rechazada; permisos, triple ámbito y rollback de intención+Outbox con disparador sintético. `cargo check --workspace --all-targets --locked`, formato, Clippy y `git diff --check` correctos. Sin API real.

## T08: Vínculos

**Status:** Done

**Description:** Implementar la relación paciente/contacto con clave compuesta, consultas en ambos sentidos y cambios de datos o actividad.

**Acceptance criteria:**
- [x] Crear, editar y cambiar actividad usa la clave patientId/contactPersonId y operaciones contractuales.
- [x] Parentesco, responsabilidad, vigencia y representación permanecen separados; el cliente no verifica representación.
- [x] Los vínculos entre entidades aún locales conservan dependencias y aislamiento de cuenta.

**Verification:**
- [x] Focused tests: `cargo test --workspace --locked components::patients`, con casos concretos de esta tarea y ejecución no vacía.
- [x] Build or broader check: `cargo check --workspace --all-targets --locked` y `git diff --check`.
- [x] Manual check: Vincular el mismo contacto a dos pacientes y consultar ambas direcciones, incluida vigencia.

**Dependencies:** T07.

**Files likely touched:**
- `src-tauri/src/components/patients/domain.rs`
- `src-tauri/src/components/patients/application.rs`
- `src-tauri/src/components/patients/adapters/outbound_relationships.rs`
- `src-tauri/src/components/patients/relationship_tests.rs`
- `src-tauri/src/components/patients/adapters/mod.rs`

**Estimated scope:** M

**Evidencia T08 (2026-11-05):** 49 pruebas reales de `components::patients` correctas (1 nueva de dominio y 4 de vínculos). Creación con clave compuesta y dependencias de las creaciones locales de paciente/contacto; consultas en ambas direcciones con nombres y teléfono actuales de lectura (no persistidos en el payload); nombres iguales no fusionan identidades; ediciones sucesivas y actividad conservan bases, versiones previstas y vigencia; null solo retira validUntil; vínculo duplicado e identificadores inexistentes rechazados. Un menor completa su ficha sin red con responsable provisional y la completitud incorpora las operaciones del vínculo como dependencias. `cargo check --workspace --all-targets --locked`, formato, Clippy y `git diff --check` correctos. Sin API real.

## T09: Protección de fichas completas

**Status:** Done

**Description:** Comprobar los efectos de una edición de contacto o vínculo sobre las fichas completas conocidas. Habilita menores y contactos compartidos sin degradación silenciosa.

**Acceptance criteria:**
- [x] El impacto inválido devuelve COMPLETENESS_CONFLICT con referencias locales afectadas y no guarda el cambio.
- [x] Pasar expresamente las fichas afectadas a draft permite preparar el cambio con sus dependencias.
- [x] Se respeta el límite contractual de dependencias sin truncarlas ni afirmar validez futura de una cita.

**Verification:**
- [x] Focused tests: `cargo test --workspace --locked components::patients`, con casos concretos de esta tarea y ejecución no vacía.
- [x] Build or broader check: `cargo check --workspace --all-targets --locked` y `git diff --check`.
- [x] Manual check: Intentar retirar el único teléfono responsable de dos fichas completas; pasarlas a draft y repetir de forma explícita.

**Dependencies:** T08.

**Files likely touched:**
- `src-tauri/src/components/patients/domain.rs`
- `src-tauri/src/components/patients/application.rs`
- `src-tauri/src/components/patients/adapters/outbound_contacts.rs`
- `src-tauri/src/components/patients/adapters/outbound_relationships.rs`
- `src-tauri/src/components/patients/completeness_tests.rs`

**Estimated scope:** M

**Evidencia T09 (2026-11-05):** 53 pruebas reales de `components::patients` correctas (4 nuevas). Retirar el único teléfono responsable o inactivar contacto/vínculo devuelve COMPLETENESS_CONFLICT con las IDs locales afectadas (dos fichas compartidas listan ambas) y no guarda nada; un adulto con documento propio y teléfono propio nunca se ve afectado. Pasar las fichas a draft permite repetir la misma edición, que incorpora la operación explícita de completitud como dependencia. Ocho fichas en draft con un contacto compartido exceden el límite de ocho dependencias y la preparación se rechaza sin perder pasos previos. `cargo check --workspace --all-targets --locked`, formato, Clippy y `git diff --check` correctos. Sin API real ni afirmaciones sobre citas.

## T19: Impacto de completitud por ficha

**Status:** Done

**Description:** Corregir `responsible_impact` para que el impacto se calcule por ficha y no por vínculo: una ficha completa solo entra en conflicto cuando el cambio le retira el soporte de **todos** sus responsables vigentes, y solo cuentan vínculos cuyas operaciones aún pueden dependerse (`can_depend`), coherente con `completion_support`. Las banderas futuras del contacto editado se calculan una vez fuera del closure y los errores de almacenamiento propagan en lugar de interpretarse como «sin teléfono». Se extrae la cola común de protección (error `COMPLETENESS_CONFLICT` con IDs, fusión de dependencias y rechazo de más de ocho) que hoy está duplicada en los `prepare` de contacto y vínculo.

**Acceptance criteria:**
- [ ] Un menor completo con dos responsables con teléfono: retirar el teléfono de uno no genera conflicto; retirar después el del otro devuelve `COMPLETENESS_CONFLICT` con la ID de la ficha y no guarda nada.
- [ ] Un vínculo cuya última operación no puede dependerse (p. ej. rechazada) no sostiene completitud ni dispara conflicto ni dependencia en el cálculo de impacto.
- [ ] La evaluación futura reutiliza las filas ya leídas (sin relecturas por vínculo) y no contiene `.ok()`/`unwrap_or` que conviertan fallos de almacenamiento en «sin teléfono».

**Verification:**
- [ ] Focused tests: `cargo test --workspace --locked components::patients`
- [ ] Build or broader check: `cargo check --workspace --all-targets --locked` y `git diff --check`
- [ ] Manual check: En la suite, un menor con madre y padre como responsables completa su ficha; retirar el teléfono del padre no bloquea la edición y retirar después el de la madre bloquea con la ID afectada.

**Dependencies:** T09.

**Files likely touched:**
- `src-tauri/src/components/patients/adapters/documents.rs`
- `src-tauri/src/components/patients/adapters/outbound_contacts.rs`
- `src-tauri/src/components/patients/adapters/outbound_relationships.rs`
- `src-tauri/src/components/patients/completeness_tests.rs`

**Estimated scope:** M

## T20: Dependencia de completitud tras edición posterior

**Status:** Done

**Description:** Hoy la operación `SetPatientCompletion` solo se incorpora como dependencia cuando es la última intención local de la ficha. Si la ficha se edita después de pasar a draft, la edición del contacto o vínculo pierde esa dependencia y puede ordenarse antes que el paso a draft. Localizar la operación de completitud explícita entre los cambios locales de la ficha y cablearla como dependencia del cambio que rompe requisitos, sin duplicar IDs y respetando el límite de ocho.

**Acceptance criteria:**
- [ ] Menor completo → paso a draft → edición de la ficha → retiro del teléfono del responsable: la preparación se acepta y `depends_on` incluye la operación `SetPatientCompletion`.
- [ ] El mismo retiro con la ficha de nuevo en complete devuelve `COMPLETENESS_CONFLICT` con su ID.
- [ ] `depends_on` no duplica IDs y preparar con más de ocho dependencias se rechaza sin pérdidas (regresión).

**Verification:**
- [ ] Focused tests: `cargo test --workspace --locked components::patients`
- [ ] Build or broader check: `cargo check --workspace --all-targets --locked` y `git diff --check`
- [ ] Manual check: Recorrer en la suite la secuencia draft → edición → retiro e inspeccionar `depends_on` de la operación resultante tras reabrir.

**Dependencies:** T19.

**Files likely touched:**
- `src-tauri/src/components/patients/adapters/documents.rs`
- `src-tauri/src/components/patients/adapters/intents.rs`
- `src-tauri/src/components/patients/completeness_tests.rs`

**Estimated scope:** S

## T21: RelationshipPatch sin retiro de responsabilidad

**Status:** Done

**Description:** El contrato fija que null solo retira `validUntil`; hoy `is_responsible_contact: Clear` se aplica como `false` y el payload emite `isResponsibleContact: null`. Rechazar `FieldChange::Clear` de `is_responsible_contact` en `RelationshipPatch::apply` (con mensaje claro, como la vigencia inicial), alinear el rechazo de `relationship_kind` Clear y eliminar la rama muerta del payload.

**Acceptance criteria:**
- [ ] `is_responsible_contact: Clear` se rechaza con `VALIDATION_ERROR`; `Set(true)`, `Set(false)` y `Keep` conservan su comportamiento.
- [ ] El payload contractual jamás emite `isResponsibleContact: null`.
- [ ] El patch vacío sigue rechazándose y los casos de vínculo existentes pasan sin cambios (regresión).

**Verification:**
- [ ] Focused tests: `cargo test --workspace --locked components::patients`
- [ ] Build or broader check: `cargo check --workspace --all-targets --locked` y `git diff --check`
- [ ] Manual check: Aplicar un patch con `isResponsibleContact: null` en dominio y comprobar el rechazo junto a un `Set(false)` válido.

**Dependencies:** T19; orden T19 → T20 → T21.

**Files likely touched:**
- `src-tauri/src/components/patients/domain.rs`
- `src-tauri/src/components/patients/adapters/outbound_relationships.rs`
- `src-tauri/src/components/patients/relationship_tests.rs`

**Estimated scope:** S

## C03: Revisión de contactos y responsables

Después de T07–T09 y T19–T21:
- [ ] Demostrar menores/contactos compartidos y cambios que rompen requisitos.
- [x] Pasan las pruebas centradas de la fase y `cargo test --workspace --locked`.
- [x] `pnpm build:desktop` y `git diff --check` correctos.
- [x] Verificaciones de T19–T21 registradas y suite de la fase reejecutada tras las correcciones.
- [ ] Evidencia registrada con límites explícitos; revisión del usuario completa.

## T10: Proyecciones remotas

**Status:** Done

**Description:** Traducir los cuatro estados remotos al almacenamiento aceptado del componente con RemoteProjector, sin modificar el transporte ni las intenciones.

**Acceptance criteria:**
- [x] Upsert traduce las cuatro entidades y valida ámbito y claves; repeticiones/versiones antiguas no retroceden vistas.
- [x] Remove/reset afectan solo las vistas del ámbito y conservan intenciones.
- [x] Un fallo local revierte el grupo/página y su avance de cursor dentro de la transacción recibida.

**Verification:**
- [x] Focused tests: `cargo test --workspace --locked components::patients`, con casos concretos de esta tarea y ejecución no vacía.
- [x] Build or broader check: `cargo check --workspace --all-targets --locked` y `git diff --check`.
- [x] Manual check: Aplicar un grupo con ficha/documento/contacto/vínculo, repetirlo y reconstruir sin perder un cambio local.

**Dependencies:** T09 y revisión del usuario en C03.

**Files likely touched:**
- `src-tauri/src/components/patients/adapters/projector.rs`
- `src-tauri/src/components/patients/adapters/mod.rs`
- `src-tauri/src/components/patients/projection_tests.rs`

**Estimated scope:** M

**Evidencia T10 (2026-11-05):** 60 pruebas reales de `components::patients` correctas (4 nuevas de proyección). Grupos con las cuatro entidades aplicados y repetidos; versiones antiguas no retroceden vistas; remove y reset borran solo las vistas del ámbito, conservan la intención local (`patient_local`) y las vistas de otra cuenta; la reaplicación reconstruye vistas aceptadas. Un vínculo con vigencia inválida revierte el grupo completo y los efectos de cursor/inbox (`sync_inbox`/`sync_remote_items` en cero). Estados con `accountId` de otra cuenta se rechazan. La validación reutiliza las reglas de dominio (documento, pareja telefónica, nombre, vigencia). `cargo check --workspace --all-targets --locked`, formato, Clippy y `git diff --check` correctos. Sin API real.

## T11: Conciliación de cambios locales

**Status:** Done

**Description:** Combinar vistas y estado de recibos para exponer cambios locales y su estado, sin dar por descargado lo que solo tiene recibo.

**Acceptance criteria:**
- [x] Accepted conserva el provisional hasta alcanzar su versión en la vista; luego retira solo ese provisional.
- [x] Conflictos, rechazos y dependencias bloqueadas conservan datos, operaciones e IDs para revisión.
- [x] Las correcciones usan operaciones nuevas por casos específicos; un resultado desconocido exige recuperar el original.

**Verification:**
- [x] Focused tests: `cargo test --workspace --locked components::patients`, con casos concretos de esta tarea y ejecución no vacía.
- [x] Build or broader check: `cargo check --workspace --all-targets --locked` y `git diff --check`.
- [x] Manual check: Simular accepted antes de la proyección, una aceptación parcial y una secuencia cuyo primer paso se rechaza.

**Dependencies:** T10.

**Files likely touched:**
- `src-tauri/src/components/patients/adapters/intents.rs`
- `src-tauri/src/components/patients/adapters/outbound.rs`
- `src-tauri/src/components/patients/adapters/outbound_contacts.rs`
- `src-tauri/src/components/patients/adapters/outbound_relationships.rs`
- `src-tauri/src/components/patients/adapters/documents.rs`
- `src-tauri/src/components/patients/reconciliation_tests.rs`

**Estimated scope:** M

**Evidencia T11 (2026-11-05):** 63 pruebas reales de `components::patients` correctas (3 nuevas de conciliación). Un recibo `accepted` sin descarga no retira el provisional; cuando la vista aceptada alcanza la versión confirmada del recibo se retira solo ese provisional (secuencia parcial: el pendiente siguiente sigue visible), conservando operación y recibo en la consulta. Rechazos conservan la secuencia completa con su código remoto, bloquean las ediciones dependientes (`DEPENDENCY_FAILED`) sin impedir operaciones ajenas. Resultados `unknown` conservan ID y contenido y admiten una edición distinta que depende del original. `cargo check --workspace --all-targets --locked`, formato, Clippy y `git diff --check` correctos. Sin API real.

## T12: Integración del componente

**Status:** Done

**Description:** Incorporar PatientsProjector a la composición usada por el ciclo normal y SSE y comprobar los recorridos del coordinador con dos bases/dispositivos.

**Acceptance criteria:**
- [x] La composición usa los proyectores existentes y Pacientes con la misma transacción, sin cambiar sus reglas de propiedad.
- [x] Dos dispositivos ejercen aceptación, conflicto y recuperación de respuesta perdida sin duplicar efectos.
- [x] Captura/retiradas/reinicio conservan intenciones y otra cuenta; una caída de API/SSE no bloquea los casos locales.

**Verification:**
- [x] Focused tests: `cargo test --workspace --locked patients_integration_tests`; ejecutar también `cargo test --workspace --locked components::patients` y las regresiones de sync afectadas.
- [x] Build or broader check: `cargo check --workspace --all-targets --locked` y `git diff --check`.
- [x] Manual check: Ejecutar el recorrido offline → reinicio → envío → descarga entre dos bases temporales con servidor local sintético.

**Dependencies:** T11.

**Files likely touched:**
- `src-tauri/src/components/projector.rs`
- `src-tauri/src/components/sync/patients_integration_tests.rs`
- `src-tauri/src/components/sync/mod.rs`
- `src-tauri/src/components/patients/test_support.rs`
- `src-tauri/src/components/patients/mod.rs`

**Estimated scope:** M

**Evidencia T12 (2026-11-05):** `patients_integration_tests`: 1 prueba real de extremo a extremo con servidor HTTP sintético correcta. Recorrido: registro de menor con responsable y completitud offline (4 operaciones) → reapertura duradera → ciclo de sincronización que envía, acepta y descarga las cuatro entidades (provisional retirado, vistas aceptadas v2/v1/v1, `patient_local` y recibos conservados) → edición de contacto desde base aceptada que recibe conflicto remoto conservando la intención y mostrando `VERSION_CONFLICT` → descarga de un retiro remoto que borra la vista aceptada sin tocar las intenciones → captura nueva que reconstruye vistas (contacto v2) conservando trabajos locales y recibos. Regresiones de sync y `components::patients` (63 pruebas) correctas. `cargo check --workspace --all-targets --locked`, formato, Clippy, `git diff --check` y `pnpm build:desktop` correctos. Sin API real (servidor sintético local).

## C04: Revisión de sincronización

Después de T10–T12:
- [ ] Demostrar que un resultado parcial o perdido no duplica efectos ni borra cambios.
- [x] Pasan las pruebas centradas de la fase y `cargo test --workspace --locked`.
- [x] `pnpm build:desktop` y `git diff --check` correctos.
- [ ] Evidencia registrada con límites explícitos; revisión del usuario completa.

**Estado C04:** entregado el 2026-11-05; pendiente de revisión del usuario. [Evidencia y límites](evidence-c04.md).

## T13: Comandos de fichas

**Status:** Done

**Description:** Añadir adaptadores de entrada Tauri específicos para consultas de pacientes/documentos y las cinco escrituras de esos grupos, sin pantalla React.

**Acceptance criteria:**
- [x] Los DTO camelCase admiten solo los campos del caso y conservan ausencia/null.
- [x] Los comandos derivan el ámbito de la sesión y comprueban acceso/allowedActions para propietario y asistente.
- [x] Las salidas separan aceptado/provisional y estados sin SQL, credenciales o errores internos crudos.

**Verification:**
- [x] Focused tests: `cargo test --workspace --locked components::patients`, con casos concretos de esta tarea y ejecución no vacía.
- [x] Build or broader check: `cargo check --workspace --all-targets --locked` y `git diff --check`.
- [x] Manual check: Ejercitar DTO y casos de entrada con sesión sintética, sin sesión y con permisos insuficientes.

**Dependencies:** T12 y revisión del usuario en C04.

**Files likely touched:**
- `src-tauri/src/components/patients/adapters/inbound_profiles.rs`
- `src-tauri/src/components/patients/ipc_profile_tests.rs`
- `src-tauri/src/components/patients/adapters/mod.rs`

**Estimated scope:** M

**Evidencia T13 (2026-11-05):** 65 pruebas reales de `components::patients` correctas (2 nuevas de IPC). Ocho comandos (`patients_register_patient`, `patients_update_patient_profile`, `patients_set_patient_completion`, `patients_set_patient_profile_status`, `patients_add_patient_identifier`, `patients_list_patients`, `patients_get_patient`, `patients_list_patient_identifiers`). DTO con `deny_unknown_fields` que rechazan campos extra, inyección de ámbito y bases ambiguas; ausencia conserva, null retira, valor fija; `issuingCountry` por defecto VE. Sin sesión seleccionada devuelve `AUTHENTICATION_REQUIRED`; el ámbito sale de la sesión y las salidas no exponen payload interno, época, fingerprint ni identificadores de dispositivo/cuenta. `cargo check --workspace --all-targets --locked`, formato, Clippy y `git diff --check` correctos. Sin API real ni pantalla React.

## T14: Comandos de contactos

**Status:** Done

**Description:** Añadir adaptadores de entrada específicos para consultas de contactos/vínculos y sus seis escrituras, conservando el mismo control nativo.

**Acceptance criteria:**
- [x] Los comandos traducen los casos de contactos/vínculos con claves y datos contractuales.
- [x] Cuenta y permisos proceden de la sesión; no se aceptan envoltorios remotos ni ámbitos arbitrarios.
- [x] Las pruebas de entrada cubren pares telefónicos, vigencia y COMPLETENESS_CONFLICT sin pantalla React.

**Verification:**
- [x] Focused tests: `cargo test --workspace --locked components::patients`, con casos concretos de esta tarea y ejecución no vacía.
- [x] Build or broader check: `cargo check --workspace --all-targets --locked` y `git diff --check`.
- [x] Manual check: Ejercitar las entradas de contacto compartido y vínculo de un menor, incluida una edición inválida.

**Dependencies:** T13.

**Files likely touched:**
- `src-tauri/src/components/patients/adapters/inbound_contacts.rs`
- `src-tauri/src/components/patients/ipc_contact_tests.rs`
- `src-tauri/src/components/patients/adapters/mod.rs`

**Estimated scope:** M

**Evidencia T14 (2026-11-05):** 67 pruebas reales de `components::patients` correctas (2 nuevas de IPC). Diez comandos (seis escrituras y cuatro consultas de contactos/vínculos). DTO que rechazan campos extra y campos de verificación de representación; par telefónico y terna documental; `validUntil` tri-estado (ausencia conserva, null retira, valor fija); `isResponsibleContact` por defecto false; flujo de menor con contacto compartido que completa y `COMPLETENESS_CONFLICT` con IDs afectadas a través del mismo camino de comando. `cargo check --workspace --all-targets --locked`, formato, Clippy y `git diff --check` correctos. Sin API real ni pantalla React.

## T15: Registro y permisos Tauri

**Status:** Done

**Description:** Registrar los 18 comandos y permisos concretos en la composición de la aplicación, sin añadir cliente TypeScript ni pantalla.

**Acceptance criteria:**
- [x] Las once escrituras y siete consultas tienen registro y permisos explícitos coherentes.
- [x] La aplicación inicia con los comandos previstos y deniega comandos o argumentos fuera de su contrato.
- [x] El ámbito efectivo procede de la sesión seleccionada y las verificaciones no exponen secretos.

**Verification:**
- [x] Focused tests: `cargo test --workspace --locked components::patients`, con casos concretos de esta tarea y ejecución no vacía.
- [x] Build or broader check: `pnpm build:desktop` y `git diff --check`.
- [x] Manual check: Abrir el ejecutable de desarrollo y comprobar una invocación autorizada y una denegada mediante el mecanismo de prueba IPC definido en T15, sin añadir UI de producto.

**Dependencies:** T14.

**Files likely touched:**
- `src-tauri/src/lib.rs`
- `src-tauri/build.rs`
- `src-tauri/capabilities/default.json`
- `src-tauri/src/components/patients/ipc_registration_tests.rs`
- `src-tauri/src/components/patients/mod.rs`

**Estimated scope:** M

**Evidencia T15 (2026-11-05):** 69 pruebas reales de `components::patients` correctas (2 nuevas de registro). Los 18 comandos registrados en `generate_handler!`, `build.rs` y `capabilities/default.json`, con 18 permisos autogenerados verificados; la prueba de coherencia recorre manifiesto, handler y capability por comando. Las consultas funcionan con los últimos permisos válidos de la sesión y una sesión sin la acción recibe `PERMISSION_DENIED` sin efectos ni operaciones. `pnpm build:desktop` genera el ejecutable y los permisos; suite completa (228 unitarias, 5 de integración, 1 de documentación), formato, Clippy y `git diff --check` correctos. Invocación real sobre el ejecutable no automatizada: el mecanismo de prueba IPC usa la sesión sintética. Sin pantalla React.

## C05: Revisión de la interfaz nativa

Después de T13–T15:
- [ ] Comprobar comandos, DTO y permisos sin añadir pantalla React.
- [ ] Pasan las pruebas centradas de la fase y `cargo test --workspace --locked`.
- [ ] `pnpm build:desktop` y `git diff --check` correctos.
- [ ] Evidencia registrada con límites explícitos; revisión del usuario completa.

## T16: Contrato del consumidor

**Status:** Done

**Description:** Ampliar las pruebas de consumidor con las once acciones de Pacientes y las cuatro proyecciones existentes, usando el bundle fijo como autoridad.

**Acceptance criteria:**
- [x] Las fixtures contrastan acciones, payloadModels, claves, versiones y disponibilidad offline con capacidades/esquemas.
- [x] Las pruebas detectan campos extra, ausencia/null y formas incorrectas sin sustituir la validación del servidor.
- [x] Las cinco huellas del bundle siguen intactas y pasan las pruebas existentes.

**Verification:**
- [x] Focused tests: `cargo test --workspace --locked components::sync::contract_tests`.
- [x] Build or broader check: `cargo check --workspace --all-targets --locked` y `git diff --check`.
- [x] Manual check: Revisar una fixture de cada grupo contra la guía y sus esquemas, incluida la clave compuesta del vínculo.

**Dependencies:** T15 y revisión del usuario en C05.

**Files likely touched:**
- `src-tauri/src/components/sync/contract_tests.rs`
- `src-tauri/src/components/sync/fixtures/patients-operations.json`

**Estimated scope:** S

**Evidencia T16 (2026-11-05):** 5 pruebas reales de `components::sync::contract_tests` correctas (1 nueva). La fixture `patients-operations.json` cubre las once acciones con payloads, claves (incluida la compuesta `patientId/contactPersonId` del vínculo), versiones esperadas y dependencias encadenadas; cada operación pasa el chequeo estructural contra `accepted-operations.schema.json`, coincide con `payloadModels` de capacidades, no es online-only, redondea por `OperationEnvelope` y rechaza campos extra. La clave compuesta coincide con la del grupo remoto fijado y un campo faltante falla la traducción. Suite completa (229 unitarias, 5 de integración, 1 de documentación), formato, Clippy y `git diff --check` correctos. No acredita API real.

## T17: API de desarrollo real

**Status:** Pending real environment

**Description:** Verificar el componente nativo con Nightingale API/OIDC de desarrollo, dos dispositivos/bases aislados y datos ficticios. No modificar otros repositorios ni servicios.

**Acceptance criteria:**
- [ ] El recorrido nativo guarda offline, reabre, sincroniza y descarga cambios aceptados usando los adaptadores reales.
- [ ] Dos dispositivos producen un conflicto y conservan las intenciones; verificar también el recorrido de contacto responsable.
- [ ] Se registran comandos, entorno, resultados y datos/dispositivos sintéticos creados, sin secretos; un bloqueo no se cuenta como éxito.

**Verification:**
- [ ] Focused tests: prueba explícita de desarrollo real con el patrón manual/ignorado del repositorio. Documentar su comando exacto antes de ejecutarla; las pruebas automáticas no acreditan esta verificación.
- [ ] Build or broader check: `cargo test --workspace --locked components::patients` y `git diff --check`.
- [ ] Manual check: Completar el acceso OIDC cuando corresponda y verificar la aceptación/descarga y conflicto reales entre dos dispositivos aislados. Si falta el entorno, mantener esta tarea pendiente de su prueba real.

**Dependencies:** T16.

**Files likely touched:**
- `src-tauri/src/components/patients/development_tests.rs`
- `src-tauri/src/components/patients/mod.rs`
- `tasks/P-20261104-01-patients-core/todo.md`

**Estimated scope:** M

**Evidencia T17 (2026-11-05):** preparada; primer intento real ejecutado y bloqueado por entorno. La API de desarrollo respondió (`GET /v1/auth/config` correcto), pero el emisor OIDC anunciado (`https://127.0.0.1:18443/realms/nightingale-test`) usa certificado autofirmado y `NIGHTINGALE_DEV_CA_CERT` no estaba configurado: `OidcProvider::discover` falló con `ProviderUnavailable` antes de abrir el navegador. Solución de entorno documentada: exportar el CA de desarrollo en `NIGHTINGALE_DEV_CA_CERT` (se extrajo el certificado del servidor a `target/dev-oidc-ca.pem` solo para la ejecución de prueba; no se añade al repo). El intento se detuvo a la espera del login del navegador; el usuario retomará T17 después. Un bloqueo no cuenta como éxito: T17 sigue pendiente. `components::patients::development_tests::manual_p17_responsible_contact_flow_with_two_devices` implementa el recorrido con los adaptadores reales: login OIDC por navegador, dos dispositivos/perfiles aislados activados sobre la cuenta sintética, registro offline de menor con responsable y completitud, ciclo de sincronización con aceptación y descarga (provisional retirado, vistas aceptadas), descarga en el segundo dispositivo y edición concurrente que conserva ambas intenciones con conflicto. Compila y queda `#[ignore]`. Comando exacto documentado en la cabecera del archivo y en `docs/operations/development.md`: `$env:NIGHTINGALE_API_BASE_URL = "<url>"; cargo test --workspace --locked -- --ignored components::patients::development_tests`. **Pendiente de entorno de desarrollo accesible desde esta máquina; un bloqueo no cuenta como éxito y las pruebas locales no la sustituyen.**

## T18: Documentación y calidad

**Status:** Done

**Description:** Documentar propiedad, API pública, puertos, reglas, adaptadores y límites del componente y entregar evidencia de calidad y prueba remota para C06.

**Acceptance criteria:**
- [x] La guía de Pacientes y las guías de integración describen la implementación final y separan lo verificado de lo pendiente.
- [x] Pasan calidad, build de escritorio y diff; no se alteran contrato, especificación o trabajo ajeno.
- [ ] El informe de cierre incluye prueba API real y mantiene C06 pendiente de revisión del usuario.

**Verification:**
- [x] Focused tests: `cargo test --workspace --locked components::patients`, `cargo test --workspace --locked components::sync::contract_tests` y `cargo test --workspace --locked patients_integration_tests`.
- [x] Build or broader check: `pnpm quality`, `pnpm build:desktop` y `git diff --check`.
- [x] Manual check: Reproducir el recorrido nativo documentado y revisar la evidencia real y sus límites antes de presentar C06.

**Dependencies:** T17.

**Files likely touched:**
- `docs/technical/components/patients.md`
- `docs/operations/core-next-tracks.md`
- `docs/operations/development.md`
- `tasks/P-20261104-01-patients-core/plan.md`
- `tasks/P-20261104-01-patients-core/todo.md`

**Estimated scope:** M

**Evidencia T18 (2026-11-05):** guía del componente `docs/technical/components/patients.md` (propiedad, datos, casos, reglas, conciliación, proyector, comandos, pruebas y fuentes); `core-next-tracks.md` marca el track Pacientes como implementado con sus exclusiones; `development.md` actualiza el estado y documenta el comando opt-in de la prueba real. `pnpm quality` (build, fmt, clippy, suite completa), `pnpm build:desktop` y `git diff --check` correctos; huellas del bundle intactas. La prueba API real sigue pendiente de entorno (T17); C06 queda pendiente de revisión del usuario.

## C06: Aceptación de Pacientes

Después de T16–T18:
- [ ] Revisar resultados locales, prueba API real y límites expresos.
- [ ] Pasan las pruebas centradas de la fase y `cargo test --workspace --locked`.
- [ ] `pnpm build:desktop` y `git diff --check` correctos.
- [ ] Evidencia registrada con límites explícitos; revisión del usuario completa.
