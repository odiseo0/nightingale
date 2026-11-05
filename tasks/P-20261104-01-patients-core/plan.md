# P-20261104-01-patients-core: Núcleo de Pacientes

**Status:** In progress
**Created:** 2026-11-04
**Updated:** 2026-11-05
**Task list:** [todo.md](todo.md)

## Aprobación y límite de ejecución

El usuario aprobó el plan el 2026-11-04: «Lo veo bien, apruebo el plan. No comiences el desarrollo».

El 2026-11-04 el usuario autorizó ejecutar T01–T03, en orden, y presentar C01. Esta instrucción sustituye «No comiences el desarrollo». Conserva alcance e IDs. El usuario aprobó C01 y autorizó continuar: «Por favor, puedes continuar con el plan. C01 está bien». Ejecutar T04–T06 en orden y detenerse en C02.

## Overview

Implementar el componente Rust `patients` para registrar, consultar, buscar y actualizar pacientes, documentos y contactos responsables. Permitirá seguir editando y completar fichas sin conexión, conservar los cambios tras reiniciar y sincronizarlos con Nightingale API.

Decisiones confirmadas:
- Incluir contactos compartidos y menores sin documento propio.
- Usar el contrato actual `api-v1.1`.
- Entregar núcleo Rust, SQLite y comandos Tauri. La pantalla React queda fuera.
- Exigir pruebas finales con la API de desarrollo y datos ficticios.

Quedan fuera datos clínicos, fusión de duplicados, consentimiento de comunicación, Patient Hub, creación de citas, importación masiva y cambios en otros repositorios. No se editará la especificación como efecto del desarrollo.

## Source of Truth

- [Master Plan](../../docs/master-plan.md), §§9.4, 10–14, 19 y 23.
- [SRS](../../docs/srs.md): registro administrativo, datos mínimos y exclusión clínica.
- [Arquitectura técnica](../../docs/technical-architecture.md): operación local, límites y sincronización.
- [Decisión de tres proyectos](../../docs/decisions/transactional-sync-three-projects.md).
- [Contrato API fijo](../../docs/contracts/api-v1/nightingale-api.md), §§5.2, 5.5, 6–10, con sus esquemas y capacidades.
- [Contratos iniciales](../../docs/operations/core-next-tracks.md), apartado Pacientes y contactos.
- [Reglas de planificación](../README.md) y [CONTRIBUTING](../../CONTRIBUTING.md).
- Código actual: componentes services, identity y sync; DesktopProjector; migraciones 0001–0006 y registro SQLite.

Los planes de motor transversal y cuenta/catálogo están completados; este plan consume sus interfaces y no reabre su historial. Al aprobar el plan, el repositorio todavía no tenía un componente patients. La composición existente ya conecta AccountProjector y ServicesProjector con el ciclo normal y SSE.

Durante la exploración previa pasaron las cuatro pruebas existentes de `components::sync::contract_tests` y las cinco huellas del bundle. Esta evidencia no acredita funcionalidad de Pacientes ni la futura prueba API real.

## Architecture Decisions

### Propiedad, datos y límites

- patients posee dominio, casos de uso y adaptadores. Otros componentes consumen su API pública; no consultan sus tablas.
- Reservar `0007_patients`, sin modificar migraciones anteriores.
- Añadir cuatro vistas aceptadas: fichas, documentos, contactos y vínculos; una tabla de cambios locales con operaciones, dependencias y datos provisionales; índices por ámbito, estado, completitud y referencias.
- Todas las filas tendrán ámbito de cuenta, usuario y dispositivo. Remove/reset y una nueva captura conservarán las intenciones locales. No usar cascadas que las borren.
- Los datos creados o descargados ya locales estarán disponibles sin HTTP y tras reiniciar; un dispositivo nuevo necesita activación y descarga para obtener datos ajenos.
- Nightingale API posee aceptación compartida, contratos y PostgreSQL. El escritorio solo usa HTTPS y no recibe acceso SQL ni credenciales PostgreSQL.
- No hay excepción cloud del Hub en este plan. Hub, comunicaciones y sus servidores quedan fuera.
- El mismo responsable integra migraciones, módulos, comandos y permisos; no hay tareas paralelas autorizadas.

