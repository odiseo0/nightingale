# P-20261022-01-screen-flows: Pantallas y flujos del producto

**Status:** Draft
**Created:** 2026-10-22
**Updated:** 2026-10-22
**Task list:** [todo.md](todo.md)

## Overview

**Autorización posterior del prototipo (2026-10-22):** el usuario pidió generar el prototipo y guardar todas las pantallas en `design-prototype/`. Esa solicitud autoriza un prototipo visual independiente, no la implementación de las funciones en Tauri ni el cierre de los checkpoints de revisión. OpenDesign queda fuera de este trabajo por indicación del usuario. Se mantiene el estado Draft de la definición funcional.

La revisión posterior elimina los estados normales de sincronización de cada fila y de la cabecera. Solo se muestran avisos cuando una desconexión, demora, rechazo o conflicto afecta al trabajo. El chip Programada usa texto oscuro explícito. Estos criterios prevalecen sobre la primera muestra descrita más abajo.

Proponer las pantallas, su contenido, la navegación y los recorridos de Nightingale. Este documento es una propuesta para revisar con el usuario, no una aprobación del diseño ni un plan para programar las pantallas. Incluye requisitos verificables y preguntas que el diseño no puede resolver por sí solo.

Solo se crean estos archivos de planificación. No se modifican el producto, el plan maestro, los planes anteriores ni el sistema de diseño. La muestra en la conversación usa datos ficticios y solo permite explorar parte de la propuesta; no es la aplicación ni prueba sus servicios.

## Source of Truth

- [SRS](../../docs/srs.md): §§4–11, RF-01–RF-16 y preguntas de §16.
- [Plan maestro](../../docs/master-plan.md): §§4–10, 14–17, 19, 22–23 y 29.
- [Arquitectura](../../docs/technical-architecture.md): §§8–12, 17 y 20.
- [ADR vigente](../../docs/decisions/transactional-sync-three-projects.md).
- Especificaciones de [API](../../docs/specifications/nightingale-api.md) y [Hub](../../docs/specifications/nightingale-patient-hub.md).
- [Sistema de diseño](../../DESIGN.md) y [reglas de diseño](../../DESIGN-RULES.md).
- [Laws of UX](../../.agents/laws-of-ux/SKILL.md): su [fuente principal](../../.agents/laws-of-ux/references/laws-of-ux.md) prevalece sobre los ejemplos de patrones y la lista de comprobación.
- Código actual: `src/App.tsx`, `src/diagnostics.ts`, `src-tauri/src/lib.rs`, `src-tauri/src/sqlite.rs`. Solo existen diagnóstico y metadatos SQLite.
- [Reglas de planificación](../README.md) y [skill de planificación](../../.agents/planning-task-breakdown/SKILL.md).
- El plan de tres proyectos sigue en revisión documental; este borrador no cierra su C01 ni cambia sus decisiones.

### Cómo leer la propuesta

**Requisito vigente** significa que la fuente de verdad ya lo exige. **Propuesta UX** significa que se recomienda una forma de mostrarlo, pendiente de revisión. **Decisión pendiente** significa que falta una regla o contrato y no debe deducirse del aspecto de una pantalla.

Las ubicaciones, nombres de pantallas y formatos que siguen son propuestas UX salvo que se indique un requisito vigente. No se han observado usuarios reales: el orden de uso es una hipótesis que debe comprobarse con un médico y un asistente.

## Architecture Decisions

