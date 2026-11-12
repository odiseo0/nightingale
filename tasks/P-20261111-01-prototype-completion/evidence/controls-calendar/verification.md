# Controles, calendario y jerarquía de la cita

Fecha: 2026-11-12. Iteración visual solicitada por el usuario sobre el prototipo original. No avanza T12 ni cierra checkpoints o la revisión pendiente de la paleta.

## Cambios

- Desplegables compartidos: chevrón Phosphor a la derecha, separación de 6 px, capa con borde y sombra, hover/foco y marca de opción seleccionada. Las opciones largas se ajustan al ancho de la pantalla. Se conserva el elemento select nativo y su validación.
- Cita: paciente y servicio juntos; fecha, horario, duración y acción principal en el bloque de mayor peso; datos de atención agrupados; acciones menos frecuentes en «Más acciones»; comunicaciones e historia como apoyo. Montserrat y Mulish, pesos reales 400/500/600; no se añade color decorativo.
- Fechas: calendario compartido para los campos date de Agenda, citas y formularios. Elección directa de mes/año, día elegido, hoy, confirmación y cancelación. El valor se escribe solo al aceptar; los campos siguen admitiendo entrada con teclado. Popover dentro del modal que lo abre, con retorno del foco. Flechas, Inicio/Fin y Re Pág/Av Pág recorren los días.
- Horarios: días, vigencia y tramos separados; horas con más peso; duración visible por tramo; mensajes breves sobre descansos. Se conservan las comprobaciones de citas afectadas y las reglas existentes.

## Verificación

[Resultados del recorrido](checks.json). Se comprobó el filtro de Agenda con teclado y retorno del foco, selección/aplicación de fecha, mes/año y 29 de febrero, cancelación con Escape y botón, calendario dentro de modal, duración del tramo, rechazo de horario que deja citas fuera, guardado válido y reprogramación con duración conservada. Tamaños revisados: predeterminado, 800×600 y 375×812. Capturas reales del navegador; extensión JPG.

El primer recorrido detectó un selector de año demasiado ancho y un recorte del calendario al descontar la barra de desplazamiento. Ambos quedaron corregidos y se volvió a comprobar la vista.

`node --check design-prototype/app.js` y revisión de espacios con `git diff --check` sobre los archivos del prototipo. No se añaden dependencias ni pruebas que repliquen el marcado.

## Capturas

- [Modal de cita](cita.jpg) y [acciones](acciones-cita.jpg).
- [Desplegable de Agenda](dropdown-agenda.jpg) y [opciones largas en móvil](dropdown-movil.jpg).
- [Calendario](calendario.jpg), [dentro de modal](calendario-en-modal.jpg) y [pantalla estrecha](calendario-movil.jpg).
- [Horarios](horarios.jpg), [editar horario](editar-horario.jpg) y [cita en móvil](cita-movil.jpg).

## Límites

No es una nueva auditoría completa de T01–T11. No se han probado lectores de pantalla, dispositivos táctiles reales ni todos los motores de navegador. En motores sin select personalizable queda el selector nativo; sin Popover API queda el campo date nativo. Las horas usan el formato visual de la configuración del navegador y guardan HH:mm. Los datos siguen siendo ficticios y temporales. La interfaz Tauri, contratos y base de datos quedan fuera de esta iteración.

## Fuentes y reglas aplicadas

DESIGN.md y DESIGN-RULES.md: contraste por peso, tamaño y espacio; color de selección/estado; sombras para capas activas. Laws of UX: similitud entre controles, proximidad entre datos de la misma tarea y convenciones conocidas del calendario. Se conserva la escala semanal por hora solicitada antes.

Plataforma nativa HTML/CSS/JavaScript, sin framework en esta carpeta. Patrones contrastados con documentación oficial:

- [MDN: select personalizable](https://developer.mozilla.org/en-US/docs/Learn_web_development/Extensions/Forms/Customizable_select): appearance base-select en control/picker, icono, opciones y selección; alternativa nativa sin soporte.
- [MDN: Popover API](https://developer.mozilla.org/en-US/docs/Web/API/Popover_API/Using): capa superior, cierre con Escape/clic fuera y popovers anidados.
- [MDN: position-anchor](https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/position-anchor) y [alternativas de posición](https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/position-try-fallbacks): menú de acciones junto al disparador, con alternativa centrada si no hay anclaje CSS.
- [WAI: calendario de selección de fecha](https://www.w3.org/WAI/ARIA/apg/patterns/dialog-modal/examples/datepicker-dialog/): foco inicial, un día en la secuencia Tab, navegación con flechas y retorno al control. La consulta de esta guía no sustituye las pruebas con ayudas técnicas.
