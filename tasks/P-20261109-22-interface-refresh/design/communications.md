# Diseño · Comunicaciones (S09)

**Plan:** [../plan.md](../plan.md) · **Tarea:** T32 · **Estado:** Diseño presentado por el proceso del plan; implementación funcional bloqueada (P-20261109-13-communications-ui).
**Fuentes:** maestro de diseño §13.1; SRS RF-05/RF-07/RF-16; plan maestro §§9.9 y 16; contrato API v1 (sin contrato de comunicaciones: ver «Requisitos de contrato»).

Especificación de diseño con datos sintéticos. Ningún envío, reintento o respuesta de la muestra es real. No hay chat libre.

## Tarea y usuario

El personal responde: «¿qué se envió, qué respondió el paciente y qué necesita seguimiento?». La superficie supervisa recordatorios y mensajes administrativos vinculados a citas, sin información clínica sensible.

## Datos necesarios y propietario

| Dato | Propietario | Notas |
|---|---|---|
| Programación, envío y resultados (entregado, respondido, fallo) | Nightingale API y sus workers | Se ejecutan fuera de las transacciones de sincronización; no bloquean el escritorio |
| Vista local con antigüedad | Coordinación sync existente | Última vista recibida; sin cálculo local de entregas |
| Paciente y cita asociados | Nightingale API | Enlace de lectura autorizado; sin datos de otros pacientes |
| Resultado de respuesta conocido | Nightingale API | Respuesta administrativa (p. ej. confirmación), no contenido clínico |

## Recorrido

1. **Lista/historial:** paciente, cita, tipo, canal, fecha prevista, estado y última actualización. Filtros por estado y tipo con peso secundario.
2. **Detalle:** contenido administrativo permitido, estado completo y resultado de respuesta conocido. Acceso contextual a la cita y a la ficha.
3. **Seguimiento:** solicitar reintento **solo cuando el contrato lo permita**; un timeout o resultado desconocido no autoriza reenvío ciego porque el resultado externo podría existir.
4. **Resultado:** el estado de la comunicación nunca confirma asistencia; la confirmación de la cita sigue siendo un estado de la cita.

## Permisos

- Consultar comunicaciones requiere permiso vigente sobre los datos asociados; se ocultan las fuera de ámbito.
- Reintentar requiere la acción contratada específica; la API revalida antes de enviar.
- Offline: última vista conocida con antigüedad; Agenda sigue funcionando.

## Estados

**Intención local → Programado → Enviado → Entregado → Respondido**, y **Fallo** como terminal conocido hasta que se resuelva. Reglas de presentación:

- Cada estado lleva texto; el color nunca es el único distintivo.
- **Entregado** no implica «confirmó asistencia»: comunicación y cita tienen estados separados.
- **Respondido** muestra el resultado administrativo conocido (p. ej. «Sí asistiré»), con su fecha.
- **Fallo** explica la causa conocida (p. ej. «número no disponible») y la acción permitida.
- **Resultado desconocido** conserva la intención y explica que se está recuperando, sin mostrar proceso técnico.

## Recuperación

- **Timeout/unknown:** sin reenvío automático; el siguiente paso se decide según contrato.
- **Respuesta perdida:** se conserva la intención y se impide duplicar el envío.
- **Caída del servicio de comunicaciones:** la vista lo indica con antigüedad; no bloquea Agenda ni citas.

## Adaptación

- Tabla con el mismo lenguaje de Agenda; a 800×600 se leen al menos dos filas o el vacío antes que los filtros.
- Nombres largos, canales ausentes y estados vacíos tienen presentación definida.

## Requisitos de contrato y pendientes

**Bloqueantes:** contrato de comunicaciones/recordatorios (listado, estados, resultado de respuesta, reintento); workers de envío en Nightingale API; política de canales y contenido permitido. No existen en contrato API v1: este diseño no los inventa.

**Pendientes explícitos:** canales alternativos, plazos de envío, proveedor oficial, número de reintentos y textos de seguimiento.

## Muestra

Vista `#communications` de [la muestra](../../../design-prototype/interface-refresh/index.html#communications): tabla con cuatro comunicaciones sintéticas en estados distintos (programado, entregado, respondido, fallo), notas de diseño y estado vacío de ejemplo. Sin controles operativos.
