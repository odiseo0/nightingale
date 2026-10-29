# Nightingale API: referencia para el escritorio

Fecha de corte: **2026-10-29**. Contrato: **v1**, revisión de modelos **api-v1.1**. Aplicación API: **0.1.0**. Destino: equipo y agentes que desarrollan la aplicación de escritorio en el repositorio `nightingale`.

Esta guía describe las funciones disponibles en Nightingale API y las reglas que el escritorio debe respetar al consumirlas. Es una copia de consulta preparada en `nightingale-api`; los modelos del servidor conservan la definición de los contratos. Los ejemplos usan datos ficticios.

## 1. Archivos que debe conservar el escritorio

Copiar completa la carpeta `contracts/api-v1/` de `nightingale-api` a una carpeta documental del escritorio, por ejemplo `docs/nightingale-api/`. Así se conservan estos enlaces sin depender de rutas entre repositorios:

| Archivo | Uso |
|---|---|
| [openapi.json](openapi.json) | OpenAPI 3.1.0: 46 rutas, 47 operaciones HTTP y 70 esquemas de entrada/salida. |
| [accepted-operations.schema.json](accepted-operations.schema.json) | Envoltorio y payloads válidos de las 42 acciones de escritura por lote. |
| [public-models.schema.json](public-models.schema.json) | Modelos HTTP en `$defs`; seleccionar el modelo concreto. |
| [sse.schema.json](sse.schema.json) | Datos de los avisos; el transporte usa el formato SSE. |
| [capabilities.json](capabilities.json) | Acciones, límites y correspondencia entre acciones y modelos de payload. |
| [manifest.json](manifest.json) | SHA-256 de los cinco JSON anteriores y conteos del contrato. |
| `nightingale-api.md` | Esta explicación del modelo, decisiones y flujos del consumidor. |

Los JSON se generan desde `src.application:app` y `src/api/*/domain`. Su campo `x-generated` identifica versión, commit de referencia, huella del árbol fuente y versiones de paquetes. La huella incluye los cambios locales de código; `sourceHead` por sí solo no identifica esos cambios. La copia del escritorio debe fijar el conjunto y sus huellas, sin editar los JSON a mano.

OpenAPI describe las rutas. El cuerpo de una operación admite distintas acciones y devuelve también rechazos por payload inválido o acción desconocida; por eso hace falta el esquema adicional `accepted-operations.schema.json` para conocer las entradas válidas de cada acción. Ninguno de esos archivos sustituye las reglas de negocio o los permisos.

## 2. Qué hace Nightingale API

Nightingale API valida y acepta el estado compartido de consultas, pacientes, contactos, servicios, agenda y citas. Usa Python, FastAPI y PostgreSQL privado. Aplica permisos vigentes, controla versiones y reservas concurrentes, conserva resultados de operaciones y entrega cambios a los dispositivos.

| Proyecto | Responsabilidad |
|---|---|
| Escritorio `nightingale` | React/Tauri para la interfaz; Rust para casos de uso y acceso remoto; SQLite para datos locales e intenciones pendientes. Puede consultar y guardar el trabajo local permitido sin red. |
| `nightingale-api` | Autorización y aceptación remota, datos compartidos, migraciones, historial, auditoría, resultados y sincronización. |
| Patient Hub | Web, backend, accesos y solicitudes del paciente, con almacenamiento propio. Su integración con esta API queda pendiente. |

El escritorio envía intenciones como `CreateAppointment` o `UpdatePatientProfile`. La API comprueba cada una y responde tras confirmar su transacción. El escritorio y el Hub acceden por HTTP; nunca reciben credenciales SQL ni escriben directamente en PostgreSQL.

## 3. Estado al corte

La primera etapa cubre una consulta independiente con un profesional propietario y asistentes, desde varios dispositivos. Tiene alta administrativa controlada, acceso por dispositivo, invitaciones, pacientes/contactos, catálogo/ofertas, agenda, ciclo de citas, captura inicial, descarga incremental y SSE.

| Área | Estado y alcance de la evidencia |
|---|---|
| Acceso, cuentas y equipo | Implementados. Pruebas previas con Keycloak y PostgreSQL aislados. |
| Pacientes, contactos y servicios | Implementados, con versiones, privacidad y controles de integridad. |
| Agenda y citas | Implementadas. Las pruebas previas cubren reservas concurrentes, bloqueo frente a reserva, transiciones e historia inmutable. |
| Sincronización | Capturas coherentes, grupos completos, cursores firmados, retención y recuperación implementados. |
| SSE | Implementado y probado previamente con dos procesos API y diez conexiones. |
| Contratos | Generados de nuevo desde la app y los modelos del árbol de esta entrega. |
| Revisión del plan | C10 aceptado. T31–T33 comprobadas; **C11 sigue pendiente de revisión y aceptación expresa**. El plan general permanece abierto. |
| Consumidor de escritorio | Su integración real con Rust/SQLite y su recuperación de extremo a extremo siguen pendientes. Esta guía no acredita ese trabajo. |
| Producción | Sin acreditación de despliegue, carga representativa, copias/restauración o uso de datos reales. |

La entrega previa para revisión C11 registró 144 pruebas correctas, migraciones hasta `20261028_01` con 28 tablas y `btree_gist`, y 16 diagnósticos estáticos previos sin cierre global del analizador. Es evidencia de aquel código y entorno. El registro de esta entrega, al final del documento, separa las comprobaciones nuevas.

La suite del árbol actual ejecuta 144 pruebas y termina con dos errores en subcasos de validación directa de duración (`True` y `1.5`). El cambio previo del commit `f572f9c` hace que el validador lance `TypeError` en vez de un error de validación de Pydantic. Se conserva ese código y se registra el límite. En consultas HTTP, los textos `durationMinutes=true` y `durationMinutes=1.5` sí devolvieron 422 en la comprobación ASGI de esta entrega. No se declara una suite global correcta.

Las medidas previas de SSE usaron 20 commits y 200 avisos: p95 local de 120,1 ms desde el callback posterior al commit y cota superior de 122,5 ms que incluye el commit. La muestra usa diez conexiones y dos procesos; no fija un objetivo de producción.

