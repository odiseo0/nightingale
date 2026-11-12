# C05: Solicitudes y espera — verificación de T11–T12

Fecha: 2026-11-12 (revisión). Alcance: revisión funcional del bloque C05 del plan [P-20261111-01](../plan.md).

**Cambio de alcance decidido por el usuario (2026-11-12):** «Quitemos eso de ofertas, los pacientes no son leads.» La pestaña y el flujo de «Ofertas» de la lista de espera se eliminaron y se sustituyeron por una asignación directa del horario compatible, confirmada por el personal. Sustituye a la verificación anterior de T12, cuya evidencia se conserva como histórica (`T12-checks-v1-ofertas.json` y capturas `v1-ofertas-*`). T11 se verificó el 2026-11-11 (`T11-checks.json`) y se contrasta en regresión.

Entorno: prototipo original servido en `http://127.0.0.1:8766` (`python -m http.server 8766 --bind 127.0.0.1 --directory design-prototype`), navegador integrado de Kimi Code Desktop, datos ficticios en memoria. Sintaxis: `node --check design-prototype/app.js` sin errores.

## T12 revisada: lista de espera sin ofertas

Recorrido verificado en navegador (ruta `#/lista-espera`):

1. **Sin conceptos comerciales.** Desaparecen la pestaña «Ofertas», el estado `waitlistOffers`, los estados de interés/vencimiento/rechazo y la entrada del catálogo «Ofertas de horario». La lista muestra paciente, servicio, preferencia, antigüedad y estado, con la nota de que el orden no fija prioridad.
2. **Horarios compatibles.** «Ofrecer horario» abre «Horarios compatibles para…» con intervalos calculados sobre el horario y la ocupación conocida (ejemplo: Control de seguimiento, 45 min → 23 oct 09:30–10:15).
3. **Asignación con confirmación del personal.** «Asignar este horario» pide confirmación («¿Asignar este horario a Valentina Herrera?») con servicio, fecha e intervalo, y el texto «Crea la cita en el ejemplo tras una aceptación ilustrada del personal…». Al confirmar, la entrada pasa a «Asignada» sin acciones y la cita queda registrada con el servicio y la duración de la entrada.
4. **Intervalo ocupado conserva la entrada.** `do-assign-slot` vuelve a comprobar la ocupación antes de crear la cita; si el intervalo ya no está disponible, avisa y conserva la entrada. Como los candidatos ya excluyen intervalos ocupados, esta comprobación es una guarda defensiva (la API real revalidaría); el comportamiento «conserva la entrada» fue verificado en navegador en la iteración anterior y el mismo código se ejerce en cada asignación.
5. **Corrección de un defecto real encontrado en esta revisión.** Las acciones con fecha/hora unidas por «:» truncaban las horas («09 – 30») porque las horas contienen dos puntos; fecha e intervalo viajan ahora como atributos de datos (`data-date`, `data-start`, `data-end`). Verificado: el diálogo de confirmación muestra «23 de octubre · 09:30 – 10:15».
6. **Teclado y tamaños.** Escape cierra los diálogos de candidatos y de confirmación. A 800×600 (emulación táctil) la tabla conserva su scroll horizontal propio sin desbordar el documento; capturas `T12-espera-800x600.png`.

Capturas a tamaño completo: `T12-espera.png`, `T12-horarios-compatibles.png`, `T12-asignar-confirm.png`. Registro estructurado en `T12-checks.json`.

## T11 en regresión

La lista de Solicitudes conserva sus cuatro estados, antigüedad relativa y el pie de pendientes; el código tocado por la revisión se limita a lista de espera y catálogo, y el guarda compartido de `dialog()` sigue ejercitándose sin errores.

## Límites y pendientes

- Resultados ficticios en memoria: ninguna acción contacta con servicios ni reserva horarios reales; la política de prioridad de la lista sigue abierta, como indica la propia pantalla.
- H01–H09 de la [auditoría heurística](../heuristics-2026-11-12/evaluacion-heuristica.md) quedan sin corregir; esta revisión no los marca como resueltos ni amplía el alcance. H08–H09 siguen previstos en T19.
- Zoom al 200 % y lector de pantalla: no comprobados en este bloque.
- La «Oferta de esta consulta» del catálogo de servicios (T09) es un concepto distinto (visibilidad y duración de servicios) y no se ve afectada por esta decisión; queda a criterio del usuario si también desea renombrarla.

## Estado

- T12: terminada en su versión revisada, criterios verificados en navegador.
- C05: pendiente de revisión del usuario; sin avanzar a T13.
