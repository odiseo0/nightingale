# Task List: P-20261104-01-patients-core

**Plan:** [plan.md](plan.md)
**Status del plan:** Approved
**Updated:** 2026-11-04

## Instrucción vigente

Aprobación del usuario el 2026-11-04: «Lo veo bien, apruebo el plan. No comiences el desarrollo».

No iniciar desarrollo hasta una nueva instrucción del usuario. T01–T18 permanecen Todo. C01–C06 no están aprobados. La preparación de estos archivos no cuenta como ejecución de T01.

**Reserva de migración:** 0007_patients; no se ha creado ni registrado.
**Responsable de integración:** quien ejecute este plan; un único responsable para migraciones, composición, comandos y permisos.
**Orden:** secuencial; cada checkpoint requiere revisión del usuario antes de la fase siguiente.
**Reglas comunes:** preservar trabajo ajeno; aplicar minimal-code antes de código y docs-first-framework-code para APIs de bibliotecas/frameworks. Usar datos ficticios.

Los filtros nuevos indicados como `Not available yet` se fijarán en la tarea indicada y deberán seleccionar pruebas reales antes de acreditar un resultado. Ninguna verificación de abajo se ha ejecutado como parte del desarrollo de este plan.

## T01: Contratos del componente

**Status:** Todo

**Description:** Concretar en el plan los contratos internos de las once escrituras y siete consultas, sin implementar código ni alterar el bundle de API. Habilita dominio, almacenamiento e IPC con una misma definición.

**Acceptance criteria:**
- [ ] Las once acciones, sus claves, versiones, payloads y permisos corresponden al contrato fijo.
- [ ] Los DTO separan aceptado/provisional y definen versión aceptada o última operación local como base de edición.
- [ ] Los contratos fijan consultas, errores y dependencias sin añadir funciones fuera del alcance aprobado.

**Verification:**
- [ ] Focused tests: `cargo test --workspace --locked components::sync::contract_tests`; cotejo de esquemas y capacidades con los contratos documentados.
- [ ] Build or broader check: `git diff --check` y revisión de enlaces del plan.
- [ ] Manual check: Revisar una ficha draft, un menor con responsable y una secuencia de ediciones contra los contratos escritos.

**Dependencies:** Ninguna; requiere autorización de inicio.

**Files likely touched:**
- `tasks/P-20261104-01-patients-core/plan.md`
- `tasks/P-20261104-01-patients-core/todo.md`

**Estimated scope:** S

## T02: Dominio de fichas y formatos

**Status:** Todo

**Description:** Crear las reglas puras de estados y formatos que usan los casos de Pacientes. Reutilizar Chrono 0.4.45 para fechas con funciones mínimas y sin actualizar otras dependencias.

**Acceptance criteria:**
- [ ] El dominio valida estados, documentos, pares telefónicos y fechas sin red ni almacenamiento.
- [ ] Las ediciones distinguen ausencia/null y no permiten atribuir verificación al cliente.
- [ ] Las pruebas cubren datos válidos e inválidos y conservan el texto significativo de documentos.

**Verification:**
- [ ] Focused tests: Not available yet - define in T02: `cargo test --workspace --locked components::patients`, con casos concretos de esta tarea y ejecución no vacía.
- [ ] Build or broader check: `cargo check --workspace --all-targets --locked` y `git diff --check`.
- [ ] Manual check: Ejecutar ejemplos válidos e inválidos de ficha/documento/teléfono desde las pruebas puras.

**Dependencies:** T01.

**Files likely touched:**
- `src-tauri/src/components/patients/domain.rs`
- `src-tauri/src/components/patients/mod.rs`
- `src-tauri/src/components/mod.rs`
- `src-tauri/Cargo.toml`
- `Cargo.lock`

**Estimated scope:** M

## T03: Persistencia local

**Status:** Todo

**Description:** Crear las tablas e índices del componente y su acceso con ámbito, conservando las migraciones previas. Habilita los casos nativos y la persistencia durable de intenciones.