### Casos de uso e interfaces

Implementar estas once escrituras, sin ampliar el contrato remoto:

| Grupo | Acciones |
|---|---|
| Fichas | RegisterPatient, UpdatePatientProfile, SetPatientCompletion, SetPatientProfileStatus |
| Documentos | AddPatientIdentifier |
| Contactos | CreateContactPerson, UpdateContactPerson, SetContactPersonStatus |
| Vínculos | CreatePatientContactRelationship, UpdatePatientContactRelationship, SetPatientContactRelationshipStatus |

Añadir siete consultas locales: listar pacientes, consultar ficha, listar documentos, listar contactos de un paciente, listar contactos, consultar contacto y listar pacientes de un contacto.

Los 18 comandos Tauri usarán el prefijo `patients_`, nombres específicos y DTO camelCase. No admitirán SQL, credenciales, una cuenta arbitraria ni un envoltorio remoto libre. Rust obtendrá el ámbito de la sesión seleccionada.

Las consultas separarán:
- Estado aceptado y versión.
- Estado local provisional.
- Operaciones pendientes, conflictos, rechazos y dependencias bloqueadas.

Un puerto `PatientQuery` ofrecerá datos administrativos aceptados a futuros consumidores como Citas, sin modelos HTTP ni acceso al almacenamiento.

### Reglas del dominio

- Separar draft/complete de active/inactive; permitir fichas incompletas en draft conforme al contrato.
- Conservar documentos como texto, incluidos prefijos, ceros y puntuación.
- Mantener phoneCode/phone juntos: ambos presentes o ambos nulos.
- Distinguir ausencia y null al editar.
- No fusionar ni vincular personas por coincidencias de nombre, teléfono o documento.
- Añadir un documento propio posterior sin cambiar la identidad del paciente.
- No permitir que el cliente declare documentos o representación como verificados.
- Desactivar un paciente no cancela citas; un responsable válido no acredita tutela ni acceso al Hub.
- Comprobar completitud con nombre, base de identificación, documento o nacimiento y teléfono propio o responsable, según el contrato.
- La edad y los requisitos a la fecha de una cita pertenecen al futuro caso de uso de Citas.
- Una edición que deje fichas completas sin requisitos devuelve COMPLETENESS_CONFLICT con las referencias locales afectadas. El usuario debe pasarlas expresamente a draft; no hay cambio silencioso de completitud.
- Para fechas, reutilizar Chrono 0.4.45, presente en Cargo.lock, como dependencia directa con las funciones mínimas; no actualizar otras dependencias.

### Trabajo sin red, secuencias y permisos

Cada guardado confirmará el cambio local y su operación Outbox en una misma transacción SQLite. Propietario y asistente podrán operar según allowedActions y los últimos permisos válidos. La API revalida el permiso vigente al aceptar; la hora del equipo no prueba autorización. Una época nueva no reasigna intenciones previas.

Para editar varias veces sin red:
- Cada cambio tiene identidad y contenido propios.
- Rust enlaza cambios sucesivos mediante dependsOn.
- La base de una edición es una versión aceptada o la última operación local de esa entidad.
- Las versiones previstas para una secuencia son provisionales; solo API acredita versiones aceptadas.
- Evitar operaciones locales sin diferencia. Respetar el límite contractual de dependencias sin omitirlas para permitir el envío.
- Un rechazo o conflicto conserva la secuencia y señala sus pasos bloqueados. No ajustar versiones ni sobrescribir estado remoto de forma automática.
- Los cambios de contactos compartidos incorporan las dependencias necesarias de las operaciones explícitas de completitud.

### Descarga y conciliación