- Propietario nativo: Rust ejecuta Pacientes, Servicios, Agenda, Citas y Lista de espera. SQLite conserva datos autorizados, operaciones pendientes e historial local cuando lo defina su caso de uso. React presenta resultados por comandos Tauri específicos.
- Sin red se consulta la información ya recibida o creada. Hoy y mañana deben sobrevivir al reinicio; una fecha no descargada no se presenta como vacía. La política de acceso local sigue pendiente, aunque los últimos permisos válidos se conservan.
- Rust envía operaciones completas a Nightingale API por HTTPS. La API verifica acceso, versiones y reglas compartidas y registra efecto, resultado idempotente, auditoría y cambios en una transacción PostgreSQL. El escritorio no accede por SQL.
- El estado de la cita y su estado de sincronización se muestran por separado. Guardado local, aceptación compartida, envío de un mensaje y confirmación del paciente son resultados distintos.
- El Hub posee accesos, política del paciente y solicitudes. Rust consume su API; su vista local indica versión/antigüedad. El Hub usa Nightingale API para citas. No se permite completar una acción remota solo desde la muestra local.
- Recordatorios, n8n y analítica pertenecen a módulos y workers de Nightingale API. Su trabajo ocurre después del commit; pueden seguir con el escritorio apagado y no conocen cambios exclusivamente locales.
- Los contratos de datos, permisos, errores y recuperación deben definirse antes de programar cada recorrido. Este borrador no introduce rutas HTTP, esquemas ni comandos definitivos.
- No hay tareas paralelas ni implementación de servidores en este plan. API posee contratos de sincronización y migraciones operativas; Hub posee su contrato público y almacenamiento privado.

### Límite entre diseño y especificación

El PMD §22 pide una interfaz inicial sin diseño final; `DESIGN.md` declara una paleta clara aprobada. Se usa esa paleta para estudiar la propuesta, sin declarar cerrado el diseño del producto ni cambiar el PMD. Las alusiones genéricas de DESIGN a resultados de salud o planes clínicos no añaden funciones clínicas al alcance de Nightingale.

## Propuesta de navegación

**Inicio habitual: Agenda, en el día de hoy.** El usuario necesita saber a quién atenderá y qué debe resolver. Los indicadores históricos quedan en su propia pantalla. Una primera instalación sin cuenta o configuración tiene un recorrido de preparación, no una agenda que simule estar lista.

```text
Acceso / preparación cuando corresponda
  └─ Agenda · Hoy
       ├─ Día / Semana / Lista de citas
       ├─ Detalle de cita → Reprogramar / Cancelar / Registrar atención
       └─ Nueva cita → Buscar paciente → Servicio y horario → Guardar localmente

Navegación de trabajo: Agenda · Pacientes · Solicitudes · Lista de espera
Seguimiento: Comunicaciones · Indicadores
Utilidades: Sincronización · Configuración
```

No se añade otro destino «Inicio» ni un destino «Citas» que repita Agenda. Las citas se consultan por fecha en Día/Semana, por búsqueda y periodo en Lista, y por paciente en su ficha. Se conserva fecha, vista, filtros y posición al volver de un detalle.

La barra superior muestra cuenta activa, usuario y estado resumido de sincronización. Un selector de cuenta o profesional solo aparece cuando existe una elección autorizada. El menú lateral conserva texto; los iconos lo acompañan. Una barra compacta da acceso a sincronización desde cualquier pantalla.

Al abrir desde cero se propone hoy; al volver desde un detalle se restaura el contexto. La restauración de la fecha entre sesiones queda para revisión, para evitar mostrar por error un día pasado.

## Catálogo de pantallas

### S01 · Acceso y preparación

- **Objetivo:** entrar a la cuenta y al equipo correctos sin confundir sesión remota con acceso local.
- **Contenido:** identidad/cuenta conocidas, estado del equipo y paso pendiente. Alta remota y registro de dispositivo requieren conexión; un equipo ya preparado puede acceder localmente según la política que se defina.
- **Preparación inicial:** cuenta y zona horaria, servicio, horario y datos administrativos necesarios. Mostrar pasos reales completados; no exigir un tutorial antes de operar. Solo quien tenga permiso configura.
- **Requisitos:** diferenciar «Hace falta conexión para activar este equipo» de «No se pudo contactar con el servicio». Una sesión remota vencida no implica borrar datos locales ni perder operaciones pendientes. No se eligen aquí contraseña local, PIN o biometría.
- **Salida:** Agenda de hoy cuando la cuenta esté lista; bloqueo o recuperación claros cuando no se permita acceso.

### S02 · Agenda: inicio y consulta de citas