Quedan fuera: gestión de clínicas y varias sedes, transferencia de cuentas, roles propios, historia clínica, pagos, archivos médicos, verificación/fusión/corrección global de identidad, cambios de referencias de citas y reapertura de estados terminales. También quedan fuera Hub, recordatorios, WhatsApp, n8n, lista de espera y analítica. El esquema puede contener relaciones o estados de soporte para usos futuros sin que exista una acción pública para gestionarlos.

## 4. Decisiones que condicionan al escritorio

| Decisión vigente | Efecto en el consumidor |
|---|---|
| Tres proyectos con propietarios distintos | El escritorio conserva su operación local; API acepta lo compartido; Hub conserva sus accesos y solicitudes. |
| Permisos vigentes al aceptar | Una operación guardada sin red puede rechazarse al volver si el usuario perdió acceso. La hora local no concede permiso anterior. |
| Token opaco por usuario/dispositivo, sin vencimiento | Guardarlo de forma segura. La API lo revoca; cerrar sesión en Keycloak no lo revoca por sí solo. |
| Una transacción por operación | Un lote puede aceptar unas operaciones y rechazar otras. La respuesta perdida exige recuperar la misma identidad de operación. |
| Versiones explícitas | Crear usa `expectedVersion=0`; modificar usa la versión remota conocida. Las fechas no sustituyen la versión. |
| Identidades separadas de fichas y permisos | Compartir un identificador no permite consultar otras cuentas ni vincular pacientes por coincidencias. |
| Catálogo del profesional y oferta por cuenta | El servicio conserva su propietario; cada oferta puede ajustar la duración de referencia. |
| Reserva global por profesional | Dos cuentas no pueden reservar al mismo profesional a la vez. La vista autorizada no revela pacientes o motivos ajenos. |
| Datos acordados conservados en la cita | Editar catálogo, nombre profesional, dirección o zona no reescribe citas anteriores. |
| Solicitud de cambio separada de la cita | Una futura solicitud del Hub no cambia estado, horario o confirmación. El personal decide y API acepta la reprogramación. |
| Interés en oferta separado de reserva | En la futura lista de espera, la respuesta del paciente no asigna el horario. El personal autoriza la reserva. |
| Un esquema editable por contrato | API posee modelos y migraciones. El escritorio consume una versión de los artefactos y diseña su SQLite según sus necesidades locales. |

Las decisiones del servidor constan en `nightingale-api/docs/decisions.md` —D01, D02, D06, D08, D09, D10 y P05/P06—. Los detalles de Hub y lista de espera aún requieren acuerdos propios.

## 5. Modelo de datos que consume el escritorio

### 5.1 Identidad, cuenta y profesional

Un usuario tiene identidad OIDC por la pareja exacta `issuer`/`subject`. Una cuenta contiene su equipo y sus datos operativos. El propietario se deriva de la propiedad de esa cuenta; los demás miembros activos son asistentes. Un perfil profesional identifica a la persona cuyo tiempo se reserva y siempre tiene usuario.

La etapa expone solo cuentas `independent`. El diseño físico admite relaciones con clínicas, pero no ofrece su gestión HTTP. Una persona puede pertenecer a varias cuentas; cada pertenencia y cada autorización de dispositivo tienen su propio alcance.

El dispositivo global pertenece al usuario. `account_device_accesses` autoriza esa instalación dentro de una cuenta. Emitir un token o aceptar una invitación no registra por sí solo la instalación en la cuenta.

### 5.2 Paciente, ficha, documentos y contactos

`patients` conserva una identidad mínima. `account_patients` conserva nombre, nacimiento, teléfono y estados administrativos de una cuenta. La lectura pública `PatientProfileResponse` reúne esa ficha bajo `accountId` y `patientId`. Una cuenta no recibe las fichas de otras por conocer `patientId`.

| Dato | Regla vigente |
|---|---|
| `completionStatus` | `draft` permite guardar datos incompletos; `complete` exige una base válida. |
| `status` | `active`/`inactive`, independiente de completitud. Inactivar no cancela citas. |
| `identificationBasis=own_document` | Nombre, documento propio `declared` o `verified`, y teléfono propio o responsable vigente con teléfono. |
| `identificationBasis=minor_without_document` | Nombre, nacimiento y responsable vigente con teléfono. Al citar, edad menor de 18 en la fecha local de la cita. |
| Documento | País emisor, `national_id`/`passport` y número de texto. Conservar prefijos, ceros y puntuación significativa. |
| `verificationStatus` | `declared`, `verified`, `invalidated`. Las acciones actuales solo añaden documentos declarados. |
| Teléfono | `phoneCode` y `phone` forman una pareja: ambos presentes o ambos nulos. No prueban identidad ni propiedad del número. |
| Contacto | Pertenece a una cuenta; puede servir a varios pacientes. No necesita usuario ni ficha de paciente. |
| Vínculo paciente/contacto | Conserva parentesco declarado, función responsable, vigencia, actividad y constancia de representación separadas. |

La API revalida requisitos a la fecha de la reserva. Menor de 18 es un criterio administrativo de esta etapa; no acredita tutela ni acceso al Hub. Para nacimiento el 29 de febrero, el cumpleaños administrativo cae el 1 de marzo en un año no bisiesto.

Las coincidencias de nombre, teléfono o documento declarado no fusionan personas. El documento del adulto no identifica al menor. Añadir después un documento propio conserva la identidad del paciente y sus citas. La API actual no permite adjuntar una identidad global arbitraria a una ficha nueva.

Si una edición de contacto o vínculo deja fichas completas sin sus requisitos, la API devuelve `COMPLETENESS_CONFLICT`. Primero hay que pasar expresamente las fichas afectadas a `draft`, con sus versiones; no hay cambio silencioso de completitud.

### 5.3 Servicio, oferta y agenda

El profesional posee el catálogo `services`. La oferta `professional_services` vincula un servicio propio con una cuenta. Su duración efectiva de referencia usa `durationMinutesOverride` cuando existe y, en otro caso, `defaultDurationMinutes`. Ambas admiten 1–1440 minutos enteros.

