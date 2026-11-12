# Evidencia C04 — Navegación (después de T08–T10)

**Fecha:** 2026-11-09
**Alcance:** Fase C del plan P-20261109-22 (T08 Controles y diálogos, T09 Estructura general, T10 Búsqueda por teclado).

## Resultados técnicos

| Comprobación | Resultado |
|---|---|
| `pnpm build` (T08, T09, T10) | PASS |
| `pnpm quality` (build + `cargo fmt --check` + clippy `-D warnings` + tests Rust) | PASS — 377 unitarias, 5 persistencia, 1 doc-test; 9 ignoradas |
| Modal común `<dialog>`+`showModal()` migrado en 7 consumidores | PASS (compilación y tipos) |
| Dependencia `@fontsource/anek-latin` retirada | PASS (package.json + pnpm-lock.yaml) |
| Rutas conservadas, incluida `/cambios` (rótulo «Por revisar») | PASS (App.tsx sin cambios de rutas) |
| Política de visibilidad de «Por revisar» (solo conflicto+rechazo+revisión requerida) | PASS (código; estados reales sintéticos NOT CHECKED) |
| Teclado de búsqueda (flechas/Enter/Escape desde campo y resultados) | PASS (compilación; interacción real NOT CHECKED) |

## Comprobaciones ambientales y humanas

| Comprobación | Estado |
|---|---|
| Abrir/cerrar modales por ratón y teclado en app nativa, retorno de foco al disparador | NOT CHECKED |
| Recorrer Tab/Shift+Tab dentro del diálogo modal | NOT CHECKED |
| Navegación estrecha (menú hamburguesa) a 800×600 con retorno de foco | NOT CHECKED |
| Búsqueda con IPC real (dos resultados con flechas, Enter, Escape, consulta con error) | NOT CHECKED |
| Cambiar cuenta autorizada sin mezclar respuestas | NOT CHECKED |
| Zoom real 200 % en shell y búsqueda | NOT CHECKED |
| Revisión expresa del usuario del checkpoint | Pendiente (la instrucción de ejecución autoriza continuar; la aceptación final se solicitará en C14) |

## Decisiones aplicadas

- Diálogo modal basado en `<dialog>` y `showModal()`; el navegador aporta condición modal, inactividad del fondo, Escape y retorno de foco. Fuente: https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/dialog#accessibility
- «Por revisar» solo aparece con asuntos que requieren intervención; el proceso normal (pending/in_flight/unknown/retryable) no genera contador ni aviso.
- Desarrollo deja de ser destino principal; sigue accesible desde Ayuda y en `/desarrollo`.
- Sidebar 216 px; topbar 52 px; navegación agrupada (trabajo diario / utilidades).
- Los pendientes NOT CHECKED no se cuentan como aprobados; se verificarán en la revisión humana final (C13/C14) o antes si el entorno lo permite.
