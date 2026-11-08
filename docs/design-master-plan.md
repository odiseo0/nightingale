# Plan Maestro de Diseño e Interfaz de Nightingale

**Versión:** 0.1\
**Fecha:** 2026-11-08\
**Estado:** Aprobado\
**Ámbito:** Estructura y reglas para el diseño, la experiencia de uso y el frontend de Nightingale

## 1. Función de este documento

Este documento organiza el diseño y la interfaz de Nightingale. Define los principios de experiencia, la arquitectura de información, las superficies, los recorridos, los componentes, los estados, las reglas de presentación y las comprobaciones que deberán respetar los trabajos posteriores.

Su finalidad es servir como referencia al crear especificaciones de interfaz, decisiones de diseño, prototipos, componentes React y planes de ejecución pequeños. Esas actividades se derivarán de este documento, pero no forman parte de él.

Este documento es la fuente de verdad UI/UX aprobada para Nightingale. Las decisiones que permanezcan etiquetadas como propuestas o pendientes conservan ese alcance y no quedan resueltas por esta aprobación general.

Este documento no contiene:

- Backlog.
- Tareas de implementación.
- Historias de usuario.
- Estimaciones.
- Fechas de entrega.
- Asignaciones de trabajo.
- Prioridades de desarrollo.
- Estados de ejecución.
- Un orden obligatorio de implementación.
- Mockups o una autorización para ejecutar diseño visual.

Cuando una decisión posterior contradiga este documento, deberá actualizarse formalmente o registrarse una excepción explícita. No se mantendrá una divergencia mediante conocimiento informal.

## 2. Lenguaje normativo

Los términos se interpretarán de la siguiente manera:

- **Deberá:** requisito obligatorio una vez aprobado este documento.
- **No deberá:** prohibición obligatoria una vez aprobado este documento.
- **Debería:** recomendación que solo podrá omitirse con una razón documentada.
- **Podrá:** opción permitida, pero no exigida.
- **Pendiente:** decisión que todavía no forma parte del diseño aprobado.

También se usarán estas etiquetas para conservar la procedencia:

- **Vigente:** exigencia respaldada por una fuente ya aprobada o por un contrato actual.
- **Propuesta UX:** decisión de presentación sometida a revisión; no modifica el dominio.
- **Implementado:** comportamiento observable en el código actual, sin convertirlo automáticamente en diseño definitivo.
- **Histórico:** evidencia de un plan o prototipo anterior, subordinada a las fuentes vigentes.

Los nombres de superficies, rutas y componentes explican una intención. Solo serán obligatorios cuando el texto los identifique expresamente como convención aprobada.

## 3. Documentos de referencia y precedencia

| Código | Documento | Responsabilidad |
|---|---|---|
| SRS | [Especificación de requisitos](srs.md) | Visión, usuarios, alcance y RF-01–RF-16 |
| PMD | [Plan Maestro de Desarrollo](master-plan.md) | Arquitectura, componentes y límites técnicos |
| DAT | [Arquitectura técnica](technical-architecture.md) | Operación local, sincronización y recuperación |
| API v1 | [Contrato de Nightingale API](contracts/api-v1/nightingale-api.md) | Aceptación remota, modelos y reglas compartidas |
| DS | [Sistema de diseño](../DESIGN.md) | Color, tipografía, iconos, geometría, componentes y tokens |
| DR | [Reglas de diseño](../DESIGN-RULES.md) | Tarea, claridad, contenido, consistencia y comportamiento |
| UX | [Laws of UX](../.agents/laws-of-ux/SKILL.md) | Proceso obligatorio de diseño y revisión |
| SF | [Flujos históricos](../tasks/P-20261022-01-screen-flows/plan.md) | Propuesta S01–S14 y requisitos UX anteriores |
| DP | [Prototipo](../design-prototype/README.md) | Exploración histórica de 34 vistas; no diseño final |

Reglas de precedencia:

1. La intención funcional procede de SRS y de las decisiones posteriores confirmadas.
2. La arquitectura y los contratos proceden de PMD, DAT, ADR y API v1.
3. La identidad visual y los valores de tokens proceden de DS.
4. Las reglas generales de diseño de producto proceden de DR.
5. Este documento organiza su aplicación a superficies, recorridos y frontend.
6. UX se aplicará en todo trabajo de interfaz, sin convertir ejemplos en requisitos arbitrarios.
7. El código y el prototipo son fuentes secundarias. Si divergen de una intención aprobada, deberá corregirse la implementación o actualizarse formalmente la intención.
8. Ante una contradicción no resuelta, el trabajo afectado deberá detenerse hasta actualizar la fuente correspondiente.

La revisión histórica SF contiene referencias anteriores a Inter y pesos 510/590. DS las sustituye: Anek Latin, Aileron y pesos 400/500/600 son las reglas actuales. DP conserva Inter como evidencia histórica y no deberá usarse para restablecer esa tipografía.

## 4. Alcance y exclusiones

### 4.1 Usuarios y contexto

La primera versión está dirigida a:

- Médicos independientes.
- Especialistas con consulta privada.
- Asistentes autorizados.
- Pacientes que reciben un acceso a una cita existente mediante Patient Hub.

El trabajo de escritorio ocurre en contextos administrativos reales: recepción, consulta de agenda, cambios frecuentes, conectividad irregular y varios dispositivos. La interfaz deberá priorizar rapidez de lectura, comparación, continuidad y prevención de errores.

### 4.2 Alcance de la experiencia

La interfaz cubre:

- Acceso, preparación y selección de cuenta.
- Agenda y disponibilidad.
- Creación y gestión de citas.
- Datos administrativos de pacientes y contactos.
- Solicitudes de cambio.
- Lista de espera y ofertas.
- Comunicaciones y respuestas.
- Indicadores operativos.
- Sincronización y resolución de conflictos.
- Configuración, permisos y dispositivos.
- Diagnóstico de desarrollo.
- Continuidad semántica con Patient Hub.

### 4.3 Exclusiones

No se diseñarán como parte de esta versión:

- Historia clínica, diagnósticos, resultados médicos o imágenes clínicas.
- Recomendaciones médicas automatizadas o predicción de ausencias.
- Check-in, filas presenciales, pagos, seguros o autorizaciones.
- Chat libre o compositor general de WhatsApp.
- Creación pública de citas o consulta de la agenda general desde Patient Hub.
- Gestión avanzada de múltiples clínicas o sedes.
- Funciones sin contrato o sin una necesidad de usuario identificada.

### 4.4 Superficies y propietarios

| Proyecto | Responsabilidad de interfaz |
|---|---|
| Nightingale | Escritorio Tauri para personal autorizado; React presenta resultados de casos de uso Rust |
| Patient Hub | Web móvil y API propias para pacientes; su diseño detallado pertenece a su repositorio |
| Nightingale API | Sin interfaz de usuario en este alcance; posee aceptación compartida y contratos |

## 5. Principios de experiencia y proceso UX

### 5.1 Tarea y jerarquía

- Cada pantalla deberá responder a una tarea principal identificable.
- Cada superficie deberá tener un elemento dominante y una acción principal coherente con esa tarea.
- Los componentes se elegirán después de comprender el recorrido, no para llenar espacio.
- Se eliminará contenido repetido, decoración sin función y opciones que no correspondan al estado o permiso actual.
- La Agenda de hoy es la propuesta de inicio habitual; los indicadores históricos no sustituirán el trabajo diario.
- Las acciones frecuentes permanecerán visibles; las poco frecuentes podrán agruparse en menús contextuales sin ocultar las necesarias.

### 5.2 Continuidad y carga cognitiva

- El sistema conservará el contexto necesario para que el usuario no memorice datos entre pantallas.
- Fecha, filtros, búsqueda, selección y posición deberán restaurarse al volver cuando la tarea lo requiera.
- Los valores que deban compararse permanecerán juntos: horario anterior/propuesto, propuesta local/estado compartido y solicitud/cita vigente.
- La complejidad técnica se traducirá a resultados comprensibles sin ocultar incertidumbre ni estados pendientes.
- Los formularios conservarán los datos ante errores y al registrar una entidad relacionada.

### 5.3 Confianza y honestidad operativa

- Guardar localmente no equivale a aceptación compartida.
- Aceptar una cita no equivale a enviar un mensaje.
- Entregar un mensaje no equivale a recibir respuesta ni confirmar asistencia.
- Solicitar un cambio no reserva otro horario.
- Expresar interés en una oferta no confirma una reserva.
- La ausencia requiere una acción del personal después del margen aplicable; no se deduce de la falta de respuesta.
- Ningún estado relevante se comunicará solo mediante color, icono, posición o animación.

### 5.4 Aplicación obligatoria de Laws of UX

Todo diseño, cambio o revisión de UI/UX deberá aplicar [Laws of UX](../.agents/laws-of-ux/SKILL.md):

1. Establecer usuario, tarea, plataforma, componente y recorrido.
2. Leer el código, estilos y estado reales cuando existan.
3. Seleccionar solo las leyes pertinentes y leer su fuente principal.
4. Relacionar cada ley con una decisión o problema concreto.
5. Priorizar violaciones confirmadas sobre sugerencias opcionales.
6. Proponer o aplicar el cambio mínimo en el stack existente cuando el trabajo lo autorice.
7. Verificar la interfaz en ejecución cuando layout, foco, movimiento o tiempo sean relevantes.

No se enumerarán todas las leyes en cada plan. Miller no impone un máximo de elementos, una transición CSS no demuestra feedback inferior a 400 ms y un checklist no acredita por sí solo preparación para publicación.

## 6. Arquitectura de información y contexto

### 6.1 Navegación propuesta

**Propuesta UX, pendiente de aprobación final:**

```text
Acceso / preparación cuando corresponda
  └─ Agenda · Hoy
       ├─ Día / Semana / Lista de citas
       ├─ Detalle → acciones de cita
       └─ Nueva cita / Reprogramar

Trabajo diario
  ├─ Agenda
  ├─ Pacientes
  ├─ Solicitudes
  └─ Lista de espera

Seguimiento
  ├─ Comunicaciones
  └─ Indicadores

Utilidades
  ├─ Cambios por revisar
  ├─ Configuración
  └─ Ayuda / Diagnóstico de desarrollo
```

