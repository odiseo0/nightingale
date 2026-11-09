# P-20261109-08-account-settings-ui: Cuenta y preferencias

**Status:** Approved — en ejecución (2026-11-09; autorización exprese y continua del usuario)
**Created:** 2026-11-09
**Updated:** 2026-11-09
**Task list:** [todo.md](todo.md)
**Roadmap:** [P-20261109-01-ux-ui-roadmap](../P-20261109-01-ux-ui-roadmap/plan.md) · R07

## Overview

Implementar S12 Cuenta con datos compartidos, zona/lugar, política de ausencia y preferencias locales, distinguiendo conexión, permiso y actualización de vista.

Depende del shell P05. No incluye dark/system sin diseño, cambios de permisos ni exportación/borrado.

## Source of Truth

- `docs/design-master-plan.md` §§14.2, 21–27 y 29–31.
- `src/features/settings/SettingsPanel.tsx` y `src/shared/api/account.ts`.
- `docs/technical/components/account.md`.

## Architecture Decisions

- Rust `account` valida versión, permiso y canal remoto. Datos compartidos usan comandos de actualización; preferencias locales usan `account_set_preference`.
- Se separan datos remotos de preferencias del dispositivo y solo lectura por permiso.
- Tema claro es el único implementado visualmente. No se ofrecerá dark hasta R19; system no inventa una paleta.
- Aceptación remota y refresco de vista se muestran por separado; `refreshStatus=failed` no se presenta como vista actualizada.
- Se reutilizan formularios/tokens y se protegen cambios sin guardar, versiones y retorno.

## Dependency Order

```text
P05 → T01 cuenta/preferencias → T02 edición remota/local → C01
C01 → T03 versiones/permiso/errores → C02
```

## Task List

- [x] T01: Implementar navegación de Configuración y vista de cuenta.

### C01: Cuenta visible
- [x] `pnpm quality` en verde; datos reales y solo lectura/permiso verificados.

- [x] T02: Implementar edición de datos/política y preferencias locales.
- [x] T03: Cubrir versiones, refresh, errores y salida sin guardar.

### C02: Cuenta y preferencias
- [x] `pnpm quality` en verde; actualización remota/local con IPC real, retorno y accesibilidad heredada.
- [~] Revisión del usuario: autorización continua; NOT CHECKED en todo.md.

## Risks and Mitigations

| Risk | Mitigation |
|---|---|
| Presentar aceptación como vista actualizada | Mostrar refresh pendiente/fallido por separado |
| Inventar dark mode | Solo tema claro; R19 posterior |
| Editar sin permiso/conexión | Solo lectura y mensaje de canal requerido |
| Perder contexto al cambiar versión | No reiniciar formulario durante edición; versiones explícitas |

## Open Questions

- Preferencia system/dark se decide en R19, no en este plan.
