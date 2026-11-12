# P-20261109-22-interface-refresh: Renovación de la interfaz de Nightingale

**Status:** Completed
**Created:** 2026-11-09
**Updated:** 2026-11-09
**Task list:** [todo.md](todo.md)
**Tipo:** Nuevo plan transversal de diseño e implementación de interfaz.

## Aprobación y límite de ejecución

El usuario aprobó el plan en esta conversación: «No lo implementes, pero apruebo el plan y quiero que escribas tal y como dice la skill en tasks/». Después autorizó su escritura: «Puedes escribirlo».

La instrucción de ejecución del 2026-11-09 autoriza T01–T03 en orden, una tarea cada vez. Solo documentación y muestra visual; no código de producto. Se detendrá en C01 para revisión expresa antes de T04. La aprobación del plan no acepta sus checkpoints.

Se conservan los IDs T01–T39 y C01–C14 del plan aprobado. No se renumerarán. Cada checkpoint requiere revisión del usuario antes del bloque siguiente.

## Registro de ejecución · 2026-11-09

T01 Done. Entregas de Día/detalle (T02) y Semana (T03) preparadas en orden. T02/T03 **Blocked para cierre**: inspección en ejecución pendiente por rechazo de file:// en el navegador integrado. No se marcan Done ni se cuentan sus criterios visuales como aprobados. Una sola tarea se trabajó a la vez; no se ejecutó T04.

Se presenta el estado y la evidencia de C01 al usuario, sin declararlo superado. [Evidencia y pasos de revisión](evidence-c01.md), [aritmética de ejemplos](geometry-c01.json), [muestra local](../../design-prototype/interface-refresh/index.html). Capturas, tamaños, teclado y render de fuentes: **NOT CHECKED**. Se espera revisión de C01 antes de T04.

### Revisión de C01 y continuación

El usuario revisó la muestra y respondió: «Lo veo bien, puedes continuar por favor». C01 aceptado; se autoriza T04 y presentación de C02, sin cambios de producto. La aceptación visual del usuario no convierte las pruebas técnicas NOT CHECKED en PASS. Se mantienen los pendientes en evidence-c01.md. T02/T03 se registran Done por entrega y aceptación del usuario, con las verificaciones ambientales abiertas.

### Entrega T04 y presentación de C02

T04 preparada en index.html, styles.css, app.js y README.md de la muestra. Se presenta [C02](evidence-c02.md). Sintaxis, estructura HTML, etiquetas, enlaces/recursos locales y diff pasan. Recorrido en navegador, teclado, foco y render: **NOT CHECKED**, sin eludir el bloqueo previo de file://. T04 queda Blocked para cierre y su casilla permanece abierta. No se ha iniciado T05 ni se ha aprobado C02. Las medidas están registradas en README para revisión de las cuatro vistas.

### C02 aprobado y criterio de fidelidad

El usuario aprobó C02 y autorizó continuar: «Aprobado […] puedes continuar». Pide mantener mayor fidelidad a Linear y Attio conforme a DESIGN.md. La implementación conservará su escala, geometría, colores, superficies y contención; no añadirá otro estilo visual. Se autorizan T05–T07 en orden y presentación de C03. No se trasladan verificaciones ambientales pendientes a PASS. T04 Done por entrega y aceptación.

### Entrega de C03

T05–T07 ejecutadas en orden. Fuentes Montserrat/Mulish integradas con los tokens y controles de DESIGN.md; rol local viewerRole y metadatos de revisión añadidos sin migración ni cambio HTTP. Build y quality final PASS: 377 unitarias, 5 persistencia y 1 doc-test; 9 ignoradas. Pruebas enfocadas: cuenta 30 correctas; sync 80 correctas y 3 ignoradas. [Evidencia C03](evidence-c03.md) y logs junto al plan.

Inspección visual de app, zoom 200 %, glifos renderizados y consulta interactiva con perfiles/revisiones: **NOT CHECKED**. Los resultados de lectura local corresponden a pruebas automatizadas, no a una sesión humana. C03 espera revisión expresa; no se ha iniciado T08. Se conserva la instrucción de fidelidad a Linear y Attio, la paleta propia y los planes previos.

### C03 aprobado y alcance limitado a diseño y prototipo

El 2026-11-09 el usuario aprobó C03 y resolvió: «Apruebo C03 y solamente hagamos los trabajos de diseño y prototipo». El alcance autorizado se limita a las entregas de diseño y muestra (T30–T36 y sus documentos en `design/` y vistas en `design-prototype/interface-refresh/`). No se ejecuta código de producto: T08–T29 y T37–T39 permanecen en su estado y ningún checkpoint de código se da por superado. Los bloques C11, C12 y C13 se presentan para revisión expresa al terminar sus tareas de diseño; la ejecución de las tareas de diseño está autorizada en secuencia. Se conservan las dependencias externas identificadas y no se implementa ninguna función sin contrato.

### Entregas de diseño T30–T36 y presentación de C11–C13

