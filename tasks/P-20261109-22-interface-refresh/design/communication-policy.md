# Diseño · Política de comunicaciones (S12)

**Plan:** [../plan.md](../plan.md) · **Tarea:** T35 · **Estado:** Diseño presentado por el proceso del plan; implementación funcional bloqueada (P-20261109-16-communication-settings-ui).
**Fuentes:** maestro de diseño §§14.5 y 15; especificación del Patient Hub; SRS §§9 y 16; contrato API v1 (sin política de comunicaciones: ver «Requisitos de contrato»).

Especificación de diseño con valores de ejemplo. Los valores de la muestra no son configuración real: las opciones quedan fijadas solo por contrato.

## Tarea y usuario

El dueño define cómo se comunica el consultorio con los pacientes: recordatorios, cancelación y los enlaces del Patient Hub. La configuración se guarda en el servicio remoto autorizado, nunca como preferencia local.

## Datos necesarios y propietario

| Dato | Propietario | Notas |
|---|---|---|
| Política de recordatorios (cantidad, anticipación) | Nightingale API / Hub | Solo opciones contratadas |
| Política de cancelación (hasta cuándo, consecuencia) | Nightingale API / Hub | Plazos pendientes de contrato |
| Enlaces del paciente (inclusión, vencimiento, verificación) | Patient Hub | El escritorio no define vencimientos |
| Efecto sobre futuras comunicaciones | Nightingale API | Un cambio no reescribe mensajes ni citas anteriores |

## Recorrido

1. **Lectura:** grupos claros —Recordatorios, Cancelación, Enlaces del paciente— con el valor vigente y su alcance.
2. **Edición:** solo campos contratados; cada opción pendiente se marca como «pendiente de contrato», no como valor editable inventado.
3. **Guardar:** requiere conexión y permiso; se explica el efecto («aplica a comunicaciones futuras») antes de confirmar.
4. **Resultado:** «Cambios guardados» afirma solo persistencia conocida; el estado aceptado se refleja al conocerse.

## Permisos

- Ver y cambiar la política exige el permiso de administración vigente; la API revalida al guardar.
- Sin permiso, la vista es de solo lectura con la limitación explicada junto a la acción.

## Estados

- **Vigente:** valor conocido con su alcance.
- **Pendiente de contrato:** opción identificada pero sin valores aprobados; no es editable.
- **Error de guardado:** se conservan los valores editados y se explica el fallo; reintentar no duplica la operación.
- **Desactualizado:** antigüedad visible junto a la política mostrada.

## Recuperación

- **Respuesta perdida:** se conserva la intención; no se repite con otro identificador.
- **Rechazo:** se explica qué valor no se aceptó y el siguiente paso permitido.
- **Offline:** lectura de la última vista conocida; la edición se explica como acción que requiere conexión.

## Adaptación

- Formulario de columna única (hasta 640 px) con grupos y ayuda breve; a 800×600 se lee el primer grupo completo.
- Valores ausentes y opciones no contratadas tienen presentación definida.

## Requisitos de contrato y pendientes

**Bloqueantes:** contrato de política de comunicaciones (opciones, valores permitidos, efecto, estados) y sus dependencias del Hub. No existe en contrato API v1: este diseño no lo inventa.

**Pendientes explícitos (visibles en la muestra como tales):** plazos y cantidad de recordatorios, política de cancelación, vencimiento de enlaces, verificación adicional y canales disponibles.

## Muestra

Vista `#policy` de [la muestra](../../../design-prototype/interface-refresh/index.html#policy): grupos con controles deshabilitados, valores de ejemplo marcados y opciones «pendiente de contrato». No guarda nada.
