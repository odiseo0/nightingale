# Diseño · Indicadores operativos (S10)

**Plan:** [../plan.md](../plan.md) · **Tarea:** T33 · **Estado:** Diseño presentado por el proceso del plan; implementación funcional bloqueada (P-20261109-14-metrics-ui).
**Fuentes:** maestro de diseño §13.2; SRS RF-11; plan maestro §§9.13 y 18; contrato API v1 (sin servicio analítico: ver «Requisitos de contrato»).

Especificación de diseño con datos sintéticos. Los valores de la muestra no son mediciones reales ni producción.

## Tarea y usuario

El dueño (y el asistente si el permiso lo permite) consulta indicadores administrativos de citas —cumplidas, confirmaciones, cancelaciones, ausencias, horarios recuperados— con contexto suficiente para entender qué mide cada valor y hasta cuándo es válido.

## Datos necesarios y propietario

| Dato | Propietario | Notas |
|---|---|---|
| Hechos aceptados y metadatos de cada indicador | Servicio analítico (Nightingale API) | La UI no calcula una segunda versión mezclando pendientes |
| Definición, numerador y denominador | Servicio analítico | Visibles junto al valor |
| Periodo, zona horaria y fecha de actualización | Servicio analítico | Antigüedad explícita |
| Estados de disponibilidad | Servicio analítico | Sin datos / desactualizado / indisponible son distintos |

## Recorrido

1. **Periodo:** selector de periodo (mes/semana) con zona de la cuenta; cambiarlo consulta el servicio analítico.
2. **Lectura:** cada indicador muestra nombre comprensible, valor con unidad, numerador/denominador, periodo, fecha de actualización y definición accesible.
3. **Detalle:** la definición explica el alcance (qué entra y qué no). Un gráfico, cuando exista, es secundario al dato y tiene equivalente tabular.
4. **Confianza:** la antigüedad se muestra siempre; un dato viejo se marca como tal, no se presenta como actual.

## Permisos

- Consultar indicadores requiere permiso de lectura de analítica; sin permiso la sección se oculta o se explica la limitación.
- La caída del servicio analítico no bloquea Agenda; la superficie muestra «indisponible» con su explicación.

## Estados

- **Con datos:** valor, unidad, numerador/denominador, periodo y actualización.
- **Sin datos:** «Sin datos para este periodo» — distinto de cero.
- **Desactualizado:** valor con su última actualización y aviso de antigüedad.
- **Indisponible:** «No se pudieron consultar los indicadores» con enlace de reintento; Agenda accesible.

## Recuperación

- **Error de consulta:** se conserva el periodo elegido y se explica el fallo; reintentar no duplica nada (es lectura).
- **Datos parciales:** cada indicador muestra su propio estado; uno caído no apaga a los demás.

## Adaptación

- Tarjetas sobrias con el texto primero; gráficos solo cuando aportan y con escala y etiquetas.
- Equivalente textual/tabular siempre presente (los propios valores y definiciones).
- A 800×600 las tarjetas pasan a una columna; el orden de lectura se mantiene.

## Requisitos de contrato y pendientes

**Bloqueantes:** servicio/contrato analítico (indicadores, definiciones, numeradores/denominadores, periodos, antigüedad esperada). No existe en contrato API v1: este diseño no lo inventa.

**Pendientes explícitos:** frecuencia de actualización, catálogo final de indicadores, umbrales de antigüedad y política de acceso por rol.

## Muestra

Vista `#metrics` de [la muestra](../../../design-prototype/interface-refresh/index.html#metrics): cuatro indicadores sintéticos con denominadores y definiciones, un ejemplo de cada estado (sin datos, desactualizado, indisponible) y notas de diseño. Sin cálculos reales.
