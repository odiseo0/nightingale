# P-20261109-17-shell-discovery-ui: Búsqueda global y navegación completa

**Status:** Approved — en ejecución (2026-11-09; autorización expresa y continua del usuario)
**Created:** 2026-11-09
**Updated:** 2026-11-09
**Task list:** [todo.md](todo.md)
**Roadmap:** [P-20261109-01-ux-ui-roadmap](../P-20261109-01-ux-ui-roadmap/plan.md) · R16

## Overview

Completar el shell con búsqueda global/Cmd+K, ayuda y navegación a superficies reales, sin placeholders ni resultados fuera de la cuenta activa.

Depende de Pacientes en lectura y P06; se amplía solo cuando existan destinos reales. No añade chat, atajos ocultos como única vía ni un gestor global de entidades.

## Source of Truth

- `docs/design-master-plan.md` §§6, 9, 19, 27–31.
- `src/app/AppLayout.tsx`, `src/App.tsx`.
- `src/shared/api/appointments.ts` y `patients.ts`.

## Architecture Decisions

- Búsqueda usa contexto local autorizado de pacientes y `listAppointments`; offline indica que solo cubre datos del equipo.
- Resultados distinguen paciente/cita, nombre y contexto permitido; sin coincidencias locales no prueba ausencia global.
- Command palette tiene alternativa visible, teclado completo, foco gestionado y `Cmd/Ctrl+K`; no se exige atajo.
- Navegación muestra solo superficies implementadas y ayuda/diagnóstico fuera del trabajo diario.
- Se conserva contexto al abrir resultado y retorno de foco al cerrar; no hay duplicado mutable global de entidades.

## Dependency Order

```text
Pacientes lectura + P06 → T01 búsqueda global → T02 navegación/ayuda → C01
C01 → T03 accesibilidad/contexto/offline → C02
```

## Task List

- [x] T01: Implementar búsqueda global con resultados reales.
- [x] T02: Completar navegación y ayuda con destinos existentes.

### C01: Descubrimiento real
- [x] `pnpm quality` en verde; búsqueda, destinos y retorno verificados.

- [x] T03: Verificar teclado, foco, offline y contexto.

### C02: Shell completo
- [x] `pnpm quality` en verde; búsqueda/navegación accesibles y sin placeholders.
- [~] Revisión del usuario: autorización continua; NOT CHECKED en todo.md.

## Risks and Mitigations

| Risk | Mitigation |
|---|---|
| Filtrar datos de otra cuenta | Consultas por contexto activo y resultados limitados |
| Confundir ausencia local con inexistencia | Texto específico de búsqueda local |
| Navegación con destinos vacíos | Rutas solo cuando son funcionales |
| Perder foco/contexto | Retorno al disparador y selección conservada |

## Open Questions

- Nuevos tipos de resultado se añadirán solo cuando tengan destino y contrato reales.
