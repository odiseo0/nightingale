# P-20261109-06-sync-review-ui: Cambios y conflictos

**Status:** Approved — en ejecución (2026-11-09; autorización expresa y continua del usuario)
**Created:** 2026-11-09
**Updated:** 2026-11-09
**Task list:** [todo.md](todo.md)
**Roadmap:** [P-20261109-01-ux-ui-roadmap](../P-20261109-01-ux-ui-roadmap/plan.md) · R05

## Overview

Construir S11 para revisar pendientes, resultados desconocidos, reintentos, rechazos y conflictos con vocabulario de trabajo y recuperación segura.

Depende de mutaciones UI reales (P06; R02/R03 para casos completos). No incluye forzar guardado, último cambio gana, borrado de cola ni nuevas mutaciones de conciliación no contratadas.

## Source of Truth

- `docs/design-master-plan.md` §§14.1, 21–25 y 27–31.
- `docs/technical/components/sync.md`.
- `src/shared/api/identity.ts`, `sync.ts`, `appointments.ts` y `account.ts`.
- `src/CorePanel.tsx` y operaciones locales de entidades.

## Architecture Decisions

- Rust `sync`/`identity` conserva Outbox, recibos, cursores y acceso; React solo presenta resultados conocidos.
- La superficie usa `identity_get_status`, `sync_list_pending` y operaciones de entidades existentes. No se añade HTTP/SQL ni reintento ciego.
- Normal: acceso discreto. Pendiente/unknown/conflicto/rechazo/revisión: enlace visible con alcance de cuenta/dispositivo.
- Conflictos comparan aceptado/provisional/operaciones; elegir otra hora lleva al editor existente R02. Descartar intención requiere comando específico y queda fuera si no existe.
- Unknown exige comprobar la misma operación; no crear otra equivalente. Permiso rechazado conserva intención y no usa hora local como prueba.
- Se reutilizan estados de `presentation.ts` y Laws of UX para atención selectiva, memoria de trabajo y recuperación.

## Dependency Order

```text
P06 → T01 centro de cambios → T02 comparación/conflictos → C01
C01 → T03 offline/permiso/recuperación → C02
```

## Task List

- [x] T01: Implementar resumen y lista de cambios reales.
- [x] T02: Implementar comparación y acciones existentes de revisión.

### C01: Revisión funcional
- [x] `pnpm quality` en verde; estados reales, texto y navegación verificados.

- [x] T03: Verificar offline, reinicio, permiso y recuperación sin duplicar.

### C02: Centro de cambios
- [x] `pnpm quality` en verde; estados reales y retorno verificados; comparación implementada.
- [~] Revisión del usuario: autorización continua; NOT CHECKED en todo.md.

## Risks and Mitigations

| Risk | Mitigation |
|---|---|
| Reenviar por timeout | Unknown exige comprobar identidad antes de otra acción |
| Ocultar trabajo pendiente | Enlace visible y estado persistente cuando requiere revisión |
| Inventar acciones de conciliación | Solo comandos existentes; descarte queda fuera sin contrato |
| Exponer vocabulario técnico | Texto de trabajo; UUID/cursores solo en detalle/diagnóstico permitido |

## Open Questions

- La resolución/descarte definitiva de cada tipo de intención requiere comandos y políticas específicas posteriores.