**Acceptance criteria:**
- [ ] 0007_patients añade cuatro vistas aceptadas y una tabla de cambios locales con el ámbito previsto.
- [ ] La migración desde esquema 6 conserva datos y un fallo revierte sin pérdida.
- [ ] Las consultas no cruzan cuenta/usuario/dispositivo y las intenciones no dependen de cascadas sobre vistas.

**Verification:**
- [ ] Focused tests: Not available yet - define in T03: `cargo test --workspace --locked components::patients`, con casos concretos de esta tarea y ejecución no vacía.
- [ ] Build or broader check: `cargo check --workspace --all-targets --locked` y `git diff --check`.
- [ ] Manual check: Abrir una base temporal de versión 6, migrar, reabrir y comprobar datos de otra cuenta.

**Dependencies:** T02.

**Files likely touched:**
- `src-tauri/migrations/0007_patients.sql`
- `src-tauri/src/platform/sqlite/mod.rs`
- `src-tauri/src/components/patients/adapters/outbound.rs`
- `src-tauri/src/components/patients/adapters/mod.rs`
- `src-tauri/src/components/patients/repository_tests.rs`

**Estimated scope:** M

## C01: Revisión de contratos y persistencia

Después de T01–T03:
- [ ] Revisar interfaces, límites y migración antes de ampliar casos de uso.
- [ ] Pasan las pruebas centradas de la fase y `cargo test --workspace --locked`.
- [ ] `pnpm build:desktop` y `git diff --check` correctos.
- [ ] Evidencia registrada con límites explícitos; revisión del usuario completa.

## T04: Registro y consulta de pacientes

**Status:** Todo

**Description:** Implementar el recorrido local de registro draft y consulta/búsqueda de fichas. Es el primer caso nativo usable sin API.

**Acceptance criteria:**
- [ ] Registrar guarda ficha provisional e intención Outbox de forma atómica.
- [ ] Las consultas y búsquedas devuelven solo datos del ámbito local y separan aceptado/provisional.
- [ ] Registro y lectura sobreviven al cierre y reapertura sin conexión.

**Verification:**
- [ ] Focused tests: Not available yet - define in T04: `cargo test --workspace --locked components::patients`, con casos concretos de esta tarea y ejecución no vacía.
- [ ] Build or broader check: `cargo check --workspace --all-targets --locked` y `git diff --check`.
- [ ] Manual check: Registrar una ficha con datos mínimos sin red, buscarla, cerrar y reabrir la base temporal.

**Dependencies:** T03 y revisión del usuario en C01.

**Files likely touched:**
- `src-tauri/src/components/patients/application.rs`
- `src-tauri/src/components/patients/adapters/outbound.rs`
- `src-tauri/src/components/patients/application_tests.rs`
- `src-tauri/src/components/patients/test_support.rs`
- `src-tauri/src/components/patients/mod.rs`

**Estimated scope:** M

## T05: Ediciones sucesivas

**Status:** Todo

**Description:** Permitir varias ediciones de datos o actividad durante la misma desconexión, con identidad propia por operación y dependencias durables.

**Acceptance criteria:**
- [ ] Las ediciones parten de versión aceptada o de la última operación local de la misma entidad y detectan bases antiguas.
- [ ] La secuencia conserva contenido/IDs y dependencias tras reabrir; sus versiones previstas no se presentan como aceptadas.
- [ ] Un cambio sin diferencia no crea otra operación y la actividad no altera citas.

**Verification:**
- [ ] Focused tests: Not available yet - define in T05: `cargo test --workspace --locked components::patients`, con casos concretos de esta tarea y ejecución no vacía.
- [ ] Build or broader check: `cargo check --workspace --all-targets --locked` y `git diff --check`.
- [ ] Manual check: Crear y editar varias veces una ficha offline; inspeccionar el orden y las bases tras reabrir.

**Dependencies:** T04.

