# Nightingale · Muestra de renovación

Muestra independiente del producto. C01 y C02 aceptados por el usuario para las cuatro vistas base. Abrir [index.html](index.html) en un navegador local. No requiere paquetes ni red. Usar solo datos ficticios.

## Vistas

- [Día](index.html#day): abrir y cerrar una fila muestra su detalle. El detalle ocupa 352 px al lado en ventana amplia y una superficie propia en estrecho.
- [Semana](index.html#week): cada evento abre su detalle completo. «Volver a Semana» o Escape devuelve el foco al evento; el calendario conserva su desplazamiento previsto.
- [Nueva cita](index.html#editor): Paciente, Servicio y Horario, con resumen antes de Guardar cita.
- Registrar paciente: subrecorrido desde Nueva cita, con retorno y selección de la ficha de ejemplo. Representa el retorno, no el formulario completo ni las reglas de identificación del registro real.

Los controles de creación y selección del editor trabajan en memoria. «Cita guardada» es un resultado visual de ejemplo: no escribe en SQLite ni crea una reserva o un mensaje. No se añade la cita a la Agenda de ejemplo. Recargar restablece los datos. Los demás controles sin comportamiento siguen deshabilitados.

## Diseños futuros (sin contrato, sin operaciones)

Entregas T30–T36 del plan P-20261109-22. Son especificación visual: los datos son sintéticos, los botones no operan y las funciones siguen bloqueadas hasta que sus contratos existan. Cada vista enlaza su especificación en `tasks/P-20261109-22-interface-refresh/design/`.

- [Solicitudes](index.html#requests): lista y detalle junto a la cita vigente sin modificar; estados de solicitud separados de los de la cita; fallo parcial de ejemplo.
- [Lista de espera](index.html#waitlist): bloques «En espera» y «Ofertas»; el orden no indica prioridad; el interés del paciente no es una reserva.
- [Comunicaciones](index.html#communications): estados programado, entregado, respondido y fallo; la entrega no confirma asistencia.
- [Indicadores](index.html#metrics): valor, unidad, numerador/denominador, periodo, actualización y definición; «sin datos», «desactualizado» e «indisponible» como estados distintos.
- [Equipo y dispositivos](index.html#team): personas, roles e invitaciones; equipos, último acceso y revocación que aplica al reconectar.
- [Política de comunicaciones](index.html#policy): grupos con controles deshabilitados; las opciones sin contrato se marcan como pendientes.
- [Patient Hub (móvil)](hub.html): entrega visual del Hub desde 320 px; confirmación, solicitud que conserva el horario, cancelación con consecuencia, enlace vencido y servicio no disponible. Es un handoff de diseño para el repositorio propietario, no una web funcional.

## Revisión de las entregas de diseño

1. Recorrer cada vista de «Diseño futuro» y confirmar que se identifica como diseño sin contrato y que sus botones no operan.
2. En Solicitudes, abrir una fila: la cita vigente aparece sin modificar y la decisión explica que elegir horario no cambia la cita.
3. En Lista de espera, comprobar que las dos vistas se distinguen y que las notas explican prioridad e interés sin prometer reserva.
4. En Comunicaciones, verificar que entregado/respondido/fallo son estados con texto y que ninguno confirma asistencia.
5. En Indicadores, comprobar que cada tarjeta tiene denominador, periodo, actualización y definición, y que los tres estados sin datos se distinguen.
6. En Equipo y dispositivos, revisar que la revocación explica su alcance y que no hay secretos ni tokens.
7. En Política, confirmar que los valores se marcan de ejemplo y que lo pendiente de contrato no es editable.
8. En hub.html a 320 px y en escritorio: objetivos táctiles legibles, una columna, y que abrir la vista de cancelación no ejecuta nada (botón deshabilitado).
9. Revisar las siete vistas a 800×600 y 1440×900; comprobar nombres largos, estados vacíos y ausencia de desbordamiento global.

Los pasos anteriores son pasos de revisión, no resultados de prueba. Los resultados de la inspección automatizada se registran en [evidencia de C11–C13](../../tasks/P-20261109-22-interface-refresh/evidence-c11-c13.md). Zoom real 200 %, lector y contraste real quedan pendientes hasta revisión humana.

## Referencia común aprobada en C02

| Elemento | Medida / decisión |
|---|---|
| Títulos | Montserrat 500; página 24 px, panel 20 px, grupos de formulario 18 px |
| Lectura, tablas y controles | Mulish 400/500/600, base 14 px; resumen 15 px |
| Recursos | WOFF2 variables locales, rango 400–600; origen, huellas y OFL en [assets/LICENSE.md](assets/LICENSE.md) |
| Color | Paleta clara y azul de DESIGN.md; estados con texto |
| Navegación amplia | 216 px; barra superior 52 px |
| Ritmo | 8/16/24/32 px; controles de al menos 36 px |
| Formulario | Hasta 640 px; resumen hasta 352 px; separación 32 px |
| Ventana estrecha | A 1100 px o menos, resumen después de campos y antes de Guardar |
| Detalle | 352 px junto a lista cuando cabe; superficie propia en estrecho |
| Semana | 96 px/hora, 48 px/media hora, al menos 224 px/día, desplazamiento propio |
| Errores | Texto junto al campo y resumen enlazado; sin depender de rojo/color |
| Retorno desde registro | Cita en DOM sin reemplazar ni reiniciar sus campos; selección del paciente y foco al selector |

Los intervalos del editor son ejemplos fijos. No calculan disponibilidad, permisos, completitud de paciente, transiciones ni reservas. Rust sigue siendo propietario de esas reglas en el producto. Se conserva Servicio, sin motivo de visita; Profesional se omite en este ejemplo de dueño con un único profesional. No se modifica ningún contrato ni dato operativo.

## Skills y fuentes

Se mantiene P-20261109-22 sin otro plan. minimal-code elige elementos nativos y los mismos archivos de muestra; no se añaden dependencias. Laws of UX: proximidad agrupa campos y grupos, memoria de trabajo mantiene resumen y valores al volver, Hick separa decisiones pertinentes sin ocultar campos necesarios, y cada diseño futuro lleva una única tarea principal con su acción principal visible y las secundarias ausentes o deshabilitadas.

Stack comprobado en manifest/lockfile: React 19.3.1, Router 7.18.5, Tauri API 2.12.4, TypeScript 6.0.3 y Vite 8.3.4. La muestra no llama a sus APIs. Las APIs web nativas no tienen versión de paquete.

Patrones consultados según docs-first-framework-code:

- [fieldset y legend](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/fieldset#try_it): grupos de formulario.
- [Radios nativos](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/input/radio#defining_a_radio_group): una elección de intervalo y teclado de plataforma.
- [Controles del formulario](https://developer.mozilla.org/en-US/docs/Web/API/HTMLFormControlsCollection/namedItem): acceso por nombre sin duplicar estado de los campos.
- [focus y preventScroll](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement/focus#parameters): foco y retorno.

[Evidencia de C01](../../tasks/P-20261109-22-interface-refresh/evidence-c01.md) · [Evidencia de C02](../../tasks/P-20261109-22-interface-refresh/evidence-c02.md) · [Evidencia de C03](../../tasks/P-20261109-22-interface-refresh/evidence-c03.md) · [Evidencia de C11–C13](../../tasks/P-20261109-22-interface-refresh/evidence-c11-c13.md).
