# P-20261109-07-access-onboarding-ui: Acceso y preparación

**Status:** Approved — en ejecución (2026-11-09; autorización expresa y continua del usuario)
**Created:** 2026-11-09
**Updated:** 2026-11-09
**Task list:** [todo.md](todo.md)
**Roadmap:** [P-20261109-01-ux-ui-roadmap](../P-20261109-01-ux-ui-roadmap/plan.md) · R06

## Overview

Diseñar S01 para acceso, selección de cuenta y preparación inicial, sustituyendo progresivamente el flujo técnico de Desarrollo sin perder recuperación ni datos locales.

Depende del shell P05. No inventa PIN, biometría, borrado local, sesión offline ni proveedor OIDC nuevo.

## Source of Truth

- `docs/design-master-plan.md` §§8–9, 21–27 y 30–31.
- `src/CorePanel.tsx`, `src/shared/api/identity.ts`, `account.ts`, `services.ts` y `scheduling.ts`.
- `docs/technical/components/identity.md`, `devices.md` y `account.md`.

## Architecture Decisions

- Rust `identity`/`devices`/`account` conserva sesión, dispositivo, permisos y datos. La activación/alta remota requiere conexión; local usa últimos permisos válidos.
- Flujo existente: `identity_begin_login`, `identity_complete_login`, `identity_cancel_login`, `identity_list_accounts`, `identity_activate_account`, `identity_get_status`.
- Preparación muestra pasos reales: cuenta/zona, perfil profesional, servicio/oferta y horario; no tutorial ni opciones inexistentes.
- Se distingue sin sesión, autenticando, dispositivo no registrado, sin cuenta, sin vista local, listo, revocado, offline y servicio no disponible.
- Una sesión remota vencida no borra datos ni pendientes. Revocación se conoce al reconectar y sigue la política nativa.
- Se conserva Desarrollo hasta que el recorrido real esté verificado; no se elimina funcionalidad antes de tiempo.

## Dependency Order

```text
P05 → T01 acceso/cuenta → T02 preparación → C01
C01 → T03 estados/offline/revocación → C02
```

## Task List

- [x] T01: Implementar acceso y selección de cuenta.
- [x] T02: Implementar preparación inicial basada en datos reales.

### C01: Entrada funcional
- [x] `pnpm quality` en verde; acceso real con servicio fixture (evidencia P05 para OIDC interactivo) y selección de cuenta.

- [x] T03: Cubrir estados, recuperación y transición a Agenda.

### C02: Acceso/preparación
- [x] `pnpm quality` en verde; acceso, preparación y transición verificados.
- [~] Revisión del usuario: autorización continua; NOT CHECKED en todo.md.

## Risks and Mitigations

| Risk | Mitigation |
|---|---|
| Simular activación offline | Estados que exigen conexión y ningún éxito local remoto |
| Perder datos al cambiar sesión | No borrar SQLite ni pendientes; revisar política nativa |
| Mostrar preparación completa sin datos | Pasos derivados de cuenta/perfil/servicio/horario reales |
| Romper flujo técnico vigente | Desarrollo permanece disponible durante migración |

## Open Questions

- Política de acceso local, sesiones persistentes y bloqueo quedan para decisión específica posterior.