No deberá añadirse un destino Inicio que duplique Agenda ni un destino Citas que repita las vistas de Agenda. La navegación conservará texto; los iconos acompañarán, no sustituirán etiquetas desconocidas.

### 6.2 Jerarquía común de una superficie

Salvo que la tarea exija otra estructura:

1. Contexto de cuenta y superficie.
2. Título y descripción breve.
3. Acción principal.
4. Navegación o filtros propios de la tarea.
5. Contenido principal.
6. Avisos relacionados con el trabajo.
7. Detalle o acciones secundarias en contexto.

Los avisos de sincronización no ocuparán la posición principal en funcionamiento normal. Un conflicto o pérdida de permiso sí podrá destacarse cuando impida completar la tarea.

### 6.3 Contexto persistente y retorno

Cada plan de superficie deberá declarar qué se conserva:

- Cuenta y usuario activos.
- Fecha y vista de Agenda.
- Filtros y búsqueda.
- Paciente o cita seleccionados.
- Orden, paginación y posición de lectura cuando sean útiles.
- Formulario y borrador solo durante el recorrido autorizado.
- Elemento que abrió un diálogo o detalle para devolverle el foco.

Al abrir una sesión nueva, la propuesta es mostrar hoy, no restaurar silenciosamente una fecha pasada. La persistencia de fecha entre sesiones queda **Pendiente**. Un borrador no se presentará como una operación guardada.

### 6.4 Búsqueda y acceso directo

La búsqueda global deberá limitarse a pacientes y citas autorizados de la cuenta activa. Podrá ofrecerse mediante `Ctrl+K`/`Cmd+K`, con alternativa visible, teclado completo y resultados que distinguan tipo, nombre y contexto permitido.

Las búsquedas offline deberán indicar que solo cubren datos disponibles en el equipo. “Sin coincidencias locales” no demuestra que una entidad no exista en otra copia autorizada.

## 7. Catálogo de superficies

Los IDs S01–S14 son identificadores estables de referencia. Las rutas de DP son ejemplos históricos y no se consideran rutas finales aprobadas.

| ID | Superficie | Tarea principal | Contenido y acciones esenciales |
|---|---|---|---|
| S01 | Acceso y preparación | Entrar y dejar lista la consulta | Sesión, dispositivo, cuenta, zona, datos iniciales, estados de acceso |
| S02 | Agenda | Saber qué citas hay y qué requiere atención | Día/Semana/Lista, fechas, filtros, disponibilidad, nueva cita |
| S03 | Detalle de cita | Revisar y actuar sin perder contexto | Paciente, intervalo, estado, lugar, contacto, acciones, comunicaciones, historia |
| S04 | Crear o reprogramar | Registrar una intención válida conservando datos | Paciente/servicio en alta; intervalo en reprogramación; disponibilidad y resumen |
| S05 | Pacientes | Encontrar o registrar una ficha administrativa | Búsqueda, contactos, completitud, estado y próxima cita |
| S06 | Ficha del paciente | Consultar identidad administrativa y citas | Datos permitidos, contactos, preferencias y citas |
| S07 | Solicitudes | Resolver peticiones del paciente | Solicitud, antigüedad, cita vigente, elección de horario y resultado |
| S08 | Lista de espera | Recuperar huecos con candidatos compatibles | Entradas, preferencias, ofertas, vencimiento y asignación por personal |
| S09 | Comunicaciones | Supervisar entregas, respuestas y problemas | Tipo, fecha, estado, antigüedad, cita y contacto |
| S10 | Indicadores | Comprender resultados administrativos | Periodo, valores, denominadores, definiciones y antigüedad |
| S11 | Sincronización y conflictos | Revisar trabajo no aceptado o divergente | Pendientes, fallos, resultados desconocidos, comparación y recuperación |
| S12 | Configuración | Mantener datos y políticas autorizadas | Cuenta, servicios, agenda, comunicaciones, equipo, dispositivos y preferencias |
| S13 | Diagnóstico | Inspeccionar el entorno de desarrollo | Versiones, adaptadores y estado técnico permitido, sin secretos |
| S14 | Patient Hub | Gestionar una cita existente | Consulta, confirmación, cancelación, solicitud y resultados conocidos |

### 7.1 Ficha normativa de superficie

Cada especificación derivada deberá completar esta ficha:

- ID, nombre y tarea del usuario.
- Usuario, rol y cuenta.
- Entrada, salida y contexto restaurado.
- Datos mostrados y propietario del caso de uso.
- Acciones disponibles y restricciones.
- Componentes y tokens empleados.
- Estado de dominio, sincronización, comunicación y permiso.
- Carga, vacío, ausencia de datos, error, éxito y conflicto.
- Comportamiento offline y recuperación.
- Contenido, responsive, teclado, foco y lector de pantalla.
- Verificaciones y trazabilidad RF.

## 8. Recorridos extremo a extremo

Los recorridos siguientes son obligatorios de especificar antes de implementar las superficies afectadas. No afirman que todos los servicios ya existan.

| Recorrido | Secuencia y resultado observable |
|---|---|
| Inicio habitual | Acceso local permitido → Agenda de hoy → abrir cita → volver a fecha, filtros y selección |
| Primera preparación | Activar dispositivo/cuenta con conexión → revisar datos iniciales → abrir Agenda cuando esté lista |
| Crear cita | Agenda/hueco/paciente → S04 → disponibilidad conocida → resumen → guardado local provisional → aceptación o revisión |
| Registrar paciente desde cita | S04 → registro explícito → volver con paciente seleccionado y resto del formulario intacto |
| Buscar paciente | Directorio o búsqueda global → resultados autorizados → ficha → retorno conservando consulta |
| Reprogramar | S03 → horario actual visible → S04 → nuevo intervalo de igual duración → provisional → aceptación → confirmación anterior invalidada |
| Cancelar | S03 → paciente, hora y consecuencia → confirmación → cancelación local provisional → aceptación y liberación |
| Confirmar por personal | S03 con cita programada y permiso → confirmar → provisional → aceptación con hora del servidor |
| Registrar atención | S03 cuando la transición sea válida → acción explícita → guardado local → aceptación |
| Registrar ausencia | S03 después de inicio + margen → confirmación del personal → guardado local → aceptación; nunca por falta de respuesta |
| Solicitud del Hub | S07 → solicitud y cita vigente → elegir nuevo intervalo → aceptación de cita → conciliación de solicitud; mostrar fallos parciales |
| Recuperar hueco | S08 → candidatos compatibles → oferta → interés → personal solicita reserva → API verifica → resultado |
| Trabajo offline | Datos locales → crear/cambiar → cerrar → abrir sin red → mismos datos y pendientes; fecha sin cobertura no aparece vacía |
| Reintento | Operación temporalmente fallida → conservación de identidad → nuevo intercambio → resultado sin duplicar efectos |
| Resultado desconocido | Respuesta perdida → comprobar la misma operación antes de repetir intención → recuperar resultado |
| Conflicto concurrente | API rechaza → S11 conserva propuesta y estado compartido → elegir alternativa o descartar explícitamente |
| Permiso revocado | Reconexión → comprobación de acceso → bloqueo o revisión según política → conservación de intenciones sin atribuir éxito |
| Configuración | S12 → editar con permiso → mostrar efecto en datos existentes → guardar por el canal autorizado → refrescar vista conocida |
| Acción del paciente | Enlace válido → Hub valida → acción → API acepta → Hub muestra resultado → escritorio recibe versión sin repetir efecto |

### 8.1 Reglas transversales de recorrido

- Toda acción mostrará recepción inmediata o progreso; el resultado remoto se informará por separado.
- Un error conservará los datos y la intención cuando el contrato lo exija.
- Un doble clic no creará una segunda intención.
- Una actualización remota no sobrescribirá silenciosamente un formulario en edición.
- Una acción destructiva mostrará sujeto, alcance, consecuencia y alternativa segura.
- Volver no perderá contexto salvo decisión explícita del usuario.
- Ningún recorrido dependerá exclusivamente de drag-and-drop, hover o atajos de teclado.

## 9. Acceso, preparación y shell

### 9.1 Acceso y primera preparación

La superficie deberá distinguir:

- Sin sesión.
- Acceso remoto en curso en el navegador externo.
- Dispositivo no registrado o no autorizado.
- Cuenta no seleccionada.
- Cuenta seleccionada sin vista local descargada.
- Preparación inicial incompleta.
- Acceso local permitido con últimos permisos válidos.
- Acceso denegado, revocado o inactivo.
- Servicio no disponible y necesidad de conexión.

La activación de un dispositivo, el alta y los cambios de permisos requieren conexión. Una sesión remota vencida no implica borrar datos locales ni pendientes. La política de acceso local, sesiones, PIN o biometría queda **Pendiente** y no se deducirá de una pantalla de acceso.

La preparación deberá mostrar pasos reales completados y solo los datos necesarios: cuenta, zona horaria, servicio, horario y datos administrativos. No se impondrá un tutorial antes de operar. Las acciones de configuración se limitarán al permiso disponible.

### 9.2 Shell de escritorio

El shell contendrá, cuando el espacio lo permita:

- Marca Nightingale propia, sin reutilizar marcas de Attio o Linear.
- Cuenta activa y selector solo cuando exista una elección autorizada.
- Navegación principal agrupada por tarea.
- Barra superior con contexto de superficie.
- Búsqueda global y ayuda.
- Perfil y configuración.
- Contenido principal con ancho y densidad apropiados.
- Acceso a cambios por revisar cuando exista algo que requiera atención.

En una ventana estrecha, la navegación podrá colapsarse en un menú con nombre accesible, foco gestionado y objetivos táctiles suficientes. No se ocultará la cuenta ni el contexto necesario para evitar acciones en el lugar equivocado.

### 9.3 Barra superior y breadcrumbs

Los breadcrumbs reflejarán la jerarquía real, no una cadena técnica de rutas. El elemento actual no se presentará como enlace redundante. La barra superior no acumulará acciones que pertenezcan al contenido.

