# P-20261111-01-prototype-completion: Completar el prototipo original

**Status:** In progress
**Created:** 2026-11-11
**Updated:** 2026-11-12
**Task list:** [todo.md](todo.md)

## Overview

Completar la experiencia de diseño en `design-prototype/index.html`, conservando la dirección que el usuario eligió: **aburrida, moderna y sencilla**. El resultado será una muestra navegable de las 14 superficies de la primera versión, con sus subrecorridos, estados y ejemplos difíciles. No basta con añadir enlaces o mostrar una confirmación sin completar el recorrido que representa.

Solicitud actual: continuar el prototipo original; tomar de las imágenes de referencia la composición, los elementos y la estética, con contenido propio de Nightingale. OpenDesign no participa. El usuario aprobó este plan el 2026-11-11 y autorizó su desarrollo. Se conservan las revisiones de cada checkpoint.

### Continuidad y límites

- Los planes [P-20261022-01](../P-20261022-01-screen-flows/plan.md) y [P-20261109-22](../P-20261109-22-interface-refresh/plan.md) están Completed. Se enlazan como antecedentes, sin reabrirlos ni cambiar su historia.
- Este trabajo tiene un artefacto independiente: el prototipo original. No modifica `design-prototype/interface-refresh/`, los planes abiertos de producto, React/Tauri/Rust, contratos o especificaciones.
- Las imágenes históricas de `previews/` documentan la composición elegida, pero no prueban el estado actual ni las funciones nuevas.
- Datos ficticios en memoria, sin servicios, persistencia entre recargas, autenticación o mensajería real. No incorporar datos de pacientes.
- La muestra del Hub sigue siendo una entrega visual para su repositorio. No se crea una aplicación web ni un servidor del Hub en este repositorio.
- Sin tema oscuro, clínica con varias sedes, historia clínica, pagos, archivos médicos, chat libre, agenda pública ni nuevas políticas de negocio.

## Source of Truth

1. [Plan maestro](../../docs/master-plan.md), §§4, 9, 14–19 y 22.
2. [Plan maestro de diseño aprobado](../../docs/design-master-plan.md), §§6–15, 18–25 y 32–33. Sus IDs S01–S14 permanecen estables.
3. [DESIGN.md](../../DESIGN.md) y [DESIGN-RULES.md](../../DESIGN-RULES.md), revisión aprobada del 2026-11-09.
4. [SRS](../../docs/srs.md), RF-01–RF-16, reglas y límites.
5. [Arquitectura](../../docs/technical-architecture.md) y [ADR de tres proyectos](../../docs/decisions/transactional-sync-three-projects.md).
6. [Contrato API v1, api-v1.1](../../docs/contracts/api-v1/nightingale-api.md), corte 2026-10-29: referencia para no ofrecer acciones inválidas. No se consume ni modifica.
7. [Prototipo original](../../design-prototype/README.md), `app.js`, `styles.css`, catálogo de 34 vistas y capturas originales.
8. Entregas visuales existentes en `design-prototype/interface-refresh/` y `tasks/P-20261109-22-interface-refresh/design/`: referencia de estados y límites, sin sustituir la composición original ni editar esos archivos.
9. [Planificación](../../.agents/planning-task-breakdown/SKILL.md), [Laws of UX](../../.agents/laws-of-ux/SKILL.md), [cambio mínimo](../../.agents/minimal-code/SKILL.md) y [reglas de tareas](../README.md).

La preferencia actual del usuario confirma la composición del original. Las reglas aprobadas nuevas prevalecen sobre sus ejemplos históricos de campos, tipografía, acciones o estados.

## Inventario: qué existe y qué falta

Inventario inicial basado en revisión estática del código y capturas históricas. La verificación nueva de cada bloque se registra por separado.