PatientsProjector traduce patient_profile, patient_identifier, contact_person y patient_contact; el vínculo usa la clave compuesta {patientId,contactPersonId}. Se incorpora a DesktopProjector para ciclo normal, SSE y recuperación por captura.

El proyector usa la transacción recibida, no hace HTTP ni modifica Outbox. Aplicar página/proyecciones/cursor o captura/baseCursor de forma atómica. Repeticiones y versiones antiguas no retroceden vistas.

Un recibo accepted no sustituye la descarga. La consulta retira un provisional confirmado cuando su vista aceptada alcanza la versión correspondiente, conservando operación y recibo.

Corregir un conflicto o rechazo conocido crea otra operación mediante el caso específico. Un resultado desconocido exige recuperar el resultado original antes de repetir la intención. La revisión conserva cambios bloqueados; no incluye una herramienta general de resolución automática de conflictos.

## Dependency Order

T01 → T02 → T03 → C01 → T04 → T05 → T06 → C02 → T07 → T08 → T09 → T19 → T20 → T21 → C03 → T10 → T11 → T12 → C04 → T13 → T14 → T15 → C05 → T16 → T17 → T18 → C06.

Un checkpoint pendiente detiene la fase siguiente hasta la revisión del usuario. Antes de cada cambio de código aplicar minimal-code tras trazar el flujo real y docs-first-framework-code cuando corresponda. El dominio local no depende de que el entorno remoto esté disponible.

## Task List

### Fase 1: Contratos y persistencia
- [x] T01: Contratos del componente.
- [x] T02: Dominio de fichas y formatos.
- [x] T03: Persistencia local.

### C01: Revisión de contratos y persistencia
- [x] Revisar interfaces, límites y migración antes de ampliar los casos de uso.
- [x] Completar verificaciones de T01–T03.
- [x] Revisión del usuario.

### Fase 2: Fichas
- [x] T04: Registro y consulta de pacientes.
- [x] T05: Ediciones sucesivas.
- [x] T06: Documentos y completitud.

### C02: Revisión del recorrido de fichas
- [x] Demostrar registro → edición → documento → completitud sin red y tras reinicio.
- [x] Completar verificaciones de T04–T06.
- [x] Revisión del usuario.

### Fase 3: Contactos
- [x] T07: Contactos.
- [x] T08: Vínculos.
- [x] T09: Protección de fichas completas.

### Correcciones C03
- [x] T19: Impacto de completitud por ficha.
- [x] T20: Dependencia de completitud tras edición posterior.
- [x] T21: RelationshipPatch sin retiro de responsabilidad.

### C03: Revisión de contactos y responsables
- [ ] Demostrar menores y contactos compartidos, incluidos cambios que rompen requisitos.
- [x] Completar verificaciones de T07–T09.
- [x] Completar verificaciones de T19–T21 y reejecutar la suite de la fase tras las correcciones.
- [ ] Revisión del usuario.

### Fase 4: Sincronización
- [x] T10: Proyecciones remotas.
- [x] T11: Conciliación de cambios locales.
- [x] T12: Integración del componente.

### C04: Revisión de sincronización
- [ ] Demostrar que resultados parciales o perdidos no duplican efectos ni borran cambios.
- [x] Completar verificaciones de T10–T12.
- [ ] Revisión del usuario.

### Fase 5: Comandos nativos
- [x] T13: Comandos de fichas.
- [x] T14: Comandos de contactos.
- [x] T15: Registro y permisos Tauri.

### C05: Revisión de la interfaz nativa
- [ ] Comprobar comandos, DTO y permisos sin añadir pantalla React.
- [x] Completar verificaciones de T13–T15.
- [ ] Revisión del usuario.

### Fase 6: Verificación y cierre
- [x] T16: Contrato del consumidor.
- [ ] T17: API de desarrollo real (pendiente de entorno; prueba manual documentada).
- [x] T18: Documentación y calidad.

