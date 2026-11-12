# Evidencia C05 — Agenda visual (después de T11–T13)

**Fecha:** 2026-11-09
**Alcance:** Fase D del plan P-20261109-22 (T11 Agenda Día, T12 Detalle, T13 Geometría semanal).

## Resultados técnicos

| Comprobación | Resultado |
|---|---|
| `pnpm build` (T11, T12, T13) | PASS |
| `pnpm quality` (build + fmt + clippy + tests Rust) | PASS — 377 unitarias, 5 persistencia, 1 doc-test; 9 ignoradas |
| `node --test tests/ui/agenda-presentation.test.mjs` | PASS — 4/4 (política Profesional, fin 24:00, dirección sin país, actividad traducida) |
| `node --test tests/ui/week-layout.test.mjs` | PASS — 7/7 (rango base, madrugada en hora real, fin nocturno, límites de día, escala 96/48 px, columnas solapadas, consecutivas) |
| Columna/filtro Profesional según rol y nº de profesionales | PASS (pruebas unitarias de la política; perfiles reales NOT CHECKED) |
| Sincronización retirada de Día, detalle y semana | PASS (código; vista real NOT CHECKED) |
| Dirección sin país en detalle; responsable sin UUID | PASS (funciones puras; datos reales NOT CHECKED) |
| Geometría semanal 96 px/hora, 160 px/día, solapamientos en columnas | PASS (pruebas; render visual NOT CHECKED) |

## Comprobaciones ambientales y humanas

| Comprobación | Estado |
|---|---|
| Comparación de Día/detalle/Semana con la muestra C02 | NOT CHECKED |
| Perfiles dueño/asistente y dueño con un profesional en la app real | NOT CHECKED |
| Datos largos, duraciones distintas (15/30/60, no redondas), simultáneas, bloqueos, medianoche en ejecución | NOT CHECKED |
| Apertura/cierre de detalle lateral y estrecho con foco y contexto | NOT CHECKED |
| 800×600 y ventana amplia | NOT CHECKED |
| Lecturas IPC de Semana sin ciclo continuo | NOT CHECKED (se aborda en T14 con dependencias estables) |
| Revisión expresa del usuario | Pendiente (aceptación final en C14) |

## Notas

- El contador de citas del Día y de la Semana describe datos locales sin afirmar exhaustividad.
- Las operaciones técnicas dejaron la vista diaria del detalle; el diagnóstico conserva su acceso.
- El estado «Actualizando…» se retiró del Día y de la Semana: la sincronización normal es invisible y el contenido conocido se conserva durante las lecturas.
