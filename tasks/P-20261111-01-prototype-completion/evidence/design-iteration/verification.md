# Iteración de diseño · Horas, modal y color

2026-11-11. Petición expresa del usuario: priorizar diseño/experiencia, separar Semana por horas, abrir citas en modal y presentar colores funcionales para su revisión. Cambio limitado al prototipo original. No modifica Tauri, contratos ni las especificaciones de diseño aprobadas.

## Cambios

- Semana: ocho divisiones de una hora por día, 96 px/hora. La posición y duración de cada cita siguen calculadas por minuto; quitar las líneas intermedias no cambia los intervalos de las citas.
- Modal nativo independiente del diálogo de confirmación: nombre, estado, datos administrativos, acciones, comunicaciones y actividad desplegable. El encabezado conserva el cierre visible al desplazar el contenido. El fondo queda fuera de la interacción y no se desplaza con el modal abierto.
- Cierre por botón, Escape y clic en el fondo; Atrás/Adelante reflejan la apertura. Al cerrar vuelven foco, filtros, búsqueda y desplazamiento. Desde una ficha de paciente se conserva esa ficha como origen.
- Propuesta de color compartida entre etiquetas, calendario y detalle. Las fichas completas quedan neutras; las incompletas muestran «Por completar» en ámbar. «Colores propuestos», fuera del trabajo diario, explica los roles sin añadir una leyenda permanente al calendario.

## Paleta pendiente de revisión

| Rol | Texto/borde | Fondo | Uso | Contraste del texto |
|---|---|---|---|---|
| Azul | #1D4ED8 | #DBEAFE | Programada, información de cita aún sin confirmar | 5,49:1 |
| Verde | #166534 | #DCFCE7 | Confirmada, Entregado | 6,49:1 |
| Ámbar | #92400E | #FEF3C7 | Ausencia, ficha por completar, solicitud que requiere atención | 6,37:1 |
| Rosa | #9F1239 | #FFF1F2 | Cancelada | 7,30:1 |
| Neutro | #475569 | #F1F5F9 | Atendida, ficha completa y datos habituales | Paleta existente |

Relaciones calculadas para esos pares sólidos. No acreditan toda la accesibilidad del prototipo. El azul de acciones, foco y selección conserva sus usos previos; la forma, el contexto y el texto distinguen su función. No se usa un color distinto por paciente, profesional o servicio. Ámbar y rosa son valores propuestos, todavía sin aprobación para DESIGN.md.

## Verificación realizada

[checks.json](checks.json) registra resultados del navegador. Pruebas centradas en los cambios:

- Cinco días × ocho divisiones, altura medida de 96 px; citas conservan sus alturas de 30/45/60 minutos y el ejemplo de 15 minutos.
- Modal en Semana sin panel lateral; primer destino de Tab en el cierre, recorrido de teclado dentro del modal y fondo no interactivo mediante showModal.
- Confirmación de cancelación sobre el detalle: cerrar vuelve a la misma cita sin cambiarla. Confirmar cancelación actualiza estado y retira mutaciones del terminal.
- Escape vuelve a Semana y al enlace de la cita. Atrás cierra y Adelante reabre.
- Lista conserva búsqueda «Ana» y foco. Semana conserva exactamente el desplazamiento observado con el modal abierto y después de cerrarlo.
- Desde la ficha de Sofía se abre el modal; reprogramar a 27 de octubre 10:00 conserva sus 45 minutos, vuelve al modal y cerrar regresa a la ficha.
- Confirmar asistencia actualiza el estado y su color en el modal. Solo lectura no ofrece acciones de modificación.
- Ventanas 1280×800, 800×600 y 375×812. Sin desbordamiento horizontal del documento en las comprobaciones de modal estrecho/móvil; nombres largos se ajustan y el contenido tiene desplazamiento propio.
- Ausencia y Cancelada muestran los pares ámbar/rosa previstos. Sin errores de consola observados.
- `node --check design-prototype/app.js` y `git diff --check -- design-prototype tasks/P-20261111-01-prototype-completion`: correctos.

Capturas: [Semana](week-hour-lines.jpg), [modal](appointment-modal.jpg), [modal 800](modal-800.jpg), [modal móvil](modal-mobile.jpg), [paleta](palette.jpg).

## Leyes aplicadas y código relevante

Semejanza: el estado conserva el mismo color entre vistas. Von Restorff: el dato por completar destaca y el caso normal se mantiene neutro. Proximidad: el modal reúne datos y acciones. Jakob: modal nativo con Escape y comportamiento de foco conocido. No se añaden límites arbitrarios al número de controles.

```css
.week-cell { height: 96px; border-left: 0; }
.badge.attention { background: var(--amber-soft); color: var(--amber); }
.week-event.attention {
  background: var(--amber-soft);
  border-left-color: var(--amber);
}
```

El modal usa la plataforma web nativa, sin biblioteca nueva: [HTMLDialogElement.showModal](https://developer.mozilla.org/en-US/docs/Web/API/HTMLDialogElement/showModal) para la capa modal y el fondo no interactivo; [cancel](https://developer.mozilla.org/en-US/docs/Web/API/HTMLDialogElement/cancel_event) para hacer que Escape cierre también la ruta. El diálogo de confirmación reutiliza el diálogo ya existente encima del detalle; cerrar esa confirmación no pierde el detalle.

## Límites y decisión pendiente

La paleta necesita revisión del usuario. Esta iteración no aprueba C03/C04/C05 ni avanza T12. No incorpora las correcciones de los otros hallazgos de la revisión T01–T11. El ejemplo mantiene los límites anteriores de datos en memoria y horario visible; no acredita autenticación, persistencia o sincronización reales. Lector de pantalla, zoom de texto al 200 % y pruebas con usuarios: NOT CHECKED.
