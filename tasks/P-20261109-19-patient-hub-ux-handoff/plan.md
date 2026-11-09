# P-20261109-19-patient-hub-ux-handoff: Handoff UX del Patient Hub

**Status:** Blocked
**Created:** 2026-11-09
**Updated:** 2026-11-09
**Task list:** [todo.md](todo.md)
**Roadmap:** [P-20261109-01-ux-ui-roadmap](../P-20261109-01-ux-ui-roadmap/plan.md) · R18

## Overview

Preparar el handoff UX de S14 para el repositorio propietario del Patient Hub: consulta, confirmación, cancelación, solicitud y resultados móviles.

**Bloqueo:** el proyecto Hub y su repositorio/contrato deben ser la autoridad. Este plan no crea una web ni implementa Hub dentro de Nightingale desktop.

## Source of Truth

- `docs/design-master-plan.md` §15 y §§21–31.
- `docs/specifications/nightingale-patient-hub.md`.
- `docs/srs.md` RF-06/RF-16.
- `design-prototype/README.md` solo como referencia histórica.

## Architecture Decisions previstas

- El Hub posee acceso, política, solicitudes y presentación final; su backend llama Nightingale API.
- Solo citas existentes y datos administrativos; no agenda pública, nuevas reservas, historia clínica ni datos de otros pacientes.
- Flujos: consulta, confirmación, cancelación con consecuencia, solicitud que conserva horario, aceptado, unknown, enlace vencido/revocado y servicio no disponible.
- GET de apertura no ejecuta acciones. Política de vencimiento/verificación sigue pendiente.
- Diseño móvil desde 320 px, objetivos 44 px, teclado/foco/contraste y lector de pantalla en el proyecto propietario.

## Dependency Order

```text
Repositorio/contrato Hub → T01 especificación UX → T02 resultados/excepciones → C01
C01 → T03 validación móvil en el proyecto propietario → C02
```

## Task List

- [ ] T01: Redactar especificación UX del Hub en su repositorio.
- [ ] T02: Definir resultados/excepciones y continuidad semántica con escritorio.

### C01: Handoff aceptado
- [ ] Revisión del propietario y contrato alineados.

- [ ] T03: Validar recorridos móviles reales en el proyecto Hub.

### C02: Validación externa
- [ ] Confirmación/cancelación/solicitud/errores verificados sin implementación en este repo.

## Risks and Mitigations

| Risk | Mitigation |
|---|---|
| Implementar Hub en desktop | Entregable documental para el repositorio propietario |
| Solicitar cambio como reserva | Mantener horario y revisión del personal |
| Exponer otras citas | Acceso limitado a una cita validada |

## Open Questions

- Vencimiento, verificación adicional, política de cancelación y propietario de implementación.
