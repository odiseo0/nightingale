# P-20261109-14-metrics-ui: Indicadores operativos

**Status:** Blocked
**Created:** 2026-11-09
**Updated:** 2026-11-09
**Task list:** [todo.md](todo.md)
**Roadmap:** [P-20261109-01-ux-ui-roadmap](../P-20261109-01-ux-ui-roadmap/plan.md) · R13

## Overview

Preparar S10 con indicadores de citas, confirmaciones, cancelaciones, ausencias y horarios recuperados, con periodo, denominadores y antigüedad.

**Bloqueo:** falta servicio/contrato analítico y definiciones finales. No se calcularán métricas mezclando operaciones locales pendientes con hechos aceptados.

## Source of Truth

- `docs/design-master-plan.md` §§13.2, 21–24 y 27–33.
- `docs/srs.md` RF-11.
- `docs/master-plan.md` §§9.13 y 18.

## Architecture Decisions previstas

- El servicio analítico entrega hechos aceptados y metadatos; la UI no calcula una segunda versión con pendientes.
- Cada indicador incluye valor/unidad, numerador/denominador, periodo/zona, actualización y definición.
- Sin datos, desactualizado e indisponible son estados distintos; caída analítica no bloquea Agenda.
- Gráficos son secundarios, con escala/etiquetas y alternativa tabular. No porcentajes de mejora sin línea base.
- Datos sintéticos para pruebas visuales se identifican como tales y no se presentan como producción.

## Dependency Order

```text
Contrato analítico → T01 métricas/definiciones → T02 gráfico/tabla → C01
C01 → T03 antigüedad/estados → C02
```

## Task List

- [ ] T01: Implementar periodo e indicadores con denominadores.
- [ ] T02: Implementar visualización y alternativa tabular.

### C01: Datos reales
- [ ] Valores/definiciones/periodo verificados con contrato real.

- [ ] T03: Cubrir sin datos/desactualizado/indisponible y accesibilidad.

### C02: Indicadores
- [ ] Datos, contexto, alternativa tabular y fallo analítico verificados.

## Risks and Mitigations

| Risk | Mitigation |
|---|---|
| Mezclar pendientes con aceptados | Fuente analítica aceptada y alcance explícito |
| Gráfico decorativo/ambiguo | Denominadores, etiquetas y tabla equivalente |
| Confundir sin datos con fallo | Estados separados y enlace a Agenda |

## Open Questions

- Frecuencia ETL, definiciones y antigüedad esperada de cada indicador.
