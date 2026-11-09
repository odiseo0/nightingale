# P-20261109-11-requests-ui: Solicitudes del paciente

**Status:** Blocked
**Created:** 2026-11-09
**Updated:** 2026-11-09
**Task list:** [todo.md](todo.md)
**Roadmap:** [P-20261109-01-ux-ui-roadmap](../P-20261109-01-ux-ui-roadmap/plan.md) · R10

## Overview

Preparar S07 para revisar solicitudes del Patient Hub junto a la cita vigente y elegir un nuevo horario sin reserva automática.

**Bloqueo:** falta contrato propietario del Hub, ciclo completo de solicitudes y conciliación Hub/API. También depende de R02. No se crearán datos, comandos ni estados locales ficticios para desbloquear.

## Source of Truth

- `docs/design-master-plan.md` §§8, 12.1, 15, 21–25 y 30–33.
- `docs/srs.md` RF-06/RF-16 y regla P05.
- `docs/specifications/nightingale-patient-hub.md`.
- R02 appointment-reschedule-ui para elección de intervalo.

## Architecture Decisions previstas

- Hub posee accesos, política y solicitudes; Rust consumirá su API y SQLite conservará una vista local con versión/antigüedad.
- Pedir cambio no modifica cita, horario ni confirmación. La acción principal será Elegir nuevo horario mediante R02, no Aceptar automáticamente.
- La conciliación mostrará por separado cambio de cita aceptado, solicitud pendiente de actualizar, fallo parcial y resultado desconocido.
- Offline mostrará última vista conocida con antigüedad; ninguna resolución local se presentará como éxito remoto.
- Se definirán comandos/DTO solo en un plan técnico aparte si el contrato los requiere.

## Dependency Order

```text
Contrato Hub + conciliación + R02 → T01 lectura → T02 elegir horario → C01
C01 → T03 conciliación/fallos parciales → C02
```

## Task List

- [ ] T01: Implementar lista/detalle de solicitudes con cita vigente y antigüedad.
- [ ] T02: Integrar elección de horario con R02 conservando solicitud y contexto.

### C01: Revisión real
- [ ] Contrato/IPC real y recorrido de revisión verificados.

- [ ] T03: Implementar conciliación, fallos parciales y vistas offline.

### C02: Solicitudes
- [ ] Recorrido Hub→escritorio→reprogramación→conciliación verificado sin duplicar efectos.

## Risks and Mitigations

| Risk | Mitigation |
|---|---|
| Reservar al aceptar solicitud | Cambio solo mediante R02 y aceptación API |
| Ocultar fallo parcial | Estados separados de cita/solicitud y resultado conocido |
| Simular Hub en memoria | Plan Blocked hasta contrato propietario real |

## Open Questions

- Rechazo, retirada, vencimiento, permisos y conciliación exacta de solicitudes.
