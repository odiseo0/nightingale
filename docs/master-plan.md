# Plan Maestro de Desarrollo de Nightingale

**Versión:** 0.3\
**Fecha:** 21/10/2026\
**Estado:** Fuente de verdad técnica\
**Ámbito:** Estructura y reglas para el desarrollo del sistema  

## 1. Función de este documento

Este documento es la fuente de verdad para organizar el desarrollo de Nightingale. Define qué partes forman el sistema, qué responsabilidad tiene cada una, cómo se relacionan y qué reglas deberán respetar sus implementaciones.

Su finalidad es servir como referencia al momento de crear especificaciones particulares, decisiones técnicas, pruebas, tareas o planes de ejecución. Esas actividades podrán derivarse de este documento, pero no forman parte de él.

Este documento no contiene:

- Backlog.
- Tareas.
- Historias de usuario.
- Estimaciones.
- Fechas de entrega.
- Asignaciones de trabajo.
- Priorización visual.
- Estados de ejecución.
- Un orden obligatorio de implementación.
- Diseño visual definitivo.

Cuando una decisión posterior contradiga este documento, deberá modificarse el plan maestro o registrarse una excepción explícita. No se deberá mantener una implementación incompatible por conocimiento informal.

## 2. Lenguaje normativo

Los términos se interpretarán de la siguiente manera:

- **Deberá:** requisito obligatorio.
- **No deberá:** prohibición obligatoria.
- **Debería:** recomendación que solo podrá omitirse con una razón documentada.
- **Podrá:** opción permitida, pero no exigida.
- **Pendiente:** decisión que todavía no puede considerarse parte definitiva del diseño.

Los ejemplos de nombres, rutas o contratos explican una intención. Solo serán obligatorios cuando el texto los identifique expresamente como convención.

## 3. Documentos de referencia

Este plan evita repetir información que ya está definida en otros documentos.

| Código | Documento | Responsabilidad |
|---|---|---|
| SRS | `docs/srs.md`, versión 0.5 | Visión, usuarios, alcance y requisitos funcionales |
| DAT | `docs/technical-architecture.md`, versión 0.3 | Arquitectura general, flujos, sincronización, despliegue y decisiones técnicas |
| PMD | Este documento | Estructura normativa para desarrollar componentes y módulos |

La [decisión de tres proyectos y sincronización transaccional por API](decisions/transactional-sync-three-projects.md), aceptada el 2026-10-21, sustituye el acceso directo del escritorio a PostgreSQL. Mantiene los casos de uso locales en Rust/SQLite. Esta revisión describe la arquitectura objetivo, no funciones implementadas.

Especificaciones iniciales de los proyectos externos: [Nightingale API](specifications/nightingale-api.md) y [Nightingale Patient Hub](specifications/nightingale-patient-hub.md).

Reglas de precedencia:

1. La intención funcional procede de la SRS y de las decisiones posteriores confirmadas.
2. La arquitectura general procede del DAT.
3. La organización interna del código y los límites de componentes proceden del PMD.
4. Una decisión registrada con posterioridad podrá reemplazar una anterior si identifica claramente qué sustituye.
5. Ante una contradicción no resuelta, la implementación deberá detenerse en el punto afectado hasta actualizar la fuente de verdad correspondiente.

Referencias principales al DAT:

- Principios de arquitectura: DAT, sección 4.
- Vista general y componentes desplegables: DAT, secciones 6 y 7.
- Modos de operación: DAT, sección 8.
- Sincronización: DAT, secciones 9 y 10.
- Estados y flujos de citas: DAT, secciones 11 y 12.
- Modelo conceptual de datos: DAT, sección 13.
- Recordatorios y cola: DAT, sección 15.
- Analítica y ETL: DAT, sección 16.
- Seguridad, despliegue y operación: DAT, secciones 17 a 22.

## 4. Alcance consolidado

### 4.1 Segmento inicial

Nightingale estará orientado inicialmente a:

- Médicos independientes.
- Especialistas con consulta privada.
- Asistentes autorizados por esos profesionales.

Las clínicas con múltiples médicos, sedes, departamentos o estructuras administrativas complejas se mantienen fuera del alcance inicial.

### 4.2 Superficies y proyectos del producto

| Proyecto | Superficies y responsabilidad |
|---|---|
| Nightingale | Escritorio para el médico y sus asistentes; casos de uso Rust y SQLite. |
| Nightingale Patient Hub | Web móvil y API propia para accesos y acciones del paciente sobre citas existentes. |
| Nightingale API | API transaccional de sincronización y acceso compartido; módulos y workers de recordatorios, integración con n8n y analítica. |

Cada proyecto tendrá repositorio propio. n8n y los proveedores de comunicación son herramientas externas, no proyectos adicionales de Nightingale. Las dos especificaciones cloud breves se conservan inicialmente en `docs/specifications/` para orientar su creación.

### 4.3 Regla del Patient Hub

El Patient Hub no permitirá crear citas nuevas. Solo podrá operar sobre citas previamente registradas por el médico o su asistente.

El paciente podrá:

- Consultar su cita.
- Confirmarla.
- Cancelarla dentro de las reglas permitidas.
- Solicitar una reprogramación.
- Consultar el estado resultante.

Una solicitud de reprogramación no modificará directamente la agenda.

El Patient Hub es una excepción explícita a la operación nativa: su servicio posee sus accesos, acciones y solicitudes. El escritorio consulta y ejecuta esas capacidades mediante su API. SQLite conserva una vista local de sus resultados, no una segunda autoridad ni una copia de sus reglas. Véase §17.

### 4.4 Regla de operación offline

Rust ejecutará los casos de uso operativos del escritorio y persistirá sus resultados en SQLite. Consultar o modificar datos locales dentro de los permisos vigentes no exigirá una petición HTTP ni una conexión activa con PostgreSQL.

El usuario podrá consultar la agenda de hoy y mañana que ya esté almacenada en el equipo, guardar cambios permitidos y conservarlos tras reiniciar. Los planes definirán la ventana de datos disponible en cada dispositivo.

El coordinador Rust enviará operaciones completas a Nightingale API por HTTPS y descargará cambios autorizados. La API revalidará acceso, versiones y reglas compartidas y confirmará sus transacciones en PostgreSQL privado. El escritorio no tendrá acceso SQL ni credenciales de esa base. No se compartirán archivos de base de datos.

PostgreSQL conservará el estado compartido. Una reserva local será válida respecto a los datos conocidos y quedará pendiente de sincronización hasta confirmar su escritura compartida. Un equipo desconectado no puede garantizar que otro equipo no reserve el mismo intervalo.

Las capacidades del Patient Hub, el envío de mensajes y el procesamiento analítico requieren sus servicios externos. Una falla de estos servicios no impedirá operar sobre los datos locales. Los resultados pendientes o desactualizados se mostrarán como tales.

### 4.5 Regla de concurrencia

Una cuenta profesional podrá utilizar varios dispositivos concurrentes. La coordinación se mantendrá sencilla mediante:

- Restricciones en PostgreSQL.
- Transacciones breves.
- Idempotencia.
- Versionado optimista.
- Recarga del estado actual ante conflictos.

No se incorporarán CRDT, bloqueos distribuidos ni event sourcing completo en el alcance inicial.

## 5. Decisiones confirmadas y abiertas

### 5.1 Confirmadas

| Área | Decisión |
|---|---|
| Escritorio | Tauri |
| Núcleo local | Rust |
| Interfaz | React y TypeScript |
| Estilos del esqueleto | TailwindCSS sin sistema visual definitivo |
| Persistencia local | SQLite por dispositivo |
| Persistencia central | PostgreSQL |
| Concurrencia | Varios dispositivos por cuenta profesional |
| Arquitectura interna | Arquitectura hexagonal |
| Organización | Empaquetado por componentes |
| Sincronización | Coordinador Rust y API transaccional HTTPS; Outbox local, idempotencia, cursores y versiones |
| Patient Hub | Solo citas existentes; su API es la única autoridad de sus capacidades y el escritorio la consume |
| Recordatorios | Ejecución cloud independiente del escritorio |
| Orquestación | n8n como adaptador, no como propietario del dominio |
| Analítica | ETL y PostgreSQL analítico separado del flujo operativo |
| UI inicial | Esqueleto Tauri funcional, sin diseño final |
| Plataformas de la base de desarrollo | Windows, macOS y Linux desde el inicio; matriz concreta en el ADR de la base de escritorio, aceptada en C01 |
| Proyectos | Nightingale (escritorio), Nightingale Patient Hub (web y API propia) y Nightingale API (Python/FastAPI, transacciones y workers), en repositorios separados |
| IA | Fuera del alcance inicial; proveedor sin definir |

### 5.2 Abiertas

| Área | Estado |
|---|---|
| Proveedor cloud | Abierto; AWS y DigitalOcean son candidatos |
| Broker o cola final | Abierto |
| Cifrado de SQLite | Abierto |
| Identidad y acceso | Alta, sesiones de API, permisos y revocación por concretar; uso local de los últimos permisos válidos; sin credenciales PostgreSQL en el escritorio |
| Duración fija o variable de citas | Abierta |
| Proveedor oficial de WhatsApp | Abierto |
| Frecuencia del ETL | Abierta |
| RPO, RTO y retención | Abiertos |

