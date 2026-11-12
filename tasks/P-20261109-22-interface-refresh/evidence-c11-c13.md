# Evidencia · C11–C13 (entregas de diseño T30–T36)

**Plan:** [plan.md](plan.md) · **Fecha:** 2026-11-09 · **Alcance:** solo trabajos de diseño y prototipo autorizados por el usuario al aprobar C03.
**Revisión:** C11, C12 y C13 aprobados por el usuario el 2026-11-09 («Apruebo estos cambios si necesitan aprobación»). Las NOT CHECKED de este documento conservan ese estado.

## Qué se entrega

| Tarea | Especificación | Muestra |
|---|---|---|
| T30 Solicitudes | [design/requests.md](design/requests.md) | `#requests` en index.html |
| T31 Lista de espera | [design/waitlist.md](design/waitlist.md) | `#waitlist` en index.html |
| T32 Comunicaciones | [design/communications.md](design/communications.md) | `#communications` en index.html |
| T33 Indicadores | [design/metrics.md](design/metrics.md) | `#metrics` en index.html |
| T34 Equipo y dispositivos | [design/team.md](design/team.md), [design/devices.md](design/devices.md) | `#team` en index.html |
| T35 Política de comunicaciones | [design/communication-policy.md](design/communication-policy.md) | `#policy` en index.html |
| T36 Patient Hub | [design/hub-handoff.md](design/hub-handoff.md) | hub.html (móvil, desde 320 px) |

## Resultados

| Comprobación | Resultado |
|---|---|
| Sintaxis `node --check app.js` | PASS |
| Integridad de rutas/secciones (script de verificación de ids) | PASS: todas las secciones de `designRoutes` existen en index.html; las seis secciones de hub.html existen |
| `git diff --check` | PASS (solo avisos CRLF en archivos Rust preexistentes del trabajo T05–T07) |
| Render `#requests` a 1440×900 (navegador integrado, HTTP local) | PASS: lista, detalle deshabilitado, estado vacío y marca «Diseño futuro · sin contrato» visibles |
| Caché de estilos durante la inspección | Detectado: el navegador sirvió styles.css en caché tras un cambio; el servidor sí servía la versión nueva. Se requiere recarga forzada en la revisión humana |
| Hueco vertical en `#requests` por `min-height` heredado | Corregido: `min-height:600px` acotado a `#day .day-layout` |

## NOT CHECKED (causa y siguiente comprobación)

- **Inspección visual completa de las siete vistas nuevas y hub.html**: el usuario tomó el control del navegador integrado durante la verificación; solo `#requests` se inspeccionó antes del takeover. Siguiente: recorrer los pasos 1–9 del README de la muestra con recarga forzada.
- **Interacción de detalle en Solicitudes** (abrir fila, comprobación de la cita sin modificar): pendiente del mismo recorrido.
- **800×600 y 1440×900 de las vistas nuevas; hub.html a 320 px**: pendiente.
- **Teclado, foco, zoom real 200 %, lector y contraste real**: no verificados; no se declara conformidad de accesibilidad.
- **Recorrido humano de las especificaciones contra contrato**: los documentos se revisaron contra el maestro de diseño, los planes bloqueados P-20261109-11/12/13/14/15/16/19 y el contrato API v1 (que no define estos dominios); la revisión de detalle es del usuario.

## Límites conservados

- Sin código de producto: T08–T29 y T37–T39 permanecen sin ejecutar.
- Sin funciones operativas ficticias: botones de las vistas nuevas deshabilitados; hub.html no ejecuta acciones; los datos son sintéticos y están marcados como diseño.
- Sin contratos inventados: cada especificación lista sus bloqueantes y pendientes explícitos.

## Verificación en navegador integrado ( continuación 2026-11-09 )

Render por HTTP local en el navegador integrado, sin toma de control por el usuario:

| Vista | Tamaño | Resultado |
|---|---|---|
| Día | 1440×900 | PASS (jerarquía fecha+título, tabla Hora/Paciente/Servicio/Estado, sin sincronización) |
| Semana | 1440×900 | PASS (96 px/hora, simultáneas en columnas, cancelada tachada, bloqueo con trama, scroll propio) |
| Solicitudes | 1440×900 | PASS tras corrección (ver abajo) |
| Editor (Nueva cita) | 1440×900 | PASS (grupos Paciente/Servicio/Horario, registro junto al selector, resumen con dirección sin país) |
| Hub móvil (hub.html) | 375×667 | PASS (una columna, lectura clara, datos sintéticos identificados) |

Defecto propio encontrado y corregido en la muestra: el encabezado compartido de Agenda permanecía visible sobre las vistas de «Diseño futuro» y duplicaba su título (`selectView()` ocultaba vistas/barra/botón pero no el encabezado). Corrección: `index.html` añade `id="agenda-heading"` y `app.js` lo oculta con el resto del cromo de Agenda en editor, registro y diseño. Verificado tras el arreglo.

Pendientes conservados: 800×600 y zoom 200 % de la muestra, interacciones (teclado/detalles desplegables), y toda la verificación humana/ambiental de la app de producto (NOT CHECKED, no cuentan como aprobados).
