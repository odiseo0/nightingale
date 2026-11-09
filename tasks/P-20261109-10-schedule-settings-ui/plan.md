# P-20261109-10-schedule-settings-ui: Horarios y bloqueos

**Status:** Approved — en ejecución (2026-11-09; autorización expresa y continua del usuario)
**Created:** 2026-11-09
**Updated:** 2026-11-09
**Task list:** [todo.md](todo.md)
**Roadmap:** [P-20261109-01-ux-ui-roadmap](../P-20261109-01-ux-ui-roadmap/plan.md) · R09

## Overview

Implementar S12 Horarios para reglas semanales, excepciones y bloqueos con fechas/horas comprensibles, vigencias, impacto en citas y permisos globales.

Depende de R08 y `scheduling`. No se permiten minutos desde medianoche/RFC3339 como entrada administrativa, ni cancelaciones implícitas de citas al cambiar agenda.

## Source of Truth

- `docs/design-master-plan.md` §§14.4, 21–27 y 29–31.
- `src/features/scheduling/AgendaEditor.tsx`, `AvailabilityPanel.tsx` y `src/shared/api/scheduling.ts`.
- `docs/technical/components/scheduling.md`.

## Architecture Decisions

- Rust `scheduling` calcula reglas, excepciones, bloqueos, disponibilidad y compatibilidad con citas; React no recalcula.
- Reglas usan día de semana, inicio/fin y vigencia; excepciones closed/replacement por fecha; bloqueos usan cuenta/global y motivo autorizado.
- Fechas/horas se editan con controles nativos y zona de cuenta, convirtiendo solo en la frontera a los campos DTO.
- Cambios que dejan citas fuera del horario/bloqueo se rechazan conservando formulario; no se cancelan/mueven citas silenciosamente.
- Solo profesional propio gestiona bloqueos globales; asistentes no ven motivo restringido. Escrituras son provisionales.

## Dependency Order

```text
R08 → T01 reglas semanales → T02 excepciones/bloqueos → C01
C01 → T03 impacto/permisos/versiones → C02
```

## Task List

- [x] T01: Implementar listado y editor de reglas semanales.
- [x] T02: Implementar excepciones y bloqueos.

### C01: Agenda configurable
- [x] `pnpm quality` en verde; reglas/excepciones/bloqueos reales y formularios accesibles.

- [x] T03: Cubrir impacto en citas, permisos globales, versiones y errores.

### C02: Horarios y bloqueos
- [x] `pnpm quality` en verde; cambios locales reales, compatibilidad y retorno.
- [~] Revisión del usuario: autorización continua; NOT CHECKED en todo.md.

## Risks and Mitigations

| Risk | Mitigation |
|---|---|
| Editar tiempo en formato técnico | Controles nativos y conversión en frontera DTO |
| Dejar citas inválidas sin aviso | Mostrar rechazo y conservar cambios para corregir |
| Exponer motivo global restringido | Presentación por permiso, sin inferencia |
| Mezclar excepción y bloqueo | Formularios/etiquetas separados por concepto |

## Open Questions

- Cambios complejos de agenda que requieran mover citas necesitan otro caso de uso explícito; están fuera.
