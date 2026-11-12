# Escala tipográfica y longitud de lectura

2026-11-12. Petición expresa del usuario: 45–75 caracteres por línea, tamaños de cuatro en cuatro y títulos compactos frente a texto pequeño con más aire. Iteración sobre el prototipo original y sus reglas; sin avance de T12 ni aprobación inferida de checkpoints o paleta.

## Aplicación

- DESIGN.md: escala actualizada; se retiran roles inferiores a 12px y tamaños intermedios; ejemplos y recomendaciones alineados. DESIGN-RULES.md y la sección 17.1 del plan maestro de diseño incorporan la petición para trabajos posteriores.
- Prototipo: tokens tipográficos de 12/16/20/24/28/32px en la referencia de raíz de 16px, expresados en rem y con raíz al 100 %. Se mantienen Montserrat, Mulish y los pesos 400/500/600.
- Títulos con interlineado 1.2; texto de apoyo de 12px con interlineado 1.666667 (20px). Lectura y controles principales de 16px con 1.5. Los nombres del calendario semanal usan 16/20 para conservar el intervalo real de la cita.
- Párrafos limitados a 58ch. Se empezó con 60ch; una línea de texto real llegaba a 77 caracteres, por lo que se ajustó la medida. No se fuerza una anchura mínima en móvil, tarjetas o modales estrechos. Las últimas líneas y los textos cortos no tienen que llegar a 45 caracteres.
- Se reduce el relleno vertical de los eventos semanales para que el texto nuevo quepa en sus 15/30/45/60 minutos. Se mantienen los 96px por hora, los días mínimos y los separadores horarios.
- El calendario de fechas se reposiciona al desplazar su contenedor. Así el desplazamiento que hace el navegador al enfocar un campo no lo cierra al abrirlo con la tipografía nueva.

## Comprobación

[Medidas reales del navegador](checks.json): Agenda, Cuenta, Horarios, calendario dentro de modal, Semana, cita con nombre largo y editor. Vista inicial 808×1061 y prueba a 375×812. Se retiró la medida temporal del navegador al terminar. Todas las muestras registradas presentan tamaños de la escala y no tienen desbordamiento horizontal del documento; tablas y Semana conservan su desplazamiento propio.

Se midieron caracteres con rangos DOM sobre párrafos reales, incluidos espacios. En las líneas completas de la muestra de lectura se observaron 65, 71, 72 y 74 caracteres. Las últimas líneas quedan más cortas cuando el texto termina. La longitud es un objetivo de lectura, no un recuento literal de la unidad ch.

Las citas semanales inspeccionadas mantienen 20/44/68/92px de altura para 15/30/45/60 minutos, con scrollHeight igual a clientHeight en las muestras de contenido. Se revisaron títulos de varias líneas y el acceso al calendario desde un modal y desde reprogramación. Escape mantiene el formulario y su fecha. No se guardaron cambios de citas ni de horarios. El registro del navegador no devolvió errores ni advertencias de la aplicación.

Comprobaciones locales: auditoría de los tamaños declarados y calculados; `node --check design-prototype/app.js`; `git diff --check` sobre los archivos cambiados. No se añaden dependencias ni pruebas de marcado.

## Capturas

- [Horario](horarios.jpg) y [Semana](semana.jpg).
- [Calendario](calendario.jpg) y [calendario móvil](calendario-movil.jpg).
- [Cita con nombre largo en móvil](cita-movil.jpg) y [retorno a la cita](cita.jpg).

No se ha repetido una auditoría completa de todas las rutas, ni comprobado zoom al 200 %, lectores de pantalla, otros motores o dispositivos táctiles reales. Esta iteración no traslada los estilos al producto Tauri.

## Fuentes de implementación

Plataforma HTML/CSS/JavaScript del prototipo, sin framework. Se reutilizan estilos y controles existentes. Las reglas de proximidad conservan la relación entre etiquetas, datos y secciones al abrir el interlineado.

- [MDN: ch y rem](https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Values/length#relative_length_units_based_on_font): ch mide el avance del cero; no equivale a un carácter cualquiera de Mulish. Las medidas relativas conservan la relación con el tamaño de fuente.
- [MDN: interlineado sin unidad](https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/line-height#prefer_unitless_numbers_for_line-height_values): el valor escala con el tamaño de cada elemento.