**Files likely touched:**
- `src-tauri/src/components/patients/application.rs`
- `src-tauri/src/components/patients/domain.rs`
- `src-tauri/src/components/patients/adapters/outbound.rs`
- `src-tauri/src/components/patients/adapters/intents.rs`
- `src-tauri/src/components/patients/application_tests.rs`

**Estimated scope:** M

## T06: Documentos y completitud

**Status:** Todo

**Description:** Implementar AddPatientIdentifier y el cambio explícito de completitud con las reglas de ficha del contrato. Habilita pacientes completos sin crear citas.

**Acceptance criteria:**
- [ ] Añadir documentos conserva PatientId y solo crea documentos declarados.
- [ ] Completar exige los datos contractuales; guardar draft conserva datos incompletos permitidos.
- [ ] Completar después de ediciones/documentos locales conserva las dependencias y la atomicidad.

**Verification:**
- [ ] Focused tests: Not available yet - define in T06: `cargo test --workspace --locked components::patients`, con casos concretos de esta tarea y ejecución no vacía.
- [ ] Build or broader check: `cargo check --workspace --all-targets --locked` y `git diff --check`.
- [ ] Manual check: Completar una ficha con documento propio y verificar que una ficha sin sus requisitos sigue draft.

**Dependencies:** T05.

**Files likely touched:**
- `src-tauri/src/components/patients/domain.rs`
- `src-tauri/src/components/patients/application.rs`
- `src-tauri/src/components/patients/adapters/outbound.rs`
- `src-tauri/src/components/patients/application_tests.rs`
- `src-tauri/src/components/patients/repository_tests.rs`

**Estimated scope:** M

## C02: Revisión del recorrido de fichas

Después de T04–T06:
- [ ] Demostrar registro → edición → documento → completitud sin red y tras reinicio.
- [ ] Pasan las pruebas centradas de la fase y `cargo test --workspace --locked`.
- [ ] `pnpm build:desktop` y `git diff --check` correctos.
- [ ] Evidencia registrada con límites explícitos; revisión del usuario completa.

## T07: Contactos

**Status:** Todo

**Description:** Implementar creación, consulta, edición y actividad de contactos administrativos de la cuenta, reutilizables entre pacientes.

**Acceptance criteria:**
- [ ] Los casos guardan contactos e intenciones sin red con control de permisos y versión.
- [ ] Los campos de contacto respetan el contrato, incluidos documento y pareja telefónica.
- [ ] Un contacto puede servir a varios pacientes sin crear otra identidad ni consultar otra cuenta.

**Verification:**
- [ ] Focused tests: Not available yet - define in T07: `cargo test --workspace --locked components::patients`, con casos concretos de esta tarea y ejecución no vacía.
- [ ] Build or broader check: `cargo check --workspace --all-targets --locked` y `git diff --check`.
- [ ] Manual check: Crear un contacto, editarlo offline y recuperar sus datos tras reabrir.

**Dependencies:** T06 y revisión del usuario en C02.

**Files likely touched:**
- `src-tauri/src/components/patients/domain.rs`
- `src-tauri/src/components/patients/application.rs`
- `src-tauri/src/components/patients/adapters/outbound_contacts.rs`
- `src-tauri/src/components/patients/contact_tests.rs`
- `src-tauri/src/components/patients/adapters/mod.rs`

**Estimated scope:** M

## T08: Vínculos

**Status:** Todo

**Description:** Implementar la relación paciente/contacto con clave compuesta, consultas en ambos sentidos y cambios de datos o actividad.

**Acceptance criteria:**
- [ ] Crear, editar y cambiar actividad usa la clave patientId/contactPersonId y operaciones contractuales.
- [ ] Parentesco, responsabilidad, vigencia y representación permanecen separados; el cliente no verifica representación.
- [ ] Los vínculos entre entidades aún locales conservan dependencias y aislamiento de cuenta.