| Superficie | Base en las 34 vistas | Ampliación o corrección prevista | Tareas |
|---|---|---|---|
| S01 Acceso/preparación | Acceso ilustrativo y lista inicial ya marcada | Acceso externo en curso, cuenta, dispositivo, datos iniciales, preparación parcial y acceso restringido | T03 |
| S02 Agenda | Día, Semana y Lista | Filtros reales del ejemplo, cobertura, contexto, semana más legible, vacíos distintos | T04 |
| S03 Detalle de cita | Datos y acciones básicas | Historia con fechas/actor, comunicaciones relacionadas, estados terminales y acciones por momento/permiso | T05 |
| S04 Alta/reprogramación | Formulario con resumen y registro relacionado | Reprogramación restringida al intervalo de igual duración, responsable, cobertura y errores que conservan datos | T05 |
| S05 Directorio | Búsqueda/lista y alta simple | Próxima cita, actividad/completitud separadas, identificación y búsqueda local | T06 |
| S06 Ficha | Contacto y tabla de citas | Documentos, responsables y relaciones, edición, historial e inactivación | T06–T07 |
| S07 Solicitudes | Lista/detalle y acceso a horario | Resultado conocido, antigüedad, solicitud modificada y conciliación parcial | T11 |
| S08 Espera | Lista, alta básica y oferta | Preferencias, horarios compatibles y asignación confirmada por el personal; sin ofertas (decisión del usuario 2026-11-12) | T12 |
| S09 Comunicaciones | Lista y detalle breve | Filtros, trazabilidad de entrega/respuesta, revisión de contacto y resultado incierto | T13 |
| S10 Indicadores | Cifras, gráfico y definición parcial | Periodo operable, actualización, equivalentes tabulares y estados de datos | T14 |
| S11 Por revisar | Vacío y conflicto de horario | Acceso condicional, conflictos/rechazos/permisos; retirar colas normales y recuperaciones automáticas | T18 |
| S12 Configuración | Seis secciones con varios cuadros ilustrativos | Cuenta/preferencias, catálogo/ofertas, horarios/excepciones/bloqueos, políticas, equipo/invitaciones y dispositivos | T08–T10, T15–T17 |
| S13 Ayuda/diagnóstico | Ayuda breve y diagnóstico genérico | Ayuda por tarea, búsqueda completa por teclado, datos técnicos aislados y sin secretos | T19 |
| S14 Patient Hub | Ocho etapas visuales | Resultado por comprobar separado de caída de servicio, cita modificada, acceso revocado y política restrictiva | T20 |

Las 14 áreas ya existen. La ampliación se mide por recorridos completos y estados, no por una cantidad arbitraria de páginas. Un menú, formulario o detalle no requiere su propia página si funciona mejor en contexto. T21 registrará el número final de rutas y variantes, sin contar diálogos como páginas ficticias.

### Problemas confirmados que se corregirán primero

- `styles.css` y README todavía definen Inter y pesos históricos; las fuentes nuevas son Montserrat/Mulish.
- `scenarioNotice()` muestra banners por desconexión y reintento automático; `activity()` expone cambios que no requieren una decisión.
- `appointmentEditor()` deja editar paciente, servicio y fin libre durante la reprogramación; el contrato conserva referencias y duración.
- `patientForm()` todavía afirma que los mínimos no se han acordado; el contrato actual distingue borrador/completo y formas de identificación.
- Configuración de horario e invitación usa cuadros ilustrativos; no representa aún todos los pasos de edición.
- `hub/no-disponible` mezcla servicio indisponible con resultado desconocido; se separarán.
- La ficha original repite horas de citas sin fecha en su tabla; al ampliar historia se incluirá el contexto temporal necesario.

## Dirección visual y criterios de aceptación comunes

