# P-20261109-12-waitlist-ui: Lista de espera y ofertas

**Status:** Blocked
**Created:** 2026-11-09
**Updated:** 2026-11-09
**Task list:** [todo.md](todo.md)
**Roadmap:** [P-20261109-01-ux-ui-roadmap](../P-20261109-01-ux-ui-roadmap/plan.md) · R11

## Overview

Preparar S08 para entradas en espera, compatibilidad, ofertas, vencimiento y asignación por personal.

**Bloqueo:** faltan dominio/contrato de lista de espera, canal de respuesta, prioridad y plazos. La respuesta del paciente expresa interés, no reserva. No se implementará un modelo ficticio.

## Source of Truth

- `docs/design-master-plan.md` §§8, 12.2, 21–25 y 30–33.
- `docs/srs.md` RF-09 y regla P06.
- Disponibilidad/agenda existentes para candidatos e intervalos.

## Architecture Decisions previstas

- Un componente propietario real definirá entradas, ofertas y asignación; React no mantendrá una segunda autoridad.
- En espera y Ofertas serán vistas separadas con preferencias, compatibilidad, vencimiento y resultado.
- El personal solicita reserva; API vuelve a validar oferta/disponibilidad. Aceptar oferta no crea cita por sí solo.
- La prioridad no se infiere del orden visual. Offline muestra última vista conocida y antigüedad.
- La asignación usará el caso nativo de citas y mostrará provisionalidad/conflictos.

## Dependency Order

```text
Contrato/dominio/política → T01 entradas → T02 ofertas → C01
C01 → T03 asignación/estados → C02
```

## Task List

- [ ] T01: Implementar entradas En espera y compatibilidad.
- [ ] T02: Implementar Ofertas y revisión de candidatos/horario.

### C01: Oferta real
- [ ] Compatibilidad/vencimiento/resultado verificados con contrato real.

- [ ] T03: Implementar asignación por personal y estados de reserva.

### C02: Recuperación de huecos
- [ ] Oferta→interés→asignación→aceptación verificada sin confundir resultados.

## Risks and Mitigations

| Risk | Mitigation |
|---|---|
| Tratar interés como reserva | Separar oferta/interés/cita y validación API |
| Inventar prioridad | No ordenar como prioridad sin política aprobada |
| Ocultar conflicto al asignar | Provisionalidad y revisión explícita |

## Open Questions

- Prioridad, canal, plazos, expiración y retirada de entradas/ofertas.