**Verification:**
- [ ] Focused tests: Not available yet - define in T08: `cargo test --workspace --locked components::patients`, con casos concretos de esta tarea y ejecución no vacía.
- [ ] Build or broader check: `cargo check --workspace --all-targets --locked` y `git diff --check`.
- [ ] Manual check: Vincular el mismo contacto a dos pacientes y consultar ambas direcciones, incluida vigencia.

**Dependencies:** T07.

**Files likely touched:**
- `src-tauri/src/components/patients/domain.rs`
- `src-tauri/src/components/patients/application.rs`
- `src-tauri/src/components/patients/adapters/outbound_relationships.rs`
- `src-tauri/src/components/patients/relationship_tests.rs`
- `src-tauri/src/components/patients/adapters/mod.rs`

**Estimated scope:** M

## T09: Protección de fichas completas

**Status:** Todo

**Description:** Comprobar los efectos de una edición de contacto o vínculo sobre las fichas completas conocidas. Habilita menores y contactos compartidos sin degradación silenciosa.

**Acceptance criteria:**
- [ ] El impacto inválido devuelve COMPLETENESS_CONFLICT con referencias locales afectadas y no guarda el cambio.
- [ ] Pasar expresamente las fichas afectadas a draft permite preparar el cambio con sus dependencias.
- [ ] Se respeta el límite contractual de dependencias sin truncarlas ni afirmar validez futura de una cita.

**Verification:**
- [ ] Focused tests: Not available yet - define in T09: `cargo test --workspace --locked components::patients`, con casos concretos de esta tarea y ejecución no vacía.
- [ ] Build or broader check: `cargo check --workspace --all-targets --locked` y `git diff --check`.
- [ ] Manual check: Intentar retirar el único teléfono responsable de dos fichas completas; pasarlas a draft y repetir de forma explícita.

**Dependencies:** T08.

**Files likely touched:**
- `src-tauri/src/components/patients/domain.rs`
- `src-tauri/src/components/patients/application.rs`
- `src-tauri/src/components/patients/adapters/outbound_contacts.rs`
- `src-tauri/src/components/patients/adapters/outbound_relationships.rs`
- `src-tauri/src/components/patients/completeness_tests.rs`

**Estimated scope:** M

## C03: Revisión de contactos y responsables

Después de T07–T09:
- [ ] Demostrar menores/contactos compartidos y cambios que rompen requisitos.
- [ ] Pasan las pruebas centradas de la fase y `cargo test --workspace --locked`.
- [ ] `pnpm build:desktop` y `git diff --check` correctos.
- [ ] Evidencia registrada con límites explícitos; revisión del usuario completa.

## T10: Proyecciones remotas

**Status:** Todo

**Description:** Traducir los cuatro estados remotos al almacenamiento aceptado del componente con RemoteProjector, sin modificar el transporte ni las intenciones.

**Acceptance criteria:**
- [ ] Upsert traduce las cuatro entidades y valida ámbito y claves; repeticiones/versiones antiguas no retroceden vistas.
- [ ] Remove/reset afectan solo las vistas del ámbito y conservan intenciones.
- [ ] Un fallo local revierte el grupo/página y su avance de cursor dentro de la transacción recibida.

**Verification:**
- [ ] Focused tests: Not available yet - define in T10: `cargo test --workspace --locked components::patients`, con casos concretos de esta tarea y ejecución no vacía.
- [ ] Build or broader check: `cargo check --workspace --all-targets --locked` y `git diff --check`.
- [ ] Manual check: Aplicar un grupo con ficha/documento/contacto/vínculo, repetirlo y reconstruir sin perder un cambio local.

**Dependencies:** T09 y revisión del usuario en C03.

**Files likely touched:**
- `src-tauri/src/components/patients/adapters/projector.rs`
- `src-tauri/src/components/patients/adapters/mod.rs`
- `src-tauri/src/components/patients/projection_tests.rs`

**Estimated scope:** M

## T11: Conciliación de cambios locales

