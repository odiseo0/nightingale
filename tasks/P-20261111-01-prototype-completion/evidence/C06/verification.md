# C06: Seguimiento — verificación de T13–T14

Fecha: 2026-11-12. Alcance: revisión funcional del bloque C06 del plan [P-20261111-01](../plan.md). El usuario aprobó C05 el 2026-11-12 («Apruebo C05») y autorizó continuar hasta este checkpoint.

Entorno: prototipo original servido en `http://127.0.0.1:8766` (`python -m http.server 8766 --bind 127.0.0.1 --directory design-prototype`), navegador integrado de Kimi Code Desktop, datos ficticios en memoria. Sintaxis: `node --check design-prototype/app.js` sin errores; `git diff --check -- design-prototype` sin advertencias de formato.

## T13: Comunicaciones

Recorrido verificado en navegador (ruta `#/comunicaciones`):

1. **Fila/detalle relacionados y sin chat libre.** Seis mensajes de ejemplo con paciente, tipo · canal, cita relacionada (enlace a la cita), fecha de envío y resultado conocido: Entregado ×2, Respondido, Programado, Revisar teléfono y Resultado desconocido. El subtítulo declara «No hay mensajería libre».
2. **Ningún estado cambia la asistencia.** Cada detalle repite «Las comunicaciones no cambian por sí solas el estado de la cita»; el «Respondido» aclara que el estado de la cita lo cambia el personal al confirmarla, no el mensaje.
3. **Filtro con limpieza.** «Filtros» abre el diálogo de estado; «Entregado» deja 2 filas con la nota «Filtrado por «Entregado»» y «Limpiar filtro» restaura las seis.
4. **Fallo y resultado desconocido sin reenvío ciego.** El detalle del fallo de teléfono no ofrece reenvío y propone «Revisar contacto»; el de «Resultado desconocido» (tiempo de espera agotado) ofrece revisar contacto o esperar la actualización, nunca reenviar a ciegas.
5. **Revisión de contacto con retorno.** «Revisar contacto» muestra teléfono registrado y canal preferido con el aviso «Este aviso no reenvía el mensaje»; «Volver al mensaje» devuelve al mismo detalle conservando el contexto.
6. **Detalle de cita.** La sección Comunicaciones de la cita muestra los mensajes reales relacionados (o «Sin comunicaciones registradas.»).
7. **Teclado y tamaños.** Escape cierra el detalle; a 800×600 (emulación táctil) la tabla conserva su scroll horizontal propio sin desbordar el documento; capturas `T13-comunicaciones-800x600.png`.

Capturas: `T13-comunicaciones.png`, `T13-detalle-fallo.png`. Registro estructurado en `T13-checks.json`.

## T14: Indicadores

Recorrido verificado en navegador (ruta `#/indicadores`):

1. **Periodo operable.** El selector abre cinco periodos; «Octubre» conserva las cifras históricas (64 · 90 % · 5 · 4; 48/64 y 8/64); «Semana pasada» muestra un conjunto coherente distinto (22 · 90 % = 18/20; 17/22 · 77 %; 3/22 · 14 %) con tabla equivalente; «Semana en curso» añade el aviso «los datos llegan hasta el 21 de octubre y las cifras pueden crecer al cerrarse».
2. **Unidad, periodo, numerador/denominador, actualización y definición.** Las tarjetas llevan unidad y numerador/denominador («45 atendidas de 50 finalizadas»); la barra lleva periodo, «Actualizado el 21 de octubre, 23:40 · Hora de Caracas»; la tabla de definiciones añade la columna Unidad con el cálculo del ejemplo.
3. **Gráfico con escala y equivalente tabular.** Barras con valor y etiqueta por tramo, nota de escala («normaliza al valor máximo del periodo») y tabla «Tramo del periodo | Citas registradas» con total.
4. **Estados distintos.** «Semana del 26 oct – 1 nov» muestra «Sin datos en este periodo» con retorno a la semana en curso; con el escenario «Servicio temporalmente no disponible» muestra «No pudimos leer el informe» con «Volver a cargar» que avisa sin fingir éxito, mientras Agenda sigue cargando sus datos locales.
5. **Teclado y tamaños.** Escape cierra el diálogo de periodo; a 800×600 las métricas se apilan 2×2 sin desbordar el documento; captura `T14-indicadores-800x600.png`.

Captura: `T14-indicadores.png`. Registro estructurado en `T14-checks.json`.

## Límites y pendientes

- Resultados ficticios en memoria: ninguna entrega, respuesta, reenvío o agregación real se simula; los periodos son conjuntos de ejemplo fijos.
- H01–H09 de la [auditoría heurística](../heuristics-2026-11-12/evaluacion-heuristica.md) quedan sin corregir; este bloque no los marca como resueltos ni amplía el alcance.
- Zoom al 200 % y lector de pantalla: no comprobados en este bloque.
- La política de avisos enlazada desde Comunicaciones corresponde a T17 y no se modificó en este bloque.

## Estado

- T13 y T14: terminadas, criterios de aceptación verificados en navegador.
- C06: pendiente de revisión del usuario; sin avanzar a T15.