Las decisiones abiertas deberán expresarse mediante puertos, contratos o límites que permitan reemplazar adaptadores. No deberán ocultarse detrás de dependencias concretas elegidas de manera accidental.

## 6. Modelo arquitectónico interno

Nightingale utilizará arquitectura hexagonal dentro de cada componente funcional.

```mermaid
flowchart TD
    I["Adaptadores de entrada"] --> A["Aplicación"]
    A --> D["Dominio"]
    A --> P["Puertos de salida"]
    P --> O["Adaptadores de salida"]
    B["Composición y bootstrap"] --> I
    B --> O
```

### 6.1 Dominio

El dominio contendrá:

- Entidades.
- Objetos de valor.
- Invariantes.
- Políticas puras.
- Servicios de dominio cuando una regla no pertenezca a una única entidad.
- Eventos de dominio.
- Errores del dominio.

El dominio no deberá depender de:

- Tauri.
- React.
- SQLite.
- PostgreSQL.
- HTTP.
- n8n.
- WhatsApp.
- El reloj del sistema.
- Generadores concretos de identificadores.
- Variables de entorno.
- Bibliotecas de infraestructura.

### 6.2 Aplicación

La capa de aplicación contendrá:

- Casos de uso.
- Comandos y consultas.
- Coordinación de transacciones.
- Puertos de entrada.
- Puertos de salida.
- DTO internos.
- Autorización contextual.
- Mapeo de errores del dominio hacia resultados de aplicación.

La aplicación podrá depender del dominio de su componente y de contratos públicos permitidos de otros componentes. No dependerá de adaptadores concretos.

### 6.3 Puertos

Los puertos describirán capacidades necesarias, no tecnologías.

Ejemplos:

- `AppointmentRepository`
- `PatientRepository`
- `UnitOfWork`
- `Clock`
- `IdGenerator`
- `MessagePublisher`
- `ReminderScheduler`
- `AuditSink`
- `SyncOperationStore`
- `AccessTokenIssuer`

Un puerto no deberá llamarse `PostgresAppointmentRepository` ni incluir detalles de una biblioteca concreta.

### 6.4 Adaptadores de entrada

Podrán incluir:

- Comandos Tauri.
- Endpoints HTTP.
- Webhooks.
- Consumidores de cola.
- Flujos invocados por n8n.
- Procesos programados.
- Herramientas administrativas limitadas.

Los adaptadores de entrada validarán forma, autenticación y traducción de transporte. Las reglas de negocio permanecerán en dominio y aplicación.

### 6.5 Adaptadores de salida

Podrán incluir:

- Repositorios SQLite.
- Repositorios PostgreSQL.
- Cliente HTTPS del coordinador nativo hacia Nightingale API.
- Clientes HTTP del Patient Hub y otros servicios de apoyo.
- Broker de mensajes.
- Proveedor de WhatsApp.
- Almacenamiento seguro del sistema operativo.
- Reloj real.
- Generador UUID.
- n8n.
- Servicio analítico.

Los adaptadores PostgreSQL operativos pertenecen a Nightingale API. El escritorio usa SQLite y clientes HTTPS; el Hub usa el contrato de API para leer y cambiar citas.

### 6.6 Composición

La composición será el único lugar que conozca las implementaciones concretas de los puertos.

Se encargará de:

- Cargar configuración.
- Construir adaptadores.
- Conectar puertos con implementaciones.
- Definir ciclos de vida.
- Iniciar procesos.
- Registrar cierres ordenados.

No contendrá reglas de negocio.

## 7. Empaquetado por componentes

Cada componente será una unidad funcional con:

- Lenguaje propio del dominio.
- Datos de los que es propietario.
- API pública interna.
- Casos de uso.
- Puertos.
- Adaptadores.
- Pruebas.

La estructura conceptual será:

```text
components/
  <component-name>/
    domain/
      entities/
      value_objects/
      policies/
      events/
      errors/
    application/
      commands/
      queries/
      use_cases/
      dto/
      ports/
    adapters/
      inbound/
      outbound/
    contracts/
    tests/
    mod-or-index
```

No todos los directorios deberán existir si están vacíos. La estructura se aplicará para clarificar límites, no para producir carpetas ceremoniales.

### 7.1 API pública del componente

Cada componente expondrá únicamente:

- Casos de uso autorizados.
- DTO contractuales.
- Eventos públicos.
- Tipos compartidos deliberadamente.

No se importarán repositorios internos, tablas, entidades privadas ni utilidades de otro componente.

### 7.2 Dependencias entre componentes

Las dependencias deberán ser explícitas y dirigidas.

Un componente podrá relacionarse con otro mediante:

- Su API de aplicación pública.
- Un puerto declarado por el consumidor.
- Un evento público.
- Un contrato de transporte versionado.

No se permitirán consultas directas a las tablas propiedad de otro componente desde la lógica de aplicación.

### 7.3 Datos compartidos

Se evitará una carpeta global de modelos que convierta todas las entidades en dependencias comunes.

Solo podrán compartirse primitivas verdaderamente transversales, como:

- `PracticeId`
- `UserId`
- `DeviceId`
- `OperationId`
- Instantes UTC.
- Códigos de error estables.
- Metadatos de auditoría.

## 8. Estructura del repositorio

El sistema tendrá tres proyectos con repositorios propios: Nightingale para el escritorio; Nightingale Patient Hub para la web y su API; Nightingale API para sincronización, aceptación transaccional y módulos/workers de recordatorios, integración con n8n y analítica. Este repositorio conserva escritorio, pruebas, contratos de consumo y documentación inicial de los otros dos. No se crean aquí servidores o aplicaciones web. n8n es una herramienta externa, no un cuarto proyecto Nightingale.

```text
nightingale/
  src/                        Interfaz React, rutas, funcionalidades y cliente Tauri
  src-tauri/
    src/
      bootstrap/              Composición cuando su tamaño requiera separarla
      components/             Componentes con dominio, aplicación y adaptadores
      shared_kernel/          Primitivas compartidas deliberadamente
      platform/               Integraciones del sistema operativo
    migrations/               Esquema SQLite
    capabilities/
    tests/
  contracts/                  Contratos de sincronización e integración externa
  test-support/               Datos ficticios y dobles de los límites externos
  docs/
  tasks/
```

### 8.1 Carácter conceptual

El [ADR de la base de escritorio](decisions/desktop-foundation.md) conserva las rutas Tauri actuales: src/ para la interfaz y src-tauri/ para Rust. El árbol fija responsabilidades; no obliga a crear carpetas vacías.

La [decisión de tres proyectos](decisions/transactional-sync-three-projects.md) mantiene esta separación. Nightingale API posee el esquema PostgreSQL operativo y sus migraciones; el escritorio y el Hub usan contratos HTTP versionados, no contratos SQL. Las especificaciones externas se trasladarán a sus repositorios de forma explícita para evitar dos fuentes editables.

### 8.2 Workspaces

El workspace Rust podrá incluir componentes nativos cuando lo justifiquen sus límites. La interfaz conserva pnpm y su lockfile. No se añadirá un workspace JavaScript para aplicaciones externas a este repositorio.

Los servicios cloud conservan sus propias herramientas, dependencias y despliegues. Su ausencia no bloquea las pruebas del dominio nativo.

### 8.3 Contratos generados

Los contratos externos tendrán una fuente versionada con propietario explícito y una política de compatibilidad entre repositorios. El plan de cada integración decidirá dónde vive esa fuente y cómo se actualiza su copia de consumo; no se mantendrán dos fuentes editables independientes.

El código generado no se editará manualmente. El contrato de sincronización remota no sustituye los casos de uso locales por llamadas HTTP. API y Hub poseen sus contratos; Rust consume versiones compatibles.

## 9. Catálogo de componentes

El catálogo define propiedad funcional. Un componente no equivale necesariamente a un proceso desplegable o microservicio. Varios componentes podrán convivir en una misma aplicación o servicio mientras mantengan sus límites internos.

```mermaid
flowchart TD
    IA["Identidad y cuenta profesional"] --> PR["Pacientes y servicios"]
    PR --> AG["Agenda y disponibilidad"]
    AG --> CI["Citas"]
    CI --> PH["Patient Hub"]
    CI --> RE["Recordatorios"]
    CI --> LE["Lista de espera"]
    CI --> SY["Sincronización"]
    CI --> AN["Eventos analíticos"]
```

La dirección del diagrama indica dependencia conceptual, no orden de implementación.

### 9.1 Identidad y acceso

**Responsabilidad:** autenticar personas y representar sus permisos.

**Propiedad:**

- Usuarios.
- Roles.
- Sesiones.
- Credenciales o referencias a un proveedor de identidad.
- Intentos de acceso relevantes.

**Casos de uso públicos:**

- Iniciar y cerrar sesión.
- Renovar una sesión.
- Consultar la identidad activa.
- Asignar o revocar un rol permitido.
- Revocar sesiones.

**Puertos requeridos:**

- Almacén de identidades.
- Verificador de credenciales.
- Emisor de sesión.
- Reloj.
- Auditoría.

**Invariantes:**

- Cada acción deberá vincularse con una identidad o actor técnico.
- Un usuario no podrá operar fuera de una cuenta profesional autorizada.
- Médico y asistente no compartirán una misma identidad.

