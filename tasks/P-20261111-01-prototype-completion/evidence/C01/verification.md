# C01 · Base visual y navegación

Fecha: 2026-11-11. T01 y T02 ejecutadas en orden. **C01 espera revisión del usuario.** T03 no ha comenzado; este informe no cierra el objetivo completo.

## Resultado

- Montserrat para encabezados y Mulish para lectura, navegación, tablas y formularios. Recursos locales idénticos por SHA-256 a los del repositorio; origen y licencia completos en `design-prototype/assets/fonts-LICENSE.txt`.
- Pesos definidos 400/500/600, sin características específicas de Inter ni síntesis de fuentes.
- 28 iconos Phosphor 2.1.10 regular incrustados como SVG en `app.js`, con licencia MIT local. Sin React ni solicitudes a un paquete/CDN al abrir el prototipo. No se requiere un sprite externo que pueda fallar al abrir `index.html` por archivo.
- Se conservan estructura, paleta, tablas abiertas y geometría del prototipo original.
- Desconexión con trabajo local permitido y reintentos automáticos sin avisos diarios. El conflicto aparece junto a la cita y mediante «Por revisar», con enlace condicional. Las pantallas ajenas a la cita no repiten el aviso.
- Dueña con un profesional: sin filtro/columna Profesional. Asistente o variante con varios: se conservan. Los registros mantienen professionalId; las variantes son datos de revisión, sin ampliar capacidades reales.
- Breadcrumbs sin enlace redundante al elemento actual. Al cerrar el detalle, vuelven fecha, filtro y foco a su cita. El cambio de un control conserva foco cuando existe el mismo elemento.
- Menú estrecho con cierre, Tab/Shift+Tab dentro de navegación, Escape y retorno al botón inicial; el fondo usa inert mientras está abierto. «Ir al contenido» enfoca main sin sustituir la ruta.

## Verificación realizada

| Comprobación | Resultado y evidencia |
|---|---|
| Sintaxis | `node --check design-prototype/app.js` correcto |
| Cambios | `git diff --check -- design-prototype tasks/P-20261111-01-prototype-completion` correcto |
| Tipografía | Agenda/ficha/editor a 1280×800 y 800×600; estilos calculados Montserrat/Mulish y sin desbordamiento del documento; seis capturas T01 |
| Recursos | Hashes de ambas fuentes coinciden con origen local; rutas Phosphor extraídas del paquete instalado con licencia conservada |
| Vistas existentes | 34 rutas del catálogo × dos tamaños: 68 cargas con encabezado, SVG consistentes y sin desbordamiento del documento; no implica verificación funcional completa de cada vista |
| Roles/escenarios | Seis combinaciones a 1280×800: dueña/un profesional, asistente/un profesional, dueña/varios, normal, offline, servicio no disponible y conflicto; ver `shellChecks` en checks.json |
| Tabla del asistente | A 800×600: documento clientWidth=785 y scrollWidth=785; tabla conserva desplazamiento propio (658 px de contenido en 547 px). Captura `T02-asistente-800x600.jpg` |
| Filtro inaccesible | Elegir Martín en variante de varios y volver a asistente/un profesional limpia la selección no disponible y vuelve a las ocho citas; no presenta un hueco ficticio |
| Retorno | Fecha 2026-10-23 y filtro Programada; abrir/cerrar a9 vuelve a esos valores y al enlace de la cita |
| Menú | A 375×812: apertura, fondo inert, Tab/Shift+Tab en los extremos, Escape y foco «Abrir menú»; sin pérdida de la cuenta |
| Acceso al contenido | Enlace de salto enfoca main y conserva `#/agenda` |
| Consola | Sin errores registrados en la pestaña de revisión |

Los datos estructurados están en [checks.json](checks.json). La muestra principal es [C01-agenda.jpg](C01-agenda.jpg). Las capturas T01 documentan la tipografía antes de T02; no se presentan como estado final de la navegación. Las capturas originales en `design-prototype/previews/` y su verification.json siguen siendo históricas.

### Corrección encontrada durante la revisión

La columna Profesional introducía un desbordamiento en la ventana de 800 px. El rótulo accesible oculto del encabezado Acciones tenía posición absoluta sin un contenedor situado dentro de la tabla. Su caja escapaba del área con desplazamiento y ampliaba el documento. Se estableció `position:relative` en `.table-wrap`, conservando su desplazamiento propio. Se descartó una prueba con contención de tamaño que no resolvía la causa. La comprobación del asistente posterior devolvió ancho y scrollWidth iguales en el documento.

## Documentación oficial consultada

- CSS nativo: [rango de peso en fuentes variables](https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/At-rules/@font-face/font-weight#setting_a_range_for_a_variable_font); se declaran los rangos 400–600.
- [Phosphor React 2.1.10](https://github.com/phosphor-icons/react/tree/v2.1.10): peso regular, viewBox de 256 y color heredado. Se conserva el trazado del paquete instalado.
- Solo para obtener el SVG durante la edición: React/React DOM **19.3.1** ya instalados; [renderToStaticMarkup](https://react.dev/reference/react-dom/server/renderToStaticMarkup) produce una cadena estática. No se introduce React en el prototipo ni se añade un generador al producto.
- DOM nativo: [focus({preventScroll:true})](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement/focus#parameters) para conservar o devolver foco sin saltos añadidos.
- DOM nativo: [inert](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement/inert) para excluir el fondo de la interacción cuando abre la navegación estrecha.

## Límites

- Datos ficticios en memoria. Sin autenticación, permisos reales, SQLite, reinicio, aceptación de reservas, comunicaciones o llamadas a servicios.
- Los estados de cita, formularios administrativos, agenda semanal y funciones futuras aún conservan límites del prototipo histórico: sus ampliaciones pertenecen a T03–T21.
- NOT CHECKED: lector de pantalla, validación con médico/asistente/paciente y auditoría completa de accesibilidad.
- NOT CHECKED en este bloque: revisión completa a 1024×768 y texto al 200 %; siguen en la verificación final prevista, sin atribuirles resultados de otras dimensiones.
- Pendiente: aceptación expresa del usuario en C01. No se autoaprueba con estas comprobaciones.
