# C07: Equipo, dispositivos y políticas — verificación de T15–T16–T17

Fecha: 2026-11-12. Alcance: revisión funcional del bloque C07 del plan [P-20261111-01](../plan.md). El usuario aprobó C06 el 2026-11-12 («Puedes continuar, apruebo C06»).

Entorno: prototipo original servido en `http://127.0.0.1:8766` (`python -m http.server 8766 --bind 127.0.0.1 --directory design-prototype`), navegador integrado de Kimi Code Desktop, datos ficticios en memoria. Sintaxis: `node --check design-prototype/app.js` sin errores; `git diff --check -- design-prototype` sin advertencias de formato.

## T15: Equipo e invitaciones

Recorrido verificado en navegador (ruta `#/configuracion/equipo`):

1. **Identidad, rol y acceso por persona.** Elena Arias (Médica) y Lucía Torres (Asistente) con correo y estado de acceso; la cabecera declara que los roles son los definidos y no hay roles personalizados; el formulario de invitación solo ofrece el rol Asistente.
2. **Invitaciones con estado y límites.** Lista con una pendiente y una vencida. El alta valida el correo con error propio que conserva el formulario y registra la invitación en memoria («no se envía ningún correo real»); la pendiente se retira con confirmación que explica el efecto sobre el enlace; la vencida se reemplaza desde un formulario precargado.
3. **Revisión y retirada de acceso.** El detalle de persona muestra rol, alcance y la salvedad de conexión; la retirada pide confirmación con sujeto, efecto («perderá acceso a la consulta… sus datos se conservan») y la salvedad de equipos desconectados; sin conexión se bloquea con «La persona conserva su acceso hasta entonces».
4. **Vista de asistente.** Sin invitar, sin retirar invitaciones y sin retirar accesos; nota «Solo la persona que administra la consulta puede invitar o retirar accesos».

Captura: `T15-equipo.png`. Registro estructurado en `T15-checks.json`.

## T16: Dispositivos

Recorrido verificado en navegador (ruta `#/configuracion/dispositivos`):

1. **Lista y detalle sin secretos.** Cuatro equipos con sistema, aplicación, último acceso conocido y estado: Actual (este equipo, sin opción de revocar), Autorizado, Por confirmar y Revocado. El detalle muestra autorización con fecha y persona, y declara que no se muestran secretos de sesión.
2. **Revocación con consecuencias y sin promesa inmediata.** Confirmación que identifica el equipo, explica que dejará de sincronizar, que los datos locales no se borran desde aquí y que el cambio se aplica al conectarse; sin conexión se bloquea conservando el estado.
3. **Siguientes pasos válidos.** «Por confirmar» ofrece «Comprobar de nuevo», que confirma la revocación; «Revocado» ofrece «Autorizar de nuevo» con confirmación, que devuelve el equipo a Autorizado.
4. **Vista de asistente.** Sin revocar, comprobar ni autorizar; nota de solo administración.

Capturas: `T16-dispositivos.png`, `T16-dispositivos-800x600.png`. Registro estructurado en `T16-checks.json`.

## T17: Políticas de comunicación

Recorrido verificado en navegador (ruta `#/configuracion/comunicaciones`):

1. **Propósitos separados.** «Recordatorios» (editable: recordatorio, anticipación, enlace en el aviso), «Acciones del paciente» (confirmar/cancelar/pedir cambio; plazo de cancelación pendiente, no fijado) y «Enlaces de cita» (vigencia y verificación no contratadas, mostradas como diseño).
2. **Ejemplos de revisión.** La anticipación lleva «Ejemplo de revisión · sin política aprobada».
3. **Guardar y fallar sin promesas.** Sin conexión, el error se muestra junto al formulario y los valores escritos se conservan (48 horas); como asistente, «Solo la persona que administra la consulta puede cambiar la política…»; el guardado exitoso avisa «No se envían mensajes por este cambio».

Captura: `T17-politica-comunicaciones.png`. Registro estructurado en `T17-checks.json`.

## Límites y pendientes

- Resultados ficticios en memoria: ninguna invitación, revocación, autorización o cambio de política se envía o aplica a un servicio real.
- H01–H09 de la [auditoría heurística](../heuristics-2026-11-12/evaluacion-heuristica.md) quedan sin corregir; este bloque no los marca como resueltos ni amplía el alcance.
- Zoom al 200 % y lector de pantalla: no comprobados en este bloque.

## Estado

- T15, T16 y T17: terminadas, criterios de aceptación verificados en navegador.
- C07: pendiente de revisión del usuario; sin avanzar a T18.
