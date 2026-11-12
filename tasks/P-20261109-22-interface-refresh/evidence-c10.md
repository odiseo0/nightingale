# Evidencia C10 — Recuperación (después de T26–T28)

**Fecha:** 2026-11-09
**Alcance:** Fase I del plan P-20261109-22 (T26 Editores de Horarios, T27 Por revisar, T28 Acceso y preparación).

## Resultados técnicos

| Comprobación | Resultado |
|---|---|
| `pnpm build` (T27, T28) | PASS |
| `pnpm quality` (build + fmt + clippy + tests Rust, incluye `sync` e `identity`) | PASS — 377 unitarias, 5 persistencia, 1 doc-test; 9 ignoradas |
| `node --test tests/ui/*.test.mjs` | PASS — 16/16 (agenda-presentación 4, week-layout 7, reviews 5) |
| Por revisar: solo intervención; sin cola/fases/contadores/botón; enlaces por clave de entidad sin barrido | PASS (pruebas de clasificación; recorrido sintético NOT CHECKED) |
| Unknown sigue automático sin duplicación (misma identidad, no listado) | PASS (clasificación probada) |
| Acceso: estados con explicación y siguiente paso; sin panel técnico del motor | PASS (código; acceso sin red NOT CHECKED) |

## Comprobaciones ambientales y humanas

| Comprobación | Estado |
|---|---|
| Conflicto, rechazo, acceso restringido y recuperación sin borrar/repetir | NOT CHECKED |
| Resolución de asuntos sintéticos de cita, paciente y acceso | NOT CHECKED |
| Caso sin asuntos (enlace fuera de navegación) | NOT CHECKED |
| 800×600 y zoom | NOT CHECKED |
| Revisión expresa del usuario | Pendiente (aceptación final en C14) |

## Notas

- Las operaciones técnicas (pending/in_flight/retryable/unknown/accepted) no aparecen en la interfaz diaria; diagnóstico conserva su acceso en Desarrollo.
- No se ofrecen acciones de forzar, descartar ni sobrescribir; la resolución usa comandos existentes.