### 9.4 Ayuda y diagnóstico

La ayuda podrá explicar la tarea actual y enlazar el catálogo de desarrollo cuando corresponda. Diagnóstico quedará fuera de la navegación diaria y no presentará el estado local como prueba de conexión o aceptación remota. No mostrará tokens, credenciales, rutas sensibles ni secretos.

## 10. Agenda y citas

### 10.1 Agenda: Día, Semana y Lista

La Agenda responderá primero “¿qué citas tengo y qué puedo hacer ahora?”.

**Día:**

- Intervalos ordenados por hora.
- Paciente, servicio, estado de cita y acciones contextuales.
- Huecos y bloqueos derivados de disponibilidad calculada, no de una lista filtrada.
- Navegación por fecha, Hoy, anterior/siguiente y filtros autorizados.
- Información de zona horaria y lugar cuando evite ambigüedad.

**Semana:**

- Comparación visual de ocupación con texto y forma, no solo color.
- Acceso al mismo detalle y acciones alternativas por formulario.
- Desplazamiento localizado si no cabe; nunca se ocultarán datos esenciales sin alternativa.
- Arrastrar, si se implementa posteriormente, no confirmará un cambio por sí solo.

**Lista:**

- Búsqueda y periodo para próximas citas e historial.
- Filtros por profesional, servicio y estado cuando estén autorizados y sean relevantes.
- Canceladas y pasadas solo cuando el filtro o contexto las incluya.
- Paginación o “Cargar más” para resultados históricos; no scroll infinito obligatorio.

Los filtros activos serán visibles y tendrán una acción clara para limpiarlos. Un filtro sin resultados no equivale a disponibilidad libre ni a ausencia de datos descargados.

### 10.2 Detalle de cita

El orden de lectura será:

1. Paciente y estado de cita.
2. Fecha, inicio/fin, servicio y lugar.
3. Sincronización solo cuando requiera atención o sea necesaria para interpretar el resultado.
4. Contacto administrativo permitido.
5. Confirmación o solicitud relacionada.
6. Comunicaciones.
7. Historia aceptada y operaciones locales separadas.

El detalle será lateral cuando quepa con texto legible y ocupará una superficie propia en ventanas estrechas. Conservará el mismo contenido y acciones en ambas presentaciones. Al cerrar un detalle lateral se devolverá el foco al elemento que lo abrió.

Las acciones se ofrecerán por transición válida, rol y momento, no mediante un selector libre de estados:

| Estado | Acciones posibles según contrato y permiso |
|---|---|
| Programada | Confirmar, reprogramar, cancelar, completar cuando haya comenzado, ausencia tras margen |
| Confirmada | Reprogramar, cancelar, completar cuando haya comenzado, ausencia tras margen |
| Cancelada | Estado terminal; consulta e historia |
| Atendida | Estado terminal; consulta e historia |
| Ausencia | Estado terminal; consulta e historia |

La interfaz no habilitará una acción solo porque aparezca en esta tabla: Rust sigue siendo la autoridad local sobre estado, tiempo y permisos.

### 10.3 Crear cita

El formulario se agrupará sin convertirlo en un asistente de varias pantallas cuando quepa:

1. Paciente.
2. Profesional y servicio cuando exista una elección real.
3. Fecha e intervalo.
4. Responsable cuando la ficha lo requiera.
5. Resumen y guardado.

Reglas de presentación:

- Buscar pacientes antes de ofrecer una nueva ficha y distinguir coincidencias con el contacto permitido.
- Registrar un paciente desde el formulario conservará los demás campos.
- La disponibilidad local indicará su cobertura: dentro de ventana descargada, sin ventana o cobertura insuficiente.
- Consultar disponibilidad no reserva.
- El resumen mostrará paciente, servicio, fecha, inicio/fin, profesional y zona cuando sean necesarios.
- Guardar producirá primero un estado local provisional, no una reserva compartida confirmada.

### 10.4 Reprogramar

El contrato actual permite cambiar el intervalo conservando la duración real; no permite cambiar paciente, profesional, servicio, cuenta o responsable mediante la acción de reprogramación. La interfaz no ofrecerá esos campos como si fueran editables en ese recorrido.

Se mostrarán juntos:

- Fecha e intervalo actuales.
- Nueva fecha e intervalo.
- Duración conservada.
- Explicación de que la confirmación anterior quedará invalidada al aceptarse el cambio.

No se afirmará que el horario anterior quedó liberado antes de la aceptación. La propuesta se conservará ante error o conflicto.

### 10.5 Historia y ocupación

La historia aceptada es inmutable y distinta del registro de operaciones locales y de la auditoría técnica. La interfaz los identificará por separado.

Solo la cancelación libera ocupación en el contrato actual. Programada, confirmada, atendida y ausencia conservan ocupación. Reprogramar crea una nueva versión con otro intervalo y estado programada. La UI no calculará estas reglas por su cuenta.

Las citas conservan snapshots de servicio, duración de referencia, profesional, lugar y zona. Los cambios posteriores de catálogo o cuenta no reescribirán visualmente esos datos históricos.

### 10.6 Estados específicos de Agenda

Toda vista de Agenda deberá distinguir:

- Cargando por primera vez.
- Actualizando conservando contenido conocido.
- Día sin citas reales.
- Filtro sin coincidencias.
- Fecha no descargada o cobertura insuficiente.
- Datos desactualizados.
- Error de lectura local.
- Cambios locales pendientes.
- Conflicto que requiere revisión.
- Falta de permiso.

## 11. Pacientes

### 11.1 Directorio y búsqueda

El directorio mostrará solo la información necesaria para encontrar y comparar fichas: nombre, contacto permitido, próxima cita y estado administrativo. No cargará toda la historia de cada paciente en la lista.

La búsqueda aceptará los campos autorizados por el caso de uso. Los resultados distinguirán:

- Coincidencias locales.
- Sin coincidencias en datos locales.
- Directorio realmente vacío.
- Búsqueda no disponible.
- Datos incompletos o no descargados.

Los nombres largos se podrán leer completos mediante wrapping, expansión o detalle; no se truncarán sin alternativa.

### 11.2 Registro y completitud

El contrato distingue `draft` y `complete`, independientemente de `active`/`inactive`. La interfaz no los combinará en un único badge ambiguo.

Una ficha completa deberá cumplir las reglas del contrato para su forma de identificación:

- Adulto o persona con documento propio: nombre, documento declarado/verificado y teléfono propio o responsable vigente.
- Menor sin documento: nombre, nacimiento y responsable vigente con teléfono; la edad se evalúa en la fecha local de la cita.

Los campos condicionales explicarán por qué se solicitan. No se añadirán correo, edad calculada o documentos clínicos por convención visual. Los datos mínimos presentados en el prototipo son históricos y están subordinados al contrato vigente.

Inactivar una ficha no cancela citas. Una edición que deje fichas completas sin requisitos producirá un conflicto de completitud y no cambiará silenciosamente su estado a borrador.

### 11.3 Documentos, teléfonos y contactos

- Los documentos conservarán prefijos, ceros y puntuación significativa; no se formatearán como números.
- País emisor, tipo y número se presentarán con etiquetas explícitas.
- `declared`, `verified` e `invalidated` no se comunicarán solo mediante color.
- Código y número telefónico forman una pareja; uno no puede aparecer sin el otro.
- Un teléfono no prueba identidad ni propiedad.
- Los contactos responsables mostrarán parentesco/función, vigencia, actividad y constancia de representación como datos separados.
- El documento de un adulto no identificará al menor.

### 11.4 Ficha del paciente

La ficha contendrá:

- Identidad administrativa y estado de completitud.
- Contactos y responsables permitidos.
- Preferencias o consentimiento cuando estén definidos.
- Próximas citas e historial paginado.
- Cambios locales pendientes cuando afecten al resultado.

Las acciones principales serán Nueva cita y Editar información administrativa, según permiso. Desactivar o fusionar son operaciones separadas; no existirá un botón genérico Eliminar. Las coincidencias de nombre, teléfono o documento no fusionarán personas ni vincularán fichas entre cuentas.

## 12. Solicitudes y lista de espera

### 12.1 Solicitudes de cambio

La lista mostrará paciente, cita, horario vigente, fecha de recepción y estado conocido de la solicitud. El detalle mantendrá juntos:

- Lo que pide el paciente.
- La cita actual sin modificar.
- La antigüedad y el resultado conocido.
- La acción para elegir un nuevo horario.

La acción principal será Revisar horario o Elegir nuevo horario, no Aceptar con asignación automática. Pedir reprogramación no cambia estado, intervalo ni confirmación de la cita.

La conciliación deberá representar fallos parciales, por ejemplo:

- Cambio de cita aceptado; falta actualizar la solicitud.
- Cambio todavía pendiente de sincronización.
- Resultado remoto desconocido.
- Solicitud desactualizada o ya modificada.

Offline solo se mostrará la última vista conocida con antigüedad. El ciclo completo de rechazo, retirada y conciliación Hub/API queda **Pendiente** y no se resolverá con botones locales.

### 12.2 Lista de espera

Las vistas En espera y Ofertas serán distintas. Las entradas mostrarán servicio, preferencias y estado; las ofertas añadirán intervalo, vencimiento y resultado.

Reglas:

- Solo se ofrecerán horarios compatibles.
- La respuesta del paciente expresa interés, no una reserva.
- El personal autorizado solicita la asignación.
- API vuelve a comprobar oferta y disponibilidad al aceptar.
- La lista no presentará un orden como prioridad si la política no está definida.

La prioridad, el canal de respuesta y los plazos quedan **Pendientes**. La interfaz deberá dejar esos límites visibles sin exponer texto especulativo en el uso normal.

## 13. Comunicaciones e indicadores

### 13.1 Comunicaciones

La superficie responderá “¿qué se envió, qué respondió el paciente y qué necesita seguimiento?”.

Cada fila podrá incluir, según contrato:

- Paciente y cita.
- Tipo de comunicación.
- Canal.
- Fecha prevista y última actualización.
- Estado.
- Resultado de respuesta conocido.
- Antigüedad de la vista local.