La autenticación concreta permanece abierta. Rust aplica el acceso local y el dominio no dependerá de un proveedor específico.

Los roles, membresías y permisos solo se modificarán mediante una operación autorizada con conexión. Sin Internet, el escritorio conservará la última versión válida de los permisos; no concederá ni cambiará permisos localmente.

Cada cambio pendiente identificará la versión de permisos utilizada y su fecha local. Las modificaciones de permisos registrarán versión y fecha de efecto desde la autoridad remota. Al reconectar se comprobarán las revocaciones y los cambios de acceso. La fecha del equipo ayuda a ordenar y auditar, pero no prueba por sí sola que una operación ocurrió antes de una revocación.

Antes de implementar identidad o sincronización se definirá la política de aceptación de operaciones anteriores a una revocación, con una referencia temporal verificable. Las operaciones dudosas se conservarán para rechazo o revisión, sin aceptarlas solo por la fecha local ni borrarlas en silencio. Nightingale API comprobará acceso y revocaciones antes de aceptar operaciones; los permisos y restricciones PostgreSQL añaden protección, además de las comprobaciones locales de Rust.

### 9.2 Cuenta profesional

**Responsabilidad:** representar el espacio aislado del médico o especialista.

**Propiedad:**

- `practice`.
- Membresías de usuarios.
- Preferencias operativas propias de la cuenta.
- Zona horaria y localización administrativa.

**Casos de uso públicos:**

- Consultar la cuenta activa.
- Actualizar datos administrativos permitidos.
- Gestionar miembros autorizados.
- Configurar reglas generales del servicio.

**Invariantes:**

- Toda entidad operativa deberá pertenecer a una cuenta profesional.
- Ninguna consulta podrá devolver datos de otra cuenta.
- El modelo deberá permitir una evolución futura hacia clínicas sin exponer funciones de clínica en la primera versión.

### 9.3 Dispositivos

**Responsabilidad:** identificar y controlar instalaciones de escritorio.

**Propiedad:**

- Identidad del dispositivo.
- Estado de autorización.
- Usuario que lo registró.
- Última sincronización conocida.
- Versión de aplicación y esquema local reportados.

**Casos de uso públicos:**

- Registrar un dispositivo.
- Renovar su autorización.
- Listar dispositivos de la cuenta.
- Revocar un dispositivo.
- Reportar versión y estado de sincronización.

**Invariantes:**

- Una operación de sincronización deberá identificar dispositivo y usuario.
- Cuando se compruebe la revocación, Nightingale API y el Hub deberán impedir nuevos intercambios del dispositivo. Un equipo sin conexión no puede conocer una revocación hasta reconectar; se aplica la política de §9.1.
- La revocación deberá activar el tratamiento local definido para credenciales y datos.

### 9.4 Pacientes

**Responsabilidad:** conservar la identidad administrativa y los datos de contacto necesarios para gestionar citas.

**Propiedad:**

- Identificador interno.
- Nombres y apellidos.
- Datos de contacto.
- Preferencias y consentimiento de comunicación.
- Estado activo o inactivo.
- Metadatos de versión y auditoría.

**Casos de uso públicos:**

- Registrar paciente.
- Consultar paciente.
- Buscar pacientes dentro de la cuenta.
- Actualizar datos administrativos.
- Desactivar o fusionar duplicados mediante una operación controlada.

**Invariantes:**

- No se almacenará información clínica en este componente.
- Las búsquedas siempre estarán limitadas por `practice_id`.
- Los datos obligatorios deberán ser mínimos y configurados de forma coherente.
- Los cambios de contacto no alterarán retroactivamente el historial de entregas.

**Eventos públicos:**

- `PatientRegistered`
- `PatientContactUpdated`
- `PatientDeactivated`

### 9.5 Catálogo de servicios

**Responsabilidad:** describir los tipos de consulta ofrecidos por el profesional.

**Propiedad:**

- Nombre del servicio.
- Duración o regla de bloques.
- Estado activo.
- Instrucciones administrativas permitidas.
- Reglas de recordatorio asociables.

**Casos de uso públicos:**

- Registrar servicio.
- Modificar configuración.
- Activar o desactivar servicio.
- Consultar servicios disponibles.

**Invariantes:**

- Una cita deberá referenciar una versión válida o una instantánea suficiente del servicio.
- Desactivar un servicio no cancelará automáticamente citas existentes.
- La duración no podrá cambiar silenciosamente citas ya programadas.

La representación de duración fija o múltiples bloques permanece abierta.

### 9.6 Agenda y disponibilidad

**Responsabilidad:** determinar qué intervalos pueden utilizarse para citas.

**Propiedad:**

- Reglas habituales de horario.
- Excepciones.
- Bloqueos.
- Ausencias del profesional.
- Zona horaria aplicable.
- Proyección de espacios disponibles.

**Casos de uso públicos:**

- Definir horario habitual.
- Añadir o retirar un bloqueo.
- Consultar disponibilidad.
- Validar un intervalo solicitado.
- Proponer alternativas disponibles.

**Puertos requeridos:**

- Repositorio de reglas de agenda.
- Consulta de ocupación proporcionada mediante contrato con Citas.
- Reloj.

**Invariantes:**

- La disponibilidad será una conclusión, no un estado duplicado sin control.
- Una excepción explícita tendrá prioridad sobre una regla habitual.
- Toda operación temporal utilizará instantes UTC internamente y la zona de la cuenta para presentación.
- La aceptación definitiva corresponde a Nightingale API dentro de una transacción PostgreSQL protegida por restricciones de disponibilidad.

**Eventos públicos:**

- `ScheduleRuleChanged`
- `ScheduleBlockCreated`
- `ScheduleBlockRemoved`

### 9.7 Citas

**Responsabilidad:** gobernar el ciclo de vida de una reserva médica administrativa.

**Propiedad:**

- Cita.
- Estado de negocio.
- Horario e intervalo.
- Paciente y servicio asociados.
- Historial de cambios.
- Confirmación.
- Resultado administrativo de asistencia.

**Casos de uso públicos:**

- Crear cita.
- Consultar cita.
- Consultar agenda de citas.
- Confirmar.
- Cancelar.
- Reprogramar.
- Registrar atención.
- Registrar ausencia.
- Incorporar el resultado versionado de una acción del Patient Hub sin repetir sus reglas.

**Invariantes:**

- Una cita deberá pertenecer a una cuenta, paciente y servicio válidos.
- Una reserva definitiva no podrá solaparse con otra reserva incompatible.
- Una reprogramación conservará el historial.
- Cambiar fecha u hora invalidará la confirmación anterior.
- Una cancelación liberará disponibilidad y no eliminará el historial.
- Ausencia y atención serán estados terminales administrativos.
- Estado de negocio y estado de sincronización serán conceptos separados.

**Eventos públicos:**

- `AppointmentCreated`
- `AppointmentConfirmed`
- `AppointmentRescheduled`
- `AppointmentCancelled`
- `AppointmentCompleted`
- `AppointmentMarkedNoShow`

Los estados y transiciones se definen en DAT, sección 11. Confirmar o cancelar desde Citas no autoriza una segunda implementación de las capacidades del Hub: el plan deberá delimitar la operación nativa y derivar por API la que pertenezca al Hub.

### 9.8 Solicitudes del Patient Hub

La propiedad y los casos de uso siguientes pertenecen al proyecto Nightingale Patient Hub, con su propio repositorio. El componente del escritorio expone consultas y adaptadores de consumo de esa API; la vista local es derivada.

**Responsabilidad:** consumir desde Rust el contrato del Patient Hub. El servicio cloud posee y valida el acceso limitado, las acciones y las solicitudes; el escritorio incorpora sus resultados sin implementar una segunda autoridad.

**Propiedad:**

- Referencias de acceso.
- Revocación y vencimiento.
- Solicitudes de reprogramación.
- Intentos relevantes.

**Casos de uso públicos:**

- Emitir acceso para una cita.
- Validar acceso.
- Consultar la vista permitida.
- Confirmar cita.
- Cancelar cita.
- Registrar solicitud de reprogramación.
- Revocar acceso.

**Invariantes:**

- Un acceso solo permitirá ver y operar sobre la cita asociada.
- El Patient Hub no expondrá la agenda general.
- El Patient Hub no creará citas nuevas.
- Una solicitud de reprogramación no reservará un horario.
- Un acceso antiguo no podrá modificar una versión incompatible de la cita.
- La respuesta no incluirá datos internos, auditoría ni información de otros pacientes.

**Eventos públicos:**

- `PatientAccessIssued`
- `PatientConfirmedAppointment`
- `PatientCancelledAppointment`
- `RescheduleRequested`

### 9.9 Recordatorios y comunicaciones

**Responsabilidad:** definir, programar y registrar comunicaciones administrativas relacionadas con citas.

**Propiedad:**

- Política de recordatorios.
- Trabajo persistente de recordatorio.
- Intentos de envío.
- Estado de entrega.
- Respuesta asociada.
- Clave idempotente.

**Casos de uso públicos:**

- Crear recordatorios para una versión de cita.
- Cancelar recordatorios obsoletos.
- Obtener trabajos vencidos.
- Marcar un trabajo en cola.
- Registrar intento y resultado.
- Aplicar respuesta reconocida.
- Programar reintento.

**Invariantes:**