`ServiceOfferResponse` tiene dos versiones: `version` de la oferta y `serviceVersion` del catálogo. Cambiar el servicio puede actualizar la proyección de la oferta sin cambiar su versión propia. El consumidor debe conservar ambas y procesar el grupo descargado; no debe descartar esa actualización solo porque `version` se repita.

La agenda combina reglas semanales, una excepción por fecha y bloqueos. Las reglas y excepciones usan minutos desde medianoche: inicio 0–1439, fin exclusivo 1–1440. `1440` representa la medianoche local del día siguiente. Las fechas de vigencia de reglas incluyen ambos extremos; los intervalos de horas incluyen inicio y excluyen fin.

Una excepción `closed` cierra el día; `replacement` sustituye su horario habitual por un intervalo. Un bloqueo usa instantes absolutos y afecta una cuenta o toda la actividad del profesional si `accountId` es nulo. Solo el profesional propio gestiona bloqueos globales; los asistentes no reciben su motivo global.

La disponibilidad une horarios, aplica excepciones y resta bloqueos y ocupación del profesional. Consultarla no reserva. Una hora local ambigua o inexistente requiere corregir la entrada; la API rechaza esa resolución con 422. Cambiar horarios, zona o bloqueos no mueve ni cancela citas: si deja una cita vigente incompatible, rechaza el cambio.

### 5.4 Cita e historia

La cita mantiene cuenta, paciente, profesional, servicio y responsable elegido. Sus instantes tienen precisión de un minuto y duración de 1–1440 minutos. Inicio y fin determinan la duración real; la referencia del servicio puede ser distinta.

| Acción | Estado previo | Estado nuevo y efecto |
|---|---|---|
| `CreateAppointment` | Sin cita | `scheduled`, versión 1 e historia inicial. |
| `ConfirmAppointment` | `scheduled` | `confirmed`, mismo intervalo y hora de confirmación del servidor. |
| `RescheduleAppointment` | `scheduled`/`confirmed` | `scheduled`, nuevo intervalo de igual duración y confirmación nula. |
| `CancelAppointment` | `scheduled`/`confirmed` | `cancelled`; conserva historia y libera ocupación. |
| `CompleteAppointment` | `scheduled`/`confirmed` | `completed`; exige haber alcanzado el inicio. |
| `MarkAppointmentNoShow` | `scheduled`/`confirmed` | `no_show`; exige inicio más margen vigente de cuenta y acción del personal. |

`cancelled`, `completed` y `no_show` son terminales. El margen de ausencia empieza en 15 minutos y admite 0–1440. Falta de respuesta del paciente no marca ausencia. Confirmar de nuevo con otra operación una cita ya confirmada da transición inválida; repetir la misma operación recupera su resultado.

La exclusión de reservas cubre al profesional entre cuentas. `scheduled`, `confirmed`, `completed` y `no_show` conservan ocupación; solo `cancelled` la libera. Dos citas contiguas pueden compartir el instante de fin/inicio.

Cada cita y versión histórica conserva nueve datos `snapshot`: nombre del servicio, duración de referencia ofrecida, nombre profesional, nombre y dirección del lugar, país, estado, ciudad y zona horaria. Reprogramar conserva esos datos y la duración real. Las acciones actuales no cambian paciente, profesional, servicio, cuenta o responsable de una cita existente.

`appointment_histories` añade una entrada por versión, desde 1 y sin huecos. El estado actual coincide con su última entrada. La API protege historia y cita en la misma transacción e impide reescribir o borrar la historia ordinaria. Este historial funcional es distinto de la auditoría técnica y del registro de sincronización.

El modelo de historia contempla actores `staff`, `patient` y `contact`, pero las acciones actuales del escritorio registran `staff`. Los valores futuros del esquema no habilitan acciones del Hub.

### 5.5 Proyecciones de sincronización

Cada `SyncItem` identifica `entityType`, `entityKey`, `entityVersion` y `state`. El tipo decide el modelo de `state`; no se debe intentar inferirlo solo por sus campos.

| `entityType` | Clave pública | Modelo de `state` |
|---|---|---|
| `account` | UUID de cuenta | `AccountResponse` |
| `account_device` | UUID de dispositivo | `AccountDeviceResponse` |
| `membership` | UUID de usuario | `MemberResponse` |
| `invitation` | UUID de invitación | `InvitationResponse` |
| `professional` | UUID de profesional | `ProfessionalResponse` |
| `patient_profile` | UUID de paciente | `PatientProfileResponse` |
| `patient_identifier` | UUID de documento | `PatientIdentifierResponse` |
| `contact_person` | UUID de contacto | `ContactResponse` |
| `patient_contact` | `{patientId,contactPersonId}` | `RelationshipResponse` |
| `service` | UUID de servicio | `ServiceResponse` |
| `service_offer` | `{professionalId,serviceId}` | `ServiceOfferResponse` |
| `schedule_rule` | UUID de regla | `ScheduleRuleResponse` |
| `schedule_exception` | UUID de excepción | `ScheduleExceptionResponse` |
| `schedule_block` | UUID de bloqueo | `ScheduleBlockResponse` |
| `appointment` | UUID de cita | `AppointmentResponse` |
| `appointment_history` | UUID de cita | `AppointmentHistoryProjection` |

La cuenta de la ruta forma parte del ámbito aunque una clave pública no la repita. Guardar y buscar entidades locales con ese ámbito evita cruces de cuenta.

`appointment_history.state.items` contiene las entradas hasta la versión descargada y conserva el intervalo de esa proyección. `patient_contact` sincroniza el vínculo; los GET usan `PatientContactResponse`, que añade nombres y teléfono actuales de esa cuenta. Esos datos relacionados no forman parte del estado persistido del vínculo.

El modelo público de documento no expone al verificador global. Las respuestas, el catálogo privado y los metadatos del equipo también aplican su audiencia. Consumir el modelo permitido no da acceso al ORM ni al resto de columnas físicas.

### 5.6 Correspondencia con PostgreSQL

El head documentado `20261028_01` contiene estas 28 tablas. Esta lista sirve para entender responsabilidades; **no define el esquema SQLite ni un contrato SQL para consumidores**.