### C06: Aceptación de Pacientes
- [ ] Revisar resultados locales, prueba API real y límites expresos.
- [x] Completar verificaciones de T16–T18.
- [ ] Revisión y aceptación del usuario.

## Test Plan

Cada tarea tendrá pruebas de comportamiento, comprobación de compilación y recorrido reproducible desde Rust o IPC. Los filtros nuevos se ligarán a pruebas reales en las tareas indicadas en todo.md.

Comandos previstos:
- `cargo test --workspace --locked components::patients`
- `cargo test --workspace --locked components::sync::contract_tests`
- `cargo test --workspace --locked patients_integration_tests`
- `pnpm quality`
- `pnpm build:desktop`
- `git diff --check`

Escenarios obligatorios:
- Guardar, editar varias veces y completar sin red; cerrar y reabrir.
- Fallar durante la escritura sin dejar solo la entidad o solo la operación.
- Adulto con documento propio y menor con responsable.
- Documento posterior del menor sin cambio de identidad.
- Contacto compartido y efectos sobre completitud.
- Repetición y respuesta perdida tras commit remoto.
- Aceptación parcial sin dar por aceptada toda la secuencia.
- Cambio de otro dispositivo con conservación de la intención en conflicto.
- Captura nueva y retiradas sin borrar trabajo pendiente.
- Aislamiento entre cuentas, permisos ausentes y revocación.
- API, Hub o SSE caídos sin impedir operaciones sobre datos locales.

El cierre requiere evidencia con API real de desarrollo. Si el entorno impide la prueba, registrar el bloqueo y mantener pendiente esa comprobación; las pruebas locales no la sustituyen. No se requiere cambiar los servidores ni probar Hub, comunicaciones, publicación o producción.

## Risks and Mitigations

| Riesgo | Impacto | Mitigación |
|---|---|---|
| Diferencias entre estado provisional y aceptación de una secuencia | Alto | Dependencias explícitas, recibos individuales, conservación de bloqueados y prueba real. No ajustar versiones de forma automática. |
| Edición de contacto compartido que rompe requisitos | Alto | Evaluar impacto local, exigir draft explícito y ordenar dependencias; API revalida. |
| Respuesta perdida o resultado desconocido | Alto | Conservar ID/contenido y consultar el resultado original antes de otra intención. |
| Remove/reset o captura que borra trabajo | Alto | Separar vistas e intenciones y probar recuperación, aislamiento y rollback. |
| Copiar la regla de escritura exclusiva del propietario de Servicios | Alto | Comprobar allowedActions; probar propietario y asistente. |
| Entorno API/OIDC no disponible | Medio | Continuar pruebas locales independientes; no acreditar ni cerrar la verificación real. |
| Ampliar el alcance mediante funciones ausentes del contrato | Medio | Dejar fusión, consentimiento y cambios de API fuera de esta entrega. |

## Assumptions and Defaults

- Búsqueda local limitada a los datos disponibles de la cuenta; sin directorio global.
- Sin pantalla React ni diseño visual en este plan.
- Sin actualización del contrato ni de la especificación como efecto del trabajo.
- Solo datos ficticios. Cifrado y uso de datos reales requieren trabajo previo.
- Conservar trabajo ajeno y planes anteriores.
- No iniciar ninguna tarea hasta una nueva instrucción del usuario que autorice el desarrollo.

## Open Questions

Ninguna decisión de alcance pendiente. T01 concretó los contratos internos dentro del alcance aprobado; cualquier incompatibilidad que exija cambiarlo se presentará al usuario antes de implementar el punto afectado.

## Contratos internos fijados en T01

Fuentes cotejadas: guía API §§5.2, 5.5, 6–10; `$defs` de accepted-operations.schema.json y public-models.schema.json; payloadModels y onlineOnlyActions de capabilities.json. No se altera el bundle.

### Once escrituras

