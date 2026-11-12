# Comprobaciones de integración y recuperación — P-20261109-22 (T38)

**Fecha:** 2026-11-09
**Entorno:** Windows (entorno de desarrollo local). macOS/Linux: fuera de alcance verificado.

## Automatizado (evidencia real)

| Comprobación | Comando | Resultado |
|---|---|---|
| Build completo del frontend (tipos + Vite) | `pnpm build` | PASS (en cada tarea y cierre) |
| Calidad: fmt, clippy `-D warnings`, tests Rust workspace | `pnpm quality` | PASS — 377 unitarias, 5 persistencia, 1 doc-test; 9 ignoradas |
| Pruebas de presentación | `node --test tests/ui/*.test.mjs` | PASS — 16/16 |
| Persistencia y recuperación del núcleo (outbox, cursores, identidad, rollback) | `core_persistence.rs` vía quality | PASS |
| Contexto de cuenta y preferencias aisladas por cuenta/usuario/dispositivo | `cargo test account` vía quality | PASS |
| Metadatos y ámbito de revisiones | `cargo test sync` vía quality | PASS |
| Casos de uso de citas, pacientes, servicios y horarios | `cargo test` workspace vía quality | PASS |
| Build de escritorio | `pnpm build:desktop` | PASS — `target\debug\nightingale.exe` compilado |

## Garantías conservadas (por construcción y pruebas Rust)

- Operación local sin HTTP: lecturas y mutaciones nativas sobre SQLite; React solo invoca comandos Tauri.
- Identidad de operaciones: reintento y respuesta perdida recuperan la misma operación (tests sync/core_persistence); la UI impide repetir la intención (savingRef, unknown no listado).
- Permisos y versiones: base y expectedVersion en cada mutación; rechazo conserva intención y formulario.
- Sincronización correcta invisible: sin avisos, columnas, contadores ni botones de proceso en las pantallas de trabajo.
- Intervención en contexto y en Por revisar; sin forzar/descartar/sobrescribir.

## Escenarios nativos no automatizables en esta sesión (NOT CHECKED)

1. Corte de red real, reinicio y recuperación con respuesta perdida (escenarios 10–12 del plan).
2. Cambio de cuenta sin mezclar datos ni respuestas antiguas (escenario 13).
3. Conflicto concurrente y revocación de permiso con equipo conectado (escenarios 14–15 con API local).
4. Recorridos con IPC real y datos sintéticos en la app nativa (escenarios 1–9).
5. macOS/Linux y validación humana.

Estos escenarios requieren el entorno de API local autorizado y sesión interactiva; quedan registrados como pendientes, no como aprobados.
