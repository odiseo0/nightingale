# P-20261109-13-communications-ui: Comunicaciones

**Status:** Blocked
**Created:** 2026-11-09
**Updated:** 2026-11-09
**Task list:** [todo.md](todo.md)
**Roadmap:** [P-20261109-01-ux-ui-roadmap](../P-20261109-01-ux-ui-roadmap/plan.md) · R12

## Overview

Preparar S09 para consultar comunicaciones programadas, enviadas, entregadas, respondidas y fallidas, con antigüedad y seguimiento autorizado.

**Bloqueo:** faltan workers y contrato de comunicaciones/recordatorios. No se creará chat libre ni se inferirá entrega/respuesta desde citas.

## Source of Truth

- `docs/design-master-plan.md` §§8, 13.1, 21–26 y 30–33.
- `docs/srs.md` RF-05/RF-07/RF-16.
- `docs/master-plan.md` §§9.9 y 16.

## Architecture Decisions previstas

- Nightingale API/workers poseen programación, envío y resultados; SQLite conserva última vista recibida con antigüedad.
- La interfaz separa intención local, programado, enviado, entregado, respondido y fallo. Entrega no confirma asistencia.
- Reintento solo cuando contrato lo permita; timeout/unknown no autoriza reenvío ciego.
- Cada fila enlaza paciente/cita permitidos y tipo/canal/fecha/estado/actualización. No contiene información clínica.
- Offline muestra última vista conocida y no bloquea Agenda.

## Dependency Order

```text
Contrato/workers → T01 lista/historial → T02 detalle/seguimiento → C01
C01 → T03 antigüedad/timeout/offline → C02
```

## Task List

- [ ] T01: Implementar comunicaciones e historial con estados reales.
- [ ] T02: Implementar detalle, contacto y reintento autorizado.

### C01: Seguimiento real
- [ ] Estados y enlaces reales verificados sin chat libre.

- [ ] T03: Cubrir antigüedad, timeout, unknown y offline.

### C02: Comunicaciones
- [ ] Resultados, reintento permitido y recuperación verificados sin duplicar envíos.

## Risks and Mitigations

| Risk | Mitigation |
|---|---|
| Confundir entrega con asistencia | Estados de comunicación y cita separados |
| Reenviar tras timeout desconocido | Comprobar resultado antes de reintentar según contrato |
| Mostrar datos actuales obsoletos | Antigüedad y última actualización explícitas |

## Open Questions

- Canales alternativos, plazos, proveedor oficial y reintentos concretos.