- **Pregunta:** «¿Qué citas tengo y qué puedo hacer ahora?»
- **Jerarquía:** título y fecha → navegación de fechas y acción «Nueva cita» → vistas Día/Semana/Lista → contenido → avisos relacionados.
- **Día, vista inicial propuesta:** filas ordenadas por hora con intervalo, nombre del paciente, servicio, estado de cita y estado de sincronización. Mostrar huecos y bloqueos a partir de la disponibilidad calculada, sin confundir un filtro vacío con un horario libre.
- **Semana:** intervalos a escala para comparar ocupación, con acceso al mismo detalle. Bloqueos y citas se distinguen por texto y forma. Mover una cita debe tener alternativa por formulario; arrastrar no confirma un cambio.
- **Lista:** búsqueda de citas por nombre/referencia autorizada, periodo, servicio y estado. Incluye pasadas y canceladas cuando el filtro lo pida; pagina los resultados históricos. Es la respuesta a «buscar una cita», sin otra sección duplicada.
- **Controles:** Hoy, anterior/siguiente, selector de fecha y filtros. Estado/servicio son filtros accesibles; no mostrar una lista de profesionales de una sola opción. Mantener visibles los filtros activos y «Limpiar filtros».
- **Avisos:** resumen breve de solicitudes y conflictos con enlace al destino. Los conteos indican su alcance y antigüedad; «no disponible» no equivale a cero. Los totales históricos no ocupan el inicio.
- **Acciones:** «Nueva cita», abrir una cita y crear desde un hueco conocido. Cambiar el horario habitual lleva a Configuración con retorno a Agenda.
- **Requisitos:** hoy/mañana locales disponibles sin red tras guardar/descargar; etiqueta «Guardada en este equipo · Pendiente de sincronizar» cuando corresponda; conservar fecha y filtros al volver. No mostrar nombres esenciales truncados sin un modo claro de leerlos.

### S03 · Detalle de cita

- **Objetivo:** revisar la cita y actuar sin perder su contexto.
- **Contenido, en orden:** paciente y estado de cita; fecha, inicio/fin, servicio y lugar; estado de sincronización; contacto administrativo; confirmación/solicitud; comunicaciones; historial.
- **Presentación:** panel junto a Agenda si cabe con texto legible; pantalla completa cuando no. El mismo contenido y acciones deben funcionar en ambos casos. En un panel, devolver el foco al elemento que lo abrió.
- **Acciones:** reprogramar, cancelar, registrar atención o ausencia y confirmar por el personal si la transición y permisos lo permiten. Emitir o consultar acceso del Hub utiliza su API y requiere resultado remoto conocido.
- **Requisitos:** no usar un selector que permita asignar cualquier estado. Mostrar acciones válidas para el estado, rol y momento; explicar las restricciones relevantes. No marcar ausencia por falta de respuesta. Conservar cancelaciones e historial.
- **Contacto:** no prometer un envío al abrir una acción; su resultado pertenece a Comunicaciones. No incluir notas clínicas ni archivos médicos.

### S04 · Crear o reprogramar una cita

- **Objetivo:** registrar una cita con datos mínimos, conservando el paciente, servicio y horario elegidos.
- **Formulario por grupos:** Paciente → Servicio → Fecha e intervalo → Resumen. No imponer un asistente de varias pantallas si el formulario cabe.
- **Paciente:** buscar entre datos locales autorizados y distinguir coincidencias por contacto permitido. «Registrar paciente» abre un paso explícito; al guardarlo, volver a la cita conservando el resto. Los campos obligatorios del paciente aún deben definirse; no añadir cédula, edad o correo obligatorio por criterio visual.
- **Horario:** mostrar solo opciones que cumplen las reglas locales conocidas y explicar por qué un intervalo no puede usarse. Duración mediante inicio/fin; la representación fija o variable sigue abierta. Si se llega desde un hueco, completar fecha e intervalo y permitir revisarlos.
- **Resumen siempre visible antes de guardar:** paciente, servicio, fecha, inicio/fin y zona horaria cuando sea necesaria para evitar ambigüedad. En reprogramación, comparar horario anterior y propuesto sin tener que recordarlo.
- **Guardado:** «Guardar cita» termina primero en «Guardada en este equipo; pendiente de sincronizar». El error conserva los campos. Un doble clic no crea otra intención. Un cambio remoto no sobrescribe el formulario en silencio.
- **Reprogramación:** mostrar que el nuevo horario sigue pendiente; no afirmar que el horario anterior se liberó en el estado compartido antes de la aceptación. Cambiar fecha/hora invalida la confirmación anterior según contrato.
- **Cancelar edición:** volver sin cambios persistidos; si hay entradas sin guardar, pedir una decisión clara. Un borrador de formulario no se presenta como operación guardada. Su persistencia automática no se añade por defecto.
- **Conflicto:** conservar la propuesta, mostrar el estado aceptado y ofrecer elegir otro horario. No reservar automáticamente una alternativa ni limitarse a «Error al guardar».

