# P-20261109-21-ux-validation: Validación UX y accesibilidad

**Status:** Approved — en ejecución (2026-11-09; autorización exprese y continua del usuario)
**Created:** 2026-11-09
**Updated:** 2026-11-09
**Task list:** [todo.md](todo.md)
**Roadmap:** [P-20261109-01-ux-ui-roadmap](../P-20261109-01-ux-ui-roadmap/plan.md) · R20

## Overview

Consolidar la validación transversal de las superficies UX/UI implementadas: comportamiento real, datos difíciles, responsive, zoom, teclado/foco, lector de pantalla, offline y recorridos con usuarios.

Depende de las superficies que se quieran auditar y de disponibilidad ambiental/participantes. No corrige hallazgos grandes dentro de una tarea implícita: cada corrección relevante reutiliza o crea un plan aprobado.

## Source of Truth

- `docs/design-master-plan.md` §§27–32.
- `.agents/laws-of-ux/SKILL.md` y `references/ux-checklist.md`.
- `DESIGN-RULES.md` §9.
- Planes operativos R01–R19 y sus NOT CHECKED.

## Architecture Decisions

- La evidencia se clasifica PASS/FAIL/NOT CHECKED/N/A; N/A y NOT CHECKED no cuentan como aprobados.
- Se usan llamadas reales y datos ficticios; compilación no sustituye recorridos.
- Se cubren viewports 800×600/1024×768/grande, 200 %, teclado/foco, lector de pantalla disponible y offline/reinicio cuando el entorno lo permita.
- Las pruebas con médico/asistente/pacientes se registran solo si ocurren; no se sustituyen por revisión técnica.
- Hallazgos se priorizan por impacto en completar tarea; correcciones se planifican sin ocultar deuda.

## Dependency Order

```text
Superficies objetivo → T01 matriz/evidencia → T02 ambiente/accesibilidad → C01
C01 → T03 usuarios/revisión/cierre → C02
```

## Task List

- [ ] T01: Consolidar matriz de recorridos y deuda de verificación.
- [ ] T02: Ejecutar verificación ambiental, responsive y accesibilidad.

### C01: Evidencia técnica
- [ ] Recorridos y checks ambientales con PASS/FAIL/NOT CHECKED/N/A documentados.

- [ ] T03: Ejecutar/registrar validación con usuarios y priorizar hallazgos.

### C02: Revisión transversal
- [ ] Evidencia completa disponible, límites explícitos y correcciones derivadas a planes propios.

## Risks and Mitigations

| Risk | Mitigation |
|---|---|
| Declarar conformidad por checklist | Evidencia observada y NOT CHECKED explícitos |
| Sustituir usuarios por inspección de código | Registro separado de pruebas humanas pendientes |
| Ocultar correcciones dentro de validación | Planes específicos para hallazgos, sin ampliar alcance |

## Open Questions

- Participantes, tecnologías de asistencia y entornos conectividad disponibles para cada ciclo de validación.
