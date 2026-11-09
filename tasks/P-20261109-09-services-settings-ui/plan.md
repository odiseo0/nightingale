# P-20261109-09-services-settings-ui: Servicios y ofertas

**Status:** Approved — en ejecución (2026-11-09; autorización exprese y continua del usuario)
**Created:** 2026-11-09
**Updated:** 2026-11-09
**Task list:** [todo.md](todo.md)
**Roadmap:** [P-20261109-01-ux-ui-roadmap](../P-20261109-01-ux-ui-roadmap/plan.md) · R08

## Overview

Implementar S12 Servicios para catálogo del profesional, ofertas por cuenta, duraciones, versiones y activo/inactivo, dejando claro que los cambios no reescriben citas anteriores.

Depende de R07 y `services`. No incluye duración personalizada en Nueva cita ni cambios de historial.

## Source of Truth

- `docs/design-master-plan.md` §§14.3, 21–27 y 29–31.
- `src/features/services/ServicesPanel.tsx` y `src/shared/api/services.ts`.
- `docs/technical/components/services.md`.

## Architecture Decisions

- Rust `services` valida catálogo/oferta, versiones, permisos y dependencias. React presenta estados aceptados/provisionales y operaciones.
- Se distinguen catálogo, oferta, defaultDurationMinutes, override/effective, version y serviceVersion.
- Formularios usan nombres/minutos comprensibles y profesionales reales, no UUID. Editar no cambia citas previas; esto se explica antes de guardar.
- Todas las escrituras son locales provisionales; sincronización/aceptación/conflictos se muestran por separado.
- Se reutilizan listas/formularios/tokens y se evita duplicar formularios para oferta existente.

## Dependency Order

```text
R07 → T01 catálogo/ofertas → T02 crear/editar → C01
C01 → T03 estados/versiones/conflictos → C02
```

## Task List

- [x] T01: Implementar catálogo y ofertas con datos reales.

### C01: Servicios visibles
- [x] `pnpm quality` en verde; versiones, duraciones y estados diferenciados.

- [x] T02: Implementar creación/edición de servicios/ofertas.
- [x] T03: Implementar activo/inactivo y tratamiento de versiones/conflictos.

### C02: Servicios configurables
- [x] `pnpm quality` en verde; operaciones locales reales y efecto explicado.
- [~] Revisión del usuario: autorización continua; NOT CHECKED en todo.md.

## Risks and Mitigations

| Risk | Mitigation |
|---|---|
| Confundir versión de servicio/oferta | DTO y textos separados para version/serviceVersion |
| Reescribir citas históricas visualmente | Snapshots intactos y aviso antes de guardar |
| Crear duplicados de oferta | Comprobar existencia y usar actualización cuando corresponda |
| Ocultar cambios pendientes | Propuesta/aceptada/operaciones separadas |

## Open Questions

- La decisión de duración fija/variable en el editor de citas se mantiene abierta; esta configuración solo administra referencia/override.
