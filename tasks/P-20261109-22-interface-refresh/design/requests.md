# Diseño · Solicitudes del paciente (S07)

**Plan:** [../plan.md](../plan.md) · **Tarea:** T30 · **Estado:** Diseño aprobado por el proceso del plan; implementación funcional bloqueada (P-20261109-11-requests-ui).
**Fuentes:** maestro de diseño §12.1 y §15; `docs/specifications/nightingale-patient-hub.md`; SRS RF-06/RF-16 y regla P05; contrato API v1 (sin endpoints de solicitudes: ver «Requisitos de contrato»).

Este documento es una especificación de diseño. Los datos de la muestra son sintéticos y no habilitan ninguna operación real. Pedir un cambio no reserva horario ni modifica la cita.

## Tarea y usuario

El personal (dueño o asistente) revisa qué pidió el paciente, compara con su cita vigente sin modificarla y elige un nuevo horario cuando corresponde. La acción principal es **Elegir nuevo horario**, no «Aceptar» con asignación automática: la reprogramación sigue el caso nativo de citas y su aceptación por API.

## Datos necesarios y propietario

| Dato | Propietario | Notas de lectura |
|---|---|---|
| Paciente y cita afectada | Nightingale API (citas compartidas) | Vista local con antigüedad; sin barrido de citas por rango |
| Qué pide el paciente (motivo en términos administrativos) | Patient Hub (solicitudes) | El escritorio la consume vía Hub API; no copia su regla |
| Fecha de recepción y estado conocido de la solicitud | Patient Hub | Estado de solicitud separado del estado de la cita |
| Horario vigente, servicio, estado y confirmación de la cita | Nightingale API | Siempre visible en el detalle, sin modificarse |
| Resultado conocido de la última operación | Coordinación sync existente | Sin payloads completos ni códigos en la vista diaria |

## Recorrido

1. **Lista:** paciente, cita (fecha y horario), fecha de recepción y estado conocido. Filtros secundarios por estado; sin columna de sincronización ni contadores.
2. **Detalle junto a la lista** (superficie propia en ventana estrecha): lo que pide el paciente; la cita actual sin modificar; antigüedad de la vista; resultado conocido.
3. **Elegir nuevo horario:** abre el flujo de reprogramación existente, con la cita y la solicitud como contexto. Guardar crea una única intención; la solicitud no cambia por sí sola.
4. **Resultado:** «Cambios guardados» solo afirma persistencia local. La aceptación compartida se refleja al conocerse; hasta entonces no se promete.

## Permisos

- Ver y resolver solicitudes requiere el permiso de gestión de citas vigente; la API vuelve a comprobarlo al aceptar.
- Sin permiso, la superficie no aparece o se explica la limitación junto a la acción; no se deduce por `canManage` ni por la hora del cliente.
- Los últimos permisos válidos permiten consultar la última vista conocida offline.

## Estados

Solicitud y cita muestran estados separados, siempre con texto (no solo color):

- Solicitud: **Por revisar** (recibida, sin decisión), **Cambio elegido** (el personal ya propuso horario; la solicitud aún no refleja el resultado), **Resultado desconocido**, **Desactualizada** (la cita cambió por otro camino), **Resuelta** (con el resultado conocido).
- Cita: los estados administrativos actuales (programada, confirmada, cancelada, atendida, ausente), sin mezclarse con los de la solicitud.
- Fallos parciales explícitos: «Cambio de cita aceptado; falta actualizar la solicitud» explica la intervención sin ofrecer resoluciones no contratadas.

## Recuperación

- **Offline:** última vista conocida con antigüedad («Última actualización: …»); ninguna resolución local se presenta como éxito remoto.
- **Respuesta perdida:** se conserva la intención; no se ofrece repetir con otro identificador; el asunto aparece en contexto y en «Por revisar» si requiere decisión.
- **Rechazo:** se explica qué no pudo completarse y el siguiente paso permitido, conservando los datos.

## Adaptación

- A 800×600 la lista muestra al menos una solicitud o el estado vacío antes que los controles; el detalle pasa a superficie propia como en Agenda Día.
- Nombres largos, datos ausentes («No registrado») y lista vacía («No hay solicitudes pendientes») tienen presentación definida.
- La revisión por teclado mantiene el patrón de fila + detalle aprobado en C02 (foco en la fila al cerrar).

## Requisitos de contrato y pendientes

**Bloqueantes para implementar:** contrato propietario del Hub para solicitudes (listado, detalle, estados); ciclo completo de rechazo, retirada y vencimiento; conciliación Hub/API tras la reprogramación; definición de qué metadatos de solicitud viajan a la vista local. El contrato API v1 actual no define endpoints de solicitudes: este diseño no los inventa.

**Pendientes explícitos (no resueltos por este diseño):** plazo de respuesta del personal, caducidad de solicitudes, notificación al paciente tras la decisión y textos exactos de conciliación.

## Muestra

Vista `#requests` de [la muestra](../../../design-prototype/interface-refresh/index.html#requests): lista con tres solicitudes sintéticas y detalle contextual con la cita vigente sin modificar. Los botones de acción están deshabilitados: representan el recorrido, no operaciones.
