# Evidencia C07 — Citas y acceso a pacientes (después de T17–T19)

**Fecha:** 2026-11-09
**Alcance:** Fase F del plan P-20261109-22 (T17 Reprogramar, T18 Acciones del ciclo, T19 Directorio de pacientes).

## Resultados técnicos

| Comprobación | Resultado |
|---|---|
| `pnpm build` (T17, T18, T19) | PASS |
| `pnpm quality` (build + fmt + clippy + tests Rust, incluye `appointments` y `patients`) | PASS — 377 unitarias, 5 persistencia, 1 doc-test; 9 ignoradas |
| Reprogramar: comparación estable, propuesta conservada ante error, intención única, mensaje limitado al guardado local | PASS (código; recorrido sintético NOT CHECKED) |
| Ciclo: confirmaciones con sujeto/fecha/hora/consecuencia sin detalles API; resultado «Cita guardada.»; estados terminales sin reapertura | PASS (código; transiciones reales NOT CHECKED) |
| Directorio: sin sincronización por fila, completitud/estado separados, contexto y foco restaurados | PASS (código; recorrido real NOT CHECKED) |

## Comprobaciones ambientales y humanas

| Comprobación | Estado |
|---|---|
| Reprogramar, confirmar, atender, ausencia y cancelar con datos sintéticos conservando reglas | NOT CHECKED |
| Directorio con nombre largo, teléfono ausente y teclado | NOT CHECKED |
| 800×600 en los tres recorridos | NOT CHECKED |
| Revisión expresa del usuario | Pendiente (aceptación final en C14) |

## Notas

- Las transiciones de estado, momentos y márgenes los sigue decidiendo Rust; la interfaz solo ofrece acciones válidas según el estado conocido.
- Ningún texto promete liberación de horario, aceptación compartida ni envío de mensajes.
