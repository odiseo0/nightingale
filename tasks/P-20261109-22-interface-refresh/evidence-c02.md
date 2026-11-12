# Evidencia C02 · Editor de cita

**Estado:** entrega preparada para revisión; C02 no aprobado. **T04 no se cierra aún:** faltan revisión en ejecución y aceptación de la muestra completa. T05 no iniciada.

## Autorización

El usuario aceptó C01: «Lo veo bien, puedes continuar por favor». Se continúa con T04 conforme al plan aprobado. La aceptación de C01 no convierte las pruebas ambientales faltantes de C01 en PASS.

## Entrega

Abrir [muestra](../../design-prototype/interface-refresh/index.html#editor) y seguir [README y medidas](../../design-prototype/interface-refresh/README.md).

Cambios principales, solo en los cuatro archivos previstos:

- index.html: Nueva cita, grupos Paciente/Servicio/Horario, resumen antes de Guardar, avisos de campo y formulario; subrecorrido de registro; escenario de guardado fuera de la interfaz de producto.
- styles.css: mismos tokens y fuentes, formulario de hasta 640 px, resumen hasta 352 px, controles de 36 px, grupos separados por 24 px, apilado a 1100 px o menos. Errores con texto y borde, sin nueva paleta semántica.
- app.js: navegación en memoria, intervalos de ejemplo, resumen, campos incompletos, error de guardado con conservación de valores, resultado de ejemplo y retorno desde registro seleccionando la ficha.
- README.md: cuatro vistas revisables juntas, pasos de revisión, medidas propuestas, fuentes oficiales, límites y comprobaciones faltantes.

El registro relacionado solo representa el retorno y selección de una ficha sintética; no especifica todo el registro ni modifica requisitos de documentos/completitud. Los intervalos fijos no calculan disponibilidad. No hay cambios de React, Tauri, Rust, SQLite, HTTP, permisos, versiones ni operaciones. No hay persistencia ni cambios en Agenda al pulsar Guardar. El mensaje de guardado pertenece al diseño de ejemplo.

## Comprobaciones

| Comprobación | Resultado | Alcance |
|---|---|---|
| Sintaxis JavaScript | PASS | node --check design-prototype/interface-refresh/app.js, Node 24.18.0 |
| Estructura HTML y recursos | PASS | IDs únicos; etiquetas for y aria-describedby/labelledby apuntan a elementos presentes; enlaces locales y archivos CSS/JS/fuentes existen |
| Documentación de APIs nativas | PASS | fieldset/legend, radios y namedItem consultados en MDN; enlaces en README |
| git diff --check y archivos nuevos | PASS | Sin errores de espacios; archivos CRLF reconocidos como finales de línea |
| Límite de cambio | PASS | Solo cuatro archivos principales de muestra y registros de este plan; producto, recursos de fuentes, 34 vistas históricas y planes funcionales anteriores se conservan |
| Grupos, resumen y estilos comunes | PASS | Inspección de código: mismos tokens y fuentes; no equivale a inspección visual |
| Incompleto/error/intervalo/retorno | NOT CHECKED | Comportamientos preparados en código; falta recorrido en navegador |
| Tab/Shift+Tab, flechas, Enter, foco | NOT CHECKED | Se mantiene el bloqueo de file:// registrado en C01. No se reintenta ni se busca eludirlo |
| Conservación observada al volver del registro y ante error | NOT CHECKED | La muestra mantiene el DOM sin reset; falta comprobar interacción real |
| 800×600 y 1440×900, zoom 200 %, fuentes renderizadas, contraste real, lector | NOT CHECKED | Sin capturas ni prueba ejecutada por el agente |
| Revisión del usuario C02 | NOT CHECKED | Se entrega el editor para revisión junto con Día/Semana/detalle |
| Build/quality/Rust/API/offline real | N/A | No hay cambios de producto y la muestra no prueba esas garantías |

No se cuentan NOT CHECKED ni N/A como aprobados. No se afirma conformidad de accesibilidad ni rendimiento medido.

## Criterios de T04

1. Campos, grupos, resumen y Guardar comparten estilos con las vistas aceptadas en C01. Inspección de código realizada; aspecto en ejecución pendiente.
2. Hay recorridos de incompleto, error, selección de intervalo y registro relacionado. Prueba de uso pendiente.
3. Las cuatro vistas tienen acceso desde la misma muestra; README registra medidas y límites. C02 sigue pendiente y producto intacto.

## Skills

planning-task-breakdown mantiene el plan y sus IDs; minimal-code usa formularios nativos y campos como estado del ejemplo, sin dependencia ni segunda copia del borrador. Laws of UX aplica proximidad a grupos y memoria de trabajo a resumen/retorno. docs-first-framework-code consulta los patrones web nativos; versiones de paquetes verificadas en README, sin uso de APIs React/Tauri.

Fuentes de los patrones: [fieldset/legend](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/fieldset#try_it), [radios](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/input/radio#defining_a_radio_group), [namedItem](https://developer.mozilla.org/en-US/docs/Web/API/HTMLFormControlsCollection/namedItem), [foco](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement/focus#parameters).

## Revisión solicitada

Ver Nueva cita, intentar guardar vacía, completar y revisar el resumen; registrar un paciente sintético a mitad y volver; probar el error de guardado con el selector al pie y después el éxito de ejemplo. Comparar las cuatro vistas. C02 requiere aprobación expresa antes de T05.

## Revisión posterior

C02 aprobado por el usuario, con la instrucción de mantener mayor fidelidad a Linear y Attio conforme a DESIGN.md. Autoriza continuar con T05. La aceptación no acredita los NOT CHECKED técnicos de esta evidencia.