- Conservar sidebar compacto, tabla abierta, barra de contexto, separadores finos y detalle lateral cuando quepa. Usar espacio antes que tarjetas sin propósito.
- Montserrat en títulos; Mulish en interfaz, tablas, formularios y valores; 400/500/600. Phosphor como único conjunto de iconos, recursos locales con licencia.
- Fondos y bordes aprobados; azul para acción/selección, verde semántico. Etiquetas neutras con texto oscuro. No introducir colores de error o advertencia sin definición aprobada: usar texto, icono y estructura.
- Una acción principal por tarea. Formularios agrupados, etiquetas visibles, secundarios en contexto, filtros activos con limpieza.
- El inicio sigue siendo Agenda · Hoy. No añadir Inicio o Citas como destinos duplicados.
- Sincronización normal invisible. Problemas que necesitan decisión junto al dato y en Por revisar, con acceso condicional. La antigüedad solo aparece donde afecta a la interpretación.
- Guardado local, aceptación compartida, entrega de mensaje y confirmación del paciente son resultados distintos. El ejemplo no presenta navegación o clic como prueba de un servicio.
- Paciente/servicio/responsable y duración se conservan al reprogramar. Los snapshots históricos no cambian al editar catálogo/cuenta.
- Dueño con un profesional: ocultar Profesional; asistentes o varios profesionales: conservarlo. La simulación usa rol y disponibilidad explícitos.
- Direcciones de lectura hasta ciudad/estado. Conservar país editable y emisor de documento.
- Nombres largos, homónimos, teléfonos ausentes, ceros en documento, menores y responsables, horarios diversos y muchas filas son ejemplos obligatorios.
- Teclado completo, Escape y retorno de foco cuando proceda, objetivos táctiles suficientes, errores asociados al campo y cambios sin guardar protegidos.
- Escritorio: 800×600, 1024×768 y 1280×800; Hub desde 320 px; texto al 200 %. Scroll propio en tablas/semana, sin desbordamiento accidental del documento.

### Aplicación concreta de Laws of UX

- Jakob: conservar navegación, tablas, filtros y edición familiar del original; no añadir patrones nuevos solo por estética.
- Proximidad y agrupación: acercar etiqueta/valor y campos relacionados; separar grupos por espacio y separadores.
- Miller: mantener horario actual/propuesto y resumen disponibles juntos; no imponer un máximo de siete controles.
- Hick: mostrar decisiones pertinentes al rol y tarea, sin ocultar opciones necesarias.
- No atribuir tiempos de respuesta o validación con usuarios a una captura o un ejemplo de código.

## Architecture Decisions

| Aspecto | Propietario real y límite de la muestra |
|---|---|
| Casos nativos | Rust: acceso local, pacientes, contactos, catálogo/ofertas, disponibilidad y citas. Este plan solo los representa con datos de ejemplo; no añade comandos Tauri. |
| Datos locales | En producto: SQLite y últimas vistas/operaciones válidas. En esta muestra: memoria JavaScript; recarga restablece datos y no demuestra RF-13. |
| Offline | Explorar datos locales, edición permitida y ausencia de cobertura. No simular persistencia tras reinicio ni cambios de permisos sin red. |
| Aceptación compartida | Nightingale API por HTTPS, versiones/permisos/idempotencia. No hay HTTP ni SQL en los recorridos del prototipo. Estados aceptados son ejemplos elegidos, no resultados reales. |
| Excepciones remotas | Hub posee acceso/política/solicitudes. API posee recordatorios, espera y analítica. Lo no contratado permanece como diseño, sin falsos envíos, reservas ni revocaciones. |
| Contratos | Copia API fija y especificaciones de cada propietario. No cambiar JSON generados, migraciones ni contratos para hacer funcionar una pantalla. |
| Integración del prototipo | Un responsable y trabajo secuencial sobre `app.js` y `styles.css`. Reutilizar funciones y controles; sin nueva biblioteca, conversión a React ni sistema de componentes paralelo. |

Las limitaciones de contrato se explican en documentación y controles de revisión del prototipo. No se llenan las pantallas de uso diario con nombres de módulos, contratos o estados técnicos. Las capacidades no definidas se muestran de solo lectura o mediante escenarios identificados, sin aparentar acciones reales.

## Dependency Order

T01 → T02 → C01 → T03 → T04 → T05 → C02 → T06 → T07 → C03 →
T08 → T09 → T10 → C04 → T11 → T12 → C05 → T13 → T14 → C06 →
T15 → T16 → T17 → C07 → T18 → T19 → T20 → C08 → T21 → C09.

No hay tareas paralelas. Cada tarea deja el prototipo navegable y conserva rutas previas o un retorno equivalente. Antes de editar se lee minimal-code y se sigue el flujo real afectado. Si se introduce una API de navegador o biblioteca, se consulta la documentación oficial pertinente según docs-first; no hace falta cambiar dependencias para este plan.

## Task List

### Base visual y navegación

