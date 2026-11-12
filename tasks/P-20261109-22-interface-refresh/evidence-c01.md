# Evidencia C01 · 2026-11-09

Estado: **C01 presentado con comprobaciones pendientes; sin aprobación**. T01 Done; T02/T03 Blocked para cierre. T04 no iniciada. Solo documentación y muestra; no cambios de producto. Las entregas de T02/T03 están preparadas, pero sus criterios visuales no se consideran verificados.

## Revisar la muestra

Abrir `design-prototype/interface-refresh/index.html` en un navegador local. No requiere servidor ni red. Día abre inicialmente; pulsar una fila abre/cierra su detalle. Semana cambia de vista; cada evento abre su detalle. «Volver a Semana» y Escape cierran el detalle con retorno al evento. Los controles de creación/acciones están deshabilitados: no ejecutan operaciones y el editor pertenece a T04.

Revisar a 800×600 y 1440×900: Día sin detalle, abrir Ana María y José, cerrar conservando contexto; abrir Semana, recorrer horas y días mediante el desplazamiento del calendario, abrir eventos y volver. Revisar 00:00 del viernes y el cruce sábado/domingo, además de 09:00–13:00 de lunes a jueves. Comprobar Tab, Shift+Tab, Enter, Espacio en las filas de Día y Escape en detalle semanal.

Todos los datos son sintéticos. Las simultáneas prueban solo composición, no reservas válidas del profesional único. No hay comandos Tauri, HTTP operativo, almacenamiento, disponibilidad ni contratos inventados. El país no se presenta ni se borra de un modelo real.

## Cambios por tarea

- T01: DESIGN.md, DESIGN-RULES.md, docs/design-master-plan.md, docs/master-plan.md y docs/technical-architecture.md. Fuentes Montserrat/Mulish, sincronización normal invisible, Por revisar condicional, dirección hasta ciudad/estado, Profesional solo oculto para dueño con un profesional, Servicio conservado e inspiración Linear/Attio/Twenty. Se conservan estados internos, permisos, transacciones, persistencia, versiones e identidad.
- T02: index.html, styles.css y tres recursos en assets/. Cuatro citas; nombres y servicios largos, teléfono/responsable ausentes, estado y dirección. Detalle lateral de 352 px en ancho amplio, superficie propia en estrecho. Fuentes locales variables, roles 400/500/600 y licencias OFL completas en assets/LICENSE.md.
- T03: app.js y ampliación de index.html/styles.css. 96 px/hora, 48 px/media hora. Días de al menos **224 px**: por encima del mínimo de 160 px del plan para dar espacio a dos columnas simultáneas. Scroll propio con cabeceras/horas sticky. Se representa todo el día, incluidos 00:00/24:00. Duraciones 15/30/37/60, cancelación, bloqueo y cruce de día. Detalle completo y retorno previstos por código; comportamiento pendiente de comprobación.

## Resultados