- El registro persistente será la fuente de verdad, no el broker.
- Un trabajo deberá verificar la versión vigente antes de enviar.
- Una clave idempotente impedirá comunicaciones equivalentes duplicadas.
- Una cita cancelada no recibirá recordatorios ordinarios posteriores.
- Los mensajes no incluirán información clínica sensible.
- La IA no será necesaria para ejecutar el flujo inicial.

**Eventos públicos:**

- `ReminderScheduled`
- `ReminderQueued`
- `ReminderSent`
- `ReminderDelivered`
- `ReminderResponded`
- `ReminderFailed`

Los estados operativos se definen en DAT, sección 15.

### 9.10 Lista de espera

**Responsabilidad:** administrar interés por horarios y ofertas derivadas de espacios liberados.

**Propiedad:**

- Entrada de lista de espera.
- Preferencias permitidas.
- Oferta.
- Vencimiento y respuesta.
- Resultado de asignación.

**Casos de uso públicos:**

- Añadir o retirar entrada.
- Consultar candidatos compatibles.
- Crear oferta.
- Aceptar o rechazar oferta.
- Vencer oferta.
- Solicitar asignación de horario a Citas.

**Invariantes:**

- Una aceptación no garantizará el horario hasta que Citas valide disponibilidad.
- Solo habrá una oferta activa por el mismo horario cuando la política así lo requiera.
- La regla de prioridad será explícita y configurable dentro de límites definidos.
- Un paciente inactivo o una entrada retirada no recibirá ofertas.

**Eventos públicos:**

- `WaitlistEntryCreated`
- `WaitlistOfferCreated`
- `WaitlistOfferAccepted`
- `WaitlistOfferExpired`
- `WaitlistSlotAssigned`

### 9.11 Sincronización

**Responsabilidad:** intercambiar operaciones y cambios entre cada SQLite local y la fuente central.

**Propiedad local:**

- `sync_outbox`.
- `sync_inbox`.
- `sync_cursor`.
- Resultado local de operaciones.
- Estado visible de sincronización.

**Estado compartido en PostgreSQL, aceptado por Nightingale API:**

- Registro de operaciones procesadas.
- Resultados idempotentes.
- Registro incremental de cambios.
- Cursores.
- Conflictos.

**Casos de uso públicos:**

- Registrar operación local.
- Enviar lote de operaciones.
- Aplicar resultado.
- Descargar cambios.
- Aplicar cambios remotos.
- Registrar conflicto.
- Reintentar operación.

**Invariantes:**

- Cambiar una entidad local y registrar su Outbox será una única transacción.
- Reenviar una operación no repetirá sus efectos.
- El orden se preservará cuando exista dependencia causal.
- Un conflicto no eliminará silenciosamente la intención local.
- Un error individual no bloqueará operaciones independientes del lote.
- El cursor avanzará únicamente después de aplicar correctamente los cambios asociados.

El coordinador Rust implementa el intercambio HTTP y la persistencia local. Nightingale API revalida las operaciones y confirma estado, idempotencia, auditoría y registro de cambios en transacciones PostgreSQL. Los resultados del Hub llegan a la misma aceptación transaccional y se reconcilian por sus versiones, sin sobrescribirlos con una versión local antigua. La semántica se define en DAT, secciones 9 y 10.

### 9.12 Auditoría

**Responsabilidad:** conservar una traza inmutable de acciones relevantes.

**Propiedad:**

- Actor.
- Cuenta.
- Dispositivo.
- Acción.
- Entidad afectada.
- Instante.
- Correlación.
- Resultado.
- Metadatos permitidos.

**Invariantes:**

- La auditoría no sustituirá el historial funcional de una entidad.
- No almacenará secretos ni cuerpos innecesarios.
- Los adaptadores no podrán modificar eventos ya registrados.
- Las acciones del paciente y actores técnicos se diferenciarán.

### 9.13 Eventos operativos y analítica

**Responsabilidad:** publicar hechos operativos estables y transformarlos en proyecciones analíticas.

**Propiedad operacional:**

- Outbox de eventos analíticos.
- Cursor de exportación.

**Propiedad analítica:**

- Hechos de citas.
- Hechos de comunicaciones.
- Dimensiones seudonimizadas.
- Agregados e indicadores.

**Invariantes:**

- Una falla analítica no afectará una transacción de citas ya confirmada.
- Los eventos deberán ser reejecutables o deduplicables.
- Los indicadores deberán documentar numerador, denominador y ventana temporal.
- La identidad personal solo se copiará cuando exista una necesidad aprobada.

### 9.14 Configuración

**Responsabilidad:** representar configuración validada sin mezclarla con reglas de negocio.

Tipos de configuración:

- Configuración de aplicación.
- Configuración por entorno.
- Configuración por cuenta profesional.
- Preferencias por dispositivo.
- Secretos.

**Invariantes:**

- Los secretos no formarán parte de archivos versionados.
- Toda configuración tendrá un valor validado o producirá un error de inicio claro.
- Una preferencia local no podrá reemplazar una política central obligatoria.

## 10. Distribución de componentes

Rust posee los casos de uso locales. Nightingale API posee la aceptación remota y vuelve a validar lo necesario para proteger datos compartidos; no confía en que el ejecutable haya ejecutado sus reglas. Patient Hub posee el acceso y la política del paciente y usa la aceptación común.

| Componente | Escritorio Rust / SQLite | PostgreSQL compartido | Servicios cloud externos |
|---|---|---|---|
| Identidad y acceso | Acceso local con los últimos permisos válidos | Identidades, versiones y permisos compartidos por concretar | Nightingale API autoriza cada operación; el Hub controla sus accesos limitados |
| Cuenta profesional | Gestión nativa y preferencias locales | Estado compartido y aislamiento | Datos mínimos necesarios para su función |
| Dispositivos | Identidad, credenciales y tratamiento de revocación | Registro y revocación con conexión | Control de acceso a cada servicio |
| Pacientes | Registro, consulta y edición local | Cambios aceptados y versiones | Nightingale API acepta operaciones; datos mínimos para Hub y comunicaciones |
| Servicios | Gestión y consulta nativas | Estado compartido | Nightingale API acepta operaciones; Hub recibe solo el servicio asociado |
| Agenda | Reglas, bloqueos y disponibilidad local | Restricciones de integridad entre dispositivos | Nightingale API valida aceptación; el Hub no consulta la agenda general |
| Citas | Ciclo operativo y reservas locales pendientes | Reservas compartidas, versiones e historial | Nightingale API acepta cambios nativos y del Hub con reglas comunes |
| Patient Hub | Cliente HTTP y vista local de resultados | Estado versionado de las acciones aceptadas | Único propietario de accesos, acciones y solicitudes del Hub |
| Recordatorios | Configuración e intención de programación; consulta de resultados | Trabajos y resultados compartidos | Módulos y workers de Nightingale API; n8n por API interna |
| Lista de espera | Gestión y decisión de asignación nativas | Oferta y reserva compartidas | Comunicaciones y respuesta limitada según contrato |
| Sincronización | Coordinador, Outbox, Inbox, cursor y conflictos | Operaciones, resultados idempotentes y Change Log | Nightingale API: autorización, aceptación y descarga |
| Auditoría | Registro local e intención pendiente | Historial compartido | Registro de las acciones propias de cada servicio |
| Analítica | Consumo de resultados cuando se integre | Eventos operativos aceptados | Módulos y workers de Nightingale API; PostgreSQL analítico separado |

### 10.1 Regla de autoridad

SQLite permite la operación autónoma del dispositivo. PostgreSQL conserva el estado final compartido; una transacción remota confirma que un cambio respeta las versiones, permisos y restricciones concurrentes.

El Hub es la autoridad de sus accesos y reglas del paciente. Tanto sus acciones sobre citas como las operaciones del escritorio pasan por Nightingale API para la aceptación compartida. El Hub no escribe directamente en PostgreSQL operativo. Los contratos comparten versiones, restricciones y reglas de transición; una vista SQLite no es otra fuente definitiva.

### 10.2 Regla de disponibilidad local

Solo se copiarán a SQLite los datos que el dispositivo necesita para operar. La ventana deberá cubrir la agenda requerida, incluidos hoy y mañana, después de su descarga o creación local; no se prometerán datos que el equipo nunca recibió.

No se utilizará la base local como réplica indiscriminada de todos los datos históricos. Su ventana y profundidad deberán poder ampliarse sin romper contratos.

## 11. Kernel compartido y primitivas

El kernel compartido será deliberadamente pequeño.

### 11.1 Tipos permitidos

- Identificadores tipados.
- `PracticeId`.
- `UserId`.
- `DeviceId`.
- `PatientId`.
- `AppointmentId`.
- `OperationId`.
- `CorrelationId`.
- Instante UTC.
- Zona horaria validada.
- Número de versión.
- Metadatos de actor.
- Resultado paginado genérico.
- Códigos de error transversales.

### 11.2 Tipos no permitidos

El kernel no contendrá:

- Entidades completas de negocio.
- Repositorios específicos.
- DTO de un componente particular.
- Modelos de base de datos.
- Tipos generados por frameworks.
- Clientes de servicios externos.
- Utilidades sin propietario claro.

### 11.3 Tiempo

