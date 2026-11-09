# P-20261109-05-agenda-views-ui: Semana y Lista de citas

**Status:** Approved — en ejecución (2026-11-09; autorización expresa y continua del usuario)
**Created:** 2026-11-09
**Updated:** 2026-11-09
**Task list:** [todo.md](todo.md)
**Roadmap:** [P-20261109-01-ux-ui-roadmap](../P-20261109-01-ux-ui-roadmap/plan.md) · R04

## Overview

Completar S02 con vistas Semana y Lista, manteniendo Día y detalle, filtros/contexto y disponibilidad sin convertir listas filtradas en prueba de huecos.

Depende de P05/P06; se coordina con reprogramación R02 para abrir el mismo detalle. No incluye drag-and-drop obligatorio, creación desde hueco ni analítica.

## Source of Truth

- `docs/design-master-plan.md` §§6–10, 18–21 y 27–31.
- `src/features/appointments/AgendaPage.tsx`, `AppointmentDetail.tsx`, `AppointmentPage.tsx`.
- `src/shared/api/appointments.ts` y `scheduling.ts`: lista, bloqueos y disponibilidad.

## Architecture Decisions

- Rust `appointments`/`scheduling` conserva citas, bloqueos y disponibilidad; React presenta vistas sin recalcular ocupación.
- Semana consulta citas del periodo y bloqueos con texto/forma; escala de horas y scroll localizado. Toda acción tiene alternativa por formulario/detalle.
- Lista añade búsqueda por paciente/referencia autorizada, periodo, profesional/servicio/estado y paginación real, incluyendo pasadas/canceladas solo por filtro.
- Día/Semana/Lista son tabs funcionales con `aria-current`, no placeholders. Fecha, vista, filtros, selección y foco se conservan entre vistas/detalle.
- Los huecos solo se derivan de disponibilidad calculada; filtros vacíos y datos no descargados no se presentan como disponibilidad.
- Se reutilizan tokens/tabla/detalle y Laws of UX para comparación, memoria de trabajo, atención selectiva y Fitts.

## Dependency Order

```text
P05/P06 → T01 Semana → C01 → T02 Lista → T03 contexto/verificación → C02
```

## Task List

- [x] T01: Implementar Semana con citas/bloqueos y detalle compartido.

### C01: Vista semanal
- [x] `pnpm quality` en verde; escala, scroll, eventos y retorno verificados.

- [x] T02: Implementar Lista con búsqueda/periodo/filtros/paginación.
- [x] T03: Integrar tabs, contexto y comprobaciones responsive/accesibilidad.

### C02: Vistas de Agenda
- [x] `pnpm quality` en verde; Día/Semana/Lista con IPC real y detalle común.
- [~] Revisión del usuario: autorización continua; NOT CHECKED en todo.md.

## Risks and Mitigations

| Risk | Mitigation |
|---|---|
| Semana ilegible en ventana estrecha | Scroll propio, etiquetas y superficie de detalle alternativa |
| Confundir filtro vacío con disponibilidad | Disponibilidad Rust y textos de cobertura |
| Perder contexto al cambiar de vista | Estado explícito de fecha/vista/filtros/selección/foco |
| Acciones solo por arrastre | Formulario y detalle equivalentes; arrastre fuera de alcance |

## Open Questions

- Crear desde hueco y drag-and-drop requieren evaluación posterior; no se añaden por espacio disponible.
