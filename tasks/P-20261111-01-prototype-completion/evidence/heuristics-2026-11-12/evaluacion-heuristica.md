# Evaluación heurística del prototipo de Nightingale

Fecha: 2026-11-12. Evaluación de los recorridos y superficies hasta T11 del prototipo original `design-prototype/`. Se señalan por separado dos herramientas visibles aún pendientes en T19: búsqueda y ayuda. No se evalúan como terminadas las tareas posteriores.

Referencia: [Jakob Nielsen, 10 Usability Heuristics for User Interface Design, Nielsen Norman Group](https://www.nngroup.com/articles/ten-usability-heuristics/).

La lista del usuario agrupa dos expresiones que NN/G trata en el criterio 4: «Use Familiar Patterns» y «Stay Consistent». Esta revisión conserva ambas bajo «Patrones y consistencia» e incluye el criterio 5, prevención de errores, presente en la referencia. «Make Actions Reversible» corresponde a control y libertad del usuario.

## Cómo interpretar el resultado

- **Mayor:** impide una tarea necesaria, descarta edición sin advertencia o confirma un resultado que otras pantallas contradicen.
- **Menor:** añade fricción, ambigüedad o falta de orientación, con otra forma de continuar. Las dos carencias de T19 conservan su condición de trabajo pendiente.
- **—:** sin defecto detectado en esa comprobación; no se asigna severidad a un resultado que pasa.
- Cada fila califica una observación en su página, no toda la heurística. Por eso un mismo principio puede pasar en una pantalla y fallar en otra.

**Resultado:** 18 observaciones: 3 fallos mayores, 6 menores (2 previstos en T19) y 9 comprobaciones que pasan. No es un porcentaje de usabilidad ni una certificación del producto.

## Tabla

| Observación | Resultado | Severidad | Detalles | Notas | Página | Heurística |
| --- | --- | --- | --- | --- | --- | --- |
| H01. Preparación confirma datos que no se reflejan. | No pasa | Mayor | Se confirma Consulta UX, 45 min y 08:00–11:00; Cuenta, Servicios y Horarios conservan valores previos. | Unificar lo confirmado con lo que muestran las demás pantallas. | Preparación → Configuración | 1. Estado visible |
| H02. Cerrar Horarios descarta la edición sin aviso. | No pasa | Mayor | Tras cambiar 12:00 a 12:15, cerrar y volver, aparece 12:00. | Proteger cambios sin guardar, como ya hace Citas. | Configuración → Horarios | 5. Prevenir errores |
| H03. No se pueden quitar tramos. | No pasa | Mayor | Se pueden añadir períodos, pero no eliminar uno añadido ni uno existente. | Añadir una salida por tramo sin perder el resto del formulario. | Configuración → Horarios | 3. Control y reversión |
| H04. El formato de 12 horas no se aplica. | No pasa | Menor | Se guarda «12 horas», pero Agenda conserva 14:00 y 15:00. | Aplicar la preferencia a las horas que muestra la interfaz. | Cuenta / Agenda | 4. Patrones y consistencia |
| H05. «Conciliación pendiente» expone lenguaje interno. | No pasa | Menor | La cita ya cambió, pero la etiqueta no aclara qué debe hacer el personal. | Explicar el estado en palabras de la consulta; validar el término con recepción. | Detalle de solicitud r4 | 2. Lenguaje del usuario |
| H06. La solicitud repite la misma explicación. | No pasa | Menor | En r4 aparece el mismo párrafo en «Estado de la solicitud» y «Conciliación». | Conservar una explicación y un siguiente paso claro. | Detalle de solicitud r4 | 8. Eliminar ruido |
| H07. El error de solapamiento orienta poco. | No pasa | Menor | Dice «una cita o un bloqueo», sin señalar la franja ocupada ni cómo corregirla. | Los campos se conservan; falta ofrecer una alternativa o acceso seguro a disponibilidad. | Nueva cita | 9. Errores útiles |
| H08. El buscador anuncia citas, pero devuelve fichas. | No pasa | Menor | La búsqueda por nombre devuelve pacientes; no ofrece resultados de citas. | Pendiente en T19; no se atribuye a una tarea terminada de T01–T11. | Búsqueda global | 7. Principiantes y expertos |
| H09. La ayuda explica la demo, no la tarea. | No pasa | Menor | El botón es visible, pero solo lleva al catálogo de pantallas y al diagnóstico. | Pendiente en T19: ayuda breve sobre la acción que el usuario quiere completar. | Ayuda global | 10. Ayuda fácil de encontrar |
| H10. La cita protege los cambios sin guardar. | Pasa | — | Al salir ofrece seguir editando o descartar; seguir editando conserva 10:30. | Buen patrón para extender a los formularios de configuración. | Reprogramar cita | 5. Prevenir errores |
| H11. Una ficha inactiva se puede reactivar. | Pasa | — | La inactivación explica su efecto; reactivar devuelve la ficha a su estado activo. | Las citas ya registradas siguen visibles; comprobación en memoria del prototipo. | Ficha de paciente | 3. Control y reversión |
| H12. El horario anterior queda disponible para comparar. | Pasa | — | Reprogramar muestra horario actual y nuevo; la solicitud conserva la cita actual. | Reduce la necesidad de recordar fechas al cambiar de tarea. | Reprogramar / Solicitudes | 6. Reconocimiento |
| H13. Los homónimos tienen un dato que los distingue. | Pasa | — | Las dos Ana María Rodríguez muestran teléfonos distintos en el directorio. | La distinción no depende solo del color o las iniciales. | Pacientes | 6. Reconocimiento |
| H14. Cerrar el detalle conserva el contexto. | Pasa | — | Se mantienen fecha y filtro Confirmada; el foco vuelve al enlace de la cita. | Evita tener que reconstruir la vista anterior. | Agenda / Detalle de cita | 4. Patrones y consistencia |
| H15. Hay acceso visible y un atajo de teclado. | Pasa | — | Buscar se abre con el botón o Ctrl+K, y Escape cierra el cuadro. | Pasa para abrir búsqueda de pacientes; el alcance de citas sigue pendiente. | Marco de navegación | 7. Principiantes y expertos |
| H16. El trabajo local no añade avisos de sincronización. | Pasa | — | Con el escenario sin conexión y datos disponibles no aparecen chips de sincronización. | El control de escenarios queda fuera de la navegación del producto. | Agenda | 8. Eliminar ruido |
| H17. El error de horario identifica qué está afectado. | Pasa | — | Indica la cita de Valentina a las 11:30, propone ajustar o reprogramar y conserva la edición. | Es más útil que el mensaje genérico de Nueva cita. | Configuración → Horarios | 9. Errores útiles |
| H18. El fallo de lectura muestra una salida. | Pasa | — | «No pudimos leer la agenda» explica la situación y ofrece «Volver a cargar». | El reintento devuelve la vista con sus datos en la simulación. | Agenda | 1. Estado visible |

## Evidencia y reproducción

El [registro del recorrido](recorrido.json) conserva el texto y los valores de las pantallas observadas. El [CSV](tabla-heuristicas.csv) contiene las mismas siete columnas de la tabla.

### Fallos mayores

**H01 — Confirmación de preparación contradictoria.** Abrir Preparación; usar «Consulta UX», «Consultorio UX», duración 45 min y horario 08:00–11:00; guardar. La confirmación muestra esos datos. Sin recargar, Cuenta mantiene «Consulta de Elena Arias» y Los Olivos; Servicios mantiene 30 min para Consulta general; Horarios mantiene 09:00–12:00 y 14:00–17:00. Nueva cita sí utiliza 45 min, lo que confirma una aplicación parcial e incoherente de esos datos. No se atribuye a una falta de persistencia real: el desacuerdo ocurre en el mismo recorrido y en memoria.

Evidencia: [confirmación](preparacion-guardada.jpg), [Cuenta después](cuenta-tras-preparacion.jpg). Registro: `setup-saved`, `setup-agenda`, `setup-account-after`, `setup-services-after`, `setup-schedule-after`, `new-appointment-layout`.

**H02 — Cierre sin protección de edición.** Editar el horario semanal, cambiar el primer fin de 12:00 a 12:15 y añadir un tramo. Cerrar no pide confirmación ni conserva un borrador. Al abrir, el fin vuelve a 12:00 y los tres tramos vuelven a dos. No se perdieron datos guardados; se descartó trabajo sin guardar, sin aviso.

Registro: `schedule-edit-before-close`, `schedule-close`, `schedule-reopen`.

**H03 — Tramos sin eliminación.** El formulario presenta Inicio/Fin por tramo y permite añadir más. No ofrece quitar un tramo existente o recién añadido. Vaciarlo no equivale a eliminarlo: los campos son obligatorios. Para retirar un añadido accidental hay que cerrar y perder el resto de la edición; tampoco hay una acción para retirar un período ya definido.

Evidencia: [formulario de tramos](tramos.jpg). Se contrastó el marcado de `intervalRows`, los controles de `ruleForm` y las acciones del prototipo; no hay un control alternativo para retirar filas.

### Otros fallos

- **H04:** `hour-format-saved` confirma 12 horas; `hour-format-agenda` conserva 14:00, 14:30 y 15:00.
- **H05–H06:** `request-partial` registra el término «Conciliación pendiente» y el párrafo duplicado. La advertencia de no crear otra reserva es útil y debe conservarse. La terminología final debe contrastarse con el personal de la consulta.
- **H07:** `appointment-overlap` conserva paciente y horas 10:30–11:15; el mensaje no identifica una franja ocupada ni propone un siguiente paso. Se califica como orientación insuficiente, no como ausencia total de validación.
- **H08:** `search-name` devuelve dos fichas para Ana María, sin citas; `search-time` no devuelve coincidencias para 09:00, aunque Agenda muestra ese horario. El código de búsqueda filtra solo nombres de pacientes. Búsqueda de citas y agrupación de resultados están pendientes en T19.
- **H09:** `help` solo describe la muestra y enlaza pantallas/diagnóstico. Ayuda por tarea está prevista en T19. El botón está bien ubicado; lo que falta es contenido útil para resolver tareas.

### Comprobaciones que pasan

- `appointment-draft-guard`, `draft-kept`: salida protegida y edición conservada al seguir.
- `patient-deactivation-confirm`, `patient-deactivated`, `patient-reactivated`: efecto explicado y reactivación disponible. Los siete enlaces a citas existentes permanecen. Desaparece el enlace a crear cita nueva al inactivar, como corresponde; ese enlace no es una cita que se haya perdido.
- `reschedule-comparison`, `request-pending`: datos de comparación disponibles en la propia tarea.
- `patient-homonyms`: teléfonos distintos para fichas con el mismo nombre.
- `context-return`: fecha 2026-10-22, filtro Confirmada y foco en `#/cita/a1` conservados tras cerrar el detalle.
- `shortcut`: Ctrl+K abre búsqueda; Escape la cierra.
- `offline-silent`: sin etiquetas de sincronización en el escenario de trabajo local permitido.
- `schedule-error`: la reducción a 11:30 identifica la cita afectada, indica cómo actuar y conserva lo escrito. [Captura](error-horario.jpg).
- `agenda-read-error`: mensaje de lectura y acción de recuperación comprobados.

## Alcance y límites

Se recorrieron Agenda, detalle, creación y reprogramación, directorio y ficha, Cuenta, Servicios, Horarios, Preparación y Solicitudes, además de búsqueda y ayuda. Se revisó el código que explica los resultados, las reglas actuales de diseño y el estado de tareas. El criterio principal fue la experiencia y el resultado visible de las acciones.

Las acciones se ejecutaron con datos ficticios en la memoria de una pestaña temporal. Se restableció la muestra al terminar. Esta revisión solo añade el informe y su evidencia: no corrige ni cambia la interfaz, no avanza tareas ni acepta checkpoints.

No es una prueba con usuarios, una auditoría completa de accesibilidad, una prueba de todos los tamaños de pantalla ni una validación de Tauri, autenticación, persistencia, permisos o sincronización reales. Los resultados Pasa se limitan a los casos observados. No se midieron tiempos de respuesta ni se usa la existencia de un estado de carga como prueba de latencia.
