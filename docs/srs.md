# Especificación de Requisitos de Software (SRS): Nightingale

> Documento centrado en la visión, el alcance y las funciones esperadas del producto. Las decisiones técnicas y de implementación quedan fuera de esta versión.

## 1. Identificación del producto

**Nombre del producto:** Nightingale  
**Descripción:** Aplicación para que médicos y especialistas independientes gestionen citas médicas, se comuniquen con sus pacientes y reduzcan el ausentismo.\
**Versión del documento:** 0.3\
**Fecha:** 19/10/2026

El segmento inicial incluye al profesional y a sus asistentes autorizados. La gestión de clínicas con varios médicos o sedes queda para una evolución posterior, conforme al plan maestro §4.1.

## 2. Problema que queremos resolver

Las clínicas y los especialistas gestionan buena parte de sus citas mediante procesos manuales y canales separados. Esto dificulta mantener actualizada la agenda, confirmar la asistencia de los pacientes y ocupar a tiempo los horarios liberados por cancelaciones.

El problema afecta principalmente a:

- **Personal administrativo:** dedica tiempo a registrar cambios, enviar recordatorios y contactar pacientes manualmente.
- **Personal médico:** pierde capacidad de atención cuando una cita queda vacía o la agenda no refleja la disponibilidad real.
- **Pacientes:** pueden olvidar sus citas, recibir información incompleta o tener dificultades para confirmar, cancelar o reprogramar.

Cuando el problema no se resuelve, se producen ausencias sin aviso, consultorios vacíos, pacientes perdidos, retrasos, trabajo administrativo repetitivo y una experiencia deficiente para todas las partes.

## 3. Visión del producto

Nightingale será un producto confiable y sencillo que realizará el trabajo tedioso relacionado con la gestión de citas. Permitirá que el personal administrativo y médico dedique más tiempo a la atención humana de los pacientes.

El producto centralizará las citas, los datos básicos de los pacientes, los cambios de estado y las comunicaciones asociadas. Su Patient Hub permitirá que el paciente consulte y gestione su propia cita mediante una experiencia sencilla, sin necesidad de entrar al panel interno de la clínica ni instalar una aplicación.

Las clínicas y los especialistas tendrán mayor seguridad sobre la disponibilidad de sus agendas, la intención de asistencia de sus pacientes y la posibilidad de recuperar horarios cancelados.

**Propuesta de valor:** Nightingale ayuda a clínicas y especialistas a gestionar sus citas de forma confiable, disminuir ausencias y recuperar horarios disponibles mediante un motor de agenda, un Patient Hub y recordatorios automatizados.

## 4. Objetivos del producto

1. Centralizar la creación y gestión de citas médicas.
2. Facilitar que los pacientes consulten, confirmen, cancelen o soliciten la reprogramación de sus citas.
3. Reducir las ausencias mediante recordatorios y seguimiento de citas sin confirmar.
4. Recuperar horarios cancelados mediante una lista de espera.
5. Mostrar indicadores básicos sobre citas, confirmaciones, cancelaciones, asistencia y ausencias.

El seguimiento clínico, las analíticas avanzadas y la gestión de récipes podrán evaluarse en versiones posteriores, pero no constituyen el foco inicial.

## 5. Usuarios y participantes

| Usuario o participante | Necesidad principal | Acciones esperadas |
|---|---|---|
| Paciente | Recordar y gestionar su cita fácilmente | Consultar los datos de su cita, confirmar, cancelar o solicitar una reprogramación desde el Patient Hub |
| Recepcionista o personal administrativo | Mantener la agenda actualizada y reducir el trabajo manual | Registrar citas, modificar estados, reprogramar, cancelar, enviar o supervisar recordatorios y gestionar la lista de espera |
| Médico o especialista | Conocer su agenda y la situación de sus pacientes citados | Consultar citas y pacientes programados, además de revisar su estado de asistencia |
| Administrador del servicio | Supervisar la operación | Consultar indicadores, configurar reglas operativas y administrar los accesos del personal |

## 6. Alcance del producto

Las listas de este documento definen el alcance esperado; no indican funciones implementadas. Los planes de `tasks/` registran el trabajo y sus comprobaciones. La base de desarrollo actual se limita al escritorio; los servicios web tendrán planes posteriores.

### 6.1 Funciones incluidas en la primera versión