### S05 · Pacientes

- **Pregunta:** «¿Quién es el paciente y cómo encuentro su cita?»
- **Contenido:** búsqueda por nombre/contacto admitido, lista de coincidencias, datos administrativos mínimos y estado activo/inactivo. No introducir documentos clínicos.
- **Acción principal:** «Registrar paciente»; cada resultado abre S06. Evitar cargar todo el historial de cada paciente en la lista.
- **Requisitos:** distinguir «sin resultados», «no hay pacientes locales» y «búsqueda limitada a datos de este equipo». Una búsqueda offline no prueba que un paciente no exista en otra copia; el tratamiento de posibles duplicados debe definirse antes de ofrecer fusión.

### S06 · Ficha del paciente

- **Contenido:** identidad administrativa, contacto, preferencias/consentimiento cuando se definan, próximas citas e historial de citas paginado.
- **Acciones:** «Nueva cita» con paciente elegido y editar datos administrativos. Desactivar o fusionar son acciones separadas, sujetas a permiso y contrato; no botones de borrado genérico.
- **Requisitos:** mostrar versiones/cambios pendientes cuando afecten al resultado. No reinterpretar un cambio de teléfono como modificación de mensajes ya entregados. Volver a la lista mantiene su búsqueda.

### S07 · Solicitudes de cambio del paciente

- **Pregunta:** «¿Qué pide el paciente y qué falta resolver?»
- **Contenido:** solicitudes del Hub, fecha de recepción, paciente/cita, horario vigente y estado recibido. En detalle, mostrar la solicitud junto a la cita, su antigüedad y resultado conocido.
- **Acción principal propuesta:** «Revisar horario», no «Aceptar» con asignación automática. El personal elige la nueva hora en S04; Rust solicita aceptación de la cita a Nightingale API y coordina la resolución con el Hub.
- **Fallo parcial:** distinguir «Cambio de cita aceptado; falta actualizar la solicitud» de «Cambio aún pendiente de sincronizar». Conservar ambas intenciones y reintentar según contrato.
- **Requisitos:** offline solo última vista conocida; no resolver solicitudes mediante una marca local de éxito. Salidas de `reschedule_requested`, rechazo/retirada y conservación de confirmación previa siguen pendientes. No deducir esas reglas de los botones de la muestra.

### S08 · Lista de espera

- **Contenido:** entradas por servicio, preferencias de horario, orden según política pendiente y ofertas con su vencimiento/resultado. Separar «En espera» de «Ofertas».
- **Acciones:** añadir/retirar una entrada, revisar candidatos compatibles para un hueco y solicitar su asignación a Citas.
- **Requisitos:** una oferta aceptada no es una reserva confirmada. La asignación sigue el caso de uso nativo y su aceptación compartida. No ordenar pacientes por un criterio arbitrario ni anunciar una reserva desde una respuesta del proveedor. La prioridad y el canal/recorrido de respuesta requieren contrato.

### S09 · Comunicaciones

