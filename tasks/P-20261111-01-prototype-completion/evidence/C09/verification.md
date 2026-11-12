# C09: Revisión completa — verificación de T21

Fecha: 2026-11-12. Alcance: cierre de cobertura y evidencia del plan [P-20261111-01](../plan.md). El usuario aprobó C05–C07 y autorizó «continúa con T18, T19, T20, T21» el 2026-11-12; C08 y C09 se presentan juntos.

Entorno: prototipo original; `node --check design-prototype/app.js` sin errores; `git diff --check -- design-prototype tasks/P-20261111-01-prototype-completion` sin advertencias de formato.

## Qué se cerró en T21

1. **Catálogo y README.** El catálogo queda en **52 rutas** en 7 grupos: se retira `/lista-espera/ofertas` (decisión del usuario: los pacientes no son leads) y se añaden `/hub/cita-modificada` y `/hub/servicio`. El README actualiza su encabezado, la lista de recorridos (espera sin ofertas, comunicaciones, indicadores, Por revisar, búsqueda/ayuda, políticas, Hub), la tabla del índice con capturas por bloque y la sección de verificación; los enlaces muertos del índice quedaron eliminados.
2. **Cobertura del despachador.** Comprobación estática con Node: las 52 rutas del catálogo resuelven a una rama explícita del despachador; ninguna cae en el respaldo silencioso (Agenda). Las rutas nuevas o cambiadas desde C04 se recorrieron además en navegador este mismo día (C05–C08).
3. **Artefacto de verificación del código final.** `design-prototype/verification-2026-11-12.json` registra fecha, catálogo, cobertura, enlaces a la evidencia C01–C08, tamaños comprobados, límites y el SHA-256 de `index.html`, `styles.css` y `app.js`. `verification.json` (2026-10-22) y los 34 PNG históricos quedan intactos e identificados como históricos.
4. **Hoja de vistas.** `design-prototype/previews/ampliacion-contact-sheet.png` compone 18 capturas post-iteración del código final, por bloque (fuente: `evidence/C09/contact-sheet.html`).
5. **Sin controles falsos.** Las acciones dependientes del escenario lo declaran (reintentos, bloqueos sin conexión) y lo no contratado se muestra de solo lectura con explicación.

## Matriz de cobertura (resumen)

| Dimensión | Estado | Dónde |
|---|---|---|
| Recorridos funcionales de los 14 bloques | Verificado en navegador | C01–C08 |
| 1280×800 | Verificado por bloque | C01–C08 |
| 800×600 con scroll propio | Verificado | C05, C06, C07, C08 |
| 320×720 Hub | Verificado | C08 |
| Teclado (Tab, Escape, Ctrl+K) | Verificado | C01–C08 |
| Cobertura de rutas del catálogo | 52/52 (estática) + recorridos reales por bloque | C09, C01–C08 |
| Texto al 200 % | **No comprobado** (la herramienta no expone zoom de página) | pendiente |
| Lector de pantalla | **No comprobado** | pendiente |
| Pruebas con usuarios | No realizadas | pendiente |

## Límites

- Datos ficticios en memoria; sin servicios, autenticación, SQLite ni sincronización reales.
- La paleta propuesta sigue pendiente de aprobación del usuario.
- H01–H07 de la [auditoría heurística](../heuristics-2026-11-12/evaluacion-heuristica.md) quedan registrados sin corregir por decisión de alcance; H08–H09 se cerraron en T19.

## Estado

- T21: terminada; el plan completo (T01–T21) queda implementado y verificado según la cobertura declarada.
- C08 y C09: pendientes de revisión del usuario.