- Registro y administración de los datos básicos de pacientes.
- Consulta del profesional, su especialidad, servicios y horarios previamente configurados.
- Creación, consulta, confirmación, cancelación y reprogramación de citas.
- Control de disponibilidad y prevención de conflictos de horario.
- Recordatorios y notificaciones automáticas mediante WhatsApp.
- Patient Hub básico accesible desde un enlace seguro para gestionar citas existentes.
- Lista de espera y recuperación de horarios cancelados.
- Registro de asistencia y ausencias.
- Panel de indicadores básicos.

### 6.2 Funciones no incluidas en la primera versión

- Administración avanzada de múltiples clínicas, sedes o redes hospitalarias.
- Autogestión completa de médicos y de la estructura organizativa de la clínica.
- Historia clínica electrónica.
- Diagnósticos o recomendaciones médicas automatizadas.
- Check-In y gestión de filas presenciales.
- Pagos, seguros y autorizaciones.
- Integraciones con sistemas clínicos o administrativos existentes, salvo el canal de mensajería necesario para los recordatorios.
- Predicción de ausencias mediante inteligencia artificial.
- Gestión de récipes o prescripciones médicas.

## 7. Funcionamiento esperado

### 7.1 Para la clínica o el especialista

El personal autorizado podrá registrar una cita asociando al paciente con un médico o especialista, un servicio, una fecha y un horario disponible. Podrá consultar la agenda, cambiar la cita, reprogramarla o cancelarla cuando sea necesario.

Antes de la consulta, Nightingale enviará los recordatorios definidos por el servicio y actualizará la cita según la respuesta del paciente. El personal podrá identificar las citas confirmadas, canceladas y pendientes de respuesta, así como registrar si el paciente asistió o se ausentó.

### 7.2 Para el paciente

El paciente no utilizará el panel interno del sistema. Recibirá una notificación por WhatsApp con la información administrativa necesaria y un acceso seguro al Patient Hub.

Desde ese acceso podrá:

- Consultar la fecha, hora, especialista, servicio y lugar de la cita.
- Confirmar su asistencia.
- Cancelar la cita.
- Solicitar una reprogramación para que el profesional o su asistente la revise.
- Consultar el estado actualizado de su cita.

El paciente no tendrá que instalar una aplicación. El acceso inicial será mediante un enlace seguro asociado con una cita existente, con vencimiento y revocación. La duración y la verificación adicional del enlace quedan pendientes. El Patient Hub no permitirá crear citas nuevas, consultar la agenda general ni reservar otro horario. Una solicitud de reprogramación no modificará la cita hasta que el personal la revise y defina el cambio, conforme al plan maestro §17.

### 7.3 Ante una cancelación o posible ausencia

Cuando un paciente cancele, el horario volverá a estar disponible. Nightingale identificará a los pacientes compatibles que estén en lista de espera y les ofrecerá la posibilidad de ocuparlo.

Si un paciente no responde a los recordatorios dentro del tiempo establecido, la cita aparecerá como pendiente de confirmación y el personal administrativo recibirá una alerta para decidir si realiza seguimiento.

La ausencia no se registrará de forma automática por falta de respuesta: deberá confirmarse después de la hora prevista de la cita.

## 8. Requisitos funcionales principales

| ID | El sistema deberá... | Prioridad |
|---|---|---|
| RF-01 | Permitir al personal autorizado registrar una cita para un paciente en un horario disponible. | Alta |
| RF-02 | Permitir consultar la agenda por fecha, médico o especialista, servicio y estado de la cita. | Alta |
| RF-03 | Impedir que un mismo horario o recurso sea reservado simultáneamente por más de una cita. | Alta |
| RF-04 | Permitir confirmar, cancelar y reprogramar una cita, conservando un registro de sus cambios. | Alta |
| RF-05 | Enviar recordatorios de cita al paciente y registrar si fueron enviados, entregados y respondidos. | Alta |
| RF-06 | Proporcionar al paciente un acceso seguro al Patient Hub para consultar y gestionar únicamente sus propias citas. | Alta |
| RF-07 | Identificar las citas pendientes de confirmación y alertar al personal administrativo. | Alta |
| RF-08 | Liberar un horario cuando una cita sea cancelada o reprogramada. | Alta |
| RF-09 | Mantener una lista de espera y ofrecer horarios liberados a pacientes compatibles. | Alta |
| RF-10 | Permitir registrar una cita como atendida, cancelada, reprogramada o ausencia sin aviso. | Alta |
| RF-11 | Mostrar indicadores básicos de citas, confirmaciones, cancelaciones, ausencias y horarios recuperados. | Media |
| RF-12 | Restringir las acciones y la información disponible según el tipo de usuario. | Alta |

## 9. Reglas del servicio