- **Pregunta:** «¿Qué se ha enviado, qué respondió el paciente y qué necesita seguimiento?»
- **Contenido:** próximas comunicaciones, historial por cita y fallos que requieren atención; destinatario mínimo, tipo, fecha prevista, estado, última actualización y resultado de respuesta conocido.
- **Acciones:** ver cita; configurar política si se tiene permiso; solicitar reintento solo cuando el contrato lo permita. No añadir un chat ni un compositor libre de WhatsApp al alcance.
- **Requisitos:** separar intención local, programado, enviado, entregado y respondido. Offline se ve el último resultado recibido con antigüedad. Un timeout de envío no autoriza reenviar ciegamente: puede haber un resultado externo desconocido. No afirmar una notificación por el solo hecho de guardar/cancelar una cita.

### S10 · Indicadores

- **Contenido:** periodo, zona horaria, fecha de actualización, definición de cada indicador y datos de citas, confirmaciones, cancelaciones, ausencias y horarios recuperados. Mostrar números y denominadores; no usar gráficos decorativos.
- **Presentación:** pocos indicadores con propósito, tendencia si hay datos comparables y tabla accesible como alternativa al gráfico. No introducir un porcentaje de mejora sin línea base.
- **Requisitos:** datos recibidos del servicio analítico; no mezclar operaciones locales pendientes con hechos aceptados como si fueran una sola medición. Diferenciar «sin datos», «desactualizado» e «indisponible». La caída analítica no impide abrir Agenda.

### S11 · Sincronización y resolución de conflictos

- **Acceso:** estado resumido siempre visible; pantalla de detalle cuando se necesita revisar.
- **Contenido:** última sincronización correcta, operaciones pendientes, fallos temporales y conflictos, con alcance por cuenta/dispositivo. Evitar mostrar Outbox, SQL o cursores como vocabulario principal.
- **Conflicto de cita:** comparar horario/estado compartidos con la intención local; explicar qué cambió y permitir revisar otra hora o descartar la intención local de forma explícita. No usar «Forzar guardado» ni «Último cambio gana» para citas.
- **Requisitos:** una caída de API mantiene cambios locales. Reintentar no duplica efectos. Una revocación puede bloquear intercambio o acceso según política pendiente; no borrar la cola ni aceptar por la hora declarada por el equipo. Una respuesta perdida deja «Resultado por comprobar», no otro guardado con nuevo identificador.

### S12 · Configuración

- **Grupos:** Cuenta y zona horaria; Servicios; Horarios y bloqueos; Comunicaciones; Usuarios y permisos; Dispositivos; Datos y seguridad local cuando se definan sus capacidades.
- **Servicios/horarios:** subpantallas con lista y editor; acceso desde Agenda cuando la tarea lo exige. Mostrar el efecto en citas ya existentes antes de guardar. Cambiar duración no modifica citas anteriores sin una operación explícita.
- **Permisos/dispositivos:** administración remota con conexión y autorización; separar de preferencias locales. No simular revocación instantánea de un equipo desconectado.
- **Requisitos:** ocultar grupos ajenos al permiso y explicar capacidades conocidas que estén temporalmente bloqueadas. No ofrecer exportaciones, copias, importaciones o borrados que aún no tengan contrato de protección y recuperación. Evitar selectores de opciones inexistentes.

### S13 · Diagnóstico de desarrollo

- Se conserva fuera de la navegación diaria, mediante ayuda/desarrollo según la distribución.
- Muestra datos técnicos permitidos sin secretos; el diagnóstico existente no demuestra que un servidor esté conectado. Esta propuesta no amplía su API.

### S14 · Patient Hub: recorrido externo, no pantalla del escritorio

- **Entrada válida:** resumen de cita, estado vigente, fecha/hora, profesional, servicio, lugar e instrucciones administrativas.
- **Acciones:** confirmar, cancelar dentro de política y solicitar cambio. Un diálogo breve explica consecuencias de cancelar. Pedir cambio no ofrece una agenda pública ni reserva un intervalo.
- **Resultados:** acción aceptada; resultado por comprobar; estado cambió y hace falta revisar; enlace vencido/revocado; servicio no disponible. Evitar mostrar información de otra cita al recuperar un enlace.
- **Requisitos:** recorrido móvil, controles fáciles de pulsar, sin registro obligatorio inventado. Abrir o previsualizar un enlace GET no ejecuta acciones. Vencimiento/verificación adicional se diseñan cuando exista política. Solo el proyecto del Hub implementará este recorrido.

