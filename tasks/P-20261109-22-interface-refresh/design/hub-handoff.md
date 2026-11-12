# Diseño · Entrega visual del Patient Hub (S14)

**Plan:** [../plan.md](../plan.md) · **Tarea:** T36 · **Estado:** Diseño presentado por el proceso del plan; handoff documental para el repositorio propietario (P-20261109-19-patient-hub-ux-handoff).
**Fuentes:** maestro de diseño §15; especificación del Patient Hub; SRS RF-06/RF-16; contrato API v1 (el Hub consume Nightingale API; su propio contrato está pendiente).

Esta entrega es **handoff de diseño**: continuidad visual y recorridos conceptuales para el repositorio del Hub. No implementa APIs, enlaces reales ni servidores, y no crea una web funcional dentro del escritorio. Los datos de [hub.html](../../../design-prototype/interface-refresh/hub.html) son sintéticos.

## Tarea y usuario

El paciente con un enlace válido consulta su cita y ejecuta las acciones que la política permite: confirmar asistencia, cancelar dentro de la política y solicitar otro horario sin reservarlo.

## Datos necesarios y propietario

| Dato | Propietario | Notas |
|---|---|---|
| Acceso, política y solicitudes del paciente | Patient Hub | Su backend llama a Nightingale API; no accede a la base operativa |
| Cita compartida (lectura/cambios) | Nightingale API | Transacciones y reglas compartidas |
| Presentación final | Repositorio propietario del Hub | Este documento es entrada, no implementación |

## Recorridos

1. **Consultar la cita:** profesional y especialidad, servicio, fecha, hora y zona, lugar e instrucciones administrativas, estado vigente. Nada más: sin otras citas, sin agenda general, sin datos de otras personas, sin historia clínica.
2. **Confirmar asistencia:** resultado explícito; el estado de la cita se refleja al conocerse.
3. **Cancelar dentro de la política:** la confirmación explica paciente/cita y que el horario podrá ofrecerse a otra persona cuando se acepte la cancelación.
4. **Solicitar otro horario:** la solicitud se recibe conservando el horario actual; pedir un cambio no reserva otra hora.
5. **Estado actualizado:** consulta repetida muestra el estado vigente conocido.

## Permisos y límites

- Un enlace valida el acceso a **una** cita; abrir o previsualizar (GET) no ejecuta ninguna acción.
- Las acciones que cambian la cita requieren la operación autorizada correspondiente; la API comprueba política, versiones e idempotencia.
- El paciente nunca crea citas nuevas ni elige libremente de una agenda pública.

## Estados y excepciones

- **Acción aceptada:** resultado conocido con sus datos (cita, horario, consecuencia).
- **Resultado por comprobar:** se conserva la intención; se impide repetir la acción mientras el resultado se desconoce.
- **Estado cambiado que requiere revisión:** explicación junto al dato y siguiente paso permitido.
- **Enlace vencido o revocado:** sin mostrar datos de la cita; indicar cómo pedir un enlace nuevo.
- **Servicio no disponible:** la cita no cambia; reintento más tarde.
- **Solicitud recibida:** horario actual conservado y resultado conocido de la solicitud.

## Adaptación móvil

- Diseño desde 320 px, una columna, lectura sin zoom.
- Objetivos táctiles de 44×44 px como objetivo práctico.
- Etiquetas persistentes, acciones con texto, sin iconos ambiguos.
- Teclado, foco, contraste y lector de pantalla se verifican en el proyecto propietario del Hub, no aquí.

## Continuidad visual

- Misma paleta clara y azul, Montserrat/Mulish y tono de voz del escritorio (tokens de DESIGN.md).
- Estados y resultados con el mismo lenguaje («tu horario actual se conserva», consecuencias explícitas).
- Los datos administrativos usan el mismo formato de lectura (dirección sin país, teléfonos legibles).

## Requisitos de contrato y pendientes

**Bloqueantes:** contrato del Hub (accesos, política, solicitudes, estados), vencimiento y verificación de enlaces, política de cancelación y propietario de la implementación. Este diseño no los resuelve.

**Pendientes explícitos:** duración del enlace, verificación adicional, canales de contacto del paciente con el consultorio y textos finales de excepciones.

## Muestra

[hub.html](../../../design-prototype/interface-refresh/hub.html): vista móvil de una cita sintética con confirmación, solicitud de cambio, cancelación con consecuencia, enlace vencido y servicio no disponible. Los botones son de ejemplo y no ejecutan acciones.