- [x] T01: Actualizar la tipografía
- [x] T02: Completar el marco de navegación

### C01: Revisión de T01–T02

- [x] Comprobaciones de las tareas registradas sin confundir pruebas de muestra con producto.
- [x] Recorrido y revisión visual del bloque, con límites pendientes explícitos.
- [x] Revisión del usuario: «Apruebo C01; continúa con T03–T05» (2026-11-11).

### Acceso, Agenda y citas

- [x] T03: Completar acceso y preparación
- [x] T04: Completar las vistas de Agenda
- [x] T05: Completar el ciclo de la cita

### C02: Revisión de T03–T04–T05

- [x] Comprobaciones de las tareas registradas sin confundir pruebas de muestra con producto. [Evidencia C02](evidence/C02/verification.md).
- [x] Recorrido y revisión visual del bloque, con límites pendientes explícitos.
- [ ] Revisión del usuario; no avanzar al siguiente bloque sin aceptación.

### Pacientes y responsables

- [ ] T06: Completar el registro de pacientes
- [ ] T07: Completar ficha y responsables

### C03: Revisión de T06–T07

- [x] Comprobaciones de las tareas registradas sin confundir pruebas de muestra con producto.
- [x] Recorrido y revisión visual del bloque, con límites pendientes explícitos.
- [x] Revisión del usuario: preaprobación explícita «Apruebo C03 si necesita aprobación» (objetivo del 2026-11-11); continúa con T08–T10.

### Configuración operativa

- [ ] T08: Completar cuenta y preferencias
- [ ] T09: Completar catálogo y ofertas
- [ ] T10: Completar horarios y bloqueos

### C04: Revisión de T08–T09–T10

- [ ] Comprobaciones de las tareas registradas sin confundir pruebas de muestra con producto.
- [ ] Recorrido y revisión visual del bloque, con límites pendientes explícitos.
- [ ] Revisión del usuario; no avanzar al siguiente bloque sin aceptación.

### Solicitudes y espera

- [x] T11: Completar solicitudes
- [x] T12: Completar lista de espera

### C05: Revisión de T11–T12

- [x] Comprobaciones de las tareas registradas sin confundir pruebas de muestra con producto. [Evidencia C05](evidence/C05/verification.md).
- [x] Recorrido y revisión visual del bloque, con límites pendientes explícitos.
- [x] Revisión del usuario: «Apruebo C05, por favor quiero que continúes hasta el siguiente checkpoint» (2026-11-12); continúa con T13–T14.

### Seguimiento

- [x] T13: Completar comunicaciones
- [x] T14: Completar indicadores

### C06: Revisión de T13–T14

- [x] Comprobaciones de las tareas registradas sin confundir pruebas de muestra con producto. [Evidencia C06](evidence/C06/verification.md).
- [x] Recorrido y revisión visual del bloque, con límites pendientes explícitos.
- [x] Revisión del usuario: «Puedes continuar, apruebo C06» (2026-11-12); continúa con T15–T17.

### Equipo, dispositivos y políticas

- [x] T15: Completar equipo e invitaciones
- [x] T16: Completar dispositivos
- [x] T17: Completar políticas de comunicación

### C07: Revisión de T15–T16–T17

- [x] Comprobaciones de las tareas registradas sin confundir pruebas de muestra con producto. [Evidencia C07](evidence/C07/verification.md).
- [x] Recorrido y revisión visual del bloque, con límites pendientes explícitos.
- [x] Revisión del usuario: «C05, C06 y C07 Aprobados, por favor continúa con T18, T19, T20, T21» (2026-11-12); continúa con T18–T21.

### Recuperación, ayuda y Hub

- [x] T18: Completar asuntos por revisar
- [x] T19: Completar búsqueda y ayuda
- [x] T20: Completar la muestra Patient Hub

### C08: Revisión de T18–T19–T20

- [x] Comprobaciones de las tareas registradas sin confundir pruebas de muestra con producto. [Evidencia C08](evidence/C08/verification.md).
- [x] Recorrido y revisión visual del bloque, con límites pendientes explícitos.
- [ ] Revisión del usuario; no avanzar sin aceptación.

### Revisión completa

- [x] T21: Cerrar cobertura y evidencia