- **Reservas:** solo el profesional o su asistente autorizado podrán crear una cita. Toda cita deberá asociarse con un paciente, un servicio, un profesional y un horario disponible. El Patient Hub solo gestionará citas existentes.
- **Disponibilidad:** un horario estará disponible cuando forme parte de la agenda habilitada y no esté reservado, bloqueado o afectado por una ausencia del profesional.
- **Confirmaciones:** el sistema solicitará confirmación mediante recordatorios configurables. La falta de respuesta no cancelará automáticamente la cita.
- **Cancelaciones:** el paciente podrá cancelar desde el Patient Hub dentro del plazo definido por la clínica. Después de ese plazo, la solicitud podrá requerir intervención del personal.
- **Reprogramaciones:** una cita solo podrá trasladarse a un horario disponible. El sistema conservará el historial del cambio y liberará el horario anterior.
- **Lista de espera:** solo se ofrecerán horarios compatibles con el servicio solicitado. El horario se asignará al primer paciente que lo acepte de forma válida, conforme a la regla de prioridad que defina la clínica.
- **Ausencias:** una cita se marcará como ausencia cuando haya pasado la hora y el margen de espera definido, y el personal confirme que el paciente no asistió.
- **Privacidad:** el paciente solo podrá consultar sus propias citas. Cada integrante del personal verá únicamente la información necesaria para su función.
- **Mensajes:** los recordatorios no deberán incluir diagnósticos, resultados médicos ni otra información clínica sensible.

Los plazos exactos para recordatorios, cancelaciones, márgenes de espera y ofertas de la lista de espera deberán ser configurables y se definirán durante la validación operativa.

## 10. Información que gestionará el producto

- Identificación y datos de contacto básicos de los pacientes.
- Profesional, especialidad y servicios disponibles.
- Horarios, bloqueos y disponibilidad.
- Citas y su historial de cambios.
- Confirmaciones, cancelaciones, reprogramaciones, asistencias y ausencias.
- Posiciones y solicitudes de la lista de espera.
- Comunicaciones enviadas y respuestas recibidas.
- Usuarios internos, funciones y permisos.

**Información que no deberá almacenarse en la primera versión:** historia clínica, diagnósticos, resultados médicos, imágenes clínicas, información financiera completa o cualquier dato que no sea necesario para administrar la cita.

## 11. Expectativas de calidad

- **Facilidad de uso:** las acciones frecuentes deberán ser claras y requerir pocos pasos tanto para el personal como para el paciente.
- **Confiabilidad:** una cita confirmada o modificada deberá mostrar el mismo estado en la agenda, el Patient Hub y las comunicaciones posteriores.
- **Disponibilidad:** el personal deberá acceder al sistema durante la operación de la consulta y el paciente deberá poder gestionar su cita cuando reciba el recordatorio.
- **Velocidad:** la consulta de agenda y las acciones habituales deberán responder sin interrumpir el trabajo de recepción. El tiempo objetivo deberá definirse antes de las pruebas de aceptación.
- **Seguridad y privacidad:** el acceso deberá estar limitado por funciones; las acciones relevantes deberán quedar registradas y los enlaces para pacientes deberán ser seguros y limitados a sus propias citas.
- **Accesibilidad:** el Patient Hub deberá funcionar correctamente desde teléfonos móviles sin exigir la instalación de una aplicación.
- **Continuidad:** ante una interrupción de conexión o del servicio de mensajería, no deberán duplicarse citas ni perderse cambios confirmados. Los mensajes pendientes deberán quedar identificados para su reintento o seguimiento manual.

## 12. Integraciones necesarias

| Sistema o servicio | Finalidad | Primera versión |
|---|---|---|
| WhatsApp | Enviar recordatorios, enlaces al Patient Hub y recibir respuestas relacionadas con la cita | Sí |
| Sistema actual de la clínica | Importar o sincronizar agendas y pacientes | No |
| Correo electrónico o SMS | Canal alternativo de notificaciones | No |
| Pagos | Cobrar o registrar pagos asociados con una cita | No |
| Seguros o autorizaciones | Verificar requisitos administrativos | No |

WhatsApp será la única integración externa considerada indispensable en la primera versión. Las demás podrán evaluarse posteriormente.

## 13. Límites y condicionantes

**Restricciones conocidas:**

- El producto inicial se concentrará en la gestión de citas y no sustituirá una historia clínica ni un sistema hospitalario completo.
- Deberá ser utilizable desde teléfonos y equipos con conectividad irregular.
- Los pacientes no deberán instalar una aplicación.
- Los mensajes deberán evitar información clínica sensible.
- La primera versión no dependerá de integraciones con los sistemas actuales de las clínicas.

