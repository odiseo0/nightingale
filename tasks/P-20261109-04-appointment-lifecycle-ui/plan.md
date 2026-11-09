# P-20261109-04-appointment-lifecycle-ui: Acciones del ciclo de cita

**Status:** Approved — en ejecución (2026-11-09; autorización expresa y continua del usuario)
**Created:** 2026-11-09
**Updated:** 2026-11-09
**Task list:** [todo.md](todo.md)
**Roadmap:** [P-20261109-01-ux-ui-roadmap](../P-20261109-01-ux-ui-roadmap/plan.md) · R03

## Overview

Implementar acciones del personal para confirmar, cancelar, registrar atención y registrar ausencia desde S03, con permisos/momento válidos, fricción proporcional y resultados locales separados de aceptación.

Depende de P06; reprogramación R02 se recomienda antes para completar el ciclo, pero no comparten archivos sin coordinación. No incluye check-in, reapertura de terminales ni ausencia automática.

## Source of Truth

- `docs/design-master-plan.md` §§8, 10.2, 20–27 y 30–31.
- `docs/contracts/api-v1/nightingale-api.md` §5.4.
- `src/features/appointments/AppointmentActions.tsx` y `BookingPanel.tsx`.
- `src/shared/api/appointments.ts`: confirm/cancel/complete/noShow, base y DTO.

## Architecture Decisions

- Rust decide transición, momento, margen y permiso. La UI presenta solo acciones permitidas por estado conocido, sin selector libre de estados.
- Confirmar no muestra diálogo innecesario; cancelar exige paciente/hora/consecuencia y conservación del historial.
- Atención exige haber alcanzado inicio; ausencia exige inicio+margen y confirmación del personal. Motivo opcional no sustituye esas reglas.
- Cada mutación usa base aceptada u operationId, progreso y prevención de doble intención. Resultados son provisionales hasta aceptación.
- Historia aceptada, operaciones locales y comunicación siguen separados. No se afirma liberación/mensaje antes del resultado.
- Se reutilizan diálogos nativos/primitives existentes y Laws of UX para fricción segura, atención selectiva y feedback.

## Dependency Order

```text
P06 → T01 confirmar/cancelar → C01 → T02 atención/ausencia → T03 historia/verificación → C02
```

## Task List

- [x] T01: Implementar Confirmar y Cancelar con consecuencia explícita.

### C01: Acciones iniciales
- [x] `pnpm quality` en verde; transiciones reales, permisos y errores verificados; revisión del usuario: autorización continua.

- [x] T02: Implementar Registrar atención y Registrar ausencia.
- [x] T03: Actualizar historia/Agenda y verificar estados/errores.

### C02: Ciclo de cita
- [x] `pnpm quality` en verde; acciones reales, estados terminales, retorno y accesibilidad verificadas donde el entorno lo permite.
- [~] Revisión del usuario: autorización continua; NOT CHECKED registrados en todo.md.

## Risks and Mitigations

| Risk | Mitigation |
|---|---|
| Marcar ausencia por falta de respuesta | Acción explícita del personal tras margen vigente |
| Cancelar sin comprender consecuencia | Diálogo con paciente/hora/liberación e historia |
| Duplicar mutaciones | Busy y una intención por acción; no crear nueva operación para comprobar unknown |
| Presentar éxito remoto ficticio | Estado provisional y actualización de vista separada |

## Open Questions

- Reintentos, unknown y conciliación completa pertenecen a R05; no se reenvían acciones ciegamente.