Se distinguirán intención local, programado, enviado, entregado, respondido y fallo. Una entrega no confirmará asistencia. Un timeout no autorizará un reenvío ciego porque el resultado externo podría existir.

Las acciones serán Ver cita, revisar contacto y solicitar reintento solo cuando el contrato lo permita. No habrá chat libre. Los mensajes no incluirán información clínica sensible.

### 13.2 Indicadores

Todo indicador mostrará, cuando aplique:

- Nombre comprensible.
- Valor y unidad.
- Numerador y denominador.
- Periodo y zona horaria.
- Fecha de actualización.
- Definición accesible.
- Fuente o alcance de datos cuando ayude a interpretarlo.

No se mezclarán operaciones locales pendientes con hechos aceptados como si fueran una única medición. “Sin datos”, “desactualizado” e “indisponible” serán estados distintos. Una caída analítica no impedirá usar Agenda.

Los gráficos serán sobrios y secundarios al dato. Tendrán etiquetas, escala comprensible, contexto y una tabla o texto equivalente. No se mostrarán porcentajes de mejora sin línea base ni gráficos decorativos.

## 14. Sincronización, configuración y ayuda

### 14.1 Centro de cambios y conflictos

El acceso a sincronización será discreto en funcionamiento normal y visible cuando haya algo que revisar. La superficie podrá mostrar:

- Última sincronización correcta conocida.
- Operaciones pendientes y su alcance.
- Reintentos temporales.
- Resultados desconocidos.
- Rechazos y revisiones.
- Conflictos.
- Cuenta y dispositivo afectados.

El vocabulario principal será orientado al trabajo: “Guardada en este equipo”, “Pendiente de aceptación”, “Resultado por comprobar” o “Necesita revisión”. Outbox, SQL, cursores y UUID podrán aparecer solo en diagnóstico o detalle técnico autorizado.

Un conflicto de cita comparará propuesta local y estado compartido, explicará qué cambió y ofrecerá elegir otra hora o descartar la intención local. No habrá “Forzar guardado” ni “Último cambio gana”.

### 14.2 Cuenta y preferencias

La configuración separará:

- Datos compartidos de la cuenta que requieren conexión y permiso.
- Preferencias del dispositivo que se guardan localmente.
- Capacidades no disponibles o de solo lectura.

La zona IANA, lugar, política de ausencia y versión aceptada serán explícitos cuando eviten ambigüedad. Los cambios remotos mostrarán si la API aceptó pero la vista local aún no se actualizó.

El tema claro es el único definido. Aunque el esqueleto pueda almacenar `system`, `light` o `dark`, este documento no define una paleta oscura ni su comportamiento final; queda **Pendiente** y requerirá un plan específico.

### 14.3 Servicios y ofertas

La configuración distinguirá:

- Catálogo del profesional.
- Oferta del servicio en una cuenta.
- Duración base y ajuste de oferta.
- Versión del servicio y versión de la oferta.
- Activo/inactivo y propuesta local/aceptada.

Cambiar un servicio no modificará citas anteriores. La UI mostrará ese efecto antes de guardar. La duración fija o variable como decisión de experiencia queda **Pendiente**, aunque el contrato siempre use inicio/fin y una duración de referencia.

### 14.4 Horarios, excepciones y bloqueos

Las subpantallas distinguirán:

- Regla semanal con vigencia.
- Excepción de cierre o reemplazo por fecha.
- Bloqueo de cuenta o global.
- Citas existentes afectadas por un cambio.

Las horas se editarán como fechas/horas comprensibles, no como minutos desde medianoche ni RFC 3339 en la interfaz de negocio. Los cambios que dejen citas fuera del horario o dentro de bloqueos serán rechazados por Rust/API y conservarán el formulario.

Los bloqueos globales solo los gestiona el profesional propio. Los asistentes no recibirán el motivo global cuando el contrato lo restrinja; la interfaz no lo inferirá.

### 14.5 Comunicaciones, equipo y dispositivos

Las políticas de recordatorios, cancelación y enlaces mostrarán solo opciones contratadas. Sus plazos concretos quedan **Pendientes**.

Usuarios y permisos indicarán que los cambios requieren conexión y autorización. Cada persona usará su propia cuenta. No se simulará una revocación instantánea en un equipo desconectado.

Dispositivos mostrará equipo, aplicación, último acceso conocido y estado de autorización. Las acciones de revocación o revisión explicarán su alcance y resultado conocido.

## 15. Patient Hub: contrato de experiencia y límite

Patient Hub es una superficie externa para pacientes, no una ruta interna del escritorio. Este documento define continuidad y restricciones; su diseño detallado e implementación pertenecen al repositorio del Hub.

### 15.1 Contenido permitido

Una cita válida podrá mostrar:

- Profesional y especialidad.
- Servicio.
- Fecha, hora y zona.
- Lugar e instrucciones administrativas.
- Estado vigente.
- Acciones permitidas por la política.

No mostrará otras citas, agenda general, datos de otros pacientes, historia clínica ni una reserva pública de horarios.

### 15.2 Recorridos

- Consultar la cita.
- Confirmar asistencia.
- Cancelar dentro de la política, con consecuencia explícita.
- Solicitar otra fecha sin reservarla.
- Consultar el estado actualizado.

Abrir o previsualizar un enlace GET no ejecutará una acción. La cancelación mostrará paciente/cita y explicará que el horario podrá ofrecerse a otra persona cuando se acepte.

### 15.3 Resultados y excepciones

La experiencia cubrirá:

- Acción aceptada.
- Resultado por comprobar.
- Estado cambiado que requiere revisión.
- Enlace vencido o revocado.
- Servicio no disponible.
- Solicitud recibida conservando el horario actual.

No se mostrará información de otra cita al recuperar un enlace. La duración del enlace y la verificación adicional quedan **Pendientes**.

### 15.4 Experiencia móvil

- Diseño desde 320 px de ancho.
- Una columna y lectura sin zoom.
- Objetivos táctiles de 44×44 px como objetivo práctico.
- Etiquetas persistentes y acciones comprensibles sin iconos ambiguos.
- Teclado, foco, contraste y lector de pantalla verificados en el proyecto del Hub.
- Sin registro obligatorio inventado ni instalación de aplicación.

## 16. Fundamentos visuales y tokens

[DESIGN.md](../DESIGN.md) es la fuente de los valores visuales. Este documento define su aplicación y no crea una segunda paleta.

### 16.1 Lenguaje visual

La interfaz transmitirá confianza, calma, claridad, privacidad y precisión. La referencia de oficio visual procede de Attio y Linear como aplicaciones administrativas, sin copiar marca, layouts propietarios ni funciones.

Se utilizarán:

- Canvas y superficies frías off-white, nunca blanco puro.
- Texto azul-gris oscuro en lugar de negro puro.
- Azul controlado para acciones e interactividad.
- Bordes finos y cambios de superficie antes que sombras.
- Radios consistentes y espaciado basado en 8 px.
- Verde solo como excepción semántica de éxito, con texto o contexto.
- Iconos Phosphor consistentes y con propósito.

No se usarán gradientes decorativos, violeta como marca, sombras pesadas, glossy 3D, emojis como iconos ni color en todas las tarjetas/estados.

### 16.2 Roles de tokens

Los planes deberán utilizar roles semánticos, no valores hexadecimales locales:

- Fondo, panel, superficie elevada e interacción.
- Texto primario, secundario, terciario y deshabilitado.
- Acento, hover, pressed, selección y texto sobre acento.
- Borde estándar, sutil, foco y selección.
- Overlay, éxito y foco.

El contraste se comprobará en cada combinación real, incluidos estados seleccionados y superficies tintadas. El azul de texto normal sobre selección azul clara seguirá la restricción de DS y usará el tono más oscuro definido.

### 16.3 Elevación y contención

- Panel = agrupación.
- Borde = contención.
- Sombra = elevación.

No se combinarán los tres al máximo en cada elemento. Las tarjetas no serán el contenedor por defecto: espacio y divisores serán preferibles cuando aporten la misma comprensión.

### 16.4 Color semántico pendiente

Los colores de error, advertencia y estado crítico no están definidos en DS. Mientras no exista una ampliación aprobada, se utilizarán texto, icono, estructura y controles semánticos sin inventar una escala cromática local. Cualquier nueva semántica exigirá contraste, significado y revisión transversal.

## 17. Tipografía, iconos y recursos

### 17.1 Font pairing

- **Anek Latin:** títulos, cabeceras, display y encabezados de tarjetas.
- **Aileron:** cuerpo, navegación, controles, formularios, tablas, captions, enlaces y texto técnico.

No se intercambiarán estos roles para resolver problemas locales. Los pesos compartidos son 400 para lectura, 500 para énfasis de UI y 600 para énfasis fuerte; 300 queda limitado a contenido no crítico.

Los tamaños conservan la escala de DS. Los roles de 10–12 px no se usarán para nombres, horas, instrucciones, acciones ni datos esenciales. Los datos técnicos usarán Aileron y `font-variant-numeric: tabular-nums` cuando la alineación sea útil; no se añadirá otra familia monoespaciada.

No se aplicarán features OpenType de Inter como `cv01` o `ss03`. Solo se activarán features soportadas por los archivos empaquetados y verificadas en render real.

### 17.2 Carga y licencias

Los planes que incorporen fuentes deberán:

- Usar archivos locales con formatos apropiados y fallback de sistema.
- Conservar licencias y atribución de Anek Latin y Aileron.
- Definir `font-display`, pesos y subsets sin depender silenciosamente de un CDN.
- Comprobar caracteres del español, signos, fechas, teléfonos y nombres reales ficticios.
- Evitar síntesis de pesos inexistentes y revisar legibilidad a tamaños pequeños.

### 17.3 Phosphor Icons

Phosphor será el único set de iconos de interfaz:

- Regular como estilo por defecto.
- Tamaño óptico consistente por región.
- Otro peso solo para una jerarquía deliberada y documentada.
- Sin mezcla decorativa de outline, fill y duotone.
- Nombre accesible en acciones icon-only.
- Texto acompañante cuando el significado no sea universal.
- Color heredado del token semántico correspondiente.