**Status:** Todo

**Description:** Combinar vistas y estado de recibos para exponer cambios locales y su estado, sin dar por descargado lo que solo tiene recibo.

**Acceptance criteria:**
- [ ] Accepted conserva el provisional hasta alcanzar su versión en la vista; luego retira solo ese provisional.
- [ ] Conflictos, rechazos y dependencias bloqueadas conservan datos, operaciones e IDs para revisión.
- [ ] Las correcciones usan operaciones nuevas por casos específicos; un resultado desconocido exige recuperar el original.

**Verification:**
- [ ] Focused tests: Not available yet - define in T11: `cargo test --workspace --locked components::patients`, con casos concretos de esta tarea y ejecución no vacía.
- [ ] Build or broader check: `cargo check --workspace --all-targets --locked` y `git diff --check`.
- [ ] Manual check: Simular accepted antes de la proyección, una aceptación parcial y una secuencia cuyo primer paso se rechaza.

**Dependencies:** T10.

**Files likely touched:**
- `src-tauri/src/components/patients/adapters/intents.rs`
- `src-tauri/src/components/patients/adapters/outbound.rs`
- `src-tauri/src/components/patients/reconciliation_tests.rs`

**Estimated scope:** M

## T12: Integración del componente

**Status:** Todo

**Description:** Incorporar PatientsProjector a la composición usada por el ciclo normal y SSE y comprobar los recorridos del coordinador con dos bases/dispositivos.

**Acceptance criteria:**
- [ ] La composición usa los proyectores existentes y Pacientes con la misma transacción, sin cambiar sus reglas de propiedad.
- [ ] Dos dispositivos ejercen aceptación, conflicto y recuperación de respuesta perdida sin duplicar efectos.
- [ ] Captura/retiradas/reinicio conservan intenciones y otra cuenta; una caída de API/SSE no bloquea los casos locales.

**Verification:**
- [ ] Focused tests: Not available yet - define in T12: `cargo test --workspace --locked patients_integration_tests`; ejecutar también `cargo test --workspace --locked components::patients` y las regresiones de sync afectadas.
- [ ] Build or broader check: `cargo check --workspace --all-targets --locked` y `git diff --check`.
- [ ] Manual check: Ejecutar el recorrido offline → reinicio → envío → descarga entre dos bases temporales con servidor local sintético.

**Dependencies:** T11.

**Files likely touched:**
- `src-tauri/src/components/projector.rs`
- `src-tauri/src/components/sync/patients_integration_tests.rs`
- `src-tauri/src/components/sync/mod.rs`
- `src-tauri/src/components/patients/test_support.rs`

**Estimated scope:** M

## C04: Revisión de sincronización

Después de T10–T12:
- [ ] Demostrar que un resultado parcial o perdido no duplica efectos ni borra cambios.
- [ ] Pasan las pruebas centradas de la fase y `cargo test --workspace --locked`.
- [ ] `pnpm build:desktop` y `git diff --check` correctos.
- [ ] Evidencia registrada con límites explícitos; revisión del usuario completa.

## T13: Comandos de fichas

**Status:** Todo

**Description:** Añadir adaptadores de entrada Tauri específicos para consultas de pacientes/documentos y las cinco escrituras de esos grupos, sin pantalla React.

**Acceptance criteria:**
- [ ] Los DTO camelCase admiten solo los campos del caso y conservan ausencia/null.
- [ ] Los comandos derivan el ámbito de la sesión y comprueban acceso/allowedActions para propietario y asistente.
- [ ] Las salidas separan aceptado/provisional y estados sin SQL, credenciales o errores internos crudos.

**Verification:**
- [ ] Focused tests: Not available yet - define in T13: `cargo test --workspace --locked components::patients`, con casos concretos de esta tarea y ejecución no vacía.
- [ ] Build or broader check: `cargo check --workspace --all-targets --locked` y `git diff --check`.
- [ ] Manual check: Ejercitar DTO y casos de entrada con sesión sintética, sin sesión y con permisos insuficientes.