| Grupo | Tablas físicas |
|---|---|
| Identidad y cuenta | `app_users`, `accounts`, `account_memberships`, `professionals`, `account_professionals` |
| Pacientes | `patients`, `patient_identifiers`, `account_patients`, `contact_people`, `patient_contact_relationships` |
| Servicios y agenda | `services`, `professional_services`, `schedule_rules`, `schedule_exceptions`, `schedule_blocks` |
| Citas | `appointments`, `appointment_histories` |
| Acceso y equipo | `devices`, `device_tokens`, `account_device_accesses`, `account_invitations` |
| Sincronización y auditoría | `operation_receipts`, `account_sync_heads`, `change_log`, `sync_snapshots`, `sync_snapshot_items`, `sync_stream_leases`, `audit_events` |

La única fuente editable del modelo físico es `nightingale-api/docs/database-schema.md`; sus migraciones implementan el esquema. El borrador `data-model.md` queda retirado. Las tablas de recibos, tokens, journal, capturas y leases son soporte del servidor; el escritorio recibe sus contratos públicos.

## 6. Acceso y permisos

1. Leer `GET /v1/auth/config`. Usar navegador externo, Authorization Code y PKCE S256 con el cliente público configurado, sin secreto distribuido.
2. Enviar el access token OIDC destinado a API a `POST /v1/auth/device-tokens`. El cuerpo contiene `deviceName`, `clientVersion` y `localSchemaVersion`; el servidor asigna usuario/dispositivo. Un ID token no sustituye esa credencial.
3. Guardar el `accessToken` opaco devuelto una sola vez. Usarlo en `Authorization: Bearer <accessToken>` en las siguientes peticiones. Su formato actual es `ngt_<tokenId>.<secret>`; el consumidor no necesita interpretarlo.
4. Leer `/v1/me`, `/v1/accounts` y el acceso de la cuenta. Autenticarse no crea consulta, pertenencia o perfil profesional. El alta de consulta usa un script administrativo del servidor.
5. Si `/access` devuelve `registration_required`, enviar `RegisterAccountDevice` con el `deviceId` autenticado como clave y `expectedVersion=0`. Solo después se permite descargar datos operativos.

Aceptar invitación usa su ruta dirigida, un `operationId` y la identidad destinataria autenticada. La invitación dura 72 horas, concede pertenencia de asistente y no envía correo. Tampoco registra la instalación en la cuenta.

| Capacidad | Propietario | Asistente |
|---|---|---|
| Leer datos operativos de la cuenta; gestionar pacientes, contactos y citas | Sí | Sí |
| Reglas, excepciones y bloqueos limitados a la cuenta | Sí | Sí |
| Catálogo propio, ofertas y nombre profesional propio | Sí | No |
| Bloqueos globales del profesional propio | Sí | No |
| Equipo, invitaciones, cuenta, política y dispositivos de cuenta | Sí | No |
| Auditoría técnica de cuenta | Sí | No |
| Dispositivos/tokens propios y aceptación de invitación propia | Sí | Sí |

Usar `AccessResponse.allowedActions` para la vista actual. En una cuenta activa con dispositivo habilitado, las listas actuales contienen 42 acciones para propietario y 27 para asistente. La lista no sustituye autorización de objeto, estado o alcance del servidor.

La revocación global de un dispositivo afecta sus tokens. La revocación dentro de una cuenta afecta solo ese ámbito. Una instalación revocada no puede restaurarse a sí misma. Perder la credencial exige nueva autenticación, otra instalación y revocación de la anterior; el servidor no recupera el secreto perdido.

Los permisos vigentes también rigen consultas de resultados previos, capturas, páginas y SSE. El escritorio conserva la intención que API rechace por revocación para que una persona autorizada la revise. Esa persona crea una operación nueva con su identidad real; no suplanta al actor anterior.

## 7. Endpoints disponibles

En las tablas, `A` significa `/v1/accounts/{account_id}`. Los nombres entre llaves son parámetros de ruta. Casi todas las rutas requieren token API; las de `A` requieren además pertenencia activa y dispositivo habilitado, salvo descubrimiento de acceso y las excepciones administrativas indicadas. Un ID ajeno no concede acceso.

### 7.1 Salud, identidad y cuenta propia

| Método | Ruta | Uso / acceso |
|---|---|---|
| GET | `/health/live` | Vida del proceso; público. |
| GET | `/health/ready` | Disponibilidad SQL y listener; público, 200/503. |
| GET | `/v1/capabilities` | Capacidades y límites implementados; público. |
| GET | `/v1/auth/config` | Emisor, audiencia y cliente público OIDC; público. |
| POST | `/v1/auth/device-tokens` | Canjear access token OIDC por token API; 201. |
| GET | `/v1/me` | Identidad propia y resumen de cuentas. |
| GET | `/v1/me/devices` | Dispositivos propios, incluidos revocados. |
| POST | `/v1/me/devices/{device_id}/rotate-token` | Rotar token del dispositivo autenticado, cuerpo `{}`; 201. |
| POST | `/v1/me/devices/{device_id}/revoke` | Revocar un dispositivo propio; 204. |
| GET | `/v1/me/invitations` | Invitaciones propias; filtro `status`. |
| POST | `/v1/me/invitations/{invitation_id}/accept` | Aceptar como destinatario con `{operationId}`; 200. |
| GET | `/v1/accounts` | Cuentas propias, incluidas inactivas. |
| GET | `/v1/me/professional` | Perfil profesional propio; 404 si no existe. |
| GET | `/v1/me/services` | Catálogo privado propio; filtros `q`, `status`. |

### 7.2 Administración de la cuenta

| Método | Ruta | Uso / acceso |
|---|---|---|
| GET | `A` | Datos de cuenta; propietario conserva lectura mínima para reactivar. |
| GET | `A/access` | Rol, estado, `permissionEpoch` y acciones; permite consultar antes del registro de instalación. |
| GET | `A/members` | Equipo; solo propietario. |
| GET | `A/invitations` | Invitaciones; solo propietario, filtro `status`. |
| GET | `A/devices` | Autorizaciones de dispositivos de esta cuenta; solo propietario. |
| GET | `A/audit-events` | Auditoría; solo propietario, filtros `from`, `until`, `action`, `objectType`, `objectId`. |
| GET | `A/professionals` | Profesionales visibles; filtro `status`. |