La selección de paquete, imports y empaquetado se decidirá en el plan de implementación mediante documentación oficial acorde a la versión instalada; no se presupone una dependencia instalada.

### 17.4 Imágenes y datos de ejemplo

Las imágenes de producto tendrán borde sutil, proporción estable y fondos aprobados. No se usarán fotografías de stock, filtros azules intensos ni imágenes clínicas innecesarias. Todos los ejemplos públicos usarán datos sintéticos sin información personal real.

## 18. Layout, densidad y adaptación

### 18.1 Ritmo y densidad

El ritmo principal será 8/16/24/32 px. Los ajustes ópticos de DS se permitirán solo cuando no destruyan ese ritmo.

La densidad dependerá de la tarea:

- Tablas y registros podrán ser compactos, manteniendo legibilidad y objetivos utilizables.
- Formularios tendrán más separación entre grupos que entre campos relacionados.
- Los datos principales conservarán espacio suficiente para evitar asociaciones erróneas.
- No se dará el mismo espacio a contenido de distinta importancia.

### 18.2 Shell y contenido

El sidebar y topbar tendrán dimensiones estables. El contenido no se limitará automáticamente al ancho de una landing cuando la tarea necesite tablas o comparación; tampoco se expandirá hasta volver ilegibles las líneas de texto.

Los detalles laterales tendrán ancho suficiente para etiquetas y valores. Si la lista y el detalle no pueden coexistir con lectura clara, el detalle pasará a su propia superficie.

### 18.3 Rangos de referencia

Las reglas CSS usarán rangos no solapados aunque la tabla conceptual de DS comparta límites:

| Contexto | Comportamiento esperado |
|---|---|
| Ventana estrecha <600 px | Una columna, navegación colapsada y acciones apiladas cuando sea necesario |
| 600–768 px | Grupos progresivos y campos en una o dos columnas según legibilidad |
| 768–1024 px | Shell completo cuando quepa, grillas ampliadas y detalle condicional |
| 1024–1280 px | Escritorio estándar y navegación completa |
| >1280 px | Más aire periférico, sin estirar texto ni separar valores relacionados |

La aplicación de escritorio se verificará al menos en 800×600, 1024×768 y una ventana grande. El Hub se verificará desde 320 px. Estos tamaños son puntos de prueba, no únicos dispositivos soportados.

### 18.4 Overflow y contenido difícil

- El documento no desarrollará overflow horizontal accidental.
- Tablas y semana podrán tener scroll propio con indicación y acceso por teclado.
- Las etiquetas permanecerán asociadas a sus valores al hacer scroll o cambiar layout.
- Nombres largos, valores ausentes, números grandes y muchas filas serán casos de diseño obligatorios.
- El wrapping será preferible al truncamiento cuando el contenido sea necesario.
- La ampliación de texto al 200 % no eliminará acciones ni información.

## 19. Catálogo de componentes

Este catálogo es conceptual: define patrones disponibles, no obliga a crear archivos o abstracciones vacías. Antes de crear un componente se evaluará elemento nativo, reutilización y cambio mínimo. Las primitivas no contendrán reglas de negocio.

### 19.1 Contrato de cada componente

Toda especificación de componente deberá declarar:

- Nombre, finalidad y cuándo no usarlo.
- Propietario y dependencias permitidas.
- Props/inputs, salidas y contenido.
- Variantes y tamaños justificados.
- Estados default, hover, focus, active, selected, disabled, loading y error cuando apliquen.
- Tokens y geometría.
- Teclado, foco, ARIA, nombre accesible y anuncios.
- Responsive, texto largo y 200 % de zoom.
- Datos ficticios y verificaciones.

### 19.2 Acciones y navegación

| Componente | Reglas esenciales |
|---|---|
| Button primary/ghost/subtle | Una jerarquía clara, estados visibles y disabled semántico |
| IconButton | Nombre accesible, borde/hit area y Phosphor consistente |
| Link | Texto claro, foco y comportamiento de navegación real |
| Tabs | Estado seleccionado semántico, teclado y overflow sin pérdida |
| SegmentedControl | Solo opciones mutuamente excluyentes y selección visible |
| Sidebar/SettingsNav | Grupos por tarea, `aria-current` y contexto estable |
| Topbar/Breadcrumbs | Contexto real sin acciones redundantes |
| Pagination/LoadMore | Posición, total conocido/desconocido y control del usuario |
| Menu | Acciones contextuales, foco gestionado y cierre predecible |
| Tooltip | Ayuda complementaria, nunca información esencial ni única vía hover |

### 19.3 Campos y formularios

| Componente | Reglas esenciales |
|---|---|
| Field/Label/Help | Etiqueta persistente, ayuda asociada y error anunciado |
| Input/Textarea | Fondo/borde DS, placeholder no sustitutivo y wrapping apropiado |
| Select nativo | Preferencia por convenciones de plataforma cuando cubra la tarea |
| Combobox/Search | Entrada flexible, resultados agrupados, teclado y ausencia de coincidencias |
| Checkbox/Radio | Etiqueta clicable, grupo y estado indeterminado solo si aplica |
| Date/TimeInput | Zona explícita, formato local y alternativa a gestos |
| FilterChip | Selección semántica, eliminación/limpieza y texto legible |
| FormActions | Principal/secundaria consistentes, progreso y prevención de doble envío |
| ErrorSummary | Resumen enlazado a campos, foco útil y conservación de valores |
| DirtyFormGuard | Decisión clara antes de perder cambios; no confundir borrador con guardado |

### 19.4 Datos y feedback

| Componente | Reglas esenciales |
|---|---|
| DataTable | Encabezados, filas legibles, comparación, scroll localizado y alternativa móvil |
| ListItem | Contenido escaneable y acciones contextuales sin tarjeta innecesaria |
| Badge/Status | Texto explícito y color secundario; estado de dominio separado de sync |
| Avatar | Iniciales decorativas/ocultas para AT cuando el nombre esté presente |
| DefinitionList/Facts | Etiquetas y valores estables, wrapping y unidades |
| Notice/InlineAlert | Texto, icono, alcance y acción cuando exista |
| EmptyState | Explicar vacío real y siguiente paso sin inventar contenido |
| Loading/Skeleton | Forma próxima al contenido y anuncio sin ruido |
| ErrorState | Causa conocida, conservación y acción de recuperación |
| StaleData | Última actualización y límite de la información |
| Toast | Confirmación breve, `aria-live` apropiado y alternativa persistente si es crítica |

### 19.5 Superficies flotantes y compuestos

| Componente | Reglas esenciales |
|---|---|
| Dialog/AlertDialog | Título, descripción, foco atrapado, retorno y consecuencia |
| Popover | Contenido complementario, cierre y posición sin ocultar foco |
| CommandPalette | Búsqueda por teclado, resultados autorizados y alternativa visible |
| AppointmentSummary | Paciente, servicio, intervalo y zona antes de guardar |
| AppointmentDetail | Composición de S03, no lógica propia de transiciones |
| CalendarWeek | Escala, eventos con texto y alternativa por formulario |
| HistoryTimeline | Cronología, actor y diferencia entre aceptado y local |
| ConflictCompare | Propuesta/estado compartido y siguiente paso explícito |
| Metric | Valor, unidad, denominador, periodo y actualización |
| Chart | Etiquetas, contexto, contraste y equivalente tabular |

## 20. Patrones compuestos

### 20.1 Formulario con resumen

El formulario y el resumen permanecerán visibles juntos cuando quepan. En una columna, el resumen aparecerá antes de la acción final sin exigir memoria. La edición actualizará el resumen sin afirmar persistencia.

### 20.2 Registro relacionado sin pérdida

Al registrar un paciente desde una cita o un contacto desde una ficha, se utilizará un subrecorrido explícito. Al terminar, se seleccionará el nuevo registro y se restaurarán campos, foco y contexto. Los cambios remotos no sustituirán silenciosamente ese borrador.

### 20.3 Detalle contextual

La lista conservará su estado al abrir/cerrar detalle. El detalle no será un modal si el trabajo exige comparar con la lista. En pantalla estrecha será una superficie propia con retorno claro.

### 20.4 Confirmación con fricción segura

Las acciones destructivas, costosas o finales mostrarán:

- Qué ocurrirá.
- A quién afecta.
- Qué se conserva.
- Qué se libera o desaparece.
- Si existe Undo o recuperación.
- Una acción segura claramente disponible.

La fricción será proporcional; no se confirmará cada acción trivial.

### 20.5 Filtros y búsqueda

Los filtros se agruparán por tarea, mostrarán selección y permitirán limpiar. Los valores no disponibles no se ofrecerán como decoración. La búsqueda tolerará entradas razonables y mostrará progreso y ausencia de coincidencias sin borrar el texto.

### 20.6 Comparación y conflicto

La comparación será lado a lado cuando quepa y apilada en estrecho, siempre con etiquetas. La propuesta local no se descartará automáticamente. La acción siguiente será explícita y no incluirá forzar sobrescritura.

## 21. Modelo de estados y verdad semántica

### 21.1 Dimensiones independientes

Ninguna entidad tendrá un único “estado” visual que mezcle estas dimensiones:

| Dimensión | Ejemplos | Regla de presentación |
|---|---|---|
| Cita | Programada, confirmada, cancelada, atendida, ausencia | Texto de dominio y acciones válidas |
| Solicitud | Recibida, en revisión, resuelta o estado contratado | Separada de la cita vigente |
| Sincronización | Pendiente, enviando, desconocido, reintento, aceptado, conflicto, rechazo, revisión | Visible cuando afecte al resultado |
| Comunicación | Programada, enviada, entregada, respondida, fallo | No equivale a asistencia |
| Permiso/acceso | Activo, solo lectura, revocado, inactivo, desconocido | Explica capacidad y siguiente paso |
| Cobertura | Dentro de ventana, sin ventana, insuficiente | Evita interpretar ausencia de datos como vacío |
| Actualidad | Actualizado, desactualizado, fecha desconocida | Muestra última actualización cuando sea útil |
| Formulario | Pristine, dirty, validando, enviando, error | Conserva intención y no implica guardado |
| Vista | Cargando, refrescando, vacía, error | No altera el estado de negocio |