| Tarea / comprobación | Resultado | Evidencia y límite |
|---|---|---|
| T01 · lectura y alineación de cinco fuentes | PASS | Revisión de reglas y diff; búsqueda con rg sin Anek/Aileron en las tres fuentes visuales. No modifica contratos de dominio. |
| T01 · conservación de planes y producto | PASS | Estado Git inicial: DESIGN.md modificado y este plan sin seguimiento. Estado final solo añade las cinco fuentes, esta muestra y registros de este plan; ningún plan anterior ni src/ ni src-tauri/ cambia. |
| T01–T03 · git diff --check | PASS | Sin errores tras quitar espacios finales que traía el texto de licencia. Comprobación adicional --no-index de archivos nuevos. |
| T02 · archivos y enlaces locales | PASS | HTMLParser: IDs únicos, enlaces internos existentes, CSS/JS presentes; URLs de fuentes apuntan a archivos locales. No se incluyen recursos remotos en ejecución. |
| T02 · recursos WOFF2 y licencias | PASS | Cabecera wOF2 y longitud interna válidas; SHA-256 en LICENSE.md. Google Fonts devuelve font-weight 400 600. No se verificó el render ni la tabla de glifos. |
| T02 · lectura de referencia histórica | PASS | Se leyeron agenda.png y cita-a1.png originales: navegación compacta, filas/separadores y detalle en contexto se conservan como base. Comparación visual nueva contra histórica: NOT CHECKED. |
| T02 · Día/detalle 800×600 y 1440×900 | NOT CHECKED | El navegador integrado rechazó file:// por su política de URL. No se buscó otro acceso para eludir el bloqueo. Faltan capturas de la muestra nueva. |
| T02 · teclado, foco y fuentes renderizadas | NOT CHECKED | Código nativo details/summary, foco visible y fuentes locales preparados; no equivalen a una prueba en ejecución. |
| T03 · sintaxis JavaScript | PASS | node --check design-prototype/interface-refresh/app.js; Node 24.18.0. |
| T03 · aritmética de ejemplos | PASS | geometry-c01.json: 12 segmentos, límites [0,1440], 30 min = 48 px, duraciones requeridas y columnas simultáneas distintas. Solo datos/cálculo, no medición de píxeles. |
| T03 · posición visual, solapamiento, scroll y overflow | NOT CHECKED | Requiere ejecución y captura a ambos tamaños. CSS y cálculo no acreditan legibilidad real. |
| T03 · Día ↔ Semana ↔ detalle y retorno | NOT CHECKED | Implementado en muestra; no probado en navegador. |
| Contraste real, zoom 200 %, lector y validación humana | NOT CHECKED | Sin evidencia en este bloque. No se afirma conformidad global ni mejora medida de uso. |
| Build/quality/Rust/API/offline real | N/A | No cambia producto; ninguna prueba funcional de persistencia/sync se da por ejecutada. |
| Revisión del usuario C01 | NOT CHECKED | Se espera revisión expresa antes de T04. |

El intento de inspeccionar glifos con fontTools no pudo ejecutarse porque el módulo no está instalado. No se añadió dependencia. Acentos, ñ, signos, teléfonos y números están en los ejemplos, pero su render sigue NOT CHECKED.

## Skills y decisiones

- planning-task-breakdown mantiene ID, tareas, dependencias y checkpoints del plan aprobado. No se crea otro plan ni se cierran planes anteriores.
- minimal-code: se trazaron el prototipo histórico y Agenda/detalle reales. La muestra necesita presentación estática, por lo que usa HTML/CSS y JavaScript nativo sin dependencias ni APIs React/Tauri. No copia funciones del prototipo a producto.
- Laws of UX: proximidad agrupa fecha/vistas; carga cognitiva retira etapas técnicas; memoria de trabajo mantiene lista junto al detalle cuando cabe; Fitts permite abrir desde toda la fila y usa controles de 36 px. Estas decisiones no acreditan latencia ni validación humana.
- docs-first-framework-code: stack consultado en manifest/lockfile: React 19.3.1, Router 7.18.5, Tauri API 2.12.4, TypeScript 6.0.3 y Vite 8.3.4. La muestra no usa sus APIs. Usa plataforma web sin versión de paquete. Navegador de ejecución: NOT CHECKED.

Fuentes consultadas para los patrones nativos:

- [@font-face: src y rango de pesos](https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/At-rules/@font-face#descriptors).
- [details/summary: activación nativa](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/details#usage_notes).
- [position sticky y contenedor de desplazamiento](https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/position#values).
- [focus con preventScroll](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement/focus#parameters).
- [hashchange para Día/Semana](https://developer.mozilla.org/en-US/docs/Web/API/Window/hashchange_event#syntax).

## Pendiente para C01

Capturas nuevas a 800×600 y 1440×900 de Día, detalle y Semana; recorrido real de teclado/foco y revisión de fuentes. Revisar si 224 px/día aporta lectura suficiente sin dificultar la comparación semanal. El usuario debe revisar C01 antes de T04. Las casillas de inspección visual permanecen abiertas.

## Revisión posterior del usuario

«Lo veo bien, puedes continuar por favor». C01 aceptado y T04 autorizado. Esta revisión no acredita tamaños, teclado, foco ni fuentes que el agente no pudo probar. Los resultados NOT CHECKED de la tabla anterior permanecen como evidencia histórica.
