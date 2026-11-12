# Diseño · Lista de espera y ofertas (S08)

**Plan:** [../plan.md](../plan.md) · **Tarea:** T31 · **Estado:** Diseño presentado por el proceso del plan; implementación funcional bloqueada (P-20261109-12-waitlist-ui).
**Fuentes:** maestro de diseño §12.2; SRS RF-09 y regla P06; decisión de tres proyectos (P06); contrato API v1 (sin dominio de espera: ver «Requisitos de contrato»).

Especificación de diseño con datos sintéticos. Ninguna entrada, oferta o asignación de la muestra opera de verdad. La respuesta del paciente expresa interés, nunca una reserva.

## Tarea y usuario

El personal recupera huecos de agenda ofreciendo intervalos compatibles a pacientes en espera y asignando una cita cuando hay interés. El paciente solo declara interés; la asignación la solicita el personal autorizado y la valida la API.

## Datos necesarios y propietario

| Dato | Propietario | Notas |
|---|---|---|
| Entrada en espera (paciente, servicio, preferencias, alta) | Componente propietario de espera (por definir) | Alta por personal o por contrato futuro; sin segundo modelo en React |
| Intervalo ofrecido y vencimiento | Mismo componente | Solo horarios compatibles con disponibilidad real |
| Respuesta del paciente (interés) | Patient Hub / canal por definir | Interés ≠ reserva; canal de respuesta pendiente |
| Asignación y validación | Nightingale API | Vuelve a comprobar oferta y disponibilidad al aceptar |
| Cita resultante | Caso nativo de citas | Conserva provisionalidad y revisión de conflictos |

## Recorrido

1. **En espera:** entradas con servicio, preferencias y antigüedad. Sin orden que sugiera prioridad.
2. **Ofertas:** intervalo ofrecido, vencimiento y respuesta conocida. Las ofertas vencidas se distinguen de las vigentes.
3. **Asignar:** el personal elige una entrada con interés y un intervalo; se envía una única intención; la API valida. El resultado conocido se muestra sin prometer nada antes de conocerse.
4. **Resultado:** la cita provisional aparece con sus reglas habituales (confirmación, ciclo); un conflicto se explica en contexto.

## Permisos

- Crear entradas, ofertas y asignaciones requiere el permiso de gestión vigente; la API revalida al aceptar.
- El paciente solo puede declarar interés sobre ofertas recibidas; no ve agenda general ni se autoasigna.
- Offline: última vista conocida con antigüedad; no se simula asignación local.

## Estados

- Entrada: **En espera**, **Con oferta vigente**, **Con interés**, **Asignada**, **Retirada** (con resultado conocido).
- Oferta: **Enviada**, **Interés recibido**, **Sin respuesta**, **Vencida**, **Aceptada**, **Rechazada**.
- Fallos parciales: oferta aceptada pero cita aún provisional; interés recibido tras vencimiento (requiere decisión del personal); respuesta desconocida.

## Recuperación

- **Vencimiento:** visible en cada oferta; un interés tardío se muestra con su antigüedad y marca la decisión como pendiente.
- **Respuesta perdida:** se conserva la intención de asignación; no se duplica; si requiere decisión aparece en «Por revisar».
- **Rechazo de la API:** se explican oferta o disponibilidad rechazadas y el siguiente paso permitido, conservando la entrada.

## Adaptación

- Dos vistas separadas («En espera» y «Ofertas») con el mismo lenguaje de tabla de Agenda.
- A 800×600 se leen al menos dos filas o el estado vacío antes que los controles.
- Nombres largos, servicios extensos, ausencia de teléfono y listas vacías tienen presentación definida.

## Requisitos de contrato y pendientes

**Bloqueantes:** dominio y contrato de lista de espera (entradas, ofertas, vencimiento, asignación); canal de respuesta del paciente; integración con disponibilidad. No existen en contrato API v1 ni en el Hub: este diseño no los inventa.

**Pendientes explícitos:** política de prioridad, plazos de vencimiento, retirada de entradas, número máximo de ofertas por hueco y textos de seguimiento al paciente.

## Muestra

Vista `#waitlist` de [la muestra](../../../design-prototype/interface-refresh/index.html#waitlist): dos bloques («En espera» y «Ofertas») con datos sintéticos, notas de diseño y estados vacíos de ejemplo. Sin controles operativos.
