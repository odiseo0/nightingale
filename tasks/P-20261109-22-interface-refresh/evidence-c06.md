# Evidencia C06 — Consulta y creación (después de T14–T16)

**Fecha:** 2026-11-09
**Alcance:** Fase E del plan P-20261109-22 (T14 Lecturas de Semana, T15 Lista de citas, T16 Nueva cita).

## Resultados técnicos

| Comprobación | Resultado |
|---|---|
| `pnpm build` (T14, T15, T16) | PASS |
| `pnpm quality` (build + fmt + clippy + tests Rust, incluye `cargo test --workspace --locked`) | PASS — 377 unitarias, 5 persistencia, 1 doc-test; 9 ignoradas |
| `node --test tests/ui/` (agenda-presentation + week-layout) | PASS — 11/11 |
| Semana: dependencias estables (useMemo), «Semana actual» con fecha real de la cuenta, posición de lectura restaurada | PASS (código; medición IPC real NOT CHECKED) |
| Lista: contexto (búsqueda/filtros/página) restaurado; texto distingue coincidencias de página del total del periodo | PASS (código; recorrido real NOT CHECKED) |
| Nueva cita: grupos Paciente/Servicio/Horario, resumen previo, registro junto al selector, dirty solo por acción del usuario, «Cita guardada.» | PASS (código; recorrido con IPC real NOT CHECKED) |

## Comprobaciones ambientales y humanas

| Comprobación | Estado |
|---|---|
| Día/Semana/Lista con IPC real, apertura, retorno y creación | NOT CHECKED |
| Conteo de lecturas IPC al montar/cambiar semana/esperar | NOT CHECKED |
| Crear cita sin doble intención por doble clic en app real | NOT CHECKED |
| Registrar paciente a mitad y recuperar el formulario completo | NOT CHECKED |
| Ficha incompleta, disponibilidad ausente y conservación de valores ante error | NOT CHECKED |
| 800×600 y ventana amplia | NOT CHECKED |
| Revisión expresa del usuario | Pendiente (aceptación final en C14) |

## Notas

- La protección de salida (useBlocker) se evalúa con refs en el momento de la navegación; precarga y selección automática ya no bloquean.
- Los textos de guardado confirman persistencia local sin prometer aceptación compartida ni envío de mensajes.