**Supuestos que deben validarse:**

- WhatsApp es un canal adecuado para la mayoría de los pacientes del segmento inicial.
- Las clínicas y los especialistas aceptarán registrar y mantener sus citas directamente en Nightingale durante la primera versión.
- Los pacientes utilizarán enlaces seguros para responder a recordatorios y gestionar sus citas.
- Una lista de espera puede administrarse con criterios simples de compatibilidad y prioridad.

**Dependencias externas:**

- Disponibilidad y condiciones del servicio oficial de WhatsApp utilizado para las comunicaciones.
- Calidad y actualidad de los números telefónicos registrados por la clínica.
- Definición de las políticas de cancelación, reprogramación y lista de espera de cada servicio.

## 14. Criterios de éxito

Los valores iniciales y las metas numéricas deberán establecerse con datos reales del primer servicio que utilice Nightingale.

| Indicador | Situación inicial | Resultado esperado |
|---|---|---|
| Porcentaje de ausencias sin aviso | Por medir | Reducción respecto a la línea base |
| Porcentaje de citas confirmadas antes de la consulta | Por medir | Aumento respecto a la línea base |
| Horarios cancelados ocupados desde la lista de espera | Por medir | Recuperación medible de horarios |
| Tiempo administrativo dedicado a recordatorios y cambios | Por medir | Reducción respecto al proceso manual |
| Mensajes con respuesta registrada | Por medir | Seguimiento confiable de las respuestas |
| Satisfacción del personal y de los pacientes | Por medir | Mejora comprobable durante el piloto |

## 15. Prioridades de la primera versión

### Es indispensable

- Registro básico de pacientes y consulta de profesionales, servicios y horarios.
- Motor de agenda con creación, confirmación, cancelación y reprogramación de citas.
- Recordatorios mediante WhatsApp y seguimiento de respuestas.
- Patient Hub básico para consultar y gestionar la cita.
- Control de estados, asistencia y ausencias.
- Lista de espera y recuperación de horarios.
- Indicadores operativos básicos.

### Es importante, pero puede esperar

- Seguimiento periódico de pacientes.
- Analíticas avanzadas y comparación histórica.
- Canales alternativos como SMS o correo electrónico.
- Configuración avanzada de múltiples clínicas y sedes.
- Gestión de récipes, después de definir su propósito y sus requisitos.

### No forma parte del producto inicial

- Historia clínica electrónica.
- Diagnósticos o recomendaciones médicas automatizadas.
- Inteligencia artificial para predecir ausencias.
- Check-In y filas presenciales.
- Pagos, seguros y autorizaciones.
- Integraciones con sistemas clínicos o administrativos existentes.

## 16. Preguntas pendientes

Las preguntas sobre el segmento inicial, la creación de citas por el paciente y el tipo de acceso al Patient Hub quedan resueltas en §§1, 7.2 y 9, conforme al plan maestro. Quedan pendientes:

1. ¿Cuánto durarán los enlaces del Patient Hub y qué acciones exigirán una verificación adicional?
2. ¿Cuántos recordatorios se enviarán y con cuánta anticipación?
3. ¿Hasta qué momento podrá el paciente cancelar o solicitar un cambio sin seguimiento adicional del personal?
4. ¿Qué criterio determinará el orden de los pacientes en la lista de espera?
5. ¿Cuánto tiempo debe pasar antes de marcar una cita como ausencia?
6. ¿Qué datos mínimos del paciente serán obligatorios?
7. ¿Qué información exacta podrá consultar el médico o especialista?
8. ¿Qué significa “gestión de récipes” dentro de la visión futura del producto?

## 17. Resumen ejecutivo

> **Nightingale** será una solución dirigida inicialmente a médicos y especialistas independientes y sus asistentes. Permitirá centralizar las citas, automatizar recordatorios y facilitar que los pacientes gestionen citas existentes mediante un Patient Hub sencillo. Su primera versión incluirá el registro básico de pacientes, el motor de agenda, las confirmaciones, cancelaciones y reprogramaciones, las notificaciones por WhatsApp, la lista de espera y los indicadores operativos. Excluirá la historia clínica, los diagnósticos automatizados, los pagos, las autorizaciones y las integraciones con sistemas clínicos existentes. El producto será considerado exitoso cuando reduzca las ausencias y el trabajo administrativo, aumente las confirmaciones y permita recuperar de forma medible los horarios cancelados.
