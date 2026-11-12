# C02 · Acceso, Agenda y citas

Fecha: 2026-11-11. T03–T05 terminadas en el prototipo original. C01 aprobado por el usuario; C02 pendiente de su revisión. T06 no iniciada.

## Alcance y entorno

HTML/CSS/JavaScript nativos, fuentes e iconos locales. Navegador en `http://127.0.0.1:8766/`, servidor temporal de archivos con Node v24.18.0. Sin nuevas dependencias ni cambios de producto, servicios o contratos. Datos ficticios en memoria; recargar restablece el ejemplo.

Se conserva el diseño claro, sencillo y compacto del original. Se aplican Hick (acciones por estado), proximidad (error junto al campo y comparación de horarios), reconocimiento (etiquetas y contexto visibles) y convenciones conocidas (tabla, pestañas y detalle lateral). Los controles de escenario quedan fuera del trabajo diario.

## Resultados

| Tarea | Comprobación | Evidencia |
|---|---|---|
| T03 | 12 variantes de acceso/preparación a 1280×800 y 800×600; secuencia de acceso y preparación, intervalo inválido con campos conservados, guardado válido y resumen | [T03-checks.json](T03-checks.json) |
| T03 | Acceso local permitido, revocación sin borrar datos y activación sin red | [Acceso revocado](T03-acceso-revocado.jpg) |
| T03 | Asistente sin preparación no recibe formulario; con cuenta ya preparada recibe su resumen | [T03-regression.json](T03-regression.json) |
| T04 | Filtrar Confirmadas mantiene el primer intervalo libre en 10:30; no usa filas ocultas como disponibilidad | [T04-checks.json](T04-checks.json) |
| T04 | Día vacío conocido, filtro vacío y fecha sin cobertura distintos; carga, actualización, datos anteriores, error y falta de permiso | [Día vacío](T04-dia-vacio.jpg), [sin cobertura](T04-sin-cobertura.jpg) |
| T04 | Semana con citas de 15/30/45/60 minutos; escala de 96 px/hora y mínimo de 160 px/día; varios profesionales sin rectángulos superpuestos | [Semana](C02-semana.jpg), [varios profesionales](T04-varios-profesionales.jpg), T04-checks.json |
| T04 | Lista carga de 8 a 16 filas; conserva búsqueda y periodo al volver del detalle | T04-checks.json |
| T05 | Crear desde intervalo libre y doble clic produce un solo registro; solapamiento y bloqueo conservan los campos | [T05-checks.json](T05-checks.json) |
| T05 | Crear desde ficha preselecciona al paciente y guarda su cita en el ejemplo | [T05-from-patient.json](T05-from-patient.json), [captura](C02-desde-paciente.jpg) |
| T05 | Reprogramar cita de 45 minutos conserva duración y referencias; solo ofrece fecha e intervalo; conserva el hecho previo en la historia | [Comparación](C02-reprogramar.jpg), T05-checks.json |
| T05 | Menor sin documento exige responsable activo; registro relacionado conserva documento declarado, campos de la cita y foco | [Registro relacionado](T05-registro-relacionado.jpg), T05-checks.json |
| T05 | Acciones dependen de permiso, estado y hora; Atendida/Ausencia/Cancelada impiden nuevas mutaciones; ausencia requiere margen y acción expresa | [Detalle terminal](T05-detalle-terminal.jpg), T05-checks.json |
| T05 | Cancelación identifica paciente, servicio y horario; cerrar no cambia la cita; cancelación local conserva ocupación anterior hasta aceptación simulada | T05-checks.json |
| T05 | Atrás conserva la URL y abre aviso de borrador; pasar a solo lectura y volver conserva campos | T05-checks.json |
| T05 | Conflicto conserva propuesta; servicio no disponible conserva una intención local, sin avisos técnicos ni doble registro | [Conflicto en editor](T05-conflicto-editor.jpg), T05-checks.json |