**Dependencies:** T12 y revisión del usuario en C04.

**Files likely touched:**
- `src-tauri/src/components/patients/adapters/inbound_profiles.rs`
- `src-tauri/src/components/patients/ipc_profile_tests.rs`
- `src-tauri/src/components/patients/adapters/mod.rs`

**Estimated scope:** M

## T14: Comandos de contactos

**Status:** Todo

**Description:** Añadir adaptadores de entrada específicos para consultas de contactos/vínculos y sus seis escrituras, conservando el mismo control nativo.

**Acceptance criteria:**
- [ ] Los comandos traducen los casos de contactos/vínculos con claves y datos contractuales.
- [ ] Cuenta y permisos proceden de la sesión; no se aceptan envoltorios remotos ni ámbitos arbitrarios.
- [ ] Las pruebas de entrada cubren pares telefónicos, vigencia y COMPLETENESS_CONFLICT sin pantalla React.

**Verification:**
- [ ] Focused tests: Not available yet - define in T14: `cargo test --workspace --locked components::patients`, con casos concretos de esta tarea y ejecución no vacía.
- [ ] Build or broader check: `cargo check --workspace --all-targets --locked` y `git diff --check`.
- [ ] Manual check: Ejercitar las entradas de contacto compartido y vínculo de un menor, incluida una edición inválida.

**Dependencies:** T13.

**Files likely touched:**
- `src-tauri/src/components/patients/adapters/inbound_contacts.rs`
- `src-tauri/src/components/patients/ipc_contact_tests.rs`
- `src-tauri/src/components/patients/adapters/mod.rs`

**Estimated scope:** M

## T15: Registro y permisos Tauri

**Status:** Todo

**Description:** Registrar los 18 comandos y permisos concretos en la composición de la aplicación, sin añadir cliente TypeScript ni pantalla.

**Acceptance criteria:**
- [ ] Las once escrituras y siete consultas tienen registro y permisos explícitos coherentes.
- [ ] La aplicación inicia con los comandos previstos y deniega comandos o argumentos fuera de su contrato.
- [ ] El ámbito efectivo procede de la sesión seleccionada y las verificaciones no exponen secretos.

**Verification:**
- [ ] Focused tests: Not available yet - define in T15: `cargo test --workspace --locked components::patients`, con casos concretos de esta tarea y ejecución no vacía.
- [ ] Build or broader check: `pnpm build:desktop` y `git diff --check`.
- [ ] Manual check: Abrir el ejecutable de desarrollo y comprobar una invocación autorizada y una denegada mediante el mecanismo de prueba IPC definido en T15, sin añadir UI de producto.

**Dependencies:** T14.

**Files likely touched:**
- `src-tauri/src/lib.rs`
- `src-tauri/build.rs`
- `src-tauri/capabilities/default.json`
- `src-tauri/src/components/patients/ipc_registration_tests.rs`
- `src-tauri/src/components/patients/mod.rs`

**Estimated scope:** M

## C05: Revisión de la interfaz nativa

Después de T13–T15:
- [ ] Comprobar comandos, DTO y permisos sin añadir pantalla React.
- [ ] Pasan las pruebas centradas de la fase y `cargo test --workspace --locked`.
- [ ] `pnpm build:desktop` y `git diff --check` correctos.
- [ ] Evidencia registrada con límites explícitos; revisión del usuario completa.

## T16: Contrato del consumidor

**Status:** Todo

**Description:** Ampliar las pruebas de consumidor con las once acciones de Pacientes y las cuatro proyecciones existentes, usando el bundle fijo como autoridad.

**Acceptance criteria:**
- [ ] Las fixtures contrastan acciones, payloadModels, claves, versiones y disponibilidad offline con capacidades/esquemas.
- [ ] Las pruebas detectan campos extra, ausencia/null y formas incorrectas sin sustituir la validación del servidor.
- [ ] Las cinco huellas del bundle siguen intactas y pasan las pruebas existentes.

