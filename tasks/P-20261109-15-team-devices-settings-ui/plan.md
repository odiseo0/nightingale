# P-20261109-15-team-devices-settings-ui: Equipo y dispositivos

**Status:** Blocked
**Created:** 2026-11-09
**Updated:** 2026-11-09
**Task list:** [todo.md](todo.md)
**Roadmap:** [P-20261109-01-ux-ui-roadmap](../P-20261109-01-ux-ui-roadmap/plan.md) · R14

## Overview

Preparar S12 Usuarios/permisos y Dispositivos para miembros, invitaciones, acceso por equipo y revocación con resultado remoto conocido.

**Bloqueo:** requiere contrato remoto de equipo/dispositivos, permisos concretos y recorrido de revocación. No se simulará cambio/revocación local instantánea.

## Source of Truth

- `docs/design-master-plan.md` §§14.5, 21–27 y 30–33.
- `docs/technical/components/identity.md` y `devices.md`.
- `docs/contracts/api-v1/nightingale-api.md`: identidad, cuenta y dispositivo.

## Architecture Decisions previstas

- Nightingale API autoriza miembros, invitaciones y acceso de dispositivos; escritorio consulta resultados por contrato.
- Cada persona usa su propia cuenta; cambios de rol/membresía/permiso requieren conexión y autorización.
- Dispositivos muestran aplicación, último acceso conocido y estado; revocación se aplica al reconectar, no instantáneamente offline.
- Permisos vigentes al aceptar se respetan y operaciones rechazadas conservan intención para revisión.
- Se ocultan datos/capacidades fuera de permiso y no se exponen tokens/credenciales.

## Dependency Order

```text
Contrato remoto/permisos → T01 miembros/invitaciones → T02 dispositivos → C01
C01 → T03 revocación/resultados → C02
```

## Task List

- [ ] T01: Implementar miembros e invitaciones reales.
- [ ] T02: Implementar dispositivos y revisión de acceso.

### C01: Administración remota
- [ ] Datos/acciones autorizados verificados con API real.

- [ ] T03: Cubrir revocación, reconexión y resultados desconocidos.

### C02: Equipo y dispositivos
- [ ] Permisos/revocación/recuperación verificados sin éxito local ficticio.

## Risks and Mitigations

| Risk | Mitigation |
|---|---|
| Simular revocación offline | Exigir conexión y mostrar resultado conocido |
| Compartir identidad entre personas | Cuenta individual y permisos por miembro |
| Exponer secretos | Solo metadatos permitidos; nunca tokens |

## Open Questions

- Roles/permisos por acción, aceptación de invitaciones y tratamiento local tras revocación.