### 21.2 Etiquetas de sincronización

| Contrato local | Texto orientado al usuario |
|---|---|
| pending | Guardada en este equipo · Pendiente de aceptación |
| in_flight | Enviando cambio |
| unknown | Resultado por comprobar |
| retryable | Se reintentará / Pendiente de reintento |
| accepted | Aceptada; actualizando la vista local |
| conflict | Necesita revisión por un cambio compartido |
| rejected | No aceptada; requiere revisión |
| review_required | Requiere revisión |

En funcionamiento normal no se repetirán etiquetas técnicas en cada fila. El detalle y el centro de cambios conservarán la información necesaria.

### 21.3 Matriz mínima de estados por superficie

Cada plan marcará Verificado, No verificado o N/A con evidencia para:

- Primera carga.
- Actualización con contenido previo.
- Vacío real.
- Sin coincidencias.
- Datos no descargados.
- Datos desactualizados.
- Error local.
- Servicio remoto no disponible.
- Offline.
- Éxito local provisional.
- Éxito aceptado.
- Conflicto.
- Rechazo.
- Permiso insuficiente.
- Texto largo y datos extremos.

No se contará N/A como aprobado ni No verificado como funcionando.

## 22. Offline, sincronización y recuperación

### 22.1 Operación local

Rust ejecuta los casos de uso y SQLite conserva los datos autorizados. La interfaz no necesitará HTTP para consultar o modificar datos locales permitidos.

Hoy y mañana disponibles localmente deberán sobrevivir al reinicio. Una fecha no descargada se presentará como falta de datos, no como agenda vacía. La ventana exacta de datos local se definirá en los planes de cada caso de uso.

### 22.2 Guardado provisional

Al guardar:

1. La UI muestra recepción/progreso inmediato.
2. Rust persiste el cambio y su operación en la misma transacción local.
3. La vista presenta el estado provisional correspondiente.
4. La sincronización solicita aceptación sin duplicar la intención.
5. La UI actualiza solo con el resultado conocido.

La aceptación remota y la posterior descarga de la vista son etapas distintas. No se afirmará actualización completa si falta refrescar.

### 22.3 Fallos y reintentos

- Un fallo temporal conserva el trabajo y permite reintento según contrato.
- Una respuesta perdida deja Resultado por comprobar; no autoriza crear otra operación equivalente.
- Un rechazo no elimina la intención ni la atribuye a otro actor.
- Un conflicto muestra estado compartido y propuesta sin elegir automáticamente.
- La recuperación no borra la cola ni presenta datos no recibidos como actuales.

### 22.4 Permisos al reconectar

Los últimos permisos válidos permiten el trabajo local previsto. Los cambios de permisos requieren conexión. API aplica los permisos vigentes al aceptar, no los que tuviera el dispositivo al escribir.

Si una operación se rechaza por falta de permiso, la interfaz conservará la intención para revisión y explicará la restricción conocida. La fecha del cliente no demuestra autorización anterior.

### 22.5 Dependencias externas

La caída de Patient Hub, mensajería o analítica no impedirá operar sobre datos locales. Sus superficies mostrarán última vista conocida, antigüedad y límite de la información. Los workers externos no conocen cambios que aún solo existen desconectados en un equipo.

## 23. Formularios y validación

### 23.1 Estructura

- Etiquetas permanentes encima o junto al control según patrón accesible.
- Ayuda cerca del campo y asociada programáticamente.
- Grupos con `fieldset`/`legend` cuando expresen relación.
- Campos condicionales solo cuando su contexto sea comprensible.
- Acciones en posición consistente y principal única.
- Resumen antes de operaciones con consecuencias.

### 23.2 Entrada flexible y formato

La interfaz aceptará formatos razonables y normalizará mediante reglas definidas, sin ocultar errores. Los identificadores técnicos no se solicitarán en flujos de negocio si existe una selección comprensible.

Las fechas/horas se presentarán en la zona de la cuenta. Los instantes ambiguos o inexistentes por cambios de horario deberán corregirse con una explicación. Los minutos desde medianoche, UUID y RFC 3339 permanecen en contratos/diagnóstico, no como entrada administrativa habitual.

### 23.3 Validación

La validación indicará qué corregir y dónde:

- Error junto al campo cuando sea localizable.
- Resumen cuando haya varios errores o el fallo sea global.
- Foco al resumen o primer campo inválido según el patrón.
- Anuncio mediante `role=alert` o mecanismo apropiado sin duplicar lectura.
- Valores conservados tras cualquier error.

Rust/API siguen validando reglas aunque React ya haya comprobado forma. Un error remoto no se traducirá a un genérico “Error al guardar” cuando exista una explicación segura y útil.

### 23.4 Envío y duplicación

Durante el envío se conservarán los valores y se indicará progreso. Se impedirá una segunda intención accidental sin bloquear la lectura ni el diagnóstico del fallo. El éxito local y el remoto se anunciarán por separado.

### 23.5 Salida y borradores

Descartar no persistirá cambios. Si hay datos modificados, se ofrecerá Seguir editando o Descartar con consecuencia clara. No se añadirá autosave ni persistencia de borradores por defecto; requerirá decisión y contrato específicos.

## 24. Feedback y estados de superficie

| Situación | Presentación requerida |
|---|---|
| Primera carga | Indicador o skeleton relacionado; no vacío prematuro |
| Actualización | Conservar contenido y mostrar actividad sin borrar contexto |
| Vacío real | Explicar que no hay elementos y ofrecer acción válida |
| Sin coincidencias | Conservar consulta/filtros y permitir limpiarlos |
| No descargado | Explicar cobertura ausente y acción para obtener datos |
| Desactualizado | Mostrar antigüedad y no afirmar estado actual |
| Error local | Mensaje útil, conservación y reintento |
| Servicio no disponible | Trabajo local conservado y alcance de la interrupción |
| Guardado local | Confirmar recepción y provisionalidad |
| Aceptado | Confirmar aceptación y estado de actualización de vista |
| Conflicto | Comparar y ofrecer decisión explícita |
| Permiso insuficiente | Explicar capacidad y alternativa autorizada |
| Acción desconocida | Resultado por comprobar, nunca nuevo intento ciego |

Los toasts serán apropiados para confirmaciones no críticas. Los errores que impiden continuar, los conflictos y los resultados desconocidos tendrán una representación persistente.

## 25. Permisos, privacidad y seguridad

### 25.1 Información mínima

Cada superficie declarará los datos necesarios para su tarea. No se mostrará información de otra cuenta ni se utilizará un identificador conocido para consultar datos no autorizados.

Los resultados de búsqueda, conteos, mensajes de error, historial y diagnóstico también respetarán el alcance del permiso. No se filtrará la existencia de entidades mediante mensajes laterales.

### 25.2 Ocultar, deshabilitar y solo lectura

- Ocultar una acción cuando no tenga sentido para el estado o rol.
- Mantener solo lectura cuando la información sea útil pero no editable.
- Deshabilitar cuando la acción sea reconocible pero temporalmente no disponible, explicando por qué si no es evidente.
- No presentar una acción remota como completada porque el control esté habilitado localmente.

### 25.3 Datos personales y técnicos

La interfaz de negocio mostrará solo datos administrativos necesarios. Los UUID, versiones, cursores, códigos y timestamps técnicos se limitarán al detalle útil o diagnóstico. Nunca se mostrarán secretos, tokens ni credenciales SQL.

### 25.4 Confirmaciones sensibles

Las acciones sobre permisos, dispositivos, cuenta, cancelaciones y cambios con impacto compartido mostrarán alcance y consecuencias. La confirmación no sustituirá la autorización en Rust/API.

## 26. Contenido y localización

### 26.1 Lenguaje del producto

La interfaz utilizará español claro, directo y administrativo. Una acción tendrá un único nombre en todo el producto. No se alternarán Eliminar/Borrar, Guardar/Crear o Reprogramar/Cambiar para el mismo resultado.

Los textos explicarán resultado y siguiente paso, evitando jerga técnica, alarmismo y lenguaje clínico fuera de alcance. Los ejemplos deberán ser ficticios y representativos.

### 26.2 Vocabulario de estados

Se mantendrá un glosario común para cita, solicitud, comunicación y sincronización. “Provisional” podrá usarse cuando ayude, pero el texto principal explicará “Guardada en este equipo · Pendiente de aceptación”.

“Confirmada” se reservará para el estado de cita. No se usará para afirmar guardado local, entrega de mensaje o descarga de vista.

### 26.3 Fecha, hora y zona

- Fechas completas cuando el día no sea evidente.
- Inicio y fin con unidad horaria y zona de la cuenta cuando evite ambigüedad.
- Periodos con límites explícitos.
- Última actualización con fecha/hora útil, no solo “hace un momento” si la precisión importa.
- UTC solo en diagnóstico o cuando sea necesario para un contrato técnico.

La fecha civil se calculará con la zona de la cuenta. No se derivará “hoy” del UTC sin zona. Los cambios DST y horas ambiguas se resolverán mediante Rust y se explicarán al usuario.

### 26.4 Datos difíciles y ausentes

Todo componente definirá comportamiento para nombres largos, caracteres acentuados, teléfonos internacionales, valores grandes, cero, ausencia de valor y contenido multilínea. Un valor ausente se mostrará como “No registrado” o equivalente cuando sea útil, nunca como cero.

Los IDs no se truncarán si el usuario debe copiarlos o distinguirlos. Las cifras tabulares se utilizarán en tiempos, métricas y columnas comparables.

## 27. Accesibilidad e interacción inclusiva

El objetivo será **WCAG 2.2 AA**, verificado por superficie. La documentación, la presencia de ARIA o un ratio de color no acreditan conformidad completa.

### 27.1 Semántica y tecnologías de asistencia