### 7.3 Pacientes, contactos y servicios

| Método | Ruta | Filtros / datos |
|---|---|---|
| GET | `A/patients` | `q`, `status`, `completionStatus`. |
| GET | `A/patients/{patient_id}` | Ficha de la cuenta. |
| GET | `A/patients/{patient_id}/identifiers` | Documentos permitidos, sin búsqueda global. |
| GET | `A/patients/{patient_id}/contacts` | Vínculos y datos relacionados; `status`. |
| GET | `A/contacts` | `q`, `status`. |
| GET | `A/contacts/{contact_id}` | Contacto de la cuenta. |
| GET | `A/contacts/{contact_id}/patients` | Vínculos con pacientes de la cuenta; `status`. |
| GET | `A/services` | Ofertas visibles; `professionalId`, `q`, `status`. |

### 7.4 Agenda, disponibilidad y citas

| Método | Ruta | Filtros / datos |
|---|---|---|
| GET | `A/schedule/rules` | `professionalId`, `isoWeekday`, `status`, `fromDate`, `untilDate`. |
| GET | `A/schedule/rules/{rule_id}` | Regla visible. |
| GET | `A/schedule/exceptions` | `professionalId`, `status`, `fromDate`, `untilDate`; rango requerido de hasta 366 días. |
| GET | `A/schedule/exceptions/{exception_id}` | Excepción visible. |
| GET | `A/schedule/blocks` | `professionalId`, `status`, `from`, `until`; rango requerido de hasta 366 días. |
| GET | `A/schedule/blocks/{block_id}` | Bloqueo visible; motivo global limitado al profesional propio. |
| GET | `A/availability` | Requiere `professionalId`, `serviceId`, `from`, `until`; `durationMinutes` opcional, rango hasta 31 días. |
| GET | `A/appointments` | Requiere `from`, `until`, hasta 366 días; filtros `professionalId`, `serviceId`, `patientId`, `status`. |
| GET | `A/appointments/{appointment_id}` | Cita visible, también fuera de la ventana descargada. |
| GET | `A/appointments/{appointment_id}/history` | Historia por versión; filtro `afterVersion`. |

### 7.5 Operaciones y sincronización

| Método | Ruta | Uso |
|---|---|---|
| POST | `A/sync/operations` | Lote de acciones tipadas; 200 con resultado por operación. |
| GET | `A/sync/operations/{operation_id}` | Resultado del actor original, con acceso vigente. |
| POST | `A/sync/snapshots` | Crear captura coherente; 201, cuerpo `{}` o fechas de ventana. |
| GET | `A/sync/snapshots/{snapshot_id}` | Metadatos de captura del usuario/dispositivo creador. |
| GET | `A/sync/snapshots/{snapshot_id}/items` | Elementos materializados con `page`/`shows`. |
| DELETE | `A/sync/snapshots/{snapshot_id}` | Liberar captura temporal; 204. |
| GET | `A/sync/changes` | Descargar grupos completos desde `cursor`; sin `page`/`shows`. |
| GET | `A/sync/events` | SSE desde `cursor`; token API en cabecera. |

Las listas y páginas de captura usan `page` desde 1, `shows=100` por defecto y máximo 500; respuesta `{items,total}`. Una página posterior al final devuelve lista vacía y conserva el total. Los filtros se aplican dentro del ámbito autorizado antes del conteo.

Las listas corrientes pueden variar entre peticiones si otro usuario cambia datos. La captura inicial ofrece la vista estable. Los listados incluyen estados sin filtro por defecto; una pantalla que quiera solo activos debe indicarlo. La búsqueda `q` se limita a la cuenta y no abre un directorio global.

Los calendarios seleccionan citas por intersección con `[from,until)`, incluidas las que empiezan antes del rango y terminan dentro. La historia ordena por versión; las citas por inicio e ID. Los detalles exactos de obligatoriedad, tipos y respuestas figuran en OpenAPI.

`/openapi.json`, `/docs` y `/redoc` se sirven solo en desarrollo/prueba. La bienvenida `/` queda fuera del contrato público exportado. La URL base debe configurarse por entorno; este conjunto no fija un servidor de producción.

## 8. Escrituras y catálogo de acciones

Todas las escrituras de negocio de cuenta pasan por `POST A/sync/operations`. Emisión/rotación/revocación global de credenciales y aceptación de invitación usan sus rutas propias.

```json
{
  "operations": [
    {
      "operationId": "019a1234-0000-7000-8000-000000000001",
      "contractVersion": 1,
      "action": "RegisterPatient",
      "entityKey": "019a1234-0000-7000-8000-000000000002",
      "expectedVersion": 0,
      "payload": {"fullName": "Paciente de prueba"},
      "dependsOn": []
    }
  ]
}
```

El ejemplo crea una identidad nueva y su ficha `draft` en la cuenta autorizada; no habilita una cita. Para completar y citar hay que cumplir los requisitos del apartado 5.2.

`entityKey` es un UUIDv7 o una clave compuesta de la tabla siguiente. La cuenta procede de la ruta. Nuevas entidades del negocio usan UUIDv7 del cliente; la emisión de dispositivos asigna sus IDs en servidor. Crear exige versión 0; cambiar exige versión positiva actual.