Todas exigen acceso operativo y la acción exacta en allowedActions del contexto local vigente. Propietario y asistente pueden ejercerlas. Ninguna es online-only. Crear exige expectedVersion=0; editar exige versión positiva. Todos los IDs son UUIDv7 canónicos. Cuenta, usuario, dispositivo y permissionEpoch proceden de la sesión Rust; no se reciben por IPC. Rust genera operationId y conserva su contenido para reintentar.

| Acción (comando Tauri previsto) | Entidad / clave | Payload contractual |
|---|---|---|
| RegisterPatient (patients_register_patient) | patient_profile / patientId | RegisterPatientPayload |
| UpdatePatientProfile (patients_update_patient_profile) | patient_profile / patientId | PatientPatch |
| SetPatientCompletion (patients_set_patient_completion) | patient_profile / patientId | PatientCompletionPayload |
| SetPatientProfileStatus (patients_set_patient_profile_status) | patient_profile / patientId | PatientStatusPayload |
| AddPatientIdentifier (patients_add_patient_identifier) | patient_identifier / identifierId | AddPatientIdentifierPayload |
| CreateContactPerson (patients_create_contact_person) | contact_person / contactPersonId | ContactData |
| UpdateContactPerson (patients_update_contact_person) | contact_person / contactPersonId | ContactPatch |
| SetContactPersonStatus (patients_set_contact_person_status) | contact_person / contactPersonId | PatientStatusPayload |
| CreatePatientContactRelationship (patients_create_patient_contact_relationship) | patient_contact / {patientId,contactPersonId} | RelationshipData |
| UpdatePatientContactRelationship (patients_update_patient_contact_relationship) | patient_contact / {patientId,contactPersonId} | RelationshipPatch |
| SetPatientContactRelationshipStatus (patients_set_patient_contact_relationship_status) | patient_contact / {patientId,contactPersonId} | PatientStatusPayload |

Los DTO de entrada son camelCase y rechazan extras. Clave y base acompañan al payload específico; no hay envoltorio remoto libre.

- RegisterPatientPayload: fullName?, birthDate?, phoneCode?, phone?, identificationBasis? admiten null; identifiers es una lista opcional de hasta 100 IdentifierInput. Registro crea draft/active, sin admitir esos estados como campos de entrada.
- IdentifierInput: id, documentType, documentNumber; issuingCountry por defecto VE. AddPatientIdentifierPayload sustituye id por patientId. No admite verificationStatus y crea declared. Conservar número como texto sin eliminar ceros, prefijos o puntuación.
- PatientPatch: solo fullName, birthDate, phoneCode, phone, identificationBasis; ausencia conserva, null retira. Rechazar patch vacío; validar teléfono sobre el resultado completo.
- PatientCompletionPayload: completionStatus obligatorio draft/complete. PatientStatusPayload: status obligatorio active/inactive. Estados independientes.
- ContactData: fullName obligatorio; phoneCode, phone, issuingCountry, documentType, documentNumber opcionales/nulos. ContactPatch admite esos mismos campos; null no puede retirar el nombre obligatorio del resultado. Documento de contacto: terna completa o nula.
- RelationshipData: relationshipKind obligatorio mother/father/guardian/other, validFrom obligatorio RFC3339 con offset, isResponsibleContact por defecto false, validUntil opcional/nulo. RelationshipPatch admite esos cuatro campos; null solo retira validUntil. El resultado exige validUntil posterior a validFrom cuando existe.
- El cliente no recibe campos de verificación como entrada: verificationStatus, representationStatus, verifiedAt o verifiedByUserId solo proceden del estado aceptado.

### Base de edición y respuesta

Base: `Accepted { version }` o `LocalOperation { operationId }`. La primera debe coincidir con la vista aceptada; la segunda debe ser la última intención de la entidad en el mismo ámbito. Versiones previstas de una secuencia son provisionales. No ajustar una base antigua de forma automática.