### C09: Revisión de T21

- [x] Comprobaciones de las tareas registradas sin confundir pruebas de muestra con producto. [Evidencia C09](evidence/C09/verification.md).
- [x] Recorrido y revisión visual del bloque, con límites pendientes explícitos.
- [ ] Revisión del usuario.


## Verification

- Sintaxis del JavaScript nativo: `node --check design-prototype/app.js`.
- Revisión de cambios: `git diff --check -- design-prototype tasks/P-20261111-01-prototype-completion`.
- Vista previa local mediante el comando ya documentado: `python -m http.server 8766 --bind 127.0.0.1 --directory design-prototype`, con Python disponible; verificar el entorno antes de iniciarlo.
- Navegador: abrir rutas/variantes desde catálogo, formularios, retorno, teclado, tamaños y zoom. Registrar dimensiones, ruta, acción, resultado y limitaciones.
- Evidencia final: `verification.json`, README y hoja de vistas. No sustituir los 34 PNG históricos ni atribuirles verificaciones nuevas.
- No ejecutar compilación Tauri ni pruebas Rust por una edición aislada del prototipo. No hay un comando automatizado actual para validar toda la muestra: T21 registra comprobaciones reales, sin inventarlo.
- Sin pruebas de servicios reales, autenticación, SQLite, reinicio, entrega o lector de pantalla si no se han realizado. Los puntos no verificados quedan NOT CHECKED.

## Risks and Mitigations

| Riesgo | Impacto | Tratamiento |
|---|---|---|
| Sustituir la estética elegida por otra renovación | Alto | Trabajar sobre el original; comparar capturas antes/después; C01/C02 con usuario. |
| Confundir muestra con producto terminado | Alto | Datos ficticios y alcance visibles en marco de revisión y README; sin llamadas externas. |
| Inventar políticas o funciones bloqueadas | Alto | Escenarios de solo lectura, contratos como límite y decisiones pendientes sin cerrar. |
| Perder trabajo de otros planes o cambios locales | Alto | Editar solo archivos de este plan y original autorizado; conservar interface-refresh y trabajo de producto. |
| Hacer muchas pantallas sin completar las acciones | Medio | Comprobar recorrido, errores y retorno por tarea, además de catálogo final. |
| Interacciones incompatibles al ampliar un archivo compartido | Medio | Una tarea a la vez, revisión de sintaxis y regresión del recorrido afectado. |
| Usar referencias oscuras como cambio de tema | Medio | Transferir composición y controles al tema claro aprobado. |

## Open Questions

Ninguna decisión de negocio bloquea la planificación. Las políticas pendientes en el maestro siguen pendientes y no se fijarán desde el prototipo.

Plan aprobado por el usuario el 2026-11-11: «Me gustaría que por favor, procedas con el desarrollo del prototipo, apruebo el Plan». Queda pendiente la revisión del usuario en cada checkpoint; la aprobación inicial no los cierra.

## Registro