## Flujos que deben poder revisarse

| Recorrido | Pasos y resultado observable |
|---|---|
| Inicio habitual | Acceso local permitido → Agenda de hoy → abrir cita → volver a la misma fecha/filtro. |
| Crear cita | Agenda/hueco o paciente → S04 → revisar resumen → guardar localmente → pending → aceptación o conflicto. |
| Paciente nuevo | S04 → registrar paciente explícitamente → volver con paciente seleccionado y resto del formulario intacto. |
| Reprogramar | S03 → anterior/propuesto juntos → S04 → pendiente → aceptación compartida → confirmación anterior invalidada según contrato. |
| Cancelar | S03 → confirmar paciente/hora y consecuencia → cancelación local pendiente → aceptación compartida y liberación; comunicación se informa aparte. |
| Registrar atención/ausencia | S03 → acción válida y explícita → guardado local → aceptación; ausencia exige hora/margen y confirmación del personal. No incorpora check-in. |
| Solicitud del Hub | S07 → revisar cita/solicitud → elegir horario → aceptación por API → coordinar resolución del Hub; mostrar cada fallo parcial. |
| Recuperar hueco | S08 → candidatos compatibles → oferta → respuesta → solicitud nativa de reserva → aceptación. Política de prioridad pendiente. |
| Trabajo offline | Datos locales → crear/cambiar → cerrar → abrir sin red → mismos datos y pendientes; fecha no descargada muestra falta de datos. |
| Conflicto concurrente | API rechaza → S11 conserva intención y estado compartido → revisar alternativa → nueva intención explícita según contrato. |
| Acción del paciente | Enlace válido → Hub valida → acción → API acepta → Hub muestra resultado conocido → escritorio recibe la versión sin repetir efecto. |

## Requisitos comunes de las pantallas

| ID | Requisito verificable |
|---|---|
| UX-01 | Cada pantalla tiene una tarea clara; el texto y la posición de una misma acción se mantienen. No se crea un máximo arbitrario de opciones por Miller. |
| UX-02 | Se distingue estado de cita, sincronización y comunicación por texto; el color nunca comunica el estado por sí solo. |
| UX-03 | Todo guardado distingue local/compartido; el formulario conserva los datos ante error y no inventa éxitos remotos. |
| UX-04 | Existen estados inicial, vacío real, sin coincidencias, carga, éxito, error, permiso insuficiente, datos no descargados, datos antiguos y conflicto cuando aplican. |
| UX-05 | Volver conserva el contexto; formularios y comparaciones evitan recordar datos de otra pantalla. Confirmar una acción seria muestra a quién y qué afecta. |
| UX-06 | Teclado completo, etiquetas persistentes, foco visible y no oculto por paneles, lectura de errores y retorno de foco. No hay funciones solo por hover o arrastre. |
| UX-07 | Texto normal con contraste al menos 4.5:1; texto esencial legible, sin roles de 10–12 px para nombres, horas, acciones o instrucciones necesarias. |
| UX-08 | Probar 800×600, 1024×768 y ventana grande, además de texto ampliado al 200 %. En ancho reducido, el detalle ocupa su propia superficie y la lista conserva los datos esenciales. Hub se prueba desde 320 px. |
| UX-09 | Seguir la paleta y los roles de DESIGN: fondos #F8FAFC/#F1F5F9/#FCFDFE, texto #111827/#475569 y azul #2563EB. Selección #DBEAFE con texto #1D4ED8; Inter Variable y alternativas de sistema. Espaciado base 8 px; pocos bordes/sombras. |
| UX-10 | Error, advertencia y conflicto no reciben nuevos colores arbitrarios: DESIGN aún no define esos colores semánticos. Mientras se revisan, la propuesta usa texto y estructura. |
| UX-11 | La interfaz confirma la recepción de la acción sin esperar una red. Un objetivo de respuesta debe medirse después; CSS o una muestra interactiva no prueban latencia ni el umbral de Doherty. |
| UX-12 | La cuenta, zona horaria y alcance de filtros evitan ambigüedad. Las búsquedas y acciones respetan permisos; no se muestran datos de otra cuenta. |

