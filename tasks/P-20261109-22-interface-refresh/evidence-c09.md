# Evidencia C09 — Lectura de configuración (después de T23–T26)

**Fecha:** 2026-11-09
**Alcance:** Fase H del plan P-20261109-22 (T23 Cuenta, T24 Servicios, T25 Listas de Horarios, T26 Editores de Horarios).

## Resultados técnicos

| Comprobación | Resultado |
|---|---|
| `pnpm build` (T23–T26) | PASS |
| `pnpm quality` (build + fmt + clippy + tests Rust, incluye `account`, `services`, `scheduling`) | PASS — 377 unitarias, 5 persistencia, 1 doc-test; 9 ignoradas |
| Cuenta: sin versión/fecha técnica en la vista; canales y permisos separados; textos de resultado sin jerga | PASS (código; perfiles reales NOT CHECKED) |
| Servicios: catálogo/oferta sin sincronización visible; efecto sobre citas anteriores explicado | PASS (código; ejecución NOT CHECKED) |
| Horarios: tres secciones con vigencias y ámbito; motivo traducido; sin columnas técnicas | PASS (código; dueño/asistente NOT CHECKED) |
| Editores: campos pertinentes, fechas/horas nativas, rechazo conserva el formulario, intención única | PASS (código; rechazo real NOT CHECKED) |

## Comprobaciones ambientales y humanas

| Comprobación | Estado |
|---|---|
| Dueño/asistente, datos largos, versión interna y retorno en las tres superficies | NOT CHECKED |
| Servicio inactivo, oferta y modificación de un servicio usado por una cita | NOT CHECKED |
| Regla con vigencia, cierre/reemplazo y bloqueo global restringido | NOT CHECKED |
| 800×600 y zoom | NOT CHECKED |
| Revisión expresa del usuario | Pendiente (aceptación final en C14) |

## Notas

- Los datos contractuales (país, versiones) se conservan; la vista diaria solo retira su lectura técnica.
- Los bloqueos globales muestran «No registrado» cuando el contrato no entrega motivo; no se infiere.
