# Pacientes, contactos y responsables

Responsabilidad: identidad administrativa de pacientes, documentos declarados,
contactos de la cuenta y vínculos paciente/contacto. No incluye datos clínicos,
fusión de duplicados, consentimiento de comunicación ni edad a fecha de cita.
Propietario y asistente operan según `allowedActions` vigentes.

Datos: patient_profile_view, patient_identifier_view, contact_person_view,
patient_contact_view y patient_local, migración `0007_patients`. Cada fila tiene
ámbito de cuenta, usuario y dispositivo. Las vistas aceptadas y las intenciones
tienen distintos escritores; remove/reset nunca borran intenciones. El vínculo usa
la clave compuesta `{patientId, contactPersonId}`.

API de C01–C05: `PatientUseCases` expone las once escrituras contractuales
(RegisterPatient, UpdatePatientProfile, SetPatientCompletion,
SetPatientProfileStatus, AddPatientIdentifier, CreateContactPerson,
UpdateContactPerson, SetContactPersonStatus, CreatePatientContactRelationship,
UpdatePatientContactRelationship, SetPatientContactRelationshipStatus) y siete
consultas locales. `PatientQuery` entrega ficha y documentos aceptados a futuros
consumidores como Citas, sin HTTP, SQL ni modelos de almacenamiento.
La lectura separa estado aceptado, provisional y operaciones con su estado.

Cada mutación offline comprueba permiso y escribe su intención con
record_operation en la misma transacción IMMEDIATE: provisional y Outbox se
confirman o revierten juntos. La base de una edición es una versión aceptada o la
última operación local de esa entidad; versiones previstas de una secuencia son
provisionales. Sin diferencia devuelve NO_CHANGES; una base antigua,
VERSION_CONFLICT; operaciones no resueltas, DEPENDENCY_FAILED. Hasta ocho
dependencias, sin duplicados ni ciclos.

Reglas de dominio: draft/complete es independiente de active/inactive. Documentos
conservan prefijos, ceros y puntuación; solo se crean declarados. phoneCode/phone
forman pareja; el documento de contacto exige terna completa o nula. Las ediciones
distinguen ausencia y null. Los nombres, teléfonos o documentos iguales no fusionan
personas; añadir un documento propio posterior no cambia la identidad del paciente.
Completar exige, según `identificationBasis`, nombre, documento declarado/verified
y teléfono propio o responsable vigente con teléfono (menor: nombre, nacimiento y
responsable). El cliente nunca verifica documentos ni representación.

Completitud compartida: una edición de contacto o vínculo que deja fichas completas
sin sus requisitos devuelve COMPLETENESS_CONFLICT con las IDs locales afectadas y
no guarda nada. Pasarlas expresamente a draft incorpora la operación de
completitud como dependencia. La evaluación usa el instante inyectado, no la zona
del equipo, y solo el ámbito local.

SqlitePatientRepository implementa lecturas y `PatientQuery`. El modelo de
conciliación retira un provisional en lectura solo cuando su operación tiene recibo
accepted y la vista aceptada alcanzó la versión confirmada; un recibo sin descarga
no retira nada. Conflictos, rechazos y resultados unknown conservan ID, contenido
y dependencias para revisión; la corrección crea una operación nueva por caso
específico. patient_local conserva sus filas retiradas (el borrado físico es
decisión de evolución futura); operación y recibo permanecen en el Outbox de sync.

PatientsProjector traduce patient_profile, patient_identifier, contact_person y
patient_contact con la transacción recibida, sin red ni intenciones. reset borra
solo las cuatro vistas del ámbito. Repeticiones y versiones antiguas no retroceden
vistas; los estados fuera del ámbito se rechazan. La validación reutiliza las
reglas de dominio. DesktopProjector compone Account, Services y Patients para el
ciclo normal, SSE y captura.

Dieciocho comandos IPC `patients_*` con permisos específicos (`allow-patients-*`),
DTO camelCase con `deny_unknown_fields`, campos tri-estado (ausencia conserva, null
retira, valor fija) y base de edición exactamente una versión aceptada positiva o
una operación local. Sin sesión devuelven AUTHENTICATION_REQUIRED; el ámbito sale
de la sesión seleccionada. Las salidas no exponen payload remoto, época,
fingerprint ni identificadores de cuenta/dispositivo. No hay pantalla React en
este plan.

Pruebas: `cargo test --workspace --locked tests::components::patients`,
`cargo test --workspace --locked patients_integration_tests`,
`cargo test --workspace --locked tests::components::sync::contract_tests`,
`pnpm quality` y `pnpm build:desktop`. Las fixtures de contrato contrastan las
once acciones, sus payloadModels y la clave compuesta con el bundle fijo; se
comprueban sus cinco hashes. La prueba HTTP loopback recorre registro de menor con
responsable, reapertura, envío, accepted, descarga, conflicto, retiro y captura.
La prueba de desarrollo real (`tests::components::patients::development_tests`, opt-in con
`--ignored`) queda pendiente de entorno y no se acredita aquí.

Fuentes: [transacciones rusqlite 0.40.2](https://docs.rs/rusqlite/0.40.2/rusqlite/struct.Transaction.html),
[bloqueo IMMEDIATE](https://docs.rs/rusqlite/0.40.2/rusqlite/struct.Connection.html#method.transaction_with_behavior),
[UPSERT SQLite](https://www.sqlite.org/lang_upsert.html),
[campos ausentes y null en Serde 1.0.229](https://serde.rs/field-attrs.html),
[comandos Tauri](https://v2.tauri.app/develop/calling-rust/#passing-arguments),
[permisos](https://v2.tauri.app/security/permissions/#app-manifest),
[Chrono 0.4.45 parse_from_rfc3339](https://docs.rs/chrono/0.4.45/chrono/struct.DateTime.html#method.parse_from_rfc3339).