- Se utilizarán elementos nativos antes que imitaciones con `div`/`span`.
- Encabezados, landmarks, listas, tablas, campos y botones conservarán una estructura significativa.
- Todo control tendrá nombre, función y estado accesibles.
- Los iconos decorativos se ocultarán de tecnologías de asistencia.
- Los cambios dinámicos se anunciarán solo cuando aporten información, sin duplicidad ni ruido.
- Las tablas conservarán encabezados y relaciones comprensibles.

### 27.2 Teclado y foco

Todo recorrido será operable con teclado:

- Orden de tabulación coherente con la lectura.
- `Tab` y `Shift+Tab` sin trampas fuera de diálogos.
- Flechas y teclas esperadas en tabs, menús, grids y combobox cuando apliquen.
- `Escape` para cerrar superficies transitorias cuando sea seguro.
- Focus visible con el outline DS y offset, no eliminado sin sustituto.
- Foco no oculto por sticky headers, paneles, diálogos o scroll.
- Retorno al elemento de apertura al cerrar detalle/diálogo.
- Foco dirigido a error, contenido nuevo o tarea siguiente solo cuando ayude.

Un enlace “Ir al contenido” permitirá saltar la navegación repetida. Los atajos tendrán alternativa visible y no interferirán con edición de texto.

### 27.3 Contraste y presentación

- Texto normal: al menos 4.5:1.
- Texto grande: al menos 3:1.
- Controles, foco y objetos gráficos significativos: requisito no textual correspondiente.
- Ningún estado dependerá solo de color, posición, forma o movimiento.
- Placeholder y disabled no se usarán para información esencial.

Las combinaciones reales se medirán con fondos finales, opacidad y estados, no solo con pares teóricos de DS.

### 27.4 Zoom, reflow y objetivos

- Texto al 200 % sin pérdida de contenido o función.
- Reflow sin overflow global accidental en los anchos objetivo.
- Scroll localizado cuando sea indispensable para tablas/semana, sin bloquear la lectura.
- WCAG AA 24×24 CSS px o excepciones como referencia mínima aplicable.
- 44×44 px como objetivo práctico para controles táctiles principales e icon-only en móvil.
- Separación suficiente para evitar activaciones accidentales.

### 27.5 Movimiento y cognición

Se respetará `prefers-reduced-motion`. No habrá parallax, animación constante, entrada decorativa ni flashes. La información necesaria permanecerá disponible sin depender de memoria, tiempo límite o reconocimiento de iconos ambiguos.

### 27.6 Verificación humana

Cada recorrido principal requerirá revisión con teclado y lector de pantalla en el entorno objetivo cuando se implemente. Las pruebas con médico/asistente y pacientes se registrarán como realizadas o pendientes; no se sustituirán por inspección de código.

## 28. Movimiento, rendimiento percibido y visualización

### 28.1 Movimiento con propósito

La animación solo explicará carga, progreso, movimiento o finalización. Será breve, precisa y desactivada/reducida según preferencia. No se utilizará para hacer parecer completa una operación cuyo resultado aún se desconoce.

### 28.2 Feedback y rendimiento

Toda acción tendrá evidencia inmediata de recepción. La espera mostrará spinner, skeleton, progreso o mensaje útil según duración y tarea. El objetivo de latencia se medirá en el entorno real; la presencia de una transición CSS no prueba el umbral de Doherty.

Las actualizaciones conservarán contenido conocido y evitarán parpadeos, saltos de layout y reemplazo completo innecesario. Las lecturas tardías no sobrescribirán una selección o respuesta más reciente.

### 28.3 Visualización de datos

- Pocos indicadores con propósito.
- Valores y unidades más prominentes que la decoración.
- Escalas y periodos comprensibles.
- Denominadores y definiciones disponibles.
- Color secundario, nunca único canal.
- Tabla o texto equivalente para gráficos.
- Sin inferir normalidad clínica de indicadores administrativos.

## 29. Arquitectura frontend

### 29.1 Stack y límites

El stack confirmado es Tauri 2, React 19, TypeScript, TailwindCSS 4, Vite, Rust y SQLite. Las versiones exactas se comprobarán en `package.json` y lockfiles antes de usar APIs cambiantes.

React deberá:

- Presentar datos y estados recibidos.
- Gestionar formularios y estado visual efímero.
- Invocar comandos mediante clientes tipados.
- Mantener accesibilidad y contexto de navegación.
- Traducir códigos de error seguros a contenido útil.

React no deberá:

- Ejecutar SQL ni acceder a PostgreSQL.
- Decidir disponibilidad, versiones, permisos o transiciones.
- Mantener secretos cloud.
- Duplicar reglas de negocio de Rust.
- Crear una segunda autoridad mutable de entidades sin necesidad.
- Ejecutar directamente servicios web o reglas del Patient Hub.

### 29.2 Organización conceptual

La estructura objetivo de PMD es una guía, no una obligación de crear carpetas vacías:

```text
src/
  app/                 providers, router y shell cuando se necesiten
  components/          primitives, feedback y forms compartidos
  features/            superficies y composición por componente funcional
  routes/              puntos de entrada de superficies
  shared/
    api/               clientes Tauri tipados y DTO de frontera
    types/             tipos verdaderamente compartidos
    testing/           apoyo de pruebas cuando sea necesario
```

La estructura actual concentra paneles en `App.tsx`; es un esqueleto y no una arquitectura visual final. Los planes de migración deberán mantener un recorrido funcional en cada paso.

Dependencias permitidas:

- `routes` puede usar `features` y componentes compartidos.
- `features` puede usar clientes contractuales y primitivas.
- Primitives no depende de features ni dominio.
- Componentes compartidos no importan pantallas concretas.
- No se crea un modelo global de entidades duplicado por conveniencia.

### 29.3 Tipos y frontera IPC

Los DTO de `src/shared/api/` serán la frontera de transporte. Los componentes podrán definir view models de presentación sin alterar la semántica contractual.

Reglas:

- Un comando por caso de uso, no comandos genéricos orientados a tablas.
- Errores estables y seguros; no interpolar objetos arbitrarios en la interfaz.
- Estado aceptado, provisional y operaciones relacionadas permanecen distinguibles.
- Fechas, versiones y permisos se interpretan según Rust/API, no mediante supuestos del navegador.
- La lectura local no necesita red; las acciones remotas se identifican explícitamente.

### 29.4 Estado frontend

Se distinguirán:

| Tipo | Ejemplos | Tratamiento |
|---|---|---|
| Datos locales persistentes | Citas, pacientes, configuración | Se consultan mediante IPC; no copia global innecesaria |
| Resultado aceptado | Versión remota e historia | Se diferencia de propuesta local |
| Operaciones | Pending, unknown, conflict | Se relacionan con entidad y revisión |
| Formulario | Valores, dirty, validación | Local al recorrido salvo necesidad justificada |
| Navegación | Fecha, filtros, selección | Conservación explícita por superficie |
| Visual efímero | Menú, diálogo, hover | No se presenta como estado de negocio |
| Conexión | Offline, sincronizando, fallo | Independiente del estado de cita |

No se añadirá un gestor global por defecto. La elección de routing, fetching, formularios o estado se hará por necesidad y con documentación oficial versionada.

### 29.5 Lecturas concurrentes y actualizaciones

Toda superficie con actualización periódica o múltiples lecturas deberá:

- Cancelar o ignorar respuestas obsoletas.
- Evitar que una respuesta antigua restaure una vista anterior.
- Conservar contenido mientras actualiza.
- No reiniciar formularios por una lectura en segundo plano.
- Limpiar timers y suscripciones al desmontar.
- Identificar cuenta/contexto para no mezclar respuestas tras cambiar de cuenta.

### 29.6 Estilos y tokens

Los tokens se definirán una vez y se consumirán mediante Tailwind/CSS según el patrón adoptado. No se repetirán valores locales para solucionar una pantalla. Las clases utilitarias no sustituirán la semántica HTML.

Los estilos globales establecerán fuente, fondo, color, foco y reducción de movimiento. Los archivos de fuentes, iconos y licencias se ubicarán en planes posteriores sin modificar este documento por rutas accidentales.

### 29.7 Dependencias y documentación

No se presupone disponibilidad de router, Phosphor, Radix, gestores de formularios o librerías de estado. Antes de añadir o usar una dependencia se deberá:

1. Comprobar imports, manifest y lockfile.
2. Evaluar elemento nativo, reutilización y cambio mínimo.
3. Consultar documentación oficial de la versión exacta.
4. Registrar patrón, motivo y consecuencias en el plan.
5. Evitar dependencias para comportamiento trivial.

## 30. Verificación de interfaz

### 30.1 Niveles de evidencia

Cada plan usará:

- **PASS:** comportamiento observado con evidencia.
- **FAIL:** comportamiento comprobado que no cumple.
- **NOT CHECKED:** aplicable pero sin evidencia.
- **N/A:** no aplica; se excluye del total.

N/A y NOT CHECKED no cuentan como aprobados. Un score parcial no demuestra preparación para publicación.

### 30.2 Comprobaciones por capa

| Capa | Evidencia esperada |
|---|---|
| Componente | Variantes, teclado, nombre, foco, estados y contenido difícil |
| Superficie | Tarea completa, contexto, retorno, estados y responsive |
| IPC real | Comandos reales, DTO, errores, provisionalidad y actualización |
| Offline | Datos previos, cambios, reinicio y pendientes |
| Fallo externo | Alcance del fallo, conservación y recuperación |
| Accesibilidad | Teclado, foco, lector de pantalla, contraste, zoom y reflow |
| Visual | Tokens, fuentes, iconos, overflow y viewports |
| Usuario | Recorrido con médico/asistente o paciente; realizado o pendiente |

### 30.3 Matriz de entorno

Escritorio, según soporte del plan:

- 800×600.
- 1024×768.
- Ventana grande.
- Texto al 200 %.
- Windows, macOS y Linux cuando el comportamiento varíe o el plan lo exija.

Patient Hub, en su repositorio:

- Desde 320 px.
- Móvil táctil y teclado.
- Conectividad irregular y resultado desconocido.

### 30.4 Comprobaciones funcionales mínimas

Cuando la superficie lo implemente:

- Crear cita y conservar formulario ante solapamiento.
- Reprogramar comparando intervalos e invalidando confirmación al aceptar.
- Cancelar con confirmación.
- Registrar paciente desde cita sin perder campos.
- Buscar y distinguir ausencia de coincidencias de ausencia de datos.
- Revisar solicitud y fallo parcial.
- Confirmar/cancelar desde Hub y comprobar resultado.
- Offline, reintento, conflicto y recuperación.
- Permisos de solo lectura y rechazo al aceptar.

### 30.5 Herramientas y comandos

Los planes usarán los comandos reales del proyecto cuando modifiquen código: `pnpm build`, `pnpm quality`, pruebas enfocadas existentes y ejecución Tauri/navegador apropiada. No se inventarán comandos de test ni se declararán ejecutados los que no lo fueron.

Los cambios solo documentales usarán `git diff --check`, validación de enlaces y revisión manual; no requerirán compilación.

## 31. Contrato para planes derivados

Todo plan de una superficie, flujo o componente deberá usar `tasks/<plan-id>/plan.md` y `todo.md`, seguir las reglas de IDs y conservar tareas XS–M con checkpoints.

### 31.1 Alcance obligatorio

El plan incluirá:

1. Usuario, tarea y resultado observable.
2. Superficie Sxx, recorrido y componentes afectados.
3. Requisito vigente, propuesta UX y pendientes por separado.
4. Propietario funcional y caso de uso nativo Rust.
5. Datos SQLite y comportamiento tras reinicio.
6. Comandos IPC/DTO y contratos externos que consuma.
7. Qué funciona sin red y qué requiere conexión.
8. Pendientes, conflictos, permisos, versiones y recuperación.
9. Navegación, rutas, entrada/salida y contexto restaurado.
10. Componentes, tokens, fuentes e iconos reutilizados.
11. Matriz de estados de §21 y feedback de §24.
12. Formularios, validación y conservación de datos.
13. Permisos, privacidad y contenido.
14. Responsive, zoom, teclado, foco y lector de pantalla.
15. Leyes UX seleccionadas y decisión concreta que justifican.
16. Trazabilidad RF y secciones de este documento.
17. Archivos probables y dependencias entre tareas.
18. Pruebas enfocadas, build y revisión manual real.
19. Decisiones abiertas y fuera de alcance.

### 31.2 Tarea pequeña verificable

Cada tarea deberá:

- Tener un solo resultado y como máximo tres acceptance checks claros.
- Mantener la aplicación en estado funcional.
- Reutilizar primero y evitar abstracciones anticipadas.
- Declarar archivos y dependencias.
- Incluir pruebas enfocadas, chequeo amplio y recorrido manual.
- Aplicar `$minimal-code` antes de código y `$docs-first-framework-code` para APIs de librerías.
- Actualizar documentación que describa el comportamiento anterior.

### 31.3 Revisión por checkpoint

Los checkpoints comprobarán:

- Build y pruebas aplicables.
- Recorrido real, no solo imports/compilación.
- Offline/fallo/recuperación cuando aplique.
- Teclado, foco, responsive y estados.
- Checklist del plan sincronizado con evidencia.
- Revisión del usuario antes del siguiente bloque cuando el plan lo exija.

### 31.4 Prohibiciones para planes derivados

- No usar el prototipo como sustituto de contratos.
- No crear cloud/web apps en este repositorio.
- No ocultar una decisión pendiente detrás de un componente.
- No implementar un catálogo completo de primitives antes de necesitarlo.
- No migrar toda la UI en una única tarea.
- No marcar completado con pruebas rojas o escenario principal sin verificar.

## 32. Trazabilidad y gobierno

### 32.1 Matriz RF ↔ experiencia

| RF | Superficies y recorridos | Verificación principal |
|---|---|---|
| RF-01 Crear cita | S02, S04, S11 | Disponibilidad, resumen, provisionalidad y conflicto |
| RF-02 Consultar agenda | S02, S03 | Fecha, filtros, vistas, detalle y cobertura |
| RF-03 Evitar doble reserva | S04, S11 | Solapamiento, conservación y conflicto concurrente |
| RF-04 Gestionar cita e historia | S03, S04, S11 | Transiciones, historia aceptada y operaciones locales |
| RF-05 Recordatorios/respuestas | S03, S09 | Separación envío/entrega/respuesta y antigüedad |
| RF-06 Acceso Patient Hub | S07, S14 | Alcance de cita, acción y resultado conocido |
| RF-07 Pendientes de confirmación | S02, S03, S09 | Filtro, texto y seguimiento sin marcar ausencia |
| RF-08 Liberar/reprogramar horario | S03, S04, S11 | Comparación y liberación solo tras aceptación aplicable |
| RF-09 Lista de espera | S08 | Compatibilidad, interés y asignación por personal |
| RF-10 Atención/cancelación/ausencia | S03 | Estado, momento, permiso y acción explícita |
| RF-11 Indicadores | S10 | Periodo, denominador, definición y antigüedad |
| RF-12 Restringir por usuario | S01, S03, S12, S13 | Datos mínimos, solo lectura y acciones por permiso |
| RF-13 Offline y reinicio | S02–S06, S08, S11 | Datos previos, guardado y persistencia |
| RF-14 Compartir sin duplicar/perder | S11 | Idempotencia, reintento, conflicto y recuperación |
| RF-15 Permisos offline/vigentes | S01, S11, S12 | Últimos permisos, rechazo y revisión |
| RF-16 Hub como autoridad de capacidades | S07, S14 | Vista local con antigüedad y resultado remoto separado |

Los planes ampliarán esta matriz con componentes, estados y comandos concretos.

### 32.2 Cambios de diseño

Un cambio transversal de tokens, tipografía, iconos, navegación, semántica de estados o accesibilidad deberá:

- Identificar superficies afectadas.
- Explicar necesidad y consecuencias.
- Actualizar DS/DR o este documento según propiedad.
- Revisar contraste, responsive y estados.
- Registrarse en un plan aprobado cuando afecte a varias partes.

Una decisión de arquitectura que afecte a varios componentes seguirá requiriendo ADR según PMD.

### 32.3 Excepciones

Toda excepción deberá identificar:

- Regla afectada.
- Necesidad.
- Alcance limitado.
- Consecuencias.
- Carácter temporal o permanente.
- Documento que deberá actualizarse.

No se aceptarán excepciones implícitas por urgencia, espacio disponible o preferencia visual.

### 32.4 Código y prototipo como evidencia

El código muestra lo implementado y DP muestra una exploración. Ninguno sustituye la intención aprobada. Cuando diverjan, se corregirá la implementación o se actualizará formalmente la fuente correspondiente.

DP no acredita accesibilidad completa, servicios, autenticación, sincronización ni validación con usuarios. Sus capturas y pruebas históricas se citarán como tales.

## 33. Decisiones abiertas

| Área | Decisión pendiente | Regla mientras permanezca abierta |
|---|---|---|
| Navegación y rutas | Aprobación final de inicio, vistas, orden y rutas | S01–S14 y propuestas no se presentan como definitivas |
| Validación | Recorridos con médico/asistente/pacientes | Registrar pendiente; no afirmar validación |
| Tema | Dark/system y comportamiento de preferencia | Solo tema claro definido |
| Semántica cromática | Error, advertencia y crítico | Texto/estructura; no colores arbitrarios |
| Duración de citas | Experiencia fija o variable | Inicio/fin del contrato; duración de referencia separada |
| Pacientes | Reglas de presentación de datos mínimos | Contrato vigente; no inferir del prototipo |
| Acceso local | Sesiones, bloqueo, política y recuperación | No inventar PIN, biometría ni borrado local |
| Permisos | Acciones y vistas concretas por rol | Solo lectura/datos mínimos según contrato |
| Recordatorios | Cantidad, anticipación y reintentos | No fijar plazos visuales |
| Cancelación/Hub | Plazos, vencimiento y verificación adicional | No inventar política |
| Solicitudes | Ciclo completo y conciliación Hub/API | Mostrar fallo parcial sin simular éxito |
| Lista de espera | Prioridad, canal y plazos | No ordenar como prioridad implícita |
| Ausencia | Configuración y explicación del margen | Acción del personal tras margen vigente |
| Indicadores | Definiciones y actualización final | Mostrar denominadores y alcance conocido |
| Restauración | Fecha entre sesiones | Propuesta de abrir hoy; decisión pendiente |
| Validación de diseño | Recorridos, prototipos y planes visuales posteriores | La aprobación del maestro no sustituye la revisión de cada trabajo derivado |

Cada cierre deberá actualizar este documento, un ADR o el plan propietario correspondiente. No se cerrará una cuestión por el aspecto de una pantalla.

## 34. Resumen normativo

Nightingale es una interfaz administrativa HealthTech para médicos, especialistas y asistentes. Su experiencia prioriza Agenda, claridad, continuidad, lectura densa y acciones explícitas. No incorpora historia clínica ni funciones fuera del alcance.

El lenguaje visual procede de DS: superficies claras, azul controlado, Anek Latin para títulos, Aileron para texto general/técnico y Phosphor Icons. Attio y Linear inspiran precisión administrativa, no identidad ni funcionalidad.

Rust ejecuta los casos de uso nativos y SQLite conserva el trabajo local. React presenta resultados mediante clientes Tauri tipados y no decide reglas de negocio. El estado de cita, solicitud, sincronización, comunicación, permiso, cobertura, formulario y vista permanece separado.

Guardar localmente no equivale a aceptación compartida; solicitar cambio no reserva; entregar mensaje no confirma asistencia; interés en oferta no asigna horario. Los errores conservan intenciones y los conflictos requieren decisiones explícitas.

Toda superficie define tarea, contexto, estados, permisos, offline, contenido, responsive, accesibilidad y verificación. Laws of UX se aplica siempre de forma contextual. Los planes pequeños se crean bajo `tasks/` y no se ejecutan sin aprobación.

Patient Hub conserva su propio repositorio y autoridad de experiencia. Este documento no implementa su web ni sus reglas. Tampoco implementa el escritorio: es la fuente de verdad UI/UX aprobada para derivar trabajo posterior igualmente sujeto a revisión.