Las reglas no llamarán directamente al reloj del sistema. Recibirán un puerto `Clock` o un instante explícito.

Todos los instantes se almacenarán en UTC. La zona horaria de la cuenta se utilizará para reglas locales y presentación.

### 11.4 Identificadores

Los clientes podrán generar UUID sin conexión. Los identificadores no codificarán información personal ni secuencias predecibles.

### 11.5 Versiones

Las entidades sincronizables tendrán un contador de versión independiente de fechas como `updated_at`.

Las fechas servirán para observabilidad y presentación. La versión servirá para control optimista.

## 12. Contratos

### 12.1 Fuente contractual

Los contratos de red y eventos deberán definirse en formatos independientes del lenguaje, por ejemplo OpenAPI, JSON Schema o una alternativa equivalente cuando se decida el transporte.

Nightingale API y Nightingale Patient Hub viven en repositorios propios. Los clientes Rust no dependerán de clases internas de sus servidores. El contrato HTTP de sincronización pertenece a Nightingale API; el de capacidades del Hub pertenece al Hub. Los contratos internos de workers siguen bajo Nightingale API. En este documento, una API pública interna de componente es una interfaz de código, no un servidor HTTP.

### 12.2 Reglas de compatibilidad

- Añadir campos opcionales será compatible.
- Eliminar o cambiar el significado de un campo requerirá una nueva versión.
- Los consumidores ignorarán campos desconocidos cuando sea seguro.
- Los valores enumerados desconocidos deberán producir un tratamiento controlado.
- Los contratos indicarán zona temporal, formato y nulabilidad.
- Los identificadores mantendrán el mismo tipo semántico entre sistemas.

### 12.3 Comandos y eventos

Un comando expresa intención y podrá ser rechazado. Un evento expresa un hecho ya aceptado.

Ejemplos:

- Comando: `CreateAppointment`.
- Evento: `AppointmentCreated`.
- Comando: `CancelAppointment`.
- Evento: `AppointmentCancelled`.

No se nombrará como evento una solicitud que todavía pueda fallar.

### 12.4 Metadatos comunes de eventos

Cada evento público incluirá:

- `event_id`.
- `event_type`.
- `event_version`.
- `occurred_at`.
- `practice_id`.
- `aggregate_id` cuando corresponda.
- `aggregate_version`.
- `correlation_id`.
- `causation_id` cuando corresponda.
- Actor permitido o referencia seudonimizada.
- Cuerpo versionado.

### 12.5 Catálogo mínimo de eventos

| Área | Eventos |
|---|---|
| Pacientes | `PatientRegistered`, `PatientContactUpdated`, `PatientDeactivated` |
| Agenda | `ScheduleRuleChanged`, `ScheduleBlockCreated`, `ScheduleBlockRemoved` |
| Citas | `AppointmentCreated`, `AppointmentConfirmed`, `AppointmentRescheduled`, `AppointmentCancelled`, `AppointmentCompleted`, `AppointmentMarkedNoShow` |
| Patient Hub | `PatientAccessIssued`, `PatientConfirmedAppointment`, `PatientCancelledAppointment`, `RescheduleRequested` |
| Recordatorios | `ReminderScheduled`, `ReminderSent`, `ReminderDelivered`, `ReminderResponded`, `ReminderFailed` |
| Lista de espera | `WaitlistEntryCreated`, `WaitlistOfferCreated`, `WaitlistOfferAccepted`, `WaitlistOfferExpired`, `WaitlistSlotAssigned` |
| Dispositivos | `DeviceRegistered`, `DeviceRevoked` |

### 12.6 Errores contractuales

Los límites del sistema utilizarán códigos estables.

Categorías mínimas:

- `VALIDATION_ERROR`
- `AUTHENTICATION_REQUIRED`
- `PERMISSION_DENIED`
- `NOT_FOUND`
- `VERSION_CONFLICT`
- `SLOT_UNAVAILABLE`
- `INVALID_STATE_TRANSITION`
- `IDEMPOTENCY_CONFLICT`
- `RATE_LIMITED`
- `DEPENDENCY_UNAVAILABLE`
- `TEMPORARY_FAILURE`
- `INTERNAL_ERROR`

Los mensajes dirigidos al usuario se traducirán en la interfaz. El contrato no expondrá detalles internos, consultas SQL ni trazas.

## 13. Persistencia

### 13.1 Separación de modelos

Se distinguirán:

- Entidad de dominio.
- Modelo de persistencia.
- DTO de transporte.
- Proyección de lectura.

No se utilizará una misma estructura para las cuatro responsabilidades por comodidad.

### 13.2 SQLite

SQLite deberá:

- Mantener integridad mediante claves y restricciones disponibles.
- Aplicar migraciones versionadas.
- Ejecutar cambios de entidad y Outbox en la misma transacción.
- Mantener índices para agenda, pacientes, estados y sincronización.
- Evitar almacenar secretos en texto plano.
- Permitir reconstrucción desde el estado cloud.

El esquema local podrá diferir del esquema PostgreSQL cuando responda a necesidades locales, siempre que el mapeo sea explícito.

### 13.3 PostgreSQL operativo

PostgreSQL deberá:

- Aplicar aislamiento por `practice_id`.
- Impedir reservas incompatibles mediante una restricción o transacción segura.
- Conservar versiones de entidades sincronizables.
- Registrar operaciones idempotentes.
- Mantener el Change Log.
- Conservar trabajos persistentes de recordatorio.
- Ejecutar migraciones compatibles con despliegues controlados.

### 13.4 Propiedad de tablas

Cada tabla tendrá un componente propietario. Otros componentes no escribirán directamente en ella.

Las consultas que combinen componentes podrán resolverse mediante:

- Proyecciones de lectura.
- Vistas controladas.
- Casos de uso compuestos.
- Eventos y modelos derivados.

### 13.5 Migraciones

Toda migración deberá incluir:

- Identificador único.
- Orden estable.
- Compatibilidad esperada.
- Tratamiento de datos existentes.
- Estrategia de fallo.

La aplicación de escritorio deberá verificar la versión del esquema antes de abrir la base local.

No se ejecutarán migraciones destructivas irreversibles sin un mecanismo de respaldo y una decisión documentada.

### 13.6 Datos de prueba

Los datos semilla para desarrollo no contendrán información real de pacientes. Se utilizarán generadores deterministas y casos representativos.

## 14. Contrato de sincronización

Esta sección define la forma que deberá tener la implementación. La semántica general se encuentra en DAT, sección 9.

### 14.1 Unidad de sincronización

La unidad será una operación, no una fila ni un archivo de base de datos.

Una operación incluirá:

- `operation_id`.
- `device_id`.
- `practice_id`.
- `actor_id`.
- `entity_type`.
- `entity_id`.
- `action`.
- `expected_version`.
- `occurred_at_local`.
- `payload`.
- Versión del contrato.
- Versión de permisos y referencia temporal verificable cuando la política de acceso lo requiera.

### 14.2 Resultado por operación

El coordinador nativo registrará el resultado individual que devuelve Nightingale API después de la aceptación transaccional:

- `accepted`.
- `duplicate`.
- `conflict`.
- `rejected`.
- `retryable_failure`.

El resultado incluirá la versión central y, cuando sea seguro, una representación suficiente del estado actual.

Nightingale API solo confirmará aceptación después del commit. La operación, su resultado idempotente, el historial/auditoría y el Change Log se guardarán en la misma transacción que su efecto. Un reintento autorizado recupera el resultado; usar el mismo identificador con otro contenido se rechaza. Una operación duplicada no permite consultar resultados fuera de los permisos vigentes.

### 14.3 Lotes

Un lote será un medio de transporte, no una transacción global obligatoria.

- Las operaciones independientes podrán completarse aunque otra falle.
- Las dependencias causales deberán declararse u ordenarse.
- El tamaño máximo será configurable.
- Un lote podrá reintentarse sin duplicar efectos.

### 14.4 Change Log

Cada cambio sincronizable aceptado generará una entrada ordenada.

La entrada contendrá:

- Cursor monotónico u opaco con orden confirmado en PostgreSQL.
- Tipo e identificador de entidad.
- Versión.
- Tipo de cambio.
- Representación o referencia necesaria.
- Instante de aceptación.

El coordinador utilizará el cursor autorizado que devuelve Nightingale API al consultar el Change Log. No lo calculará a partir de la hora local. Su asignación, orden de confirmación y recuperación deberán definirse antes de implementar el intercambio.

### 14.5 Aplicación local de cambios remotos

Los cambios descargados se aplicarán en una transacción local por página o unidad segura.

El cursor solo avanzará después de completar la transacción.

Si un cambio remoto afecta una entidad con una operación local pendiente:

- Se evaluará si son compatibles.
- Se conservará la intención local.
- Se marcará conflicto cuando no exista una regla segura.
- No se aplicará “último escritor gana” a horario o estado crítico de cita.

### 14.6 Política de conflictos

| Tipo | Política |
|---|---|
| Reserva concurrente | PostgreSQL acepta una; la otra queda rechazada como `SLOT_UNAVAILABLE` |
| Edición sobre versión antigua | `VERSION_CONFLICT` y recarga del estado central |
| Cancelación del paciente frente a cambio local antiguo | La cancelación central prevalece |
| Confirmación frente a reprogramación | La reprogramación invalida la confirmación |
| Cambio de contacto concurrente | Podrá aplicarse último cambio aceptado con auditoría |
| Operación duplicada | Se devuelve el resultado idempotente previo |