| Grupo | Acciones por lote | Clave y regla principal |
|---|---|---|
| Cuenta | `UpdateAccountDetails`, `UpdateAccountPolicy`, `SetAccountStatus` | UUID de cuenta; propietario. Política usa `noShowGraceMinutes`. Detalles admiten patch de nombre, zona y lugar. |
| Invitaciones | `CreateInvitation`, `RevokeInvitation` | UUID de invitación; propietario. Crear exige `inviteeUserId` ya identificado; revocar usa `{}`. |
| Equipo | `SetMembershipStatus` | UUID del usuario miembro; propietario, `status`. No transfiere propiedad. |
| Dispositivo en cuenta | `RegisterAccountDevice`, `RevokeAccountDevice`, `RestoreAccountDevice` | UUID de dispositivo; payload `{}`. Registro inicial solo propio; revocar/restaurar exige propietario. |
| Profesional | `UpdateProfessionalProfile` | UUID del profesional propio; propietario, `publicName`. |
| Pacientes | `RegisterPatient`, `UpdatePatientProfile`, `SetPatientCompletion`, `SetPatientProfileStatus` | UUID del paciente; registro inicial puede incluir documentos. Completar usa `completionStatus`; actividad usa `status`. |
| Documento | `AddPatientIdentifier` | UUID del documento nuevo; payload `patientId`, país emisor, tipo y número. |
| Contactos | `CreateContactPerson`, `UpdateContactPerson`, `SetContactPersonStatus` | UUID del contacto; datos administrativos o `status`. |
| Vínculos | `CreatePatientContactRelationship`, `UpdatePatientContactRelationship`, `SetPatientContactRelationshipStatus` | `{patientId,contactPersonId}`; parentesco, función responsable, vigencia o `status`. |
| Catálogo | `CreateService`, `UpdateService`, `SetServiceStatus` | UUID del servicio; solo profesional propietario. Nombre y duración base; creación incluye `professionalId`. |
| Oferta | `OfferService`, `UpdateServiceOffer`, `SetServiceOfferStatus` | `{professionalId,serviceId}`; solo profesional propietario. Ajuste de duración nulo usa base. |
| Regla semanal | `CreateScheduleRule`, `UpdateScheduleRule`, `SetScheduleRuleStatus` | UUID de regla; profesional, día ISO, minutos locales y vigencia. |
| Excepción | `CreateScheduleException`, `UpdateScheduleException`, `SetScheduleExceptionStatus` | UUID de excepción; profesional, fecha local, clase y extremos de reemplazo si corresponde. |
| Bloqueo | `CreateScheduleBlock`, `UpdateScheduleBlock`, `SetScheduleBlockStatus` | UUID de bloqueo; profesional, ámbito de cuenta, instantes y `reasonCode`. Global solo profesional propio. |
| Cita | `CreateAppointment`, `ConfirmAppointment`, `RescheduleAppointment`, `CancelAppointment`, `CompleteAppointment`, `MarkAppointmentNoShow` | UUID de cita; reglas y estados del apartado 5.4. |

Para crear una cita, el payload requiere `patientId`, `professionalId`, `serviceId`, `startsAt` y `endsAt`; `responsibleContactId` es opcional salvo cuando la base del paciente lo exige. Confirmar usa `{}`. Reprogramar recibe solo inicio y fin. Cancelar, completar y marcar ausencia admiten `changeReason` administrativo opcional. El servidor obtiene actor, estado, confirmación y snapshots; el cliente no los asigna.

`UpdateServiceOffer` exige que `durationMinutesOverride` esté presente y admite nulo para volver a la base. En patches, ausencia conserva el dato y nulo solo lo retira donde el modelo lo permita. Un patch vacío se rechaza. Un cambio sin diferencia produce `NO_CHANGES`, sin versión funcional nueva.

`dependsOn` admite hasta ocho IDs del mismo actor/cuenta, ya aceptados o anteriores en el lote. Duplicados de ID, ciclos, dependencias futuras o envoltorios mal formados producen 422 antes de procesar. Una dependencia inexistente/rechazada produce `DEPENDENCY_FAILED`; una dependencia con fallo temporal mantiene posibilidad de reintento. Las operaciones independientes pueden confirmar aunque otra falle.

Las acciones solo con conexión figuran en `capabilities.json`: `AcceptInvitation`, `CreateInvitation`, `RegisterAccountDevice`, `RestoreAccountDevice`, `RevokeAccountDevice`, `RevokeInvitation`, `SetAccountStatus`, `SetMembershipStatus`, `UpdateAccountDetails`, `UpdateAccountPolicy`. `AcceptInvitation` usa su ruta propia y no integra las 42 acciones del lote.

### Resultados y reintentos

La respuesta del lote contiene `results` y `correlationId`. Cada resultado identifica `operationId`, `status`, `entityVersion`, `state` y `error`; `duplicate` incorpora `originalResult`.

| `status` | Tratamiento del escritorio |
|---|---|
| `accepted` | El servidor confirmó efecto y resultado. Registrar el resultado de la intención local. |
| `duplicate` | Recuperar el resultado original, que también puede ser un rechazo o conflicto. No aplicar otra escritura. |
| `conflict` | Revisar versión/estado remoto o disponibilidad. No sobrescribir el estado nuevo. |
| `rejected` | Mostrar el código y conservar la intención para corrección o revisión. Un cambio de intención usa otro ID. |
| `retryable_failure` | Consultar o reintentar la misma intención cuando se recupere la dependencia. No fijar rechazo definitivo. |

La clave idempotente combina cuenta, actor y `operationId`. Mantener el mismo ID y contenido semántico en cada reintento, incluso desde otra instalación autorizada del mismo usuario. Cambiar acción, payload, versión esperada, dependencias o `occurredAtLocal` con el mismo ID produce `IDEMPOTENCY_CONFLICT`.

Ante una respuesta perdida, consultar `GET A/sync/operations/{operation_id}` o repetir la misma operación. Un 404 de consulta no autoriza por sí solo a crear otro ID: también puede reflejar falta de visibilidad. El resultado completo dura al menos 90 días; después queda un recibo mínimo que impide repetir el efecto. `RESULT_EXPIRED` exige revisar el estado actual antes de crear una nueva intención.

HTTP 200 del lote no implica que todas las operaciones se aceptaron. Un fallo de acceso puede terminar la petición después de que operaciones previas del mismo lote ya confirmaron. Si no se conoce qué confirmó, conservar todos los IDs y recuperar sus resultados con acceso válido.

## 9. Captura inicial, descarga y avisos

### Captura inicial

1. Autenticar usuario y habilitar instalación en la cuenta.
2. Enviar `POST A/sync/snapshots` con `{}` o `{appointmentsFrom,appointmentsUntil}`. Son fechas locales; si se envían, no admiten nulo.
3. Conservar `snapshotId`, `window`, `baseCursor`, `expiresAt` e `itemCount`. La ventana por defecto va de hoy local menos 30 días a hoy más 181, límite exclusivo; incluye hoy más 180. Máximo: 366 días.
4. Descargar `A/sync/snapshots/{snapshot_id}/items?page=1&shows=100` y el resto de páginas. La captura dura 15 minutos, conserva orden y contenido, y vuelve a exigir acceso vigente.
5. Al completar la descarga, instalar la vista remota y `baseCursor` juntos en una transacción SQLite. Conservar las intenciones locales pendientes al sustituir la vista remota.
6. Liberar el derivado mediante DELETE cuando deje de hacer falta. No borra datos de negocio ni recibos.