Las pruebas usan grupos independientes de datos ficticios; un identificador de cita puede corresponder a distintas altas después de recargar. `T03-regression.json` sustituye la observación inicial del asistente después de preparar la cuenta.

## Tamaños, foco y contexto

- [surface-checks.json](surface-checks.json): 48 rutas del catálogo × tres tamaños (1280×800, 1024×768 y 800×600), 144 cargas con encabezado visible y sin desbordamiento horizontal del documento. Es una prueba de carga y ajuste; no cierra las funciones de T06–T21.
- [context-layout-checks.json](context-layout-checks.json): a 800×600, abrir una cita con el puntero y cerrarla conserva exactamente el desplazamiento semanal (x 325,527; y 414,545) y el de la página (257,455); el foco vuelve al enlace de la cita. Escala medida entre 10:00 y 11:00: 96 px.
- El detalle estrecho recibe foco en el nombre del paciente; el resumen del editor queda antes de las acciones finales. [Semana estrecha](C02-semana-800.jpg), [editor estrecho](C02-editor-800.jpg).
- Teclado en campos y controles nativos, retorno al enlace de origen, protección al navegar y diálogos de decisión comprobados. No se acredita una auditoría completa de accesibilidad.
- Sin errores de consola observados al terminar. Se retiró la dimensión temporal de ventana usada para las pruebas y se dejó Agenda con los datos iniciales.
- `node --check design-prototype/app.js`: PASS. `git diff --check -- design-prototype tasks/P-20261111-01-prototype-completion`: PASS.

## Límites

- NOT CHECKED: autenticación o permisos reales, Rust, SQLite, sincronización, persistencia tras reinicio, entrega de mensajes, lector de pantalla y pruebas con usuarios.
- NOT CHECKED: zoom de texto al 200 %. La revisión completa de tamaños y accesibilidad sigue en T21.
- Preparación recoge cuenta, lugar, zona, primer servicio y horario; guarda resumen y servicio en memoria. La disponibilidad de Agenda aún usa los días y horas fijos del ejemplo (19–30 de octubre, lunes a viernes, 09–12 y 14–17). La edición completa y su enlace a Agenda corresponden a T08–T10.
- El ejemplo usa 15 minutos como margen de ausencia inicial del contrato vigente. El control de hora permite comprobar las restricciones sin cambiar la hora del equipo. No se deduce ausencia de falta de respuesta.
- El escenario normal representa aceptación conocida; sin red o servicio, guarda actividad local del ejemplo. No prueba conciliación distribuida. T18 completará los recorridos de revisión y recuperación.
- El registro relacionado de paciente usa documento propio declarado y teléfono; el menor sin documento ya preparado permite revisar el responsable obligatorio. Directorio, alta administrativa completa y responsables editables siguen en T06–T07.
- No hay reservas, envíos ni consultas a servicios. Las capturas históricas y `design-prototype/verification.json` no se modifican para atribuirles estas pruebas.

## Fuentes técnicas

Se reutilizan las APIs nativas del prototipo, sin biblioteca nueva: [CSS Grid](https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/grid-template-columns), [posición sticky](https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/position#sticky_positioning), [scrollTop](https://developer.mozilla.org/en-US/docs/Web/API/Element/scrollTop), [replaceState](https://developer.mozilla.org/en-US/docs/Web/API/History/replaceState) y [hashchange](https://developer.mozilla.org/en-US/docs/Web/API/Window/hashchange_event). `replaceState` conserva la ruta del borrador sin disparar otro `hashchange`.

La implementación toma las reglas de las fuentes locales vigentes: `docs/master-plan.md`, `docs/design-master-plan.md`, `docs/contracts/api-v1/nightingale-api.md`, `DESIGN.md` y `DESIGN-RULES.md`. No cambia esas fuentes.

## Revisión del usuario

C02 queda listo para revisar acceso/preparación, Día/Semana/Lista y creación/reprogramación/acciones de cita. Según el plan aprobado, T06–T07 comienzan después de la aceptación expresa de C02.
