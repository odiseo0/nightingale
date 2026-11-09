# P-20261109-20-theme-semantics-ui: Tema y semántica cromática

**Status:** Blocked
**Created:** 2026-11-09
**Updated:** 2026-11-09
**Task list:** [todo.md](todo.md)
**Roadmap:** [P-20261109-01-ux-ui-roadmap](../P-20261109-01-ux-ui-roadmap/plan.md) · R19

## Overview

Definir, solo por decisión expresa del usuario, una ampliación para dark/system y colores de error, advertencia y crítico con significado y contraste consistente.

**Bloqueo:** actualmente solo el tema claro está aprobado y los colores semánticos de error/advertencia/crítico no están definidos. No se ejecutará una inversión mecánica de la paleta ni nuevos colores locales.

## Source of Truth

- `DESIGN.md` §§1–3, 6–11.
- `DESIGN-RULES.md` §§2–4.
- `docs/design-master-plan.md` §§14.2, 16, 21, 27 y 33.

## Architecture Decisions previstas

- La ampliación empieza por significado y roles semánticos, no por preferencias de componentes concretos.
- Dark/system requiere una paleta propia revisada, `color-scheme`, preferencia persistida y comportamiento de cambio sin pérdida de foco/contenido.
- Error/advertencia/crítico usan texto/icono/estructura además de color; no se confunden con estados de cita/sincronización/clínicos.
- Se validan texto, controles, foco, selección, disabled, gráficos y superficies reales con WCAG objetivo.
- Todo cambio transversal exige actualizar DESIGN y planes afectados antes de implementar tokens.

## Dependency Order

```text
Decisión del usuario → T01 semántica → T02 dark/system → C01
C01 → T03 contraste/verificación transversal → C02
```

## Task List

- [ ] T01: Definir y aprobar roles de error/advertencia/crítico.
- [ ] T02: Definir y aprobar paleta dark/system.

### C01: Especificación aprobada
- [ ] DESIGN actualizado por decisión expresa y roles coherentes.

- [ ] T03: Implementar tokens y verificar todas las superficies.

### C02: Ampliación visual
- [ ] Contraste/estados reales verificados sin alterar significado de dominio.

## Risks and Mitigations

| Risk | Mitigation |
|---|---|
| Invertir colores mecánicamente | Diseño de roles propio y contraste real |
| Comunicar estado solo por color | Texto/icono/estructura obligatorios |
| Cambiar semántica clínica por estética | Revisión de significado por superficie |

## Open Questions

- Si dark/system se implementa, preferencia inicial, cambios de tema y combinaciones semánticas completas.