**Verification:**
- [ ] Focused tests: `cargo test --workspace --locked components::sync::contract_tests`.
- [ ] Build or broader check: `cargo check --workspace --all-targets --locked` y `git diff --check`.
- [ ] Manual check: Revisar una fixture de cada grupo contra la guía y sus esquemas, incluida la clave compuesta del vínculo.

**Dependencies:** T15 y revisión del usuario en C05.

**Files likely touched:**
- `src-tauri/src/components/sync/contract_tests.rs`
- `src-tauri/src/components/sync/fixtures/patients-operations.json`

**Estimated scope:** S

## T17: API de desarrollo real

**Status:** Todo

**Description:** Verificar el componente nativo con Nightingale API/OIDC de desarrollo, dos dispositivos/bases aislados y datos ficticios. No modificar otros repositorios ni servicios.

**Acceptance criteria:**
- [ ] El recorrido nativo guarda offline, reabre, sincroniza y descarga cambios aceptados usando los adaptadores reales.
- [ ] Dos dispositivos producen un conflicto y conservan las intenciones; verificar también el recorrido de contacto responsable.
- [ ] Se registran comandos, entorno, resultados y datos/dispositivos sintéticos creados, sin secretos; un bloqueo no se cuenta como éxito.

**Verification:**
- [ ] Focused tests: Not available yet - define in T17: prueba explícita de desarrollo real con el patrón manual/ignorado del repositorio. Documentar su comando exacto antes de ejecutarla; las pruebas automáticas no acreditan esta verificación.
- [ ] Build or broader check: `cargo test --workspace --locked components::patients` y `git diff --check`.
- [ ] Manual check: Completar el acceso OIDC cuando corresponda y verificar la aceptación/descarga y conflicto reales entre dos dispositivos aislados. Si falta el entorno, mantener esta tarea pendiente de su prueba real.

**Dependencies:** T16.

**Files likely touched:**
- `src-tauri/src/components/patients/development_tests.rs`
- `src-tauri/src/components/patients/mod.rs`
- `tasks/P-20261104-01-patients-core/todo.md`

**Estimated scope:** M

## T18: Documentación y calidad

**Status:** Todo

**Description:** Documentar propiedad, API pública, puertos, reglas, adaptadores y límites del componente y entregar evidencia de calidad y prueba remota para C06.

**Acceptance criteria:**
- [ ] La guía de Pacientes y las guías de integración describen la implementación final y separan lo verificado de lo pendiente.
- [ ] Pasan calidad, build de escritorio y diff; no se alteran contrato, especificación o trabajo ajeno.
- [ ] El informe de cierre incluye prueba API real y mantiene C06 pendiente de revisión del usuario.

**Verification:**
- [ ] Focused tests: `cargo test --workspace --locked components::patients`, `cargo test --workspace --locked components::sync::contract_tests` y `cargo test --workspace --locked patients_integration_tests`.
- [ ] Build or broader check: `pnpm quality`, `pnpm build:desktop` y `git diff --check`.
- [ ] Manual check: Reproducir el recorrido nativo documentado y revisar la evidencia real y sus límites antes de presentar C06.

**Dependencies:** T17.

**Files likely touched:**
- `docs/technical/components/patients.md`
- `docs/operations/core-next-tracks.md`
- `docs/operations/development.md`
- `tasks/P-20261104-01-patients-core/plan.md`
- `tasks/P-20261104-01-patients-core/todo.md`

**Estimated scope:** M

## C06: Aceptación de Pacientes

Después de T16–T18:
- [ ] Revisar resultados locales, prueba API real y límites expresos.
- [ ] Pasan las pruebas centradas de la fase y `cargo test --workspace --locked`.
- [ ] `pnpm build:desktop` y `git diff --check` correctos.
- [ ] Evidencia registrada con límites explícitos; revisión del usuario completa.
