# Matriz de accesibilidad y adaptación — P-20261109-22 (T37)

**Fecha:** 2026-11-09
**Criterio:** WCAG 2.2 AA por superficie. N/A y NOT CHECKED no cuentan como aprobados. Zoom real, lector de pantalla y validación humana no se sustituyen por inspección de código.

## Cobertura automática (evidencia real)

| Superficie / aspecto | Método | Resultado |
|---|---|---|
| Política Profesional (rol × nº profesionales) | node:test agenda-presentation | PASS |
| Dirección de lectura sin país; actividad traducida | node:test agenda-presentation | PASS |
| Fin de intervalo 24:00 | node:test agenda-presentation | PASS |
| Geometría semanal (rango, escala, solapamientos) | node:test week-layout | PASS |
| Clasificación de intervención de revisiones | node:test reviews | PASS |
| Enlaces de revisión por clave de entidad | node:test reviews | PASS |
| Tipos, build y lint de todo el frontend | pnpm build, pnpm quality | PASS |
| Reglas de dominio, persistencia y sincronización | cargo test --workspace --locked | PASS (377+5+1; 9 ignoradas) |

## Matriz por superficie

| Superficie | 800×600 | 1024×768 | ≥1440 | Zoom 200 % | Teclado/foco | Lector | Estados (vacío/error/largo) |
|---|---|---|---|---|---|---|---|
| Shell y navegación | NC | NC | NC | NC | NC | NC | NC |
| Búsqueda global (Ctrl+K) | NC | NC | NC | NC | NC | NC | NC |
| Agenda Día | NC | NC | NC | NC | NC | NC | NC |
| Detalle de cita | NC | NC | NC | NC | NC | NC | NC |
| Semana | NC | NC | NC | NC | NC | NC | NC |
| Lista de citas | NC | NC | NC | NC | NC | NC | NC |
| Nueva cita / Reprogramar | NC | NC | NC | NC | NC | NC | NC |
| Acciones del ciclo | NC | NC | NC | NC | NC | NC | NC |
| Directorio y ficha de paciente | NC | NC | NC | NC | NC | NC | NC |
| Registro/edición de paciente | NC | NC | NC | NC | NC | NC | NC |
| Cuenta / Servicios / Horarios | NC | NC | NC | NC | NC | NC | NC |
| Por revisar | NC | NC | NC | NC | NC | NC | NC |
| Acceso y preparación | NC | NC | NC | NC | NC | NC | NC |
| Ayuda / Diagnóstico | NC | NC | NC | NC | NC | NC | NC |

NC = NOT CHECKED: aplicable pero sin evidencia en entorno (navegador tomado por el usuario en sesiones previas; lector no disponible; validación humana pendiente).

## Diseño aplicado en código (hipótesis verificables en revisión)

- Foco visible global (`:focus-visible` 2 px azul con desplazamiento) y `prefers-reduced-motion` respetado en App.css.
- Diálogos con `<dialog>`/`showModal()`: condición modal, Escape y retorno de foco nativos.
- Skip link «Ir al contenido»; landmarks (nav/header/main); tablas con `th scope="col"`.
- Nombres accesibles en controles de icono (`aria-label`) y estados seleccionados semánticos (`aria-current`, `aria-selected`, `aria-pressed`).
- Texto, además del color, en estados (etiquetas de estado, canceladas con tachado + texto, bloqueos con trama + etiqueta).
- Contraste documentado en DESIGN.md para los pares principales; las combinaciones reales requieren medición en revisión.

## Pendientes de la auditoría humana

- Recorrido completo con teclado y lector por superficie.
- Zoom real al 200 % y reflow en los cuatro tamaños.
- Validación con médico/asistente (agenda, paciente, crear/reprogramar, conflicto).
- macOS/Linux fuera del entorno de desarrollo actual (Windows).