- 2026-11-11: revisión de fuentes, código original y capturas históricas; solicitud adicional del usuario: aburrido, moderno y sencillo. Se crea plan Draft. No se ha editado el prototipo ni ejecutado un recorrido nuevo en navegador.
- 2026-11-11: aprobación expresa del usuario; comienza T01, seguida de T02 hasta C01. Se conserva el resto del alcance y los puntos de revisión.
- 2026-11-11: T01/T02 terminadas. C01 presentado con [evidencia](evidence/C01/verification.md); revisión del usuario pendiente. T03 no iniciada. El objetivo completo sigue abierto.
- 2026-11-11: el usuario aprueba C01 y autoriza T03–T05 en orden. Comienza T03; próxima revisión C02.
- 2026-11-11: T03–T05 terminadas; C02 listo para revisión del usuario con evidencia funcional, capturas y 144 cargas de rutas en tres tamaños. El horario editable completo queda en T08–T10; zoom al 200 % y lector de pantalla no comprobados. T06 no iniciada; el objetivo completo sigue abierto.
- 2026-11-11: el usuario aprueba C02 («Apruebo C02») y pide continuar el prototipo hasta completar las pantallas restantes. Comienza T06, directorio y registro de pacientes conforme a las formas de identificación del contrato.
- 2026-11-11: T06 terminada. Directorio con contacto/actividad/completitud/estado separados; alta de adulto, menor y borrador; pareja telefónica y documentos con prefijos; edición con conflicto de completitud. Evidencia en `evidence/C03/`; el catálogo pasa a 51 rutas. Comienza T07, ficha y responsables.
- 2026-11-11: T07 terminada. Ficha con próximos encuentros e historial paginado; responsables con parentesco, función, vigencia, representación y actividad separados; alta desde ficha y cita; inactivación de vínculo con conflicto de completitud e inactivación de ficha sin cancelar citas. Evidencia en `evidence/C03/`. C03 (T06–T07) listo para revisión del usuario; comienza T08, cuenta y preferencias.
- 2026-11-11: T08 terminada. Cuenta con datos compartidos (lugar, dirección hasta ciudad/estado, zona con alcance) separados de preferencias locales; solo lectura para asistentes; cambios compartidos bloqueados sin conexión; citas históricas conservan su lugar. Evidencia en `evidence/C04/T08-checks.json`. Comienza T09, catálogo y ofertas.
- 2026-11-11: T09 terminada. Catálogo del profesional con duración base separada del ajuste de oferta; activación por niveles con confirmación y alcance explícito; oferta desactivada fuera de la nueva cita y cita previa con servicio y duración conservados. Evidencia en `evidence/C04/T09-checks.json`. Comienza T10, horarios y bloqueos.
- 2026-11-11: T10 terminada. Reglas semanales con vigencia y tramos dinámicos, excepciones de cierre/reemplazo, bloqueos con alcance y motivo, rechazo de cambios que cruzan citas listando afectados con formulario conservado, asistente sin motivos globales, y Agenda/Semana/Nueva cita calculados sobre las ventanas definidas. Evidencia en `evidence/C04/T10-checks.json`. C04 (T08–T10) listo para revisión del usuario; comienza T11, solicitudes.
- 2026-11-11: T11 terminada. Solicitudes con estados (pendiente, resuelta, desactualizada, conciliación pendiente), antigüedad relativa, cita vigente junto a la petición y resolución al guardar el horario sin duplicar cambios. Evidencia reordenada por bloque en `evidence/C04` y `evidence/C05`. Comienza T12, lista de espera.
- 2026-11-11: iteración de diseño solicitada expresamente tras la revisión T01–T11: separadores semanales por hora, detalle de cita en modal y propuesta semántica de color. La petición sustituye el panel lateral del prototipo; la paleta queda pendiente de revisión sin modificar DESIGN.md. Alcance y [evidencia](evidence/design-iteration/verification.md) separados del avance de tareas y de los otros fallos de la revisión.
- 2026-11-11: continuación visual solicitada con nuevas referencias: jerarquía de pesos y grises, iconos de datos e iniciales con fondos variados y estables por persona. Se aplica al prototipo mediante componentes compartidos; no se modifica base de datos ni se aprueba la paleta por inferencia. [Evidencia de tipografía e identidad](evidence/typography-identity/verification.md).