El servidor materializa datos y posición base bajo una misma vista PostgreSQL. No mantiene una transacción abierta entre páginas. Si la captura vence o cambia su alcance, hay que empezar otra captura explícita; no mezclar páginas de capturas distintas.

La captura contiene datos operativos y referencias necesarias: fichas, contactos/vínculos, servicios/ofertas permitidos, agenda y citas que intersectan ventana con su historia. Incluye referencias inactivas necesarias para interpretar las citas. El propietario recibe su catálogo privado y datos administrativos permitidos; el asistente recibe su vista limitada.

### Descarga incremental

Pedir `GET A/sync/changes?cursor=<cursor_aplicado>`. La respuesta contiene `groups`, `nextCursor` y `hasMore`. El servidor examina hasta 100 grupos completos por petición; no admite `page`/`shows` ni divide un grupo.

Aplicar los cambios de la página y guardar `nextCursor` en la misma transacción SQLite. Cada grupo contiene hechos de una operación y puede afectar varias entidades. Respetar esa unidad al aplicarlo. Si falla la transacción local, conservar el cursor anterior y volver a descargar.

Continuar mientras `hasMore=true`, aunque `groups` esté vacío: el servidor puede avanzar sobre grupos de otra audiencia o fuera de ventana sin entregar su contenido. La versión de entidad y el ID de operación no sustituyen al cursor aplicado. Aceptar una escritura tampoco acredita que el escritorio haya descargado los cambios anteriores.

`changeKind=upsert` entrega estado; `remove` retira la entidad de la vista. Si una cita sale de ventana, se retiran cita e historia locales sin borrar su historia funcional en API. Conservar aparte las intenciones que dependan de ellas para revisión.

El cursor se liga a cuenta, usuario, dispositivo, audiencia, época de acceso y ventana, con firma y plazo de 90 días. La retención del journal puede exigir recuperación antes de que venza la firma. Cambiar zona no recalcula la ventana capturada; cambiar ventana exige otra captura. Los grupos se conservan al menos 90 días y una posición fuera de retención exige recuperación explícita.

### SSE

Abrir `GET A/sync/events?cursor=<cursor_aplicado>` con `Authorization`. El cliente Rust debe poder enviar esa cabecera y leer SSE; no se selecciona aquí una biblioteca.

| Evento | Datos y efecto |
|---|---|
| `changes_available` | `{accountId,cursor}`; iniciar descarga con el último cursor aplicado local. El cursor del aviso conserva la posición dada al abrir el stream. |
| `heartbeat` | `{serverTime}` cada 15 segundos; el servidor también revalida acceso. |
| `access_revoked` | Sin datos; cierra el stream. Detener el acceso y revisar permisos. |

El campo `id` SSE y `Last-Event-ID` son referencias del aviso, no prueba de aplicación. Al reconectar, usar el cursor aplicado más reciente. El servidor sugiere reintento de 3000 ms; journal y heartbeat permiten recuperar avisos perdidos.

Hay un máximo compartido de dos conexiones por dispositivo/cuenta entre procesos. Tras una caída, un registro puede ocupar cupo hasta 45 segundos. El servidor desconecta consumidores lentos y limita la cola pendiente; el cliente recupera por cursor. Perder el listener degrada `/health/ready` a 503, mientras las rutas de datos siguen disponibles.

## 10. Formatos, límites y errores

| Dato o límite | Contrato |
|---|---|
| JSON y filtros públicos | camelCase; acciones y códigos conservan su forma exacta. Los parámetros de ruta usan los nombres de OpenAPI. |
| IDs | UUIDv7 canónicos. Conservarlos como identidad estable. |
| Instantes | RFC 3339 con offset explícito; respuesta UTC con `Z`. Sin fechas sin zona ni valores infinitos. |
| Fechas y zonas | `YYYY-MM-DD` y zonas IANA. No deducir zona del teléfono o documento. |
| Citas | Extremos sin segundos/fracciones; 1–1440 minutos. |
| Lote | 1–100 operaciones, hasta 1 MiB de cuerpo y 64 KiB por operación UTF-8. |
| Listas y páginas de captura | `page>=1`; `shows` de 1 a 500, por defecto 100. |
| Captura | 15 minutos; ventana máxima 366 días. |
| Cambios/resultados completos | Retención mínima 90 días. El recibo mínimo permanece. |
| Correlación | `X-Correlation-ID` del servidor y `correlationId` en errores/resultados; no concede permisos. |

La API rechaza campos extra y parámetros no admitidos. País emisor y país del lugar usan códigos ISO alfa-2 asignados; `phoneCode` es un prefijo telefónico distinto. Los modelos validan forma y longitud posible del teléfono, sin probar titularidad. El cliente debe respetar ausencia frente a nulo y no completar campos omitidos antes de decidir qué patch quiere enviar.

Los errores HTTP usan `{code,message,correlationId,details}`. `details` contiene ubicaciones/códigos seguros. Usar `code` para decidir el comportamiento; el texto del mensaje puede cambiar.