Cada paso depende del anterior; documentos y vínculos incorporan creaciones locales referidas. Contactos compartidos incorporan las operaciones explícitas que pasan fichas afectadas a draft. Hasta ocho IDs, sin duplicados, ciclos ni truncamiento. Exceder el límite rechaza la preparación sin perder pasos previos.

Una escritura devuelve operationId, clave, provisional, expectedVersion y dependsOn; no una aceptación remota. Sin diferencia devuelve NO_CHANGES sin operación nueva. Guardar provisional y Outbox mediante record_operation en la misma transacción. Resultado desconocido exige recuperar el original; corregir rechazo/conflicto crea otra operación por el caso concreto.

Consulta por entidad: clave, `accepted: { state, version } | null`, `provisional: { state, operationId, expectedVersion } | null`, `operations: [{ operationId, dependsOn, status, error }]`. Estados derivados de sync distinguen pendientes, unknown, conflictos, rechazos y dependencias bloqueadas. Un recibo accepted no retira el provisional hasta que la vista alcance la versión confirmada; nunca dar versión aceptada al provisional.

Errores: VALIDATION_ERROR (forma, fecha, teléfono, documento, base o dependencia inválidos), AUTHENTICATION_REQUIRED, PERMISSION_DENIED, NOT_FOUND, VERSION_CONFLICT, NO_CHANGES, COMPLETENESS_CONFLICT con IDs locales afectados, DEPENDENCY_FAILED e INTERNAL_ERROR seguro. Conservar códigos remotos sin SQL ni datos de otra cuenta.

### Siete consultas y puerto

| Comando previsto | Entrada local |
|---|---|
| patients_list_patients | q?, status?, completionStatus?, page=1, shows=100 (1–500) |
| patients_get_patient | patientId |
| patients_list_patient_identifiers | patientId |
| patients_list_patient_contacts | patientId, status? |
| patients_list_contacts | q?, status?, page=1, shows=100 (1–500) |
| patients_get_contact | contactPersonId |
| patients_list_contact_patients | contactPersonId, status? |

Todas usan SQLite con ámbito obligatorio y últimos permisos válidos, sin HTTP. Sin filtro incluyen ambos estados. q busca nombre/documento/teléfono solo del ámbito. Orden estable por ID; items/total tras filtrar. Dato ajeno equivale a no encontrado. Un vínculo puede reunir datos actuales en lectura, sin copiarlos a su estado aceptado.

`PatientQuery` entrega ficha administrativa, documentos y responsables aceptados por referencias tipadas y ámbito explícito; no expone HTTP, SQL ni modelos de almacenamiento. Los casos internos combinan aceptado/provisional por separado. El puerto no decide edad o validez para una cita futura.

### Persistencia y cotejo manual

0007_patients crea patient_profile_view, patient_identifier_view, contact_person_view, patient_contact_view y patient_local. Las cuatro primeras conservan estados públicos aceptados y versiones, con índices por filtros/referencias. patient_local conserva operationId, acción, claves de entidad, expectedVersion, operación base, dependsOn, payload, provisional y orden local. Todas exigen cuenta/usuario/dispositivo. No hay cascadas desde vistas a intenciones. La época del permiso vive en Outbox y no reasigna intenciones.

Ejemplos revisados: `{}` registra draft sin requisitos de complete; un menor usa nombre, birthDate y responsable vigente con teléfono, sin usar el documento del adulto como propio; registro seguido de dos ediciones usa bases de operación previa, dependencias y versiones previstas 0/1/2, sin afirmar aceptación. Completar own_document exige nombre, documento declared/verified y teléfono propio o responsable; minor_without_document exige nombre, nacimiento y responsable con teléfono. Edad a fecha de cita queda fuera.

## Entrega C01

T01–T03 completos. [Evidencia y límites de C01](evidence-c01.md). Pruebas y build correctos. C01 aprobado por el usuario el 2026-11-04. Continuación autorizada hasta C02.

## Entrega C02