- 2026-11-12: iteración visual solicitada expresamente: desplegables con chevrón, lista separada y hover; jerarquía del modal de cita; calendario y definición de días, vigencia y tramos. Verificación de teclado, reprogramación, validación de horarios y pantallas estrechas en [evidence/controls-calendar](evidence/controls-calendar/verification.md). No avanza T12 ni implica aprobación de checkpoints o paleta.
- 2026-11-12: se incorporan por petición directa las reglas de 45–75 caracteres por línea de lectura, tamaños tipográficos en múltiplos de 4 e interlineado compacto en títulos y abierto en texto pequeño. Aplicadas al prototipo y documentadas en DESIGN.md, DESIGN-RULES.md y sección 17.1 del plan de diseño. [Medidas y capturas](evidence/type-rhythm/verification.md). No avanza T12 ni cierra puntos de revisión.
- 2026-11-12: T12 terminada. Lista de espera con alta y edición de entradas (validación propia que conserva campos), detalle de oferta con intervalo, vencimiento del ejemplo, respuesta y siguiente paso por estado; asignación solo con confirmación del personal, rechazada con entrada conservada cuando el intervalo está ocupado y creada al liberarse, con enlace «Ver cita». Guarda compartida de `dialog()` para sustituir contenido sobre un diálogo abierto (patrón nativo documentado en MDN). Verificación en navegador con teclado y 800×600 en [evidencia C05](evidence/C05/verification.md). C05 (T11–T12) listo para revisión del usuario; H01–H09 de la auditoría heurística siguen sin corregir y no se marca avance de T13.
- 2026-11-12: decisión directa del usuario sobre la revisión de C05: «Quitemos eso de ofertas, los pacientes no son leads.» T12 se revisa: se eliminan la pestaña y el flujo de Ofertas (estado, detalle, interés/vencimiento/respuesta) y la entrada del catálogo; «Ofrecer horario» pasa a mostrar horarios compatibles con asignación directa confirmada por el personal y recheck defensivo que conserva la entrada si el intervalo está ocupado. Se corrige un defecto encontrado en la revisión: las acciones con fecha/hora unidas por «:» truncaban las horas; fecha e intervalo viajan ahora como atributos de datos. Evidencia v1 conservada como histórica; verificación nueva en [evidencia C05](evidence/C05/verification.md). C05 sigue pendiente de aceptación; sin avance de T13. La «oferta» del catálogo de servicios (T09) es un concepto distinto y queda fuera de esta decisión.
- 2026-11-12: el usuario aprueba C05 («Apruebo C05…»). T13–T14 terminadas. Comunicaciones con filtro por estado y limpieza, detalle que relaciona paciente/cita/tipo/canal/envío/resultado, fallo de teléfono y «Resultado desconocido» sin reenvío ciego, «Revisar contacto» con retorno al mensaje conservando contexto y mensajes reales en el detalle de cita. Indicadores con cinco periodos operables y conjuntos de ejemplo coherentes, unidad/numerador/actualización/ definición por dato, gráfico con escala y equivalente tabular, y estados «periodo en curso», «sin datos» y «servicio no disponible» distintos con Agenda utilizable. Verificación en navegador con teclado y 800×600 en [evidencia C06](evidence/C06/verification.md). C06 (T13–T14) listo para revisión del usuario; sin avance de T15.
- 2026-11-12: el usuario aprueba C06 («Puedes continuar, apruebo C06»). T15–T17 terminadas. Equipo con personas (rol definido, sin roles personalizados), invitaciones pendiente/vencida con alta validada, retirada con confirmación de sujeto y efecto, y retirada de acceso bloqueada sin conexión; dispositivos con cuatro estados (Actual/Autorizado/Por confirmar/Revocado), detalle sin secretos, revocación con consecuencias y salvedad offline, y siguientes pasos válidos («Comprobar de nuevo», «Autorizar de nuevo»); políticas separadas en recordatorios, acciones del paciente y enlaces, con «ejemplo de revisión» en lo no aprobado y guardado sin promesa de envío. Asistente en solo lectura en las tres secciones. Verificación en navegador en [evidencia C07](evidence/C07/verification.md). C07 listo para revisión del usuario; sin avance de T18.
- 2026-11-12: el usuario aprueba C05–C07 («C05, C06 y C07 Aprobados, por favor continúa con T18, T19, T20, T21»). T18–T21 terminadas; C08 y C09 presentados juntos. Por revisar derivado del estado real con contador condicional y archivo local; búsqueda agrupada de pacientes y citas con Ctrl+K, consulta restaurada y ayuda por tarea (H08–H09 cerrados); Hub con cita modificada y servicio no disponible separados del resultado por comprobar, verificado a 320 px. Cierre T21: catálogo en 52 rutas sin ofertas, cobertura del despachador 52/52, README actualizado, `verification-2026-11-12.json` con SHA-256 y hoja de vistas `previews/ampliacion-contact-sheet.png`. Zoom al 200 % y lector de pantalla no comprobados. Verificación en [evidencia C08](evidence/C08/verification.md) y [evidencia C09](evidence/C09/verification.md). El plan completo queda implementado; falta la revisión del usuario de C08–C09 y su decisión sobre H01–H07 y la paleta.