Accesibilidad: se propone WCAG 2.2 AA como objetivo verificable, sin afirmar conformidad por esta revisión. Revisar [foco no oculto](https://www.w3.org/WAI/WCAG22/Understanding/focus-not-obscured-minimum) y [tamaño de objetivos](https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum): AA establece 24×24 píxeles CSS o sus excepciones; los 44×44 para interacción táctil son el objetivo práctico de DESIGN, no el mínimo AA general. El recorrido nativo también necesita pruebas con tecnologías de asistencia del sistema.

## Leyes seleccionadas y decisiones que explican

| Ley o principio | Aplicación concreta |
|---|---|
| Cognitive Load / Hick | Abrir en el trabajo diario; reducir destinos duplicados; elegir valores de contexto, sin esconder opciones necesarias. |
| Jakob / Mental Model | Agenda con fechas y vistas conocidas; navegación estable; buscar pacientes antes de crear otra ficha. Son hipótesis de uso que se validan, no garantías. |
| Proximity / Chunking | Separar paciente, horario y resultado; mantener cada acción con su cita y cada etiqueta con su campo. |
| Working Memory / Miller | Horario anterior y propuesto juntos; conservar paciente y filtros entre pasos. No imponer siete enlaces o columnas. |
| Fitts | Acciones con etiqueta, tamaño y separación suficientes; no depender de iconos pequeños para operar. |
| Selective Attention / Von Restorff | Una acción principal por tarea; destacar conflictos concretos y pendientes relevantes, sin volver toda la agenda una alerta. |

No se enumeran todas las leyes ni se tratan sus ejemplos como requisitos universales. No hay pantallas de negocio actuales que permitan declarar violaciones confirmadas; lo anterior fundamenta decisiones propuestas. La exigencia de código de la skill se concreta en la muestra de navegación de la conversación, sin cambiar React ni introducir funcionalidad del producto durante planificación.

## Trazabilidad de requisitos

| SRS | Pantallas / flujos |
|---|---|
| RF-01, RF-02, RF-03 | S02–S04: agenda, lista, formulario y aceptación/conflicto. |
| RF-04, RF-08, RF-10 | S03–S04, S11: transiciones, historial y disponibilidad. |
| RF-05, RF-07 | S02, S03, S09: pendientes de confirmación, trabajos y respuestas. |
| RF-06, RF-16 | S07, S14: capacidades del Hub, alcance del paciente y vista derivada. |
| RF-09 | S08: lista, oferta, respuesta y aceptación de la reserva. |
| RF-11 | S10: indicadores con periodo, denominador y antigüedad. |
| RF-12, RF-15 | S01, S11, S12 y reglas comunes de autorización. |
| RF-13, RF-14 | S02–S06, S08, S11: datos locales, reinicio, pendientes y recuperación. |

## Dependency Order

```text
T01: navegación y tarea diaria → T02: recorrido de cita → C01
C01 → T03: pendientes y fallos → T04: recorrido del paciente → C02
```

Estas tareas ordenan la revisión de diseño. La redacción de este borrador no las marca como aceptadas. Tras aprobar la definición, la implementación necesitará otro plan con contratos y pruebas por caso de uso; no se autoriza aquí.

## Task List

### Phase 1: Operación diaria
- [ ] T01: Validar el inicio y la navegación.
- [ ] T02: Validar el recorrido de una cita.

### C01: Recorrido diario revisado
- [ ] El usuario revisa inicio, vistas, contenido y acciones de citas.
- [ ] Médico/asistente recorren escenarios con datos ficticios o se registra que esa prueba está pendiente.

### Phase 2: Continuidad y paciente
- [ ] T03: Validar pendientes y recuperación.
- [ ] T04: Validar el recorrido externo del paciente.

### Prototipo visual solicitado después de la revisión
- [x] T05: Crear y verificar todas las superficies en `design-prototype/`.

### C02: Definición revisada
- [ ] Se revisan fallos, permisos y separación local/remota sin inventar reglas pendientes.
- [ ] El usuario acepta la definición y decide si quiere un plan de implementación posterior.

## Risks and Mitigations

| Riesgo | Impacto | Tratamiento |
|---|---|---|
| Dar por cerrado el diseño por una muestra convincente | Alto | Mantener Draft, usar datos ficticios y validar recorridos antes de programar. |
| Confundir guardado con reserva, envío o respuesta del paciente | Alto | Textos distintos y estado visible por cada resultado. |
| Resolver permisos o reprogramación mediante decisiones visuales | Alto | Señalar decisiones pendientes y no mostrar éxito sin contrato. |
| Perder texto o acciones en la ventana actual de 800×600 | Alto | Detalle completo en superficie propia cuando no cabe y pruebas con texto ampliado. |
| Convertir el inicio en un panel de cifras o avisos | Medio | Priorizar citas del día; enlaces al seguimiento en su contexto. |
| Adoptar patrones genéricos de salud que amplíen el alcance | Alto | Sin historia clínica, resultados, diagnósticos, check-in o pagos. |

## Open Questions

Para revisión UX: confirmar Agenda de hoy como inicio; lista diaria como vista inicial; detalle lateral solo con espacio; orden de navegación; necesidad de restaurar fecha entre sesiones. No se presentan como decisiones ya tomadas por el usuario.

Antes de implementar los recorridos afectados: datos mínimos del paciente; política de acceso local y sesiones; permisos por acción; cifrado/claves; aceptación tras revocación; duración de citas; estados completos de solicitudes y conciliación Hub/API; prioridad de espera; políticas de cancelación y recordatorio; definición de indicadores; ventana local y recuperación de cursores. Las preguntas ya existentes siguen en sus fuentes; este borrador no las resuelve.

## Comprobaciones del borrador

2026-10-22: 14 superficies definidas, con trazabilidad de RF-01–RF-16. Se comprobaron los enlaces locales de los dos archivos, espacios al final de línea e identificadores/atributos estáticos de la muestra; sin incidencias. `git diff --check` pasó para cambios versionados y una comprobación directa cubrió los documentos nuevos aún no seguidos por Git. El estado del repositorio solo añade la carpeta de este plan; no se cambió código de producto ni planes anteriores.

La muestra pasó comprobación de sintaxis JavaScript y no contiene llamadas de red. Se revisaron sus rutas por lectura del código. El navegador rechazó abrir el archivo local por su política de protocolos; no se ejecutó una inspección visual ni pruebas de interacción en navegador, teclado, lector de pantalla o tamaños de ventana. Estos controles quedan sin verificar, no aprobados. El contenedor de vista previa generado no cambia el producto. No se ejecutaron compilaciones ni pruebas de servicios: este trabajo define una propuesta UX y no modifica la aplicación.

### Entrega posterior: prototipo solicitado en design-prototype/

2026-10-22: T05 terminada. Se creó un prototipo independiente con 34 rutas para las 14 superficies y variantes, un índice y 34 capturas. La fuente Inter y su licencia están incluidas. Esta entrega usa las correcciones de sincronización discreta, texto oscuro de Programada y densidad visual revisadas con el usuario.

Se verificaron las 34 rutas en navegador a 1280×720, 800×600 y 320×720 sin desbordamiento horizontal del documento. Se probaron creación, solapamiento, reprogramación, protección de edición, cancelación, registro/búsqueda de pacientes, alta desde cita con conservación del horario, confirmación/solicitud del Hub y escenarios de desconexión/conflicto. Se inspeccionaron las capturas de Agenda, detalle, editor y Hub. `node --check design-prototype/app.js`, enlaces, recursos, espacios y `git diff --check` pasan. La vista previa usa HTTP en 127.0.0.1 y sirve solo la carpeta del prototipo.

La evidencia de esta entrega sustituye las limitaciones de inspección de la primera muestra, sin atribuirle retrospectivamente pruebas. La revisión con el usuario, personal médico, lector de pantalla y los servicios reales siguen fuera de estos resultados. Los checkpoints T01–T04/C01/C02 no se cierran por generar el prototipo. La aplicación, sus dependencias, las especificaciones y los planes anteriores no cambian.
