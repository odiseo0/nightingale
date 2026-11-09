# P-20261109-03-appointment-reschedule-ui: Reprogramar cita

**Status:** Approved — en ejecución (2026-11-09; P06 Completed; autorización continua del usuario)
**Created:** 2026-11-09
**Updated:** 2026-11-09
**Task list:** [todo.md](todo.md)
**Roadmap:** [P-20261109-01-ux-ui-roadmap](../P-20261109-01-ux-ui-roadmap/plan.md) · R02

## Overview

Implementar reprogramación de citas programadas/confirmadas comparando horario actual y propuesto, conservando duración y explicando invalidación de confirmación antes de guardar localmente.

Depende de P06. No permite cambiar paciente, profesional, servicio, cuenta o responsable, ni fuerza disponibilidad/aceptación remota.

## Source of Truth

- `docs/design-master-plan.md` §§8, 10.2–10.5, 20–24 y 27–31.
- `docs/contracts/api-v1/nightingale-api.md` §5.4.
- `src/shared/api/appointments.ts`: `rescheduleAppointment`, detalle/historia y `baseOf`.
- `src/shared/api/scheduling.ts`: disponibilidad local/cobertura.
- Editor P06 y componentes de disponibilidad/resumen.

## Architecture Decisions

- Rust `appointments`/`scheduling` validan estado, duración, disponibilidad, versión y permisos. React presenta opciones y conserva intención.
- Ruta propuesta `/cita/:appointmentId/reprogramar`, accesible desde detalle solo si la transición local es válida.
- El intervalo actual permanece visible junto al nuevo; duración real se conserva y no se edita en este plan.
- Nuevos horarios proceden de `getLocalAvailability`; cobertura y provisionalidad son visibles. Consultar no reserva.
- `rescheduleAppointment` usa base aceptada u operationId y produce estado programada/provisional, no confirmada.
- No se afirma liberación del horario anterior ni aceptación hasta conocer el resultado. Errores/conflictos conservan propuesta.
- Se reutilizan tokens/componentes y Laws of UX para memoria de trabajo, comparación, proximidad y feedback.

## Dependency Order

```text
P06 → T01 comparación y contexto → T02 disponibilidad → C01
C01 → T03 guardado/historia/retorno → C02
```

## Task List

- [x] T01: Añadir ruta y comparación actual/propuesto.
- [x] T02: Seleccionar nuevo intervalo con disponibilidad real.

### C01: Propuesta válida
- [x] `pnpm quality` en verde; comparación, duración, cobertura y formulario accesible.

- [x] T03: Guardar reprogramación, tratar errores y actualizar detalle/historia.

### C02: Reprogramación local
- [x] `pnpm quality` en verde; reprogramación real, confirmación invalidada según DTO, retorno y estados.
- [~] Revisión del usuario: autorización continua; NOT CHECKED registrados en todo.md.

## Risks and Mitigations

| Risk | Mitigation |
|---|---|
| Editar entidades prohibidas por reprogramación | Mostrar paciente/servicio/profesional como contexto de solo lectura |
| Cambiar duración accidentalmente | Seleccionar intervalos de igual duración validados por Rust |
| Afirmar confirmación/liberación prematura | Texto provisional y aceptación separada |
| Perder propuesta ante conflicto | Conservar formulario y comparación para elegir otra hora |

## Open Questions

- Resolución completa de conflictos compartidos se integra con R05; esta etapa conserva y presenta el fallo sin forzar sobrescritura.