### 14.7 Recuperación de un dispositivo

Un dispositivo nuevo o reconstruido deberá:

- Autenticarse.
- Registrar o recuperar su identidad autorizada.
- Crear una base local vacía en la versión correcta.
- Descargar un snapshot o páginas iniciales.
- Establecer un cursor.
- Continuar con cambios incrementales.

No reutilizará una copia local perteneciente a otro dispositivo sin un procedimiento explícito.

## 15. Citas y disponibilidad

### 15.1 Reserva definitiva

El componente Citas de Rust valida y guarda la reserva local. Su coordinador envía la operación completa a Nightingale API, que revalida permisos, versiones, transiciones y disponibilidad y confirma la reserva compartida mediante una transacción PostgreSQL. La validación local no permite eludir la aceptación remota.

El caso de uso deberá:

1. Validar cuenta, actor, paciente y servicio.
2. Validar estado del servicio.
3. Validar disponibilidad.
4. Impedir solapamiento.
5. Crear o modificar la cita.
6. Incrementar versión.
7. Registrar historial y auditoría.
8. Publicar los eventos correspondientes dentro de una estrategia Outbox.

Esta enumeración define responsabilidades de una operación, no una tarea de desarrollo.

### 15.2 Cita provisional local

La creación offline deberá:

- Validarse contra la copia local.
- Guardarse con estado de sincronización `pending`.
- Mostrar claramente su carácter provisional.
- No activar recordatorios cloud hasta ser aceptada.
- Convertirse en definitiva o conflicto al sincronizar.

### 15.3 Bloques de agenda

El diseño deberá soportar una representación que impida solapamientos sin depender de la interfaz.

Hasta decidir duración fija o variable:

- El contrato de Citas utilizará intervalos de inicio y fin.
- El dominio no asumirá que todas las citas duran lo mismo.
- La UI mínima podrá ofrecer bloques configurables para simplificar pruebas.

### 15.4 Transiciones

Las transiciones autorizadas seguirán DAT, sección 11. Cada transición deberá estar implementada como intención de dominio, no como asignación libre de un campo `status`.

## 16. Recordatorios, cola y n8n

### 16.1 Separación de responsabilidades

| Parte | Responsabilidad |
|---|---|
| Citas en Rust | Registrar la intención de recordatorio y los cambios de cita; compartirlos al sincronizar |
| Servicio cloud de recordatorios | Identificar y programar trabajos sobre datos compartidos; poseer su ejecución |
| Planificador | Identificar trabajos que deben entrar en cola |
| Broker | Transportar el identificador del trabajo |
| Worker o n8n | Ejecutar el flujo de integración |
| Adaptador de WhatsApp | Comunicarse con el proveedor |
| Webhook | Traducir resultados externos |
| PostgreSQL | Conservar el estado definitivo |

El servicio de recordatorios y sus ejecutores son módulos y workers de Nightingale API. Persisten trabajos y los procesan después del commit; las transacciones de sincronización no esperan a proveedores, n8n, Hub o analítica. n8n conserva su función de herramienta externa.

### 16.2 Fuente persistente

El escritorio puede registrar una intención de recordatorio sin conexión; esta no implica que la nube lo haya programado ni enviado. Un cambio solo local no es visible para el ejecutor cloud hasta su intercambio.

El broker no será la fuente de verdad. `reminder_job` conservará:

- Identificador.
- Cita y versión de cita.
- Tipo de recordatorio.
- Fecha programada.
- Estado.
- Clave idempotente.
- Intentos.
- Próximo reintento.
- Resultado final.

### 16.3 Publicación segura

La transición de trabajo persistente a cola deberá tolerar fallos entre la escritura en PostgreSQL y la publicación.

Se podrá utilizar:

- Outbox de trabajos.
- Consulta periódica de trabajos vencidos con bloqueo seguro.
- Capacidad equivalente del broker elegido.

La decisión concreta deberá conservar la posibilidad de recuperar trabajos no publicados.

### 16.4 Ejecución

El ejecutor recibirá `reminder_job_id`, consultará el estado actual y verificará:

- Que la cita siga activa.
- Que la versión coincida.
- Que el mensaje no haya sido enviado.
- Que exista consentimiento o habilitación aplicable.
- Que la plantilla sea válida.

Después registrará el intento y su resultado mediante la API interna.

### 16.5 Papel de n8n

n8n será un adaptador de orquestación.

Los flujos versionados deberán:

- Recibir identificadores, no conjuntos completos de datos sensibles cuando no sea necesario.
- Consultar la API interna.
- Utilizar credenciales administradas.
- Informar resultados mediante contratos estables.
- Ser idempotentes o apoyarse en idempotencia del servicio.
- Evitar lógica de disponibilidad o transiciones propias de Citas.

### 16.6 Reintentos y errores

Los errores se clasificarán en:

- Temporales: red, límite de proveedor, indisponibilidad.
- Permanentes: número inválido, plantilla rechazada, acceso revocado.
- De negocio: cita cancelada, versión obsoleta, comunicación no permitida.

Solo los temporales producirán reintentos automáticos.

## 17. Patient Hub

### 17.1 Límite funcional

El Patient Hub será una interfaz pública limitada y un servicio externo propio. Es la excepción explícita a la operación nativa: tanto el paciente como el escritorio utilizan su API para las capacidades que el Hub posee.

El servicio valida y registra sus acciones incluso con los escritorios apagados. Rust no duplica sus reglas de acceso del paciente ni mantiene una segunda versión definitiva de sus resultados. El Hub envía sus acciones sobre citas a Nightingale API, que aplica las reglas compartidas y persiste la aceptación en PostgreSQL. El escritorio conserva una proyección local versionada.

Sin conexión, el escritorio podrá consultar su última vista del Hub, marcada como desactualizada. No mostrará una acción del Hub como completada sin su aceptación. Si un plan permite guardar intenciones pendientes para esa API, deberá definir su revalidación, idempotencia y conflictos antes de implementarlas. Las funciones nativas ajenas al Hub seguirán disponibles.

No será una versión web del escritorio.

Deberá permitir únicamente:

- Validar el acceso a una cita existente.
- Presentar información administrativa permitida.
- Confirmar.
- Cancelar dentro de las reglas vigentes.
- Solicitar reprogramación.
- Mostrar el resultado de la acción.

### 17.2 Contrato de acceso

El acceso deberá estar asociado con:

- Una cita.
- Una versión o política de actualización.
- Un vencimiento.
- Un alcance de acciones.
- Un estado de revocación.

El token no incluirá datos personales legibles. El servidor almacenará solo la representación necesaria para validarlo de forma segura.

### 17.3 Vista pública

La respuesta destinada al paciente podrá incluir:

- Profesional.
- Servicio.
- Fecha y hora.
- Ubicación administrativa.
- Estado visible.
- Acciones disponibles.
- Instrucciones administrativas aprobadas.

No incluirá:

- Auditoría.
- Notas internas.
- Datos de otros pacientes.
- Historia clínica.
- Diagnóstico.
- Datos de sincronización.
- Identificadores internos innecesarios.

### 17.4 Acciones

- **Confirmar:** aplicará sobre la versión vigente o devolverá el estado actualizado.
- **Cancelar:** el Hub valida su política y pide a Nightingale API aceptar la cancelación sobre la versión compartida, liberar disponibilidad y registrar su resultado. Rust lo incorpora sin volver a ejecutar las reglas del Hub.
- **Solicitar reprogramación:** creará una solicitud separada; no consultará ni reservará la agenda.

### 17.5 Protección contra abuso

El adaptador web deberá contemplar:

- Límites de solicitudes.
- Registro de intentos relevantes.
- Respuestas que no permitan enumerar citas.
- Protección frente a reutilización indebida.
- Revocación de enlaces.
- Validación de origen cuando corresponda.

## 18. Analítica y ETL

La arquitectura general se define en DAT, sección 16. Esta sección fija los contratos para su desarrollo.

ETL y entrega de indicadores pertenecen a módulos y workers de Nightingale API. Su procesamiento no participa en la transacción de aceptación de operaciones.

### 18.1 Fuente de datos

La analítica consumirá eventos aceptados o un Change Log autorizado. No ejecutará consultas arbitrarias de alto costo sobre el PostgreSQL operativo desde el panel.

### 18.2 Modelo analítico inicial

Hechos previstos:

- Cita creada.
- Cita confirmada.
- Cita cancelada.
- Cita reprogramada.
- Cita atendida.
- Ausencia.
- Recordatorio programado.
- Recordatorio enviado.
- Entrega registrada.
- Respuesta registrada.
- Oferta de lista de espera.
- Horario recuperado.

Dimensiones previstas:

- Cuenta profesional.
- Servicio.
- Fecha.
- Franja horaria.
- Canal.
- Estado.

La dimensión paciente utilizará un identificador seudonimizado salvo que una necesidad aprobada exija otra cosa.

### 18.3 Definición de indicadores

Cada indicador deberá tener una ficha contractual con:

- Nombre.
- Propósito.
- Numerador.
- Denominador.
- Exclusiones.
- Zona horaria.
- Ventana temporal.
- Frecuencia de actualización.
- Nivel de agregación.
- Tratamiento de correcciones tardías.