T04–T06 completos. [Evidencia y límites de C02](evidence-c02.md). La suite completa, compilación, formato, Clippy y build de escritorio pasan. C02 aprobado por el usuario el 2026-11-05: «Apruebo C02». T07 no iniciado; la siguiente sesión puede ejecutar T07–T09 y detenerse en C03.

## Instrucción vigente tras C02

El usuario aprobó C02 el 2026-11-05. T07–T09 constituyen la siguiente fase autorizada, en orden, con parada en C03 para revisión. Se deja su ejecución para la otra sesión solicitada por el usuario; ninguna de esas tareas se ha iniciado aquí.

## Entrega C03

T07–T09 completos, más correcciones T19–T21 de la revisión. [Evidencia y límites de C03](evidence-c03.md). C03 aprobado por el usuario el 2026-11-05 tras sus correcciones y commit.

## Entrega C04

T10–T12 completos. [Evidencia y límites de C04](evidence-c04.md). C04 aprobado por el usuario el 2026-11-05 (commit `0be5631`).

## Entrega C05

T13–T15 completos. [Evidencia y límites de C05](evidence-c05.md). C05 aprobado por el usuario el 2026-11-05 («Continue»).

## Entrega C06

T16 y T18 completos; T17 preparada y pendiente de su entorno real (prueba manual opt-in documentada). [Evidencia y límites](evidence-c06.md). Suite completa, `pnpm quality` y build de escritorio en verde. C06 aprobado por el usuario el 2026-11-05: «Puedes continuar».

## Instrucción vigente tras la entrega de C06

T16–T18 se ejecutaron en la sesión del 2026-11-05 tras la aprobación de C05. T17 solo podrá marcarse completa cuando la prueba manual contra Nightingale API/OIDC de desarrollo pase en su entorno; hasta entonces C06 queda pendiente de revisión del usuario.

## Correcciones de C03 (revisión 2026-11-05)

La revisión de C03 (ejes Estándares y Spec) detectó tres defectos confirmados en T07–T09 y dos señales de robustez. Se corrigen en este mismo plan con T19–T21, antes de la aprobación de C03. No se inicia T10.

Defectos confirmados:

1. **Falso COMPLETENESS_CONFLICT con otro responsable vigente** — `responsible_impact` (`adapters/documents.rs`) evalúa cada vínculo del contacto editado de forma aislada y no comprueba si la ficha conserva soporte con otro responsable; un menor completo con madre y padre recibe conflicto al retirar el teléfono de uno solo. Contradicen «sin sus requisitos» (Reglas del dominio) y el comportamiento de `completion_support`, que sí admite varios responsables.
2. **Dependencia de completitud perdida tras una edición posterior** — la dependencia de la operación `SetPatientCompletion` solo se cablea cuando es la última intención local de la ficha; si la ficha se edita después de pasar a draft, la edición del contacto/vínculo no depende del paso a draft y puede ordenarse antes en el envío («Contactos compartidos incorporan las operaciones explícitas que pasan fichas afectadas a draft»).
3. **RelationshipPatch admite retirar la responsabilidad** — `is_responsible_contact: Clear` se aplica como `false` y el payload emite `isResponsibleContact: null`, cuando el contrato fija que «null solo retira validUntil».

Robustez incluida en T19: el closure de evaluación futura del vínculo releía los contactos por iteración y convertía errores de almacenamiento en «sin teléfono» (`.ok()…unwrap_or((false, false))`); se calcula una vez y propaga. La cola de protección duplicada entre los `prepare` de contacto y vínculo se extrae a un helper común.

Queda fuera, con justificación: el hard-error `INTERNAL_ERROR` cuando un vínculo carece de contacto visible (solo posible con estado local corrupto; el contrato admite «INTERNAL_ERROR seguro»), la ubicación del cálculo de impacto en `documents.rs` (juicio de diseño, sin impacto de corrección) y los estados con primitivos en `application.rs` (precedente de `IdentifierState`).
