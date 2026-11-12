# Jerarquía tipográfica e iniciales · 2026-11-11

Petición del usuario: tomar de las referencias la jerarquía de peso/color del texto, aprovechar iconos y usar iniciales con fondos variados para las personas. Se conserva la estética sencilla, el contenido propio y el alcance del prototipo. No se cambian modelos, contratos, base de datos ni la paleta aprobada de DESIGN.md.

## Propuesta aplicada

- Nombre del paciente y hora inicial en tablas: Mulish 600, #111827. Servicio y datos principales: peso 500. Etiquetas, hora final, notas y metadatos: peso 400 y grises existentes #475569/#5B6778. No se usa opacidad para debilitar texto necesario.
- Modal: avatar de la persona en el encabezado, fecha y hora a 16 px/600, etiquetas grises con iconos Phosphor de calendario, reloj, servicio, profesional, lugar y contacto. Cada icono conserva su etiqueta visible y queda fuera del nombre accesible mediante el SVG compartido.
- Tabla: iconos de propiedad en encabezados; no se añade un icono repetido a cada celda. El calendario conserva nombres con peso 600 y datos de apoyo más discretos.
- Iniciales en Agenda, directorio, ficha, modal, responsables, búsqueda y perfil. La asignación toma el identificador estable de la persona, no su nombre. Dos pacientes llamados Ana María Rodríguez conservan sus respectivas claves y en el ejemplo usan tonos distintos. Las iniciales se muestran en mayúsculas.
- Seis fondos suaves con letras oscuras. La asignación es variada y determinista, no cambia en cada render. Los colores pueden repetirse: no reemplazan nombre/contacto ni expresan estado de cita.
- «Colores propuestos» incluye una muestra de los seis fondos. La propuesta continúa pendiente de revisión; no se interpreta este cambio como aceptación de la paleta previa.

## Fondos de iniciales

| Tono | Letra | Fondo | Contraste calculado |
|---|---|---|---|
| Azul | #1E40AF | #DBEAFE | 7,15:1 |
| Violeta | #5B21B6 | #EDE9FE | 7,57:1 |
| Rosa | #9D174D | #FCE7F3 | 6,71:1 |
| Verde azulado | #115E59 | #CCFBF1 | 6,73:1 |
| Melocotón | #9A3412 | #FFEDD5 | 6,38:1 |
| Cian | #155E75 | #CFFAFE | 6,49:1 |

Cálculos de pares sólidos; no acreditan accesibilidad completa. Los grises mantienen los tokens actuales. La ampliación violeta/cian/melocotón se limita a las iniciales de identidad; no añade otros colores de estado, gráficos ni fondos grandes.

## Comprobaciones

[checks.json](checks.json) registra la revisión en navegador:

- Filtrar Ana conserva `p1 / avatar--cyan`.
- La misma persona conserva ese tono en el modal y después de recargar.
- Editar el nombre de p1 conserva clave y tono. Después se recargó el ejemplo para retirar ese cambio de prueba.
- Los homónimos p1 y p11 conservan claves distintas y tonos cian/azul en el directorio.
- Nombres a peso calculado 600; fecha del modal a 16 px y peso 600; etiquetas #5B6778; seis iconos con sus etiquetas.
- Responsables c1/c2 con sus iniciales y claves; sin desbordamiento horizontal observado en la ficha a 1280×800.
- Modal a 800×600 y nombre largo a 375×812 sin desbordamiento horizontal del documento ni del diálogo.
- Semana conserva ocho divisiones por día de 96 px y el modal sigue cerrando con Escape. No se cambian intervalos ni reglas de cita.
- Sin errores de consola observados. `node --check design-prototype/app.js` y `git diff --check -- design-prototype tasks/P-20261111-01-prototype-completion`: correctos.

Capturas: [Agenda](agenda.jpg), [modal](appointment-modal.jpg), [modal móvil](modal-mobile.jpg), [ficha y responsables](patient-responsibles.jpg), [muestra de color](palette.jpg).

## Leyes y cambio mínimo

Semejanza y continuidad: una persona conserva su apariencia entre vistas. Proximidad: icono y etiqueta quedan juntos. Von Restorff: nombres/horarios destacan sobre metadatos; el color de estado conserva un propósito distinto. Se reutilizan Mulish, Montserrat, Phosphor y el helper de avatar, sin nueva dependencia.

```css
.data-table .name { font-weight:600; color:var(--ink); }
.data-table .time .subtle { font-weight:400; color:var(--subtle); }
.avatar.avatar--cyan { background:#CFFAFE; color:#155E75; }
```

No se implementa la sugerencia futura de guardar un tono en la base de datos. El prototipo usa el identificador como clave para obtener un tono estable, sin persistencia añadida. Pruebas con usuarios, lector de pantalla y texto al 200 %: NOT CHECKED. La revisión se centra en presentación; no acredita correcciones de los otros hallazgos funcionales ni avanza tareas pendientes.