Ejemplo conceptual:

```text
Nombre: Porcentaje de ausencias sin aviso
Numerador: citas marcadas como no_show
Denominador: citas completadas + no_show
Exclusiones: canceladas y reprogramadas antes del horario
```

### 18.4 Idempotencia del ETL

El ETL deberá:

- Mantener un cursor.
- Deduplicar por `event_id`.
- Permitir reprocesamiento controlado.
- Aplicar correcciones sin duplicar hechos.
- Registrar su última ejecución exitosa.

### 18.5 Servicio analítico

La API analítica solo ofrecerá consultas predefinidas y autorizadas. No expondrá SQL ni acceso genérico al almacén.

El panel analítico podrá ser web, pero compartirá identidad y aislamiento por cuenta con el sistema central.

## 19. Seguridad transversal

Las responsabilidades generales se definen en DAT, sección 17.

### 19.1 Límites de confianza

Se considerarán límites de confianza:

- React hacia Rust mediante IPC de Tauri.
- Coordinador Rust hacia Nightingale API por HTTPS.
- Escritorio hacia API del Patient Hub y otros servicios de apoyo.
- Web y escritorio hacia API del Hub; backend del Hub hacia Nightingale API.
- Servicios cloud hacia PostgreSQL y broker.
- n8n hacia API interna.
- Webhooks externos hacia el receptor.
- ETL hacia fuentes operativas y analíticas.

Todo dato que cruce un límite deberá validarse nuevamente.

### 19.2 Escritorio

- React no recibirá secretos permanentes.
- Los comandos Tauri serán específicos y limitados por capacidades.
- Rust validará identidad, forma e invariantes.
- Las credenciales locales utilizarán almacenamiento seguro del sistema operativo.
- La base local deberá cifrarse antes de producción con datos reales.
- Los registros locales evitarán información personal innecesaria.

### 19.3 Cloud

- PostgreSQL operativo estará en red privada. Solo Nightingale API y sus procesos autorizados tendrán credenciales con permisos mínimos; escritorio y Hub no tendrán acceso SQL. API comprobará identidad, cuenta, dispositivo o servicio y permisos para cada operación.
- Los servicios utilizarán identidades separadas.
- Los secretos se administrarán fuera del repositorio.
- Los permisos seguirán mínimo privilegio.
- Todo tráfico externo utilizará TLS.
- Las operaciones internas sensibles estarán autenticadas.

### 19.4 Aislamiento por cuenta

Toda consulta operativa deberá requerir contexto de cuenta. No se confiará en un `practice_id` enviado por el cliente sin verificar membresía.

Las pruebas de aislamiento serán obligatorias para repositorios, conexiones PostgreSQL, endpoints de apoyo y consultas analíticas. La seguridad remota no dependerá solo de Rust ni de un practice_id enviado por el cliente. El acceso local usa los últimos permisos válidos conforme a §9.1.

### 19.5 Datos personales

Se aplicará minimización:

- Solo datos necesarios para citas y comunicación.
- Sin historia clínica en el alcance inicial.
- Sin diagnósticos en mensajes.
- Sin datos personales completos en colas cuando baste un identificador.
- Sin cuerpos sensibles en logs generales.

### 19.6 Auditoría frente a observabilidad

La auditoría registra quién hizo qué. La observabilidad explica cómo se comportó el sistema. No se utilizará una como sustituto de la otra.

## 20. Modelo de errores

### 20.1 Categorías internas

- Error de validación.
- Violación de invariante.
- Transición inválida.
- Falta de autorización.
- Conflicto de versión.
- Recurso no disponible.
- Dependencia temporalmente indisponible.
- Error permanente de integración.
- Error interno inesperado.

### 20.2 Conversión por capas

- El dominio devolverá errores semánticos.
- La aplicación añadirá contexto del caso de uso.
- El adaptador traducirá el error al transporte.
- La interfaz mostrará un mensaje apropiado.
- La observabilidad conservará información técnica mediante un identificador de correlación.

No se mostrarán trazas ni detalles de infraestructura al usuario.

### 20.3 Reintentos

Los casos de uso no reintentarán automáticamente errores de negocio. Solo los adaptadores o coordinadores designados reintentarán fallos clasificados como temporales.

## 21. Observabilidad y operación

### 21.1 Correlación

Cada solicitud, operación de sincronización, ejecución de recordatorio y proceso ETL tendrá un identificador de correlación.

Los eventos derivados conservarán `correlation_id` y, cuando corresponda, `causation_id`.

### 21.2 Logs

Los logs serán estructurados y contendrán:

- Instante.
- Nivel.
- Servicio o proceso.
- Versión.
- Correlación.
- Categoría del evento.
- Resultado.
- Duración cuando corresponda.

No contendrán tokens, contraseñas ni información médica.

### 21.3 Métricas mínimas

- Tiempo de guardado local, respuesta de Nightingale API, duración de transacciones, espera por bloqueos y demora de sincronización, medidos por separado; errores de las API de apoyo.
- Estado de conexiones privadas de Nightingale API a PostgreSQL.
- Operaciones de sincronización aceptadas y rechazadas.
- Conflictos por categoría.
- Tamaño y antigüedad de Outbox.
- Trabajos de recordatorio por estado.
- Reintentos y errores permanentes.
- Entregas y respuestas.
- Cursor y retraso del ETL.
- Errores del Patient Hub.

### 21.4 Salud

Los procesos cloud expondrán comprobaciones separadas de:

- Proceso activo.
- Preparación para recibir tráfico.
- Dependencias esenciales disponibles.

Una falla del Hub, de recordatorios o de analítica no deberá impedir el arranque ni los casos de uso nativos que no requieren ese servicio. Su estado se informará por separado.

## 22. Esqueleto funcional de la interfaz

El esqueleto existirá para integrar y comprobar funcionalidad. No representa el diseño visual definitivo.

### 22.1 Principios

- Utilizar componentes simples y accesibles.
- Mantener jerarquía visual mínima.
- Evitar decisiones de marca, color, ilustración o animación.
- No crear un sistema de diseño definitivo.
- No acoplar los casos de uso a los componentes React.
- Permitir inspeccionar estados de sincronización y errores.

### 22.2 Rutas o superficies mínimas

| Superficie | Finalidad técnica |
|---|---|
| Acceso | Probar sesión, dispositivo y cuenta activa |
| Agenda | Consultar citas y disponibilidad local |
| Editor de cita | Crear, cancelar y reprogramar |
| Pacientes | Registrar, buscar y actualizar datos administrativos |
| Solicitudes | Revisar reprogramaciones del Patient Hub |
| Lista de espera | Registrar entradas y procesar ofertas |
| Centro de sincronización | Mostrar conexión, Outbox, última sincronización y conflictos |
| Configuración | Probar preferencias y datos de cuenta permitidos |
| Diagnóstico de desarrollo | Inspeccionar versión, dispositivo y estado de adaptadores sin exponer secretos |

### 22.3 Organización del frontend

```text
src/
  app/
    providers/
    router/
    shell/
  components/
    primitives/
    feedback/
    forms/
  features/
    appointments/
    patients/
    schedule/
    waitlist/
    sync/
    settings/
  routes/
  shared/
    api/
    types/
    validation/
    testing/
```

### 22.4 Dependencias del frontend

- `routes` podrá utilizar `features` y componentes compartidos.
- `features` podrá utilizar contratos generados y primitivas visuales.
- `components/primitives` no dependerá de un componente funcional.
- Ningún componente React ejecutará SQL.
- La comunicación local se realizará mediante una capa de cliente Tauri tipada.
- La comunicación web se realizará mediante clientes generados o contractuales.

### 22.5 Estado persistente, compartido y visual

Se distinguirán:

- Datos persistentes locales.
- Datos sincronizados.
- Estado de formularios.
- Estado visual efímero.
- Estado de conexión.

No se mantendrá una segunda copia mutable de entidades completas en un gestor global sin necesidad.

### 22.6 TailwindCSS

TailwindCSS se utilizará para estructura y legibilidad básica. El esqueleto podrá emplear un conjunto neutral de clases, pero no fijará la identidad visual final.

## 23. Estrategia de pruebas

Las pruebas verificarán límites y comportamiento, no detalles accidentales de implementación.

### 23.1 Pruebas de dominio

Deberán cubrir:

- Invariantes.
- Transiciones.
- Políticas temporales.
- Conflictos.
- Idempotencia lógica.
- Casos límite.

No utilizarán bases de datos ni red.

### 23.2 Pruebas de aplicación

Utilizarán puertos falsos o en memoria para verificar:

- Coordinación de casos de uso.
- Transacciones esperadas.
- Publicación de eventos.
- Autorización.
- Traducción de errores.

### 23.3 Pruebas de adaptadores

Verificarán:

- Repositorios SQLite.
- Repositorios PostgreSQL.
- Migraciones.
- Contratos HTTP o transporte elegido.
- Broker.
- Webhooks.
- Almacenamiento seguro.

Las pruebas PostgreSQL utilizarán una instancia real aislada, no una simulación SQLite del comportamiento PostgreSQL.

### 23.4 Pruebas de contratos

Productores y consumidores verificarán la misma fuente contractual.

Deberán detectar:

- Campos eliminados.
- Cambios incompatibles.
- Enumeraciones no tratadas.
- Errores con forma incorrecta.
- Eventos con versión inválida.

### 23.5 Pruebas de sincronización

Escenarios mínimos:

- Crear offline y sincronizar.
- Reenviar la misma operación.
- Interrumpir la conexión después del commit PostgreSQL y antes de registrar el resultado local.
- Dos dispositivos editan versiones distintas.
- Dos dispositivos reservan el mismo horario.
- Descargar varias páginas de cambios.
- Fallar antes de avanzar el cursor.
- Recuperar un dispositivo nuevo.
- Revocar un dispositivo con operaciones pendientes.
- Conservar permisos locales sin red y prohibir su edición offline.
- Reconciliar una revocación con la versión de permisos y fechas verificables; rechazar que baste cambiar el reloj local.
- Incorporar una cancelación del Hub sin duplicar su efecto ni sobrescribirla con una operación antigua.
- Enviar directamente a Nightingale API una operación que omita o manipule las validaciones Rust; comprobar rechazo por autorización o reglas compartidas.
- Reusar un identificador de operación con contenido distinto y comprobar que no cambia el efecto aceptado.
- Cancelar por el Hub y reintentar tras perder la respuesta, sin doble efecto ni resultado de éxito inventado.

### 23.6 Pruebas de recordatorios

Escenarios mínimos:

- Programación inicial.
- Reprogramación de una cita.
- Cancelación antes del envío.
- Trabajo entregado dos veces por la cola.
- Error temporal y reintento.
- Error permanente.
- Webhook duplicado.
- Respuesta sobre una versión antigua.

### 23.7 Pruebas del esqueleto Tauri

La interfaz mínima deberá permitir verificar de extremo a extremo:

- Crear paciente.
- Crear cita.
- Operar sin conexión.
- Mostrar cambio pendiente.
- Recuperar conexión.
- Resolver o presentar conflicto.
- Recibir una actualización creada desde otra sesión.
- Consultar hoy y mañana sin servicios externos, después de guardar o descargar esos datos.
- Cerrar y reabrir sin conexión conservando la agenda y los cambios pendientes.
- Mostrar la última vista del Hub sin afirmar que está actualizada ni completar sus acciones sin aceptación.

### 23.8 Reloj y aleatoriedad

Las pruebas utilizarán relojes e identificadores controlables. No dependerán de la hora real ni de UUID impredecibles cuando afecten el resultado.

### 23.9 Datos de prueba

Se mantendrán builders y fixtures reutilizables por componente. Los fixtures no se convertirán en un modelo global compartido que rompa encapsulación.

## 24. Configuración y secretos

### 24.1 Orden de configuración

La carga podrá combinar:

- Valores seguros por defecto.
- Archivo de desarrollo no sensible.
- Variables de entorno.
- Gestor de secretos.
- Configuración central por cuenta.
- Preferencias locales permitidas.

La precedencia deberá documentarse y probarse.

### 24.2 Validación

Los servicios validarán toda configuración al iniciar. Un valor ausente o inválido producirá un error claro antes de recibir trabajo.

### 24.3 Secretos

Nunca se versionarán:

- Contraseñas.
- Tokens.
- Claves de cifrado.
- Credenciales de proveedores.
- Enlaces firmados reales.
- Copias de bases con información personal.

Los ejemplos utilizarán nombres y valores ficticios.

## 25. Construcción, versiones y publicación

### 25.1 Versionado

Se versionarán de forma identificable:

- Aplicación de escritorio.
- API de servicios de apoyo.
- Esquema SQLite.
- Esquema PostgreSQL.
- Contratos.
- Eventos.
- Flujos n8n.
- ETL.

### 25.2 Compatibilidad

El contrato HTTP de sincronización, las versiones del escritorio y las API del Hub deberán declarar su compatibilidad. Nightingale API gestiona el esquema operativo sin exponerlo como contrato SQL a los consumidores. Una actualización del esquema o del servicio no dejará inutilizable sin aviso una instalación soportada. Los planes coordinarán migraciones entre repositorios y evitarán que cada cliente migre el esquema compartido sin control.

### 25.3 Escritorio

La distribución deberá contemplar:

- Firma.
- Verificación de actualización.
- Migración local.
- Recuperación ante fallo de actualización.
- Canal de pruebas separado.

### 25.4 Artefactos cloud

Cada servicio deberá producir un artefacto reproducible e identificable. La configuración de entorno permanecerá fuera del artefacto.

### 25.5 Flujos n8n

Los flujos se almacenarán como definiciones versionadas sin credenciales. Los cambios manuales en producción deberán reconciliarse con la fuente versionada.

## 26. Calidad de un componente

Un componente podrá considerarse conforme con el plan cuando:

- Su responsabilidad esté delimitada.
- Declare qué datos posee.
- Exponga una API pública mínima.
- Mantenga dominio y aplicación independientes de adaptadores.
- Defina sus puertos.
- Implemente adaptadores sin filtrar detalles al dominio.
- Utilice errores estables.
- Publique eventos versionados cuando corresponda.
- Incluya pruebas de dominio, aplicación y adaptadores relevantes.
- Respete aislamiento por cuenta.
- Añada observabilidad sin exponer datos sensibles.
- Documente decisiones abiertas o excepciones.

Esta lista es un criterio de conformidad, no una lista de tareas ni una secuencia de desarrollo.

## 27. Reglas de documentación

### 27.1 Decisiones arquitectónicas

Una decisión que afecte a varios componentes deberá registrarse como ADR.

Un ADR incluirá:

- Contexto.
- Decisión.
- Alternativas consideradas.
- Consecuencias.
- Estado.
- Fecha.
- Decisión reemplazada, si existe.

### 27.2 Documentación por componente

Cada componente tendrá un README breve con:

- Responsabilidad.
- Propiedad de datos.
- API pública.
- Puertos.
- Eventos.
- Invariantes.
- Adaptadores disponibles.
- Enlaces a contratos y ADR.

No repetirá explicaciones generales de arquitectura hexagonal.

### 27.3 Diagramas

Los diagramas explicarán relaciones que no sean evidentes mediante texto. No se utilizarán para reproducir listas de archivos o tablas que puedan mantenerse con mayor facilidad.

### 27.4 Código como fuente secundaria

El código refleja una implementación. Este documento define la intención. Si divergen, deberá corregirse la implementación o actualizarse formalmente la intención.

## 28. Excepciones

Una excepción a este plan deberá:

- Identificar la regla afectada.
- Explicar la necesidad.
- Limitar su alcance.
- Registrar consecuencias.
- Indicar si es temporal o permanente.
- Definir qué documento debe actualizarse.

No se aceptarán excepciones implícitas basadas únicamente en urgencia o conveniencia local.

## 29. Decisiones que deberán cerrarse en documentos separados

Este plan conserva abiertos los puntos que requieren evaluación específica:

1. Contrato HTTP transaccional de sincronización, orden confirmado del Change Log y compatibilidad; la propiedad de migraciones operativas corresponde a Nightingale API.
2. Contratos HTTP de los servicios de apoyo, propietario de su fuente y frontera exacta Citas/Patient Hub.
3. Proveedor cloud.
4. Versiones y arquitecturas de distribución soportadas; Windows, macOS y Linux ya están confirmados para la base de desarrollo.
5. Estrategia de cifrado de SQLite.
6. Alta de identidad, sesiones de API, permisos remotos y política temporal verificable para operaciones pendientes ante revocación.
7. Cola o broker definitivo.
8. Proveedor de WhatsApp.
9. Duración fija o variable de servicios.
10. Política exacta de recordatorios.
11. Política de lista de espera.
12. Frecuencia y modelo físico del ETL.
13. RPO, RTO y retención.
14. Estrategia futura de inteligencia artificial.

Cada cierre deberá producir una modificación de este documento o un ADR referenciado desde él.

## 30. Resumen normativo

Este repositorio contiene el escritorio Tauri, React, TypeScript, TailwindCSS, Rust y SQLite, organizado por componentes con dominio, aplicación y adaptadores separados. La interfaz inicial facilita integración y pruebas; no fija el diseño visual definitivo.

Rust ejecuta los casos de uso nativos sobre SQLite sin Internet. Su coordinador sincroniza operaciones de alto nivel por HTTPS con Nightingale API. API autoriza y revalida su aceptación en PostgreSQL privado con restricciones, transacciones, idempotencia y versiones. El escritorio no tiene credenciales SQL. Los cambios locales pendientes y sus conflictos permanecen visibles.

El Patient Hub es una excepción explícita: su API posee sus capacidades y Rust la consume sin duplicar su lógica ni su estado definitivo. Sin conexión se ofrece la última vista conocida, no una acción remota ficticiamente completada.

Los permisos locales conservan su última versión válida; cambiarlos requiere conexión. La reconciliación usa versiones y fechas verificables, nunca solo la hora declarada por el equipo.

Nightingale, Nightingale Patient Hub y Nightingale API tienen repositorios separados. El Hub usa API para guardar cambios compartidos de citas. Recordatorios, integración con n8n y analítica pertenecen a módulos y workers de Nightingale API, fuera de las transacciones de sincronización. Su ausencia no bloquea el núcleo local. Este documento define responsabilidades y no ordena tareas de desarrollo.
