# Contratos iniciales para los siguientes tracks

El motor transversal habilita estos cuatro trabajos; este documento no los implementa ni aprueba sus planes. Antes de empezar cada uno se concreta su DTO, caso Rust, proyector, tablas, comando, migración y pruebas. Antes de trabajo paralelo se revisan los contratos compartidos. El integrador conserva `lib.rs`, `build.rs`, capacidades y registro de migraciones.

## Pacientes y contactos

Propietario: componente `patients`. Casos nativos: guardar ficha borrador, actualizar datos administrativos, completar ficha, añadir documento, guardar contacto y vínculo. Datos locales: identidad/ficha por cuenta, documentos, contactos, vínculos e intenciones. No incluye historia clínica.

Contrato API: `patient_profile`, `patient_identifier`, `contact_person`, `patient_contact`; este último usa `{patientId,contactPersonId}`. Acciones: `RegisterPatient`, `UpdatePatientProfile`, `SetPatientCompletion`, `SetPatientProfileStatus`, `AddPatientIdentifier`, `CreateContactPerson`, `UpdateContactPerson`, `SetContactPersonStatus`, `CreatePatientContactRelationship`, `UpdatePatientContactRelationship`, `SetPatientContactRelationshipStatus`. Los modelos y reglas concretos pertenecen al [contrato importado](../contracts/api-v1/nightingale-api.md#52-paciente-ficha-documentos-y-contactos).

Offline: guardar con el último permiso válido y emitir una intención durable. API comprueba completitud, representación y permisos al aceptar. Un conflicto conserva ambas versiones para revisión. Las consultas públicas para Citas deben devolver referencias y datos administrativos necesarios; no ofrecen SQL ni tablas ajenas.

Estado: implementado en el plan P-20261104-01-patients-core (ver
[guía del componente](../technical/components/patients.md)). La pantalla React,
la edad a fecha de cita y la prueba con API de desarrollo real quedan fuera o
pendientes de su entorno.

## Cuenta y catálogo de servicios

Propietarios: configuración de cuenta y componente `services`, con puertos públicos distintos. Casos nativos: consultar preferencias, editar datos/política online, crear servicio y ofrecerlo en una cuenta. Datos: vista de cuenta, profesional, catálogo, ofertas e intenciones propias.

Contrato API: `account`, `professional`, `service`, `service_offer`; oferta usa `{professionalId,serviceId}`. `UpdateAccountDetails`, `UpdateAccountPolicy` y `SetAccountStatus` requieren conexión y permiso de propietario. Catálogo/oferta usan `CreateService`, `UpdateService`, `SetServiceStatus`, `OfferService`, `UpdateServiceOffer`, `SetServiceOfferStatus`; nombre propio usa `UpdateProfessionalProfile`. La lista vigente de acciones y online-only se consume de API/capabilities, no se copia por rol.

Offline: catálogo/ofertas permitidos conservan intenciones; cambios administrativos no simulan aceptación. Guardar `version` de oferta y `serviceVersion` del catálogo. Igual versión de oferta puede traer un catálogo más nuevo. Agenda consume un puerto que devuelve servicio, profesional, duración de referencia y versiones; no lee sus tablas.

## Agenda y disponibilidad

Propietario: componente `scheduling`. Casos nativos: editar reglas, excepciones y bloqueos, consultar una vista local de disponibilidad. Datos: reglas semanales, excepciones por fecha y bloqueos aceptados, más intenciones pendientes. Consume el catálogo por puerto; no posee servicios ni pacientes.

Contrato API: `schedule_rule`, `schedule_exception`, `schedule_block`; claves UUIDv7. Acciones `Create/Update/SetStatus` con los nombres contractuales `CreateScheduleRule`, `UpdateScheduleRule`, `SetScheduleRuleStatus` y sus equivalentes `ScheduleException`/`ScheduleBlock`. Los minutos locales, fechas IANA, bloques globales y alcance de asistente siguen el [contrato](../contracts/api-v1/nightingale-api.md#53-servicio-oferta-y-agenda).

Offline: la disponibilidad calculada es provisional. No reserva ni acredita hueco compartido. Un bloqueo rechazado o conflicto conserva la intención. Citas consulta por puerto disponibilidad de referencia y versiones; API hace la comprobación compartida final.

## Citas e historia

Propietario: componente `appointments`. Casos nativos: crear, confirmar, reprogramar, cancelar, completar y marcar ausencia. Depende de puertos de Pacientes, Servicios y Agenda; conserva citas, historia aceptada e intenciones de cuenta. No posee contactos o catálogo.

Contrato API: `appointment` y `appointment_history`, ambas con UUID de cita. Acciones `CreateAppointment`, `ConfirmAppointment`, `RescheduleAppointment`, `CancelAppointment`, `CompleteAppointment`, `MarkAppointmentNoShow`. Se conservan snapshots e historia del servidor; las nueve referencias acordadas no se recalculan desde catálogo actual. La historia aceptada no se reescribe.

Offline: una cita guardada queda pendiente de aceptación compartida. Dependencias causales enlazan pacientes/servicios previos cuando corresponde. `SLOT_UNAVAILABLE` y `VERSION_CONFLICT` no se resuelven por hora local ni por sobrescritura forzada. Salir de ventana retira cita/historia aceptadas y conserva intenciones. El Hub y sus solicitudes requieren otro plan y repositorio.

## Recorrido para añadir un caso

1. Leer el contrato importado y fijar acción, clave, payload, versión esperada, permisos y ausencia frente a nulo. La API mantiene la única fuente editable.
2. Reservar una migración **desde `0005`** en el nuevo plan, sin tocar `0003`/`0004`. Declarar tablas del componente y separar intención de vista aceptada. Este documento no asigna esos números.
3. Implementar dominio sin almacenamiento/framework. El caso Rust consulta los puertos necesarios, valida permiso y abre una transacción SQLite propia. Escribe su cambio local y llama a `sync::application::record_operation` con esa misma transacción.
4. Implementar `RemoteProjector::project/reset` para sus tablas aceptadas. Componer los proyectores concretos en el integrador; ninguno realiza HTTP, cambia Outbox o produce efectos externos en esa transacción.
5. Añadir un comando de caso de uso, DTO camelCase y permiso Tauri explícito. React invoca ese comando, no acepta SQL, token o un envío remoto arbitrario. No reutilizar el comando de diagnóstico para exponer estados sensibles.
6. Probar sin red, reapertura, rechazo/conflicto, replay y rollback con Outbox en la transacción. Ejecutar contrato, calidad y revisión del checkpoint del nuevo plan.

Ejemplo sintético: una ficha borrador usa `RegisterPatient`, clave UUIDv7 nueva, versión esperada cero y `payload={"fullName":"Paciente de prueba"}`. El caso local guarda el borrador y su intención juntos. Un recibo `accepted` resuelve la intención; la página/captura actualiza la vista aceptada mediante el proyector. Ninguno de esos pasos permite saltarse la autorización de API.

La guía de [sync](../technical/components/sync.md), el [ADR](../decisions/desktop-core-engine.md) y la [guía de desarrollo](development.md) describen los puertos existentes. Cifrado de SQLite, claves, instaladores, publicación y datos reales siguen fuera de esta entrega.
