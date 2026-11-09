# P-20261109-16-communication-settings-ui: Política de comunicaciones

**Status:** Blocked
**Created:** 2026-11-09
**Updated:** 2026-11-09
**Task list:** [todo.md](todo.md)
**Roadmap:** [P-20261109-01-ux-ui-roadmap](../P-20261109-01-ux-ui-roadmap/plan.md) · R15

## Overview

Preparar S12 Comunicaciones para políticas de recordatorios, cancelación y enlaces del Patient Hub, solo con opciones contratadas.

**Bloqueo:** depende de R12 y de plazos/verificación/políticas concretas. No se copiarán valores de ejemplo del prototipo como configuración real.

## Source of Truth

- `docs/design-master-plan.md` §§14.5, 15, 21–27 y 30–33.
- `docs/specifications/nightingale-patient-hub.md`.
- `docs/srs.md` §§9 y 16.

## Architecture Decisions previstas

- API/Hub posee política de recordatorios/cancelación/accesos; escritorio no define plazos localmente.
- Formularios mostrarán solo opciones existentes: cantidad/anticipación de recordatorios, inclusión de enlace, cancelación y verificación cuando estén contratadas.
- Antes de guardar se explicará efecto en futuras comunicaciones sin reescribir mensajes/citas anteriores.
- Guardado remoto requiere conexión/permiso y muestra resultado/refresh separados.
- No se inventarán vencimientos, canales alternativos ni verificación adicional.

## Dependency Order

```text
R12 + políticas → T01 recordatorios → T02 cancelación/enlaces → C01
C01 → T03 efecto/resultado/errores → C02
```

## Task List

- [ ] T01: Implementar política de recordatorios contratada.
- [ ] T02: Implementar política de cancelación y enlaces.

### C01: Política real
- [ ] Opciones y efecto verificados con contrato real.

- [ ] T03: Implementar guardado remoto, refresh y errores.

### C02: Configuración de comunicaciones
- [ ] Política guardada, resultado conocido y vista actualizada verificados.

## Risks and Mitigations

| Risk | Mitigation |
|---|---|
| Convertir ejemplos en política | Solo valores del contrato vigente |
| Aplicar cambios a historial | Explicar alcance futuro sin reescribir mensajes |
| Simular guardado local remoto | Conexión/permiso y refresh explícitos |

## Open Questions

- Plazos, cantidad de recordatorios, cancelación, vencimiento y verificación adicional.
