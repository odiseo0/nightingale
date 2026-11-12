# Evidencia C08 — Pacientes (después de T20–T22)

**Fecha:** 2026-11-09
**Alcance:** Fase G del plan P-20261109-22 (T20 Ficha administrativa, T21 Registro/edición con documento, T22 Recorrido del menor).

## Resultados técnicos

| Comprobación | Resultado |
|---|---|
| `pnpm build` (T20, T21, T22) | PASS |
| `pnpm quality` (build + fmt + clippy + tests Rust, incluye `patients`) | PASS — 377 unitarias, 5 persistencia, 1 doc-test; 9 ignoradas |
| Ficha: identidad/contacto/estado dominan; documentos y responsables agrupados; «No registrado» en ausencias; sin columnas técnicas | PASS (código; datos reales NOT CHECKED) |
| Registro/edición: línea base separa precarga de edición (useBlocker solo con cambios del usuario) | PASS (código; recorrido NOT CHECKED) |
| Menor: campos condicionales, relación/vigencia, anti-duplicación en reintentos (paciente/contacto/vínculo) | PASS (código; menor sintético NOT CHECKED) |
| Retorno a borrador de cita tras registrar ficha | PASS (flujo existente verificado; recorrido real NOT CHECKED) |

## Comprobaciones ambientales y humanas

| Comprobación | Estado |
|---|---|
| Ficha completa/incompleta, adulto/menor y alta desde cita con retorno al borrador | NOT CHECKED |
| Salir sin cambios / salir con cambios / volver desde error | NOT CHECKED |
| Rechazo de documento conservando valores | NOT CHECKED |
| 800×600 y zoom | NOT CHECKED |
| Revisión expresa del usuario | Pendiente (aceptación final en C14) |

## Notas

- Una edición concurrente ajustó los textos de guardado del editor a «…en este equipo.» (persistencia local, sin promesa de aceptación); se respetó y se conservó.
- Ningún texto promete aceptación compartida; los pasos multi-paso muestran progreso real del trabajo del usuario.
