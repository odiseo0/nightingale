# P-20261109-18-diagnostics-ui: Diagnóstico de desarrollo

**Status:** Approved — en ejecución (2026-11-09; autorización expresa y continua del usuario)
**Created:** 2026-11-09
**Updated:** 2026-11-09
**Task list:** [todo.md](todo.md)
**Roadmap:** [P-20261109-01-ux-ui-roadmap](../P-20261109-01-ux-ui-roadmap/plan.md) · R17

## Overview

Convertir el diagnóstico técnico existente en S13 accesible desde Ayuda/Desarrollo, con versiones, dispositivo y estado de adaptadores permitidos, sin convertirlo en prueba de conexión o aceptación remota.

Depende del shell P05 y queda fuera de la navegación diaria. No muestra secretos, SQL, tokens ni datos personales.

## Source of Truth

- `docs/design-master-plan.md` §§9.4, 25 y 29–31.
- `src/App.tsx`, `src/shared/api/diagnostics.ts`, `storage.ts`, `identity.ts` y `sync.ts`.
- `src/CorePanel.tsx`.

## Architecture Decisions

- Rust expone metadatos permitidos mediante comandos existentes; React presenta información local sin inferir salud cloud.
- Se distinguen versión de app/plataforma/arquitectura, esquema/applicationId, sesión local y último intercambio conocido.
- Diagnóstico no prueba autenticación, API conectada ni aceptación de operaciones.
- Errores de lectura usan texto seguro y no interpolan objetos internos.
- Desarrollo conserva acceso a casos técnicos necesarios hasta su migración específica.

## Dependency Order

```text
P05 → T01 diagnóstico real → T02 estados/seguridad → C01
C01 → C02 revisión final
```

## Task List

- [x] T01: Implementar S13 con metadatos reales.
- [x] T02: Cubrir errores, sesión y límites de interpretación.

### C01/C02: Diagnóstico
- [x] `pnpm quality` en verde; datos reales, ausencia de secretos y errores verificados.
- [~] Revisión del usuario: autorización continua; NOT CHECKED en todo.md.

## Risks and Mitigations

| Risk | Mitigation |
|---|---|
| Presentar diagnóstico local como cloud saludable | Texto explícito de alcance local |
| Exponer secretos | Solo DTO permitidos y ningún campo arbitrario |
| Llevar diagnóstico a navegación diaria | Acceso desde Ayuda/Desarrollo |

## Open Questions

- Nuevos datos de adaptadores solo se añaden cuando tengan contrato seguro y utilidad de diagnóstico.