T30–T36 ejecutadas en orden, solo documentación y muestra. Siete especificaciones en `design/` (requests, waitlist, communications, metrics, team, devices, communication-policy, hub-handoff) y ocho vistas nuevas en la muestra (`#requests`, `#waitlist`, `#communications`, `#metrics`, `#team`, `#policy` y `hub.html` móvil). Sintaxis, integridad de rutas y `git diff --check`: PASS. Render de `#requests` a 1440×900 en navegador integrado por HTTP local: PASS (supera el bloqueo file:// de C01/C02). Inspección completa de las demás vistas, interacciones, 800×600, hub.html a 320 px, teclado, zoom y lector: **NOT CHECKED** por toma de control del navegador por el usuario. [Evidencia C11–C13](evidence-c11-c13.md). Se corrige un defecto propio: el `min-height` de la lista del Día se acotó a `#day` para no deformar las vistas nuevas.

C11, C12 y C13 se presentan juntos para revisión expresa. Ningún checkpoint se da por superado; las casillas de revisión del usuario permanecen abiertas.
### C11–C13 aprobados

El 2026-11-09 el usuario aprobó los cambios presentados: «Apruebo estos cambios si necesitan aprobación». C11, C12 y C13 quedan revisados por el usuario. Las comprobaciones ambientales NOT CHECKED (inspección completa de vistas, 800×600, hub.html a 320 px, teclado, zoom 200 % y lector) conservan ese estado y no se cuentan como aprobadas. T29 y T37 siguen sin ejecutar por el alcance limitado a diseño y prototipo.

## Registro de ejecución · continuación 2026-11-09 (autorización de cierre total)

Con la instrucción «Tienes autorización de terminar todo el plan» se ejecutaron secuencialmente T08–T29 (código de producto) y T37–T39 (auditoría, integración y cierre), respetando las dependencias y conservando los registros por tarea. Resultados: pnpm build y pnpm quality PASS en cada fase (377 unitarias, 5 persistencia, 1 doc-test; 9 ignoradas); node --test de presentación 16/16 (agenda-presentación, week-layout, reviews); pnpm build:desktop PASS (nightingale.exe). Evidencias: evidence-c04.md a evidence-c10.md, evidence-c11-c13.md, accessibility-matrix.md, integration-checks.md, user-validation.md y evidence.md.

El usuario aprobó los checkpoints C11, C12 y C13 en esta continuación; C04–C10 y C14 quedan con revisión expresa pendiente o parcial, y toda verificación humana/ambiental sigue NOT CHECKED sin contar como aprobada. El 2026-11-09 el usuario aceptó el cierre («Aprobado»): C14 quedó superado y el plan pasó a Completed.

## Overview

### Contexto y resultado esperado

Nightingale ya tiene una interfaz funcional sobre React, Tauri, Rust y SQLite. Permite trabajar con citas, pacientes, servicios, horarios, acceso y configuración. Sin embargo, su presentación conserva decisiones propias de una interfaz de comprobación técnica: textos sobre aceptación remota, etiquetas repetidas, formularios demasiado anchos y una distribución que dedica mucho espacio a controles secundarios.

El [prototipo existente](../../design-prototype/README.md) se acerca más al resultado que busca el usuario. Presenta mejor las tablas, la navegación, los formularios y el detalle junto a la lista. Servirá como referencia de composición, sin convertir sus datos ficticios ni sus funciones de ejemplo en capacidades reales del producto.

La renovación tomará de [Linear](https://linear.app/), [Attio](https://attio.com/) y [Twenty](https://twenty.com/es/product) sus elementos de interfaz y patrones de uso: navegación lateral, barras compactas, tablas, filtros, paneles de detalle, formularios, menús y jerarquía tipográfica. La categoría Healthtech define las tareas y los datos de Nightingale; el color propio será su principal diferenciador visual.

El objetivo es renovar toda la interfaz actual y mejorar su uso: facilitar la lectura, reducir tareas repetidas, conservar el contexto y hacer más claros los resultados y errores.

### Decisiones confirmadas con el usuario

- Crear un plan visual nuevo y transversal.
- Conservar los planes funcionales existentes y su historial.
- Renovar todas las pantallas funcionales de la app.
- Diseñar también las pantallas futuras, sin implementar funciones que carecen de contrato.
- Conservar el servicio actual de la cita; aplazar un posible campo nuevo de motivo de visita.
- Hacer invisible la sincronización normal.
- Mostrar problemas que requieren intervención junto al dato afectado y en **Por revisar**.
- Revisar primero una muestra de **Agenda Día, Semana, detalle y editor de cita**.
- Usar **Montserrat** para títulos y **Mulish** para la interfaz de lectura y trabajo.

La aprobación de este plan no cierra ni sustituye otros planes. Tampoco traslada sus comprobaciones pendientes al estado de aprobadas.

### Estado observado y relación con otros planes

La revisión anterior ejecutó la app con `pnpm tauri dev` y observó Agenda, Semana, detalle, pacientes, ficha, editor, configuración y búsqueda. Se compararon con código y capturas del prototipo y con las referencias visuales del usuario. Esa revisión es evidencia de partida, no prueba de la renovación aún no realizada.

El único cambio previo de esta conversación es el reemplazo de fuentes en `DESIGN.md`. La app todavía usa Anek Latin y Aileron. `DESIGN-RULES.md` y el maestro de diseño conservan referencias antiguas; T01 alineará las fuentes cuando se autorice ejecutar.

Se mantienen separados los planes existentes:

- [Base visual y Agenda](../P-20261108-05-agenda-ui-foundation/plan.md) y [Nueva cita](../P-20261108-06-appointment-creation-ui/plan.md): recorridos previos, sin reabrir su historial.
- [Roadmap UX/UI](../P-20261109-01-ux-ui-roadmap/plan.md): referencia del trabajo funcional.
- P-20261109-02 a P-20261109-10, P-20261109-17 y P-20261109-18: implementación funcional relacionada; se conserva su estado.
- P-20261109-11 a P-20261109-16, P-20261109-19 y P-20261109-20: dependencias y diseños pendientes; este plan no los desbloquea.
- [Validación UX](../P-20261109-21-ux-validation/plan.md): deuda ambiental e historial de evidencia que no equivalen a aprobación de la nueva interfaz.
- Núcleos de Pacientes, Citas, Agenda, Cuenta y Servicios: conservan sus contratos y pendientes técnicos.

Los registros de varios planes figuran abiertos aunque sus checklists contienen tareas ejecutadas. Esta discrepancia no se resolverá cerrándolos dentro de este plan. Antes de cada tarea se comprobarán cambios concurrentes y dependencias funcionales.

## Source of Truth

### Lecturas obligatorias

Antes de ejecutar cada tarea, el implementador deberá leer:

1. [DESIGN.md](../../DESIGN.md): fuentes, colores, superficies, geometría, iconos y estados visuales.
2. [DESIGN-RULES.md](../../DESIGN-RULES.md): tarea principal, claridad, contenido, acciones y composición.
3. [Plan Maestro de Diseño e Interfaz](../../docs/design-master-plan.md): superficies, recorridos, estados, accesibilidad y límites del frontend.
4. [Plan Maestro de Desarrollo](../../docs/master-plan.md), especialmente alcance, propietarios, sincronización y frontend.
5. [Arquitectura técnica](../../docs/technical-architecture.md), [SRS](../../docs/srs.md) y [decisión de tres proyectos](../../docs/decisions/transactional-sync-three-projects.md).
6. [Contrato API v1](../../docs/contracts/api-v1/nightingale-api.md) antes de cualquier trabajo que toque su consumo.
7. [Laws of UX](../../.agents/laws-of-ux/SKILL.md) y las entradas pertinentes de sus referencias.
8. [Reglas de planes](../README.md), código real y checklist de la tarea.

Las decisiones expresas de esta conversación sustituyen las reglas anteriores sobre fuentes y exposición de la sincronización. T01 alineará los documentos para que el implementador no tenga que resolver contradicciones por su cuenta. Los cambios documentales se limitarán a esas decisiones aprobadas; no alterarán invariantes del dominio.

Cada tarea de código deberá aplicar [minimal-code](../../.agents/minimal-code/SKILL.md) después de seguir el flujo real. Las decisiones sobre APIs de React, React Router, Tauri u otras bibliotecas deberán aplicar [docs-first-framework-code](../../.agents/docs-first-framework-code/SKILL.md) con las versiones del lockfile.

### Código y herramientas de partida

- `src/App.tsx` define rutas; `src/app/AppLayout.tsx` contiene estructura, cuenta, búsqueda y navegación.
- `src/App.css` contiene fuentes, tokens, controles, tablas, detalle y calendario.
- `src/features/` contiene las superficies funcionales; los paneles técnicos se conservan en Desarrollo.
- `src/shared/api/` contiene clientes Tauri tipados; React no ejecuta SQL ni operaciones HTTPS de dominio.
- Rust conserva las lecturas, mutaciones, operaciones, permisos y persistencia.
- Versiones resueltas observadas: React 19.3.1, React Router 7.18.5, API Tauri 2.12.4, Tailwind 4.3.3, TypeScript 6.0.3 y Vite 8.3.4. Node 24.18.0 y pnpm 11.17.0 están fijados por el proyecto. Se comprobarán de nuevo antes de usar una API.

### Aplicación concreta de Laws of UX

| Problema | Principio que orienta el cambio | Aplicación |
|---|---|---|
| Información técnica repetida | Atención selectiva y carga cognitiva | Retirar información de sincronización normal |
| Controles que desplazan las citas | Proximidad y agrupación | Reunir fecha, vistas y filtros por función |
| Pérdida de contexto | Memoria de trabajo | Conservar fecha, filtros, selección y retorno |
| Formularios extensos | Agrupación y ley de Hick | Mostrar campos necesarios y grupos claros |
| Acciones poco claras | Ley de Fitts y convenciones conocidas | Controles legibles, posición estable y etiquetas |
| Esperas y actualizaciones | Umbral de Doherty | Respuesta inmediata y contenido estable durante la lectura |

Estas leyes no fijarán cantidades arbitrarias de columnas, opciones o componentes. Las tareas leerán las entradas aplicables en `references/laws-of-ux.md`; una transición CSS o la presencia de un spinner no prueban una latencia medida.

## Reglas obligatorias de la renovación

### Sincronización invisible

La interfaz diaria no mostrará:

- Columnas de sincronización.
- Etiquetas como «Aceptada», «Enviando» o «Actualizando la vista local».
- Contadores de operaciones pendientes.
- Bordes discontinuos que distingan datos locales.
- Mensajes de éxito que describan el proceso técnico.
- Botones de sincronización en las pantallas de trabajo.
- Avisos por cada actualización en segundo plano.

El proceso seguirá conservando operaciones, versiones, permisos, reintentos y resultados desconocidos. El cambio afecta a su presentación, no a su funcionamiento.

| Situación | Presentación acordada |
|---|---|
| Lectura y guardado local correctos | Resultado breve: «Cita guardada» o «Cambios guardados» |
| Envío o reintento automático | Sin aviso |
| Falta de red que no impide la tarea local | Sin interrupción |
| Acción que requiere conexión | Explicar la limitación junto a la acción |
| Conflicto que necesita una decisión | Explicar el problema junto al dato y añadirlo a Por revisar |
| Rechazo o pérdida de acceso | Explicar qué no pudo completarse y el siguiente paso permitido |
| Resultado desconocido | Conservar la intención; impedir una repetición que pueda duplicar el efecto |

«Guardado» describirá la persistencia local realizada. No afirmará que se envió un mensaje, se liberó un horario o se obtuvo una reserva compartida cuando el resultado aún no se conoce.

**Por revisar** aparecerá en la navegación solo cuando existan asuntos que requieren intervención. No mostrará tareas normales de sincronización. El diagnóstico técnico permanecerá separado del uso diario.

### Direcciones

Las direcciones de lectura terminarán con **ciudad y estado**, por ejemplo:

> Dirección del consultorio · Barquisimeto, Lara

- No añadir el país ni su código.
- Evitar comas, separadores o líneas vacías cuando falte un dato.
- Aplicar el mismo formato en detalle, resumen y vistas de lectura de configuración.
- Conservar el país almacenado y los campos necesarios para editar una dirección.
- Conservar el país emisor de documentos de identidad: no forma parte de esta regla.

### Profesional y servicio

- Mantener la columna **Servicio** y su dato actual.
- No crear ni renombrar ese dato como un motivo clínico.
- Ocultar columna y filtro Profesional cuando el usuario sea el dueño y la cuenta tenga un único profesional disponible.
- Mostrar Profesional cuando el contexto requiera distinguirlo, incluidos asistentes y cuentas con más de un profesional.
- No deducir el rol a partir del permiso de editar configuración.
- No eliminar `professionalId` de consultas ni operaciones.

### Tipografía

```css
--font-display: "Montserrat", system-ui, sans-serif;
--font-sans: "Mulish", system-ui, sans-serif;
```

- Montserrat: títulos de página, encabezados de sección y títulos de panel.
- Mulish: navegación, controles, formularios, mensajes, tablas, fechas y valores.
- Mantener los roles 400, 500 y 600.
- Usar archivos locales y pesos reales; conservar origen y licencia.
- No depender de un servicio de fuentes durante el uso de la app.
- Revisar acentos, ñ, signos, teléfonos, fechas y números.
- Revisar el espaciado con las fuentes nuevas; no asumir que las medidas anteriores producen el mismo resultado.

### Estilo común

- Conservar la paleta clara y azul de `DESIGN.md`.
- Reservar el azul para acciones principales, selección y foco.
- Usar texto, además del color, para comunicar estados.
- Preferir separadores y grupos a tarjetas alrededor de cada bloque.
- Usar sombras en superficies flotantes, no en todas las secciones.
- Mantener Phosphor como único conjunto de iconos.
- Dar a cada pantalla una tarea y una acción principal claras.
- Mantener visibles las acciones frecuentes y reunir las secundarias en un menú.
- Retirar números de superficie, códigos internos, versiones y fechas técnicas del contenido de trabajo.

## Composición e interacción

### Estructura general

La base para la muestra será:

| Elemento | Regla inicial |
|---|---|
| Navegación lateral | 216 px en escritorio; grupos por tarea |
| Barra superior | Aproximadamente 52 px; contexto, búsqueda y ayuda |
| Título de página | Montserrat, 24 px, peso 500 |
| Texto de trabajo | Mulish, 14–16 px según función |
| Etiquetas esenciales | Al menos 14 px |
| Metadatos secundarios | 12–13 px cuando no sean necesarios para ejecutar una acción |
| Espaciado | Ritmo 8/16/24/32 px |
| Controles de escritorio | Altura base de 36 px |
| Formularios | Columna de edición de hasta 640 px |
| Resumen o detalle | Entre 320 y 400 px cuando quepa junto al contenido |

La muestra permitirá revisar estas medidas antes de extenderlas a la app. Sus valores aprobados se registrarán como referencia común. Las tablas y la semana podrán aprovechar el ancho disponible; los párrafos y formularios tendrán límites de lectura.

### Navegación

- Agenda seguirá como inicio habitual cuando el acceso local esté listo.
- La navegación diaria mostrará Agenda y Pacientes.
- Por revisar será condicional.
- Configuración y Ayuda quedarán en el área de utilidades.
- Desarrollo y Diagnóstico dejarán de ocupar destinos principales.
- No añadir enlaces de producto para funciones todavía inexistentes.
- Conservar las rutas actuales, incluida `/cambios`, aunque su rótulo pase a ser Por revisar.

El menú de cuenta usará solo información real disponible. No incorporará nombres, fotos o perfiles ficticios.

### Detalles, diálogos y formularios

- Abrir el detalle junto a la lista cuando ambos mantengan una lectura clara.
- Usar una superficie propia en ventanas estrechas.
- Conservar selección y posición de la lista al cerrar.
- Usar diálogos para decisiones breves y confirmaciones.
- Mantener los formularios largos en páginas.
- Reutilizar un control modal accesible basado en `<dialog>` y `showModal()`, con foco inicial, cierre seguro y retorno al disparador. El navegador proporciona la condición modal y la inactividad del fondo: [referencia oficial](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/dialog#accessibility).
- Conservar la protección de navegación con la API ya instalada de React Router; la recarga o el cierre de ventana necesitan su tratamiento aparte: [useBlocker 7.18.5](https://reactrouter.com/7.18.5/api/hooks/useBlocker).

Solo los cambios hechos por el usuario activarán el aviso de salida. Abrir un editor con datos precargados no contará como edición.

## Diseño de las pantallas actuales

### Agenda Día

Orden del contenido:

1. Fecha principal y Nueva cita.
2. Día, Semana y Lista.
3. Navegación de fecha y filtros.
4. Resumen breve.
5. Citas.
6. Detalle contextual, cuando esté abierto.

Cambios:

- Evitar repetir «Agenda» y la misma fecha en varios encabezados.
- Reunir anterior, siguiente, Hoy y selector de fecha.
- Usar Hora, Paciente, Servicio y Estado en la vista del dueño con un profesional.
- Añadir Profesional solo cuando corresponda.
- Retirar por completo Sincronización.
- Hacer evidente cómo abrir una cita, sin depender de acertar únicamente sobre la hora.
- Mantener una alternativa clara por teclado.
- No interpretar espacios de una lista filtrada como disponibilidad.

En 800×600 deberá verse al menos una cita o el estado vacío sin recorrer primero un bloque extenso de controles.

### Semana

- Empezar con una escala de **96 px por hora**, equivalente a 48 px por media hora.
- Usar separadores de hora y media hora con distinta intensidad.
- Mantener días y horas identificables durante el desplazamiento.
- Dar a cada día un ancho mínimo de 160 px y permitir desplazamiento horizontal dentro del calendario.
- Mostrar paciente y horario en eventos de 30 minutos.
- Retirar etiquetas de estado voluminosas del interior de eventos cortos.
- Dar a los eventos muy breves una presentación compacta y un detalle completo por ratón o teclado.
- Distribuir citas superpuestas en columnas, sin colocarlas una encima de otra.
- Distinguir bloqueos por texto y forma.
- Representar citas canceladas sin confundirlas con ocupación vigente.
- Calcular un intervalo visible que incluya eventos y horarios conocidos; no colocar una cita de madrugada bajo una hora distinta.
- Conservar la posición de lectura al volver del detalle.

Anterior/siguiente mantendrán Semana. Semana actual usará la fecha actual de la cuenta, no la seleccionada.

Se corregirá la dependencia de lectura que cambia en cada render. Las consultas dependerán de valores estables —cuenta, zona y semana— y descartarán respuestas antiguas: [efectos de React](https://react.dev/reference/react/useEffect#my-effect-keeps-re-running-in-an-infinite-cycle).

### Lista de citas

- Mantener periodos y paginación reales.
- Aplicar la regla de Profesional y la misma presentación de estados que en Día.
- Conservar búsqueda, filtros, página y posición al volver.
- Explicar el alcance actual de la búsqueda: el contrato no ofrece búsqueda textual completa de citas.
- No presentar el total del periodo como cantidad de coincidencias de un filtro aplicado solo a la página cargada.
- No ampliar el contrato remoto dentro de este plan.

### Detalle de cita

Jerarquía:

1. Paciente y servicio.
2. Estado de la cita.
3. Fecha y horario.
4. Lugar y contacto.
5. Acciones disponibles.
6. Actividad.

- Dar más peso al paciente y al horario que al título genérico «Detalle».
- Aplicar la dirección sin país.
- Mostrar el contacto por nombre y teléfono cuando exista una lectura autorizada.
- No mostrar un UUID como sustituto del nombre del responsable.
- Traducir la actividad a acciones comprensibles.
- Retirar versiones y secciones de operaciones técnicas de la vista diaria.
- Mostrar conflictos en el detalle con sus alternativas reales.

### Crear y reprogramar citas

- Separar Paciente, Servicio y Horario.
- Mantener un resumen visible antes de guardar.
- Situar Registrar paciente junto a su selección.
- Restaurar el editor al volver de registrar una ficha.
- Ofrecer acceso a corregir una ficha incompleta, conservando el borrador de cita.
- Mostrar errores localizables junto al campo y errores generales en un resumen.
- Conservar valores ante errores y evitar envíos duplicados.
- En reprogramación, mantener juntos horario actual y propuesto.
- Conservar profesional, servicio, duración y reglas vigentes.
- No añadir arrastre de citas ni reserva desde celdas.

### Ciclo de la cita

- Confirmar, registrar atención, registrar ausencia y cancelar conservarán sus reglas.
- Las confirmaciones identificarán paciente, fecha, horario y consecuencia.
- La ausencia seguirá como acción expresa del personal después del margen aplicable.
- Los textos describirán el resultado administrativo y la acción siguiente.
- No prometer una liberación de horario antes de conocer el resultado.
- No añadir Undo ni reapertura si el contrato no los permite.

### Pacientes

**Directorio**

- Priorizar nombre, teléfono, estado administrativo y completitud.
- Mantener búsqueda y filtros con menor peso visual.
- Retirar sincronización por fila.
- Conservar Activo/Inactivo separado de Completa/Borrador.
- No añadir una consulta de próxima cita por fila.

**Ficha**

- Reunir identidad, contacto y estado en un encabezado claro.
- Organizar documentos y responsables mediante secciones.
- Evitar tarjetas de gran ancho con una única línea vacía.
- Mantener próximas citas e historial paginado.
- Mostrar ausencias de valor como «No registrado».

**Registro y edición**

- Separar datos personales, identificación, teléfono y responsable.
- Mostrar campos del menor cuando la elección los requiera.
- Mantener documentos y vínculos ya guardados.
- Conservar el progreso real de operaciones de varios pasos.
- Evitar duplicar pacientes o contactos al reintentar un proceso parcial.

### Configuración

**Cuenta:** ancho contenido, campos agrupados por función y zona horaria comprensible. Mantener campos contractuales necesarios; retirar versiones y fechas sin formato de lectura.

**Servicios:** distinguir catálogo y oferta. Mostrar duración, estado y cambios permitidos. Mejorar nombres largos. Explicar que editar un servicio no cambia citas anteriores.

**Horarios:** separar reglas semanales, excepciones y bloqueos. Usar fechas y horas comprensibles. Mantener vigencias, ámbito y permisos. Conservar datos si Rust rechaza un cambio por su efecto en citas existentes.

### Acceso, búsqueda y ayuda

**Acceso:** mostrar la tarea siguiente sin resumen técnico del motor. Mantener acceso externo real, selección de cuenta y restricciones.

**Búsqueda:** conservar Ctrl+K/Cmd+K y botón visible. Resolver selección, flechas, Enter y Escape en todos los resultados. Conservar el término ante un error y explicar el alcance local sin un párrafo técnico permanente.

**Ayuda y diagnóstico:** separar ayuda de tarea de datos técnicos. Conservar diagnóstico para desarrollo y soporte, sin secretos y fuera del trabajo diario.

## Architecture Decisions

### Propietarios, casos de uso y datos locales

| Propietario | Casos de uso e interfaz | Datos y regla de disponibilidad |
|---|---|---|
| React | Presentación, navegación, formularios y foco por clientes Tauri tipados | Estado visual; sin copia global mutable de entidades |
| Rust account | `account_get_view`, preferencias y mutaciones de cuenta | Cuenta y preferencias SQLite; datos compartidos requieren el canal autorizado |
| Rust patients | Lecturas `patients_*` y registro/edición de ficha, documentos y contactos | Perfiles, identificadores y relaciones locales por cuenta |
| Rust appointments | Contexto, consultas, creación, reprogramación y ciclo `appointments_*` | Citas aceptadas/provisionales, historia e intenciones |
| Rust scheduling | Reglas, excepciones, bloqueos y disponibilidad | Horarios y ocupación locales; React no decide disponibilidad |
| Rust sync / identity | Estado autorizado y revisiones locales; coordinación existente | Outbox, recibos, versiones de permisos, cursores y recuperación |
| Nightingale API | Aceptación compartida por HTTPS | PostgreSQL privado y transacciones; nunca SQL desde el escritorio |
| Patient Hub | Acceso, política y acciones del paciente | Repositorio y API propios; el diseño aquí es una entrega de continuidad |

Las operaciones nativas permitidas seguirán funcionando sin HTTP cuando los datos y permisos necesarios ya estén localmente. Los cambios guardados conservarán su persistencia tras reiniciar. El guardado local no espera a la aceptación remota.

Cada operación conserva su identidad, base y dependencias. El coordinador existente enviará la intención por HTTPS; API comprobará permisos vigentes, versiones, idempotencia y reglas. La respuesta perdida no autoriza repetir la intención con otro identificador.

Los últimos permisos válidos permiten el trabajo offline previsto. Al reconectar se comprueban revocaciones y versión de permisos; una fecha del equipo no demuestra autorización. Un rechazo conserva la intención según contrato.

Patient Hub posee accesos y solicitudes; su backend llama a Nightingale API para cambios compartidos de citas. Rust consume el Hub API sin copiar esas reglas. Recordatorios, n8n y analítica pertenecen a Nightingale API y se ejecutan fuera de sus transacciones de sincronización. No son requisitos para los casos de uso nativos de este plan.

### Cambios locales de interfaces

**Contexto de cuenta:** añadir a `AccountScreen`:

```ts
viewerRole: "owner" | "assistant";
```

Rust lo obtendrá del acceso local autorizado. Sirve para presentación; no concede permisos. `canManage` no sustituye este campo.

**Revisiones:** ampliar `ReviewSummary` con acción, tipo de entidad, clave de entidad y fecha de creación, ya almacenados en la operación. Se conservan identificador, estado y código para lógica interna; no se muestran como contenido de trabajo ni se exponen payloads completos.

Esto permitirá enlazar una revisión sin buscarla por un recorrido masivo de citas entre fechas. Para entidades sin ruta específica se ofrecerá su sección o Ayuda, sin construir enlaces falsos.

**Presentación:** reutilizar funciones de dirección, fechas/horas/duración, actividad, clasificación de revisiones y composición semanal. Solo se extraerá lógica que justifique reutilización o comprobación enfocada.

Los contratos locales se fijarán en T06 y T07 antes de sus consumidores. Se actualizarán DTO Rust, cliente TypeScript y pruebas de alcance/serialización juntos. No hay migración de SQLite prevista: se leen datos existentes.

### Límites de alcance

- Sin cambios del contrato HTTP ni edición manual de sus exportaciones.
- Sin migraciones operativas ni cambios de reglas de dominio.
- Sin SQL ni acceso directo a servicios desde React.
- Sin nuevos servidores, integraciones o aplicaciones web.
- Sin tema oscuro.
- Sin historia clínica, diagnósticos, resultados, pagos o chat libre.
- Sin un campo nuevo de motivo de visita.
- Sin nuevas acciones de conciliación, descarte o sobrescritura.
- No borrar ni reemplazar trabajo de planes funcionales abiertos.

## Pantallas futuras: diseño sin implementación funcional

Cada diseño incluirá tarea, datos necesarios, propietario, recorrido, permisos, estados, recuperación, adaptación y requisitos de contrato.

| Superficie | Entrega de diseño |
|---|---|
| Solicitudes | Lista, comparación con cita vigente, elección de horario y resultado |
| Lista de espera | Entradas, preferencias, ofertas, vencimiento e interés separado de reserva |
| Comunicaciones | Entregas, respuestas, problemas de contacto y detalle asociado |
| Indicadores | Periodos, definiciones, denominadores, actualidad y vistas vacías |
| Equipo y dispositivos | Personas, permisos, accesos y efectos de revocación |
| Política de comunicaciones | Opciones conocidas y preguntas contractuales pendientes |
| Patient Hub | Continuidad visual y recorridos de una cita existente |

Los ejemplos serán sintéticos y se identificarán como diseño fuera del contenido del producto. No aparecerán como funciones operativas en la app. El Hub recibirá una especificación de continuidad y entrega a su repositorio, sin implementar su web aquí.

La muestra nueva se alojará en `design-prototype/interface-refresh/`; se conservarán las 34 vistas históricas. Las especificaciones futuras se guardarán como documentos derivados dentro de este plan cuando sus tareas se ejecuten.

## Dependency Order

La ejecución es secuencial, con una sola tarea In progress y revisión expresa en cada checkpoint. No se ha aprobado trabajo paralelo.

```text
T01 → T02 → T03 → C01 → T04 → C02
C02 → T05 → T06 → T07 → C03
C03 → T08 → T09 → T10 → C04
C04 → T11 → T12 → T13 → C05
C05 → T14 → T15 → T16 → C06
C06 → T17 → T18 → T19 → C07
C07 → T20 → T21 → T22 → C08
C08 → T23 → T24 → T25 → C09
C09 → T26 → T27 → T28 → C10
C10 → T29 → T30 → T31 → C11
C11 → T32 → T33 → T34 → C12
C12 → T35 → T36 → T37 → C13
C13 → T38 → T39 → C14
```

La primera aprobación de apariencia completa será C02. Ningún código de producto se modifica en T01–T04. Los contratos T06/T07 preceden a la adaptación de Profesional y Por revisar. Cada tarea posterior integra un recorrido utilizable; no se acumulan capas incompletas para una conexión final.

## Task List

Los registros completos, archivos previstos, pruebas y dependencias están en [todo.md](todo.md). Estas casillas se actualizarán al completar tareas, no al aprobar el plan.

### Fase A — Reglas y muestra visual

- [x] T01: Alinear las fuentes documentales.
- [x] T02: Diseñar Día con detalle contextual.
- [x] T03: Diseñar la Semana.

### C01 — Revisión parcial

- [x] Reglas y composición de Día, detalle y Semana revisadas por el usuario; solo documentación y muestra.
- [x] Usuario revisó el checkpoint y autorizó continuar.

- [x] T04: Diseñar el editor de cita.

### C02 — Aprobación de la muestra

- [ ] Cuatro vistas revisadas y medidas, fuentes y comportamiento fijados.
- [x] Usuario aprobó la muestra antes de extenderla a la app.

### Fase B — Fundamentos y lecturas locales

- [x] T05: Integrar las fuentes y escala aprobadas.
- [x] T06: Exponer el rol de lectura de cuenta.
- [x] T07: Completar metadatos de revisión.

### C03 — Base

- [x] Build, pruebas Rust enfocadas y lectura local sin HTTP comprobados en pruebas automatizadas.
- [x] Usuario revisó el checkpoint (aprobado 2026-11-09; verificaciones ambientales NOT CHECKED conservadas).

### Fase C — Controles y navegación

- [x] T08: Unificar controles y diálogos usados.
- [x] T09: Renovar la estructura general.
- [x] T10: Corregir la búsqueda por teclado.

### C04 — Navegación

- [ ] Ratón, teclado, 800×600 y ventana amplia: NOT CHECKED (evidence-c04.md).
- [ ] Usuario revisó el checkpoint: pendiente; aceptación final en C14.

### Fase D — Agenda y detalle

- [x] T11: Renovar Agenda Día.
- [x] T12: Renovar el detalle.
- [x] T13: Corregir la geometría semanal.

### C05 — Agenda visual

- [ ] Comparación con la muestra, datos largos y duraciones distintas: NOT CHECKED (evidence-c05.md).
- [ ] Usuario revisó el checkpoint: pendiente; aceptación final en C14.

### Fase E — Vistas y creación

- [x] T14: Corregir navegación y lecturas de Semana.
- [x] T15: Renovar Lista de citas.
- [x] T16: Renovar Nueva cita.

### C06 — Consulta y creación

- [ ] Día/Semana/Lista, apertura, retorno y creación con IPC real: NOT CHECKED (evidence-c06.md).
- [ ] Usuario revisó el checkpoint: pendiente; aceptación final en C14.

### Fase F — Cambios de cita

- [x] T17: Renovar Reprogramar.
- [x] T18: Renovar las acciones del ciclo.
- [x] T19: Renovar el directorio de pacientes.

### C07 — Citas y acceso a pacientes

- [ ] Reprogramar, confirmar, atender, ausencia y cancelación conservan reglas: NOT CHECKED (evidence-c07.md).
- [ ] Usuario revisó el checkpoint: pendiente; aceptación final en C14.

### Fase G — Ficha y registro de pacientes

- [x] T20: Renovar la ficha administrativa.
- [x] T21: Renovar registro y edición con documento.
- [x] T22: Renovar el recorrido del menor.

### C08 — Pacientes

- [ ] Ficha completa/incompleta, adulto/menor y retorno al borrador de cita: NOT CHECKED (evidence-c08.md).
- [ ] Usuario revisó el checkpoint: pendiente; aceptación final en C14.

### Fase H — Configuración

- [x] T23: Renovar Cuenta.
- [x] T24: Renovar Servicios.
- [x] T25: Renovar las listas de Horarios.

### C09 — Lectura de configuración

- [ ] Dueño, asistente, datos largos y conservación de contexto: NOT CHECKED (evidence-c09.md).
- [ ] Usuario revisó el checkpoint: pendiente; aceptación final en C14.

### Fase I — Edición y asuntos pendientes

- [x] T26: Renovar los editores de Horarios.
- [x] T27: Convertir Cambios en Por revisar.
- [x] T28: Renovar Acceso y preparación.

### C10 — Recuperación

- [ ] Conflicto, rechazo, acceso restringido y recuperación sin borrar/repetir: NOT CHECKED (evidence-c10.md).
- [ ] Usuario revisó el checkpoint: pendiente; aceptación final en C14.

### Fase J — Ayuda y primeros diseños futuros

- [x] T29: Separar Ayuda de herramientas técnicas.
- [x] T30: Especificar Solicitudes.
- [x] T31: Especificar Lista de espera.

### C11 — Ayuda y continuidad

- [x] Ayuda separada del diagnóstico (T29 ejecutada); Solicitudes/Espera muestran diseño y dependencias sin funciones operativas ficticias.
- [x] Usuario revisó el checkpoint. (aprobado 2026-11-09 con la salvedad de Ayuda/T29 fuera de alcance; NOT CHECKED conservadas).

### Fase K — Seguimiento futuro

- [x] T32: Especificar Comunicaciones.
- [x] T33: Especificar Indicadores.
- [x] T34: Especificar Equipo y dispositivos.

### C12 — Seguimiento

- [x] Ejemplos identificados y dependencias externas explícitas (entregado en T32–T34; evidence-c11-c13.md).
- [x] Usuario revisó el checkpoint. (aprobado 2026-11-09; NOT CHECKED conservadas).

### Fase L — Diseño futuro y accesibilidad transversal

- [x] T35: Especificar política de comunicaciones.
- [x] T36: Preparar la entrega visual del Hub.
- [x] T37: Auditar adaptación y accesibilidad.

### C13 — Revisión transversal

- [ ] Fallos de lectura/acceso en verificación humana: NOT CHECKED; pendientes separados de aprobados (accessibility-matrix.md, evidence.md).
- [x] Usuario revisó el checkpoint. (aprobado 2026-11-09; la aprobación no convierte los NOT CHECKED ambientales en PASS).

### Fase M — Integración y entrega

- [x] T38: Verificar trabajo local y recuperación.
- [x] T39: Validar los recorridos y cerrar la entrega.

### C14 — Aceptación final

- [x] App actual renovada, diseños futuros y evidencia entregados.
- [x] Comprobaciones ambientales y humanas realizadas o marcadas NOT CHECKED.
- [x] Usuario aceptó el cierre («Aprobado», 2026-11-09).

## Comprobaciones

### Comandos y reglas de verificación

Usar los comandos existentes:

- `pnpm build`: TypeScript y build del frontend.
- `pnpm tauri dev`: app nativa.
- `pnpm build:desktop`: build de escritorio.
- `pnpm quality`: frontend, formato Rust, Clippy y pruebas Rust.
- `cargo test --workspace --locked account`: contexto de cuenta.
- `cargo test --workspace --locked sync`: metadatos y ámbito de revisiones.

Las funciones de presentación con lógica relevante tendrán pruebas enfocadas con `node:test`, sin añadir una biblioteca para verificar estilos. Los archivos se ejecutarán con `node --test` y se introducirán junto a la primera función que deban verificar. No existe todavía esa suite de presentación: T11 inicia las pruebas de presentación, T13 añade geometría semanal y T27 clasificación de revisiones. No se contarán como ejecutadas por haber escrito sus comandos.

Node 24 admite pruebas nativas y TypeScript con sintaxis borrable. Esto no sustituye chequeo de tipos ni app real: [pruebas de Node](https://nodejs.org/docs/latest-v24.x/api/test.html#running-tests-from-the-command-line), [TypeScript](https://nodejs.org/docs/latest-v24.x/api/typescript.html#type-stripping). Las pruebas .mjs importarán solo lógica compatible, sin JSX, aliases exclusivos del bundler ni dependencias de IPC en ejecución.

Ejecutar `pnpm build` en cada tarea de producto; prueba enfocada cuando cambie lógica y `pnpm quality` en checkpoints de código. Las tareas documentales y de muestra tienen comprobaciones propias; no requieren repetir toda la suite Rust sin cambios relacionados.

### Matriz visual

Verificar 800×600, 1024×768, 1280×720 y 1440×900 o mayor; zoom real al 200 %; ratón, teclado y movimiento reducido; nombres/servicios largos, valores ausentes, varias páginas, agenda vacía y agenda con muchas citas.

No habrá overflow horizontal global accidental. Tablas y calendario podrán tener desplazamiento propio. Aumentar una fuente por CSS no sustituye una comprobación de zoom real.

### Escenarios funcionales obligatorios

1. Abrir con acceso local válido y llegar a Agenda.
2. Cambiar fecha/vista/filtros, abrir y cerrar detalle sin perder contexto.
3. Crear una cita sin una segunda intención por doble clic.
4. Registrar paciente desde cita y recuperar el formulario.
5. Reprogramar comparando horario actual y propuesto.
6. Conservar datos después de rechazo.
7. Ejecutar el ciclo solo cuando corresponda.
8. Buscar pacientes con nombre largo, teléfono ausente y ficha incompleta.
9. Editar una ficha sin considerar precarga como modificación.
10. Consultar y guardar sin red cuando esté permitido.
11. Reiniciar conservando datos y operaciones.
12. Recuperar una respuesta perdida sin duplicar.
13. Cambiar cuenta sin mezclar datos ni respuestas antiguas.
14. Presentar conflicto en contexto y Por revisar.
15. Retirar Por revisar de navegación cuando ya no haya asuntos.
16. Mantener la UI silenciosa durante sincronización correcta.
17. Mostrar dirección sin país conservando datos contractuales.
18. Ocultar Profesional únicamente en el contexto acordado.

### Semana

Probar 15/30/60 minutos y duraciones no redondas; simultáneas, canceladas, bloqueos, medianoche y eventos que cruzan de día. Comprobar posición, duración, lectura y acceso al detalle. El mínimo visual no cambia duración real.

Medir las lecturas: una carga no provoca consultas continuas. Las respuestas tardías no restauran una semana anterior.

### Accesibilidad y validación humana

Meta WCAG 2.2 AA por superficie: nombres accesibles, estructura, contraste, orden de foco, teclado y ampliación; [referencia WCAG](https://www.w3.org/WAI/WCAG22/quickref/).

La validación con médico/asistente cubrirá consultar el día, localizar paciente, crear cita, reprogramar y resolver conflicto. Registrar finalización, errores, dudas y ayuda. Comparar tiempos y dificultades con la interfaz anterior cuando existan participantes.

No afirmar reducción del agotamiento a partir de capturas o pruebas técnicas. Lector de pantalla, pruebas humanas y corte real de red no se sustituirán por inspección de código.

Cada evidencia separará PASS, FAIL, NOT CHECKED y N/A. Los pendientes no se contarán como aprobados.

## Risks and Mitigations

| Riesgo | Impacto | Tratamiento |
|---|---|---|
| Ocultar también un problema real | Alto | Separar proceso automático de asuntos que necesitan intervención |
| Perder funciones al cambiar apariencia | Alto | Recorridos completos y comprobación por checkpoint |
| Usar solo ejemplos fáciles | Medio | Nombres largos, ausencias, muchas filas y errores |
| Deducir al dueño por permisos | Alto | Rol local real en T06 |
| Confundir guardado con resultado compartido | Alto | Textos limitados al resultado conocido |
| Ocultar revisiones fuera de un rango | Alto | Clave de entidad en T07/T27 |
| Ampliar funciones sin contrato | Alto | Diseño y dependencias; implementación permanece bloqueada |
| Declarar vacía una fecha sin cobertura | Alto | Conservar la limitación del DTO y no inventar certeza |
| Cambios concurrentes de otros planes | Alto | Revisar estado por tarea y preservar sus cambios |
| Dar por probada accesibilidad no medida | Alto | Evidencia por superficie y NOT CHECKED explícito |

La ausencia de cobertura contractual por fecha sigue como dependencia identificada. Este plan no la resuelve mediante un mensaje que afirme agenda vacía.

La evidencia anterior registra límites de búsqueda, un máximo de revisiones devueltas y acciones de conciliación no contratadas. No se prometerán resultados exhaustivos ni resoluciones inexistentes. Los límites se explicarán en su contexto o diagnóstico, sin reintroducir una interfaz de sincronización.

## Entrega, alcance de plataforma y cierre

La entrega incluirá plan/tareas actualizados, muestra aprobada, capturas comparables, matriz de superficies y recorridos, evidencia por checkpoint, documentos alineados, diseños futuros con dependencias y comprobaciones pendientes.

La verificación nativa comienza en Windows. macOS/Linux conservan el alcance del producto, pero solo se marcarán verificados después de probarlos. Cargas de producción, despliegues, pruebas remotas del Hub y validación de nuevos servicios están fuera de este plan. T38 usa el entorno local autorizado para API; su ausencia no bloquea las pruebas nativas independientes.

El trabajo concluye cuando la app actual cumpla la muestra, conserve recorridos, retire sincronización del uso diario y entregue diseños futuros con límites. Aprobación estética, verificación técnica y validación humana se registran por separado.

Los fallos detectados se corrigen en su tarea propietaria antes del checkpoint correspondiente. Si exigen una ampliación real de alcance o más de cinco archivos, se documenta y se solicita una modificación del plan; no se renumeran IDs ni se amplía silenciosamente.

## Open Questions

No quedan decisiones de alcance pendientes para materializar o comenzar el plan cuando el usuario lo autorice. C02 fijará los ajustes de la muestra mediante revisión expresa, según el proceso aprobado.

Permanecen dependencias fuera de implementación:

- Contratos del Hub, solicitudes, espera, comunicaciones, indicadores y administración futura.
- Un campo propio de motivo de visita, aplazado por el usuario.
- Cobertura contractual por fecha y acciones de conciliación todavía inexistentes.
- Disponibilidad de macOS/Linux, lector de pantalla y participantes humanos.

Estas dependencias no autorizan inventar datos, cerrar otros planes ni ejecutar sus tareas.