| Respuesta / código | Acción del consumidor |
|---|---|
| 401 `AUTHENTICATION_REQUIRED` | Revisar credencial/dispositivo; volver a acceso si corresponde. Conservar IDs de operaciones con resultado desconocido. |
| 403 `PERMISSION_DENIED` | Detener el acceso afectado y revisar permisos; conservar intenciones. |
| 404 `NOT_FOUND` | Recurso inexistente o no visible. No deducir datos de otra cuenta. |
| Conflicto `VERSION_CONFLICT` | Leer estado autorizado actual y revisar la intención; no sobrescribir por fuerza. |
| Conflicto `SLOT_UNAVAILABLE` | Revisar agenda/horario; la consulta de disponibilidad no garantizaba reserva. |
| Conflicto `IDEMPOTENCY_CONFLICT` | Se reutilizó un ID con otro contenido. Revisar el registro local. |
| Rechazo `COMPLETENESS_CONFLICT` | Resolver requisitos de ficha/contactos con operaciones explícitas. |
| Rechazo `INVALID_STATE_TRANSITION` / `NO_CHANGES` | Revisar estado y acción; no añadir una versión por insistencia. |
| 410 `CURSOR_EXPIRED` | Crear otra captura y reconstruir vista remota, conservando intenciones locales. Incluye firma, alcance, época, plazo o frontera inválidos. |
| 410 `SNAPSHOT_EXPIRED` / `SNAPSHOT_SCOPE_CHANGED` | Descartar ese derivado y pedir otra captura con permisos actuales. |
| 410 `RESULT_EXPIRED` | Revisar estado actual; no repetir automáticamente con otro ID. |
| 413 / 422 | Corregir tamaño o estructura. Un payload de negocio inválido dentro de un envoltorio válido puede producir rechazo por operación con HTTP 200. |
| 429 `CONNECTION_LIMIT` | Cerrar streams sobrantes o esperar su liberación; conservar cursor aplicado. |
| 503 / `retryable_failure` | Reintentar la misma identidad cuando se recupere la dependencia. No asumir rechazo definitivo si el resultado es desconocido. |

Aceptar invitación tiene su respuesta propia y puede rechazar con HTTP 409; los conflictos de negocio del lote usan HTTP 200 con resultado individual. Una captura ajena, inexistente o retirada físicamente devuelve el mismo 404. Mientras exista y el creador tenga acceso, la API distingue vencimiento y cambio de época con 410.

## 11. Trabajo que debe resolver el escritorio

Este apartado fija obligaciones del consumidor; no afirma que el repositorio `nightingale` ya las cumpla.

- Separar estado remoto aceptado, intención local y estado visible de sincronización. Un rechazo remoto no borra el trabajo guardado sin red.
- Conservar el contenido y el ID de una operación enviada hasta resolver su resultado; no crear otro ID por timeout, caída o cambio de instalación.
- Preparar las acciones administrativas solo con conexión y usar permisos vigentes al volver a sincronizar.
- Guardar tokens en almacenamiento seguro y tratar por separado revocación global, acceso por cuenta y logout OIDC.
- Aplicar captura completa/base y página/cursor de forma atómica en SQLite; soportar repetición, grupos vacíos con avance y retiradas de vista.
- Respetar `entityType`, claves compuestas, audiencia y las dos versiones de oferta. Validar los modelos de entrada sin convertir el esquema SQL remoto en el esquema local por defecto.
- Mostrar conflictos para revisión. Una resolución autorizada puede crear otra intención; no debe repetir un efecto cuyo resultado aún se desconoce.
- Conservar el mismo instante al mostrar zonas distintas. La presentación para telemedicina debe indicar fecha y zona, sin mover la cita al cambiar la zona de la cuenta.
- Probar con dos dispositivos, respuesta perdida, permisos revocados, cursor/captura vencidos, caída SSE, cita fuera de ventana y reinicio durante escritura local.

## 12. Origen, actualización y comprobaciones

Fuentes propietarias en el repositorio `nightingale-api`: `docs/nightingale-api.md` para alcance; `docs/decisions.md` para acuerdos; `docs/api-v1.md` para reglas HTTP; `docs/database-schema.md` para diseño físico; `src/application.py`, `src/api/api.py` y los modelos/casos para comportamiento ejecutable. El estado y evidencia se registran en `tasks/P-20261024-01-desktop-shared-api/todo.md`. Esta guía resume esas fuentes y no traslada su propiedad al escritorio.

Para actualizar el conjunto, desde `nightingale-api` con su entorno existente:

```powershell
uv run --locked python -m scripts.export_contracts
uv run --locked python -m scripts.export_contracts --check
```

El exportador no conecta a PostgreSQL, no abre OIDC y no arranca el listener. Contrasta las rutas y las acciones con el catálogo de diseño y conserva origen, versiones y huellas. Regenerar los JSON, revisar esta guía y copiar el conjunto; no corregir solo el OpenAPI del escritorio.

La entrega conserva el router central de `src/api/api.py` y lo conecta a la app. El paquete `src/api/__init__.py` no importa endpoints al importar un modelo. La conexión usa el patrón documentado de [FastAPI para incluir routers](https://fastapi.tiangolo.com/tutorial/bigger-applications/#include-the-apirouter-s-for-users-and-items), contrastado con FastAPI 0.159.1 instalado; no añade dependencias.

Comprobación de la entrega, terminada el 2026-10-29:

- Exportación desde la app y comprobación reproducible del conjunto; 46 rutas, 47 operaciones HTTP, 70 esquemas y 42 acciones. El contenido HTTP coincide con el artefacto anterior salvo metadatos de origen.
- Catálogo de endpoints de esta guía contrastado con OpenAPI; todas las acciones enumeradas, 428 referencias JSON locales resueltas, hashes del manifiesto y enlaces locales comprobados. El ejemplo pasa los modelos reales de envoltorio y payload.
- Suite actual: 144 pruebas en 64,838 segundos, con los dos errores de subcasos ya descritos. La ejecución inicial dentro del aislamiento de Windows no acredita resultados: además del efecto de importación ya corregido, `asyncio` se bloqueaba en `socketpair`. La repetición fuera de ese aislamiento completó la suite.
- Build offline correcto: wheel de 185 entradas y sdist de 186, con 125 fuentes Python idénticas al árbol en ambos y sin archivos privados. No se ejecuta HTTP desde un wheel instalado.

Las pruebas PostgreSQL/Keycloak, concurrencia y SSE citadas en el estado son las registradas antes de esta entrega; no se repitieron aquí. Tampoco se repitió el analizador estático. El cambio actual conecta rutas y prepara documentación/contratos; no altera las migraciones ni las reglas de negocio.

La comprobación del consumidor con Rust/SQLite, el despliegue y los servicios de producción requieren evidencia propia. Este conjunto aporta el contrato y las reglas para realizar ese trabajo.
