# Task List: P-20261109-22-interface-refresh

**Plan:** [plan.md](plan.md)
**Updated:** 2026-11-09 (C14 aprobado; plan Completed)

## Autorización y ejecución

La instrucción del 2026-11-09 autoriza ejecutar T01–T03 y presentar C01. No autoriza cambios de producto ni T04 antes de revisión expresa. Los checkpoints siguen pendientes de aceptación.

C01 aceptado después: «Lo veo bien, puedes continuar por favor». Se autoriza T04 y presentación de C02. Las verificaciones ambientales pendientes siguen NOT CHECKED.

C02 aprobado por el usuario: «Aprobado […] puedes continuar». Se refuerza la fidelidad a Linear y Attio según DESIGN.md; conservar la paleta propia. Se autoriza T05–T07 y presentación de C03. Las pruebas no ejecutadas siguen NOT CHECKED.

Se conservan T01–T39 y C01–C14. Ejecución secuencial; una sola tarea In progress. Cada tarea depende de la anterior o del checkpoint previo. Antes de cada tarea se leen las fuentes obligatorias del plan y se comprueban cambios concurrentes de planes funcionales.

Los archivos marcados «nuevo» son destinos previstos, no archivos creados al materializar este plan. Una extracción indicada «si hace falta» es una división local del bloque real: no autoriza crear una abstracción sin uso ni ampliar el resultado. Los artefactos de evidencia se guardarán en este plan durante la ejecución. Las pruebas de presentación todavía no existen: las tareas que las introducen lo indican. Ningún comando o recorrido listado a continuación se considera ejecutado por su inclusión.

Los registros de evidencia, plan y todo acompañan la tarea; los límites de hasta cinco archivos se aplican al cambio principal previsto. Si el trabajo real rebasa ese límite, se propone un ajuste explícito sin renumerar IDs ni ampliar el alcance en silencio.

## Phase A — Reglas y muestra visual

## T01: Alinear las fuentes documentales

**Status:** Done

**Description:** Formalizar las decisiones aprobadas sobre tipografía, sincronización invisible, direcciones, adaptación de Profesional e inspiración visual. Cambiar solo las reglas afectadas; conservar contratos de dominio e historial. Esta tarea se ejecutará únicamente después de una instrucción de inicio.

**Acceptance criteria:**
- [x] DESIGN.md, DESIGN-RULES.md y el maestro de diseño usan Montserrat/Mulish y las reglas acordadas, sin instrucciones activas contradictorias.
- [x] Las menciones de presentación en arquitectura y plan maestro respetan la sincronización invisible sin alterar aceptación, permisos, versiones ni persistencia.
- [x] Ningún plan anterior cambia de estado, se cierra o se sustituye; no cambia código ni el prototipo.

**Verification:**
- [x] Focused tests: Revisar las reglas contra la sección «Reglas obligatorias» del plan y buscar términos antiguos con rg; no hay pruebas ejecutables de comportamiento para este cambio documental.
- [x] Build or broader check: git diff --check.
- [x] Manual check: Leer las cinco fuentes en orden y comprobar que un implementador encuentra las mismas instrucciones de UI.

**Dependencies:** None

**Files likely touched:**
- `DESIGN.md`
- `DESIGN-RULES.md`
- `docs/design-master-plan.md`
- `docs/master-plan.md`
- `docs/technical-architecture.md`

**Estimated scope:** M

## T02: Diseñar Día con detalle contextual

**Status:** Done

**Description:** Crear la primera muestra estática nueva de Agenda Día y su detalle lateral en design-prototype/interface-refresh/. Reutilizar la composición del prototipo histórico, con las fuentes aprobadas y ejemplos difíciles, sin cambiar la aplicación ni las 34 vistas originales.

**Acceptance criteria:**
- [ ] Día con y sin detalle muestra paciente, horario, servicio, estado y acción principal en una jerarquía común; no muestra sincronización normal.
- [ ] La dirección termina en ciudad/estado; nombres largos y datos ausentes tienen una presentación clara a 800×600 y en ventana amplia.
- [ ] La muestra carga sus recursos localmente, conserva origen/licencia de las fuentes e identifica los datos sintéticos fuera del contenido del producto.

**Verification:**
- [ ] Focused tests: Inspección visual de Día y detalle con los mismos datos; comprobar navegación/lectura con teclado y carga local de fuentes.
- [x] Build or broader check: git diff --check.
- [ ] Manual check: Comparar con agenda.png y cita-a1.png históricos, conservando el contexto de la lista y la legibilidad de la muestra nueva.

**Dependencies:** T01

**Files likely touched:**
- `design-prototype/interface-refresh/index.html (nuevo)`
- `design-prototype/interface-refresh/styles.css (nuevo)`
- `design-prototype/interface-refresh/assets/montserrat-latin-variable.woff2 (nuevo)`
- `design-prototype/interface-refresh/assets/mulish-latin-variable.woff2 (nuevo)`
- `design-prototype/interface-refresh/assets/LICENSE.md (nuevo)`

**Estimated scope:** M

**Resultado de ejecución:** Entrega de muestra preparada. Inspección visual, teclado y render de fuentes: **NOT CHECKED** por bloqueo de apertura file:// del navegador integrado. Después, el usuario revisó y aceptó C01; entrega Done por esa aceptación. Las comprobaciones técnicas faltantes permanecen NOT CHECKED.

## T03: Diseñar la Semana

**Status:** Done

**Description:** Ampliar la muestra con el calendario semanal, sus límites horarios y eventos. Definir el aspecto de duraciones cortas, solapamientos y bloqueos antes de trasladar su geometría a la app.

**Acceptance criteria:**
- [ ] La base de 96 px/hora y 160 px mínimos por día permite leer paciente y horario en eventos de 30 minutos; horas/medias horas se distinguen.
- [ ] Eventos muy breves, simultáneos, cancelados y bloqueos tienen presentación clara y acceso al detalle completo, sin etiquetas técnicas ni capas que tapen otros eventos.
- [ ] El calendario mantiene desplazamiento propio y cabeceras identificables, sin overflow horizontal del documento en los tamaños de revisión.

**Verification:**
- [ ] Focused tests: Comprobar ejemplos de 15/30/60 minutos, duración no redonda, simultáneas y límites de día; medir visualmente posición y duración.
- [x] Build or broader check: git diff --check.
- [ ] Manual check: Cambiar entre Día, detalle y Semana, y revisar el calendario en 800×600 y 1440×900.

**Dependencies:** T02

**Files likely touched:**
- `design-prototype/interface-refresh/index.html`
- `design-prototype/interface-refresh/styles.css`
- `design-prototype/interface-refresh/app.js (nuevo)`

**Resultado de ejecución:** Muestra semanal preparada después de la entrega de Día. Sintaxis y aritmética de ejemplos: PASS. Revisión en ejecución, medidas visuales y teclado: **NOT CHECKED** por bloqueo de apertura file://. Después, el usuario aceptó C01 y autorizó continuar; entrega Done por esa aceptación, sin convertir los NOT CHECKED en PASS.

**Estimated scope:** M

## C01: Revisión parcial — después de T01–T03

- [x] Reglas y composición de Día, detalle y Semana aceptadas por revisión del usuario; no acredita las comprobaciones ambientales pendientes.
- [ ] Muestra revisada a 800×600 y ventana amplia, con datos sintéticos y sin cambios de producto.
- [x] Revisión expresa del usuario recibida antes de T04.

## Phase A — Cierre de la muestra

## T04: Diseñar el editor de cita

**Status:** Done

**Description:** Completar la muestra representativa con Nueva cita: grupos de paciente, servicio y horario, resumen y estados de validación. Los comportamientos son ejemplos en memoria, no operaciones reales.

**Acceptance criteria:**
- [ ] Campos, grupos, resumen y Guardar siguen las mismas fuentes, proporciones y controles de Día/Semana/detalle.
- [ ] La muestra cubre valores incompletos, error, selección de intervalo y retorno de un registro de paciente sin perder el resto del formulario.
- [x] Las cuatro vistas pueden revisarse juntas y sus medidas/decisiones quedan identificadas para C02, sin cambiar código de producto.

**Verification:**
- [ ] Focused tests: Recorrer el formulario de ejemplo con teclado y comprobar validación, resumen y conservación de valores.
- [x] Build or broader check: git diff --check.
- [ ] Manual check: Revisar las cuatro vistas con el usuario y documentar ajustes solicitados sin dar C02 por aprobado.

**Dependencies:** C01

**Files likely touched:**
- `design-prototype/interface-refresh/index.html`
- `design-prototype/interface-refresh/styles.css`
- `design-prototype/interface-refresh/app.js`
- `design-prototype/interface-refresh/README.md (nuevo)`

**Estimated scope:** M

**Resultado de ejecución:** Entrega del editor preparada en los cuatro archivos previstos. Sintaxis, estructura/enlaces/recursos y diff: PASS. Recorrido real, teclado, foco, render y revisión del usuario: **NOT CHECKED**. Bloqueada para cierre mientras faltan esas comprobaciones; se presenta C02 sin aprobarlo. Ver [evidence-c02.md](evidence-c02.md).

## C02: Aprobación de la muestra — después de T04

- [ ] Día, Semana, detalle y editor comparten el sistema y se revisan juntos.
- [ ] Medidas, tipografía, errores y retorno quedan fijados; recursos locales y datos difíciles comprobados.
- [x] Usuario aprueba la muestra antes de extenderla a la app.

## Phase B — Fundamentos y lecturas locales

**Revisión posterior:** C02 aprobado por el usuario. T04 Done por entrega y aceptación; los NOT CHECKED técnicos conservan ese resultado.

## T05: Integrar las fuentes y escala aprobadas

**Status:** Done

**Description:** Llevar las fuentes locales y medidas de la muestra aprobada a los estilos globales del escritorio. Mantener las dependencias antiguas instaladas hasta retirarlas en T08, para no romper tareas intermedias.

**Acceptance criteria:**
- [ ] Montserrat sirve títulos y Mulish lectura/controles; archivos locales, pesos reales y fallback funcionan sin CDN.
- [ ] La escala, espacios y tamaños globales siguen C02 y mantienen foco visible y movimiento reducido.
- [ ] Español, signos, fechas, números y nombres largos se leen correctamente; la aplicación sigue compilando.

**Verification:**
- [ ] Focused tests: pnpm build; inspeccionar carga local de fuentes y pesos en Agenda, editor y una tabla, sin afirmar una revisión completa de esas superficies.
- [x] Build or broader check: pnpm quality.
- [ ] Manual check: Comparar la app con C02 a tamaño normal y zoom real 200 %, comprobando que no se usa una fuente de red.

**Dependencies:** C02

**Files likely touched:**
- `src/App.css`
- `src/assets/fonts/montserrat-latin-variable.woff2 (nuevo)`
- `src/assets/fonts/mulish-latin-variable.woff2 (nuevo)`
- `src/assets/fonts/LICENSE.md`

**Estimated scope:** M

**Resultado:** Build y quality PASS: 375 pruebas unitarias, 5 de persistencia y 1 doc-test; 9 ignoradas. Ambas fuentes se empaquetan sin cambios de bytes. La escala y los controles existentes ya usan las medidas y tokens de DESIGN.md; no se añade otro estilo. Inspección visual de Agenda/editor/tabla, glifos renderizados y zoom real: **NOT CHECKED**.

## T06: Exponer el rol de lectura de cuenta

**Status:** Done

**Description:** Añadir viewerRole a AccountScreen desde el acceso autorizado Rust y mantener su tipo en TypeScript. Este dato permitirá adaptar Profesional sin inferir el rol a partir de canManage.

**Acceptance criteria:**
- [x] account_get_view devuelve viewerRole owner/assistant desde el acceso local correcto, incluido dueño sin todos los permisos de gestión.
- [x] La lectura no exige HTTP ni cambia autorizaciones; ámbitos de cuenta/usuario/dispositivo y denegación de acceso se conservan.
- [x] DTO, consumidor TypeScript y pruebas de lectura/serialización se actualizan juntos, sin migración ni cambio del contrato HTTP.

**Verification:**
- [x] Focused tests: cargo test --workspace --locked account; añadir los casos dueño/asistente, permisos de gestión distintos y lectura sin red en las pruebas propietarias existentes.
- [x] Build or broader check: pnpm quality.
- [ ] Manual check: Consultar la cuenta desde la app con perfiles autorizados y comprobar rol sin usar identity_list_accounts como requisito de red.

**Dependencies:** T05

**Files likely touched:**
- `src-tauri/src/components/account/application.rs`
- `src-tauri/src/components/account/adapters/inbound.rs`
- `src-tauri/src/tests/components/account/mod.rs`
- `src/shared/api/account.ts`

**Estimated scope:** M

**Resultado:** 30 pruebas de cuenta PASS; quality PASS (376 unitarias, 5 persistencia, 1 doc-test; 9 ignoradas). Rol dueño/asistente, dueño sin gestión, serialización, acceso denegado y preferencias aisladas por cuenta/usuario/dispositivo cubiertos con SQLite local. Consulta desde app con perfiles reales: **NOT CHECKED**. Primer intento de pruebas bajo sandbox falló por acceso HTTP local; repetición autorizada fuera del sandbox pasó sin cambios de código.

## T07: Completar metadatos de revisión

**Status:** Done

**Description:** Ampliar la lectura local ReviewSummary con acción, tipo/clave de entidad y fecha ya persistidos. Conservar el filtro de ámbito y los criterios existentes de revisión para permitir enlaces directos en T27.

**Acceptance criteria:**
- [x] Cada revisión incluye metadatos mínimos para identificar su entidad sin consultar citas por un rango de fechas.
- [x] La respuesta conserva aislamiento de cuenta/usuario/dispositivo y no expone payloads completos, secretos ni información de otra cuenta.
- [x] Rust y tipos TypeScript permanecen coherentes; las pruebas cubren metadatos, ausencia de datos ajenos y resultado vacío.

**Verification:**
- [x] Focused tests: cargo test --workspace --locked sync; cubrir serialización y ámbitos junto a las lecturas propietarias.
- [x] Build or broader check: pnpm quality.
- [ ] Manual check: Abrir revisiones reales de prueba de una cuenta y verificar metadatos internos sin cambiar aún la presentación pública de T27.

**Dependencies:** T06

**Files likely touched:**
- `src-tauri/src/components/sync/adapters/outbound.rs`
- `src-tauri/src/components/identity/desktop.rs`
- `src-tauri/src/tests/components/sync/reconciliation_tests.rs`
- `src/shared/api/identity.ts`
- `src/shared/api/sync.ts`

**Estimated scope:** M

**Resultado:** Pruebas sync PASS (80 correctas, 3 ignoradas). Quality final PASS: 377 unitarias, 5 persistencia y 1 doc-test, 9 ignoradas. Lecturas sin escritura, claves simples/compuestas, códigos, fecha, aislamiento y serialización cubiertos. Consulta interactiva desde app: **NOT CHECKED**. Ver [evidence-c03.md](evidence-c03.md).

## C03: Base — después de T05–T07

- [x] pnpm quality y pruebas account/sync enfocadas pasan; cambios de lectura no exigen migración ni HTTP.
- [ ] Fuentes locales, viewerRole y metadatos de revisión funcionan con aislamiento y sin red.
- [ ] Revisión expresa del usuario recibida antes del bloque siguiente.

**Estado de C03:** Aceptado por el usuario el 2026-11-09. Fuentes empaquetadas y lecturas locales verificadas por pruebas; render, zoom y recorridos interactivos: **NOT CHECKED**. Con la aprobación, el usuario limitó el alcance a los trabajos de diseño y prototipo: se ejecutan T30–T36 en secuencia; T08–T29 y T37–T39 permanecen sin ejecutar.

## Phase C — Controles y navegación

## T08: Unificar controles y diálogos usados

**Status:** Done

**Resultado de ejecución:** Modal común `src/components/primitives/Modal.tsx` basado en `<dialog>`+`showModal()` (MDN); siete consumidores migrados (AppLayout, CommandPalette, PatientEditorPage, AppointmentEditorPage, AccountSettingsPage, ReschedulePage, LifecycleActions). Estilos `.modal`/`.modal::backdrop` reemplazan a `.modal-backdrop`/`.modal-card` (retirados por quedar huérfanos). Dependencia `@fontsource/anek-latin` retirada de package.json/pnpm-lock; LICENSE.md actualizada (sección Anek eliminada; Aileron marcada como recurso heredado sin uso). pnpm build PASS. Apertura/cierre por ratón y teclado en app nativa: NOT CHECKED.

**Description:** Aplicar los controles de la muestra y crear solo el control modal compartido que necesitan varios recorridos. Usar HTML nativo y los paquetes existentes. Retirar la dependencia de Anek al dejar de usarla, sin añadir un catálogo anticipado de componentes.

**Acceptance criteria:**
- [ ] Botones, campos y foco siguen la base aprobada; los controles esenciales mantienen etiquetas y dimensiones utilizables.
- [ ] El diálogo modal común gestiona apertura, foco inicial, Escape/cancelación segura y retorno; el fondo no recibe interacción modal.
- [ ] La dependencia de fuente sin uso se retira coherentemente de manifest/lockfile y se conserva atribución de los recursos que permanezcan.

**Verification:**
- [ ] Focused tests: pnpm build; probar el modal en un consumidor del mismo cambio o una comprobación de desarrollo local, sin dejar una primitiva sin uso en producto.
- [ ] Build or broader check: pnpm quality.
- [ ] Manual check: Abrir/cerrar por ratón y teclado; recorrer Tab/Shift+Tab y verificar retorno al disparador.

**Dependencies:** C03

**Files likely touched:**
- `src/App.css`
- `src/components/primitives/Modal.tsx (nuevo)`
- `src/app/AppLayout.tsx`
- `package.json`
- `pnpm-lock.yaml`

**Estimated scope:** M

## T09: Renovar la estructura general

**Status:** Done

**Resultado de ejecución:** AppLayout renovado: sidebar 216 px con grupo «Trabajo diario» (Agenda, Pacientes, Acceso condicional), «Por revisar» solo cuando hay intervención (conflicto+rechazo+revisión requerida; sin contadores de proceso normal), utilidades al pie (Configuración). Desarrollo sale de la navegación principal (sigue accesible desde Ayuda y su ruta). Topbar 52 px. Lectura de cuenta consolidada con guarda de respuestas obsoletas; rutas conservadas (incluida /cambios con rótulo «Por revisar»). pnpm build PASS. Comprobación manual en app nativa (ratón/teclado/cambio de cuenta): NOT CHECKED.

**Description:** Adaptar sidebar/topbar, contexto de cuenta y utilidades a C02. Mostrar Agenda/Pacientes y reservar Por revisar a intervenciones reales, manteniendo rutas y un contexto local pequeño compartido.

**Acceptance criteria:**
- [ ] Navegación, contexto y proporciones siguen la muestra; Desarrollo deja de ser un destino principal y no aparecen funciones futuras.
- [ ] Por revisar solo aparece por asuntos que requieren intervención, no por pending/in_flight/unknown/retryable normales ni por contadores de sincronización.
- [ ] El contexto de cuenta/rol no duplica entidades completas; cambiar cuenta o ruta no mezcla respuestas, y la navegación estrecha tiene retorno de foco.

**Verification:**
- [ ] Focused tests: pnpm build; comprobar condiciones del enlace con estados reales conocidos y validar rutas previas, incluida /cambios.
- [ ] Build or broader check: pnpm quality.
- [ ] Manual check: Navegar Agenda/Pacientes/Configuración, cambiar cuenta autorizada y abrir/cerrar menú estrecho por teclado.

**Dependencies:** T08

**Files likely touched:**
- `src/app/AppLayout.tsx`
- `src/app/AccountContext.tsx (nuevo si hace falta compartir la lectura pequeña)`
- `src/App.tsx`
- `src/App.css`

**Estimated scope:** M

## T10: Corregir la búsqueda por teclado

**Status:** Done

**Resultado de ejecución:** Manejador de teclado común (contenedor del diálogo): flechas recorren resultados desde el campo o desde un resultado (foco rodante + aria-selected), Enter abre el resultado activo o el primero desde el campo, Escape cierra con retorno de foco al disparador (showModal nativo). Consulta conservada ante error; alcance local explicado sin prometer exhaustividad. Migrado al Modal común de T08. pnpm build PASS. Recorrido manual con IPC real: NOT CHECKED.

**Description:** Aplicar el diálogo común a la búsqueda y completar la interacción en toda la lista de resultados, no solo en el campo. Mantener la búsqueda local autorizada y sus límites reales.

**Acceptance criteria:**
- [ ] Ctrl+K/Cmd+K y el botón abren la misma búsqueda; flechas, Enter y Escape funcionan desde campo y resultados.
- [ ] La consulta sobrevive a fallos; resultados de paciente/cita tienen contexto legible y no prometen exhaustividad fuera de los datos/rangos disponibles.
- [ ] El foco permanece en la superficie modal y vuelve al disparador al cerrar; no se pierden borradores al abrir un resultado.

**Verification:**
- [ ] Focused tests: pnpm build; recorrer al menos dos resultados con flechas y Enter, cerrar desde un resultado con Escape y comprobar consulta con error.
- [ ] Build or broader check: pnpm quality.
- [ ] Manual check: Buscar paciente/cita por ratón y teclado, abrir resultado y regresar al contexto de partida.

**Dependencies:** T09

**Files likely touched:**
- `src/features/search/CommandPalette.tsx`
- `src/components/primitives/Modal.tsx`
- `src/App.css`

**Estimated scope:** M

## C04: Navegación — después de T08–T10

- [x] pnpm quality pasa (377+5+1; 9 ignoradas); modales, rutas y búsqueda compilan y tipan. Función en app nativa: NOT CHECKED. Evidencia: [evidence-c04.md](evidence-c04.md).
- [ ] Ratón, teclado, foco y navegación estrecha a 800×600 y ventana amplia: NOT CHECKED.
- [ ] Revisión expresa del usuario: pendiente; la instrucción de ejecución autoriza continuar y la aceptación se solicitará en C14.

## Phase D — Agenda y detalle

## T11: Renovar Agenda Día

**Status:** Done

**Resultado de ejecución:** Vista Día renovada: encabezado con fecha principal (Montserrat) y Nueva cita; navegación de fecha y filtros reunidos en una barra; resumen breve sin avisos de actualización (contenido estable); tabla sin columna Sincronización ni etiquetas de proceso; columna y filtro Profesional condicionales (dueño + un profesional) vía viewerRole T06 y showsProfessionalColumn; paciente y hora abren el detalle; caption redundante retirada. Pruebas node:test nuevas (tests/ui/agenda-presentation.test.mjs): política Profesional + fin 24:00, 2 PASS. pnpm build PASS. Comparación manual con C02 y perfiles dueño/asistente: NOT CHECKED.

**Description:** Aplicar la muestra a la agenda funcional, agrupando fecha/vistas/filtros y adaptando Profesional al rol local y al contexto disponible. Iniciar las pruebas de presentación puras que necesita esta política.

**Acceptance criteria:**
- [ ] La tabla diaria usa Hora/Paciente/Servicio/Estado en dueño con un profesional, conserva Profesional cuando corresponde y mantiene professionalId en operaciones.
- [ ] Desaparecen columna/etiquetas de sincronización, redundancias de fecha y avisos normales; una cita o el vacío son visibles inicialmente a 800×600.
- [ ] Filas, filtros y detalle conservan teclado, fecha, selección y retorno; las funciones extraídas tienen pruebas enfocadas.

**Verification:**
- [ ] Focused tests: Not available yet - define in T11: crear pruebas node:test para la política dueño/asistente/un profesional y funciones puras usadas; ejecutar node --test tests/ui/agenda-presentation.test.mjs después de crearlas. Ejecutar pnpm build.
- [ ] Build or broader check: pnpm quality.
- [ ] Manual check: Comparar Día con C02 en perfiles dueño/asistente, con/sin filtros, tabla larga y ventana pequeña.

**Dependencies:** C04

**Files likely touched:**
- `src/features/appointments/AgendaPage.tsx`
- `src/features/appointments/presentation.ts`
- `src/App.css`
- `tests/ui/agenda-presentation.test.mjs (nuevo)`

**Estimated scope:** M

## T12: Renovar el detalle

**Status:** Done

**Resultado de ejecución:** Detalle con jerarquía aprobada: título = paciente, servicio+duración como subtítulo; hechos en orden Estado, Fecha e intervalo, Teléfono, Lugar (dirección sin país vía formatAddressLine), Profesional, Confirmación, Contacto responsable (nombre resuelto por listPatientContacts; nunca UUID). «Historia aceptada» pasa a «Actividad» con acciones traducidas (historyActionLabel) y sin versiones técnicas; sección «Operaciones locales» retirada de la vista diaria; aviso de provisionalidad eliminado (sincronización invisible). Pruebas: dirección y actividad añadidas (4 PASS). pnpm build PASS. Comprobación manual (responsable, dirección parcial, conflicto, detalle estrecho): NOT CHECKED.

**Description:** Dar prioridad a paciente, horario y acciones; aplicar dirección común, actividad legible y conflictos en contexto. Conservar la superficie propia cuando lista y detalle no caben juntos.

**Acceptance criteria:**
- [ ] Paciente, servicio, horario, contacto y acciones tienen la jerarquía aprobada y dirección sin país; no se usa un UUID como nombre de responsable.
- [ ] Actividad y mensajes describen trabajo real; versiones y operaciones técnicas dejan la vista diaria, conservando el historial persistido.
- [ ] Abrir/cerrar mantiene lista y foco; un conflicto conserva comparación y solo ofrece acciones existentes.

**Verification:**
- [ ] Focused tests: node --test tests/ui/agenda-presentation.test.mjs; añadir dirección con campos ausentes y etiquetas de actividad cuando se introduzca esa lógica. Ejecutar pnpm build.
- [ ] Build or broader check: pnpm quality.
- [ ] Manual check: Abrir cita con y sin responsable/contacto, dirección parcial, historia y conflicto; probar detalle estrecho y lateral.

**Dependencies:** T11

**Files likely touched:**
- `src/features/appointments/AppointmentDetail.tsx`
- `src/features/appointments/AppointmentPage.tsx`
- `src/features/appointments/presentation.ts`
- `src/App.css`
- `tests/ui/agenda-presentation.test.mjs`

**Estimated scope:** M

## T13: Corregir la geometría semanal

**Status:** Done

**Resultado de ejecución:** Geometría extraída a week-layout.ts (visibleMinuteRange amplía el rango a eventos reales en horas enteras; eventBox a 96 px/hora; assignColumns distribuye solapamientos sin taparse). WeekPage reescrito: rejilla con cabeceras y horas pegajosas, días de 160 px mínimos con scroll propio, eventos de 30 min con paciente+horario, eventos breves compactos con detalle completo en title/focus, canceladas con estilo tachado sin confusión de ocupación, bloqueos por texto y trama. Pruebas tests/ui/week-layout.test.mjs: 7 PASS. pnpm build PASS. Comparación visual con C02 (15/30/60, no redondas, simultáneas, bloqueos, medianoche): NOT CHECKED.

**Description:** Trasladar el calendario aprobado a eventos reales, separando el cálculo comprobable de posiciones/solapamientos de su presentación. No modificar disponibilidad ni duración de las citas.

**Acceptance criteria:**
- [ ] Escala, cabeceras y desplazamiento siguen C02; paciente/horario de 30 minutos y detalle de eventos breves son accesibles.
- [ ] Eventos solapados, canceladas y bloqueos no se tapan ni confunden ocupación; eventos fuera del horario inicial se representan en su hora real.
- [ ] Pruebas de geometría cubren límites de día y duraciones variadas; el tamaño visual no cambia instantes ni duración.

**Verification:**
- [ ] Focused tests: Not available yet - define in T13: crear pruebas de geometría con node:test y ejecutar node --test tests/ui/week-layout.test.mjs; ejecutar pnpm build.
- [ ] Build or broader check: pnpm quality.
- [ ] Manual check: Comparar con C02 usando citas de 15/30/60 minutos, duración no redonda, simultáneas, bloqueos y medianoche.

**Dependencies:** T12

**Files likely touched:**
- `src/features/appointments/WeekPage.tsx`
- `src/features/appointments/week-layout.ts (nuevo)`
- `src/App.css`
- `tests/ui/week-layout.test.mjs (nuevo)`

**Estimated scope:** M

## C05: Agenda visual — después de T11–T13

- [x] pnpm quality pasa (377+5+1; 9 ignoradas); node --test agenda-presentation 4/4 y week-layout 7/7 PASS. Evidencia: [evidence-c05.md](evidence-c05.md).
- [ ] Día/detalle/Semana con C02, nombres largos, roles y duraciones distintas: NOT CHECKED.
- [ ] Revisión expresa del usuario: pendiente; aceptación final en C14.

## Phase E — Vistas y creación

## T14: Corregir navegación y lecturas de Semana

**Status:** Done

**Resultado de ejecución:** weekStartCivil y days memorizados (useMemo) para que la lectura dependa de valores estables (zona, semana) y no se repita por identidad en cada render; guarda de generación ya descartaba respuestas obsoletas. «Semana actual» usa civilToday(zone) en lugar de la fecha seleccionada. Posición de scroll del calendario conservada y restaurada vía sessionStorage al volver del detalle; contenido conocido se mantiene durante lecturas (sin parpadeo ni avisos). pnpm build PASS. Conteo de lecturas IPC en app real y cambios rápidos de fecha/cuenta: NOT CHECKED.

**Description:** Corregir la dependencia cambiante de los efectos de Semana y separar cambio de periodo de cambio a Día. Mantener contenido durante lecturas y restaurar posición al volver del detalle.

**Acceptance criteria:**
- [ ] Anterior/siguiente permanecen en Semana; Semana actual usa hoy en la zona de la cuenta y cada cabecera permite abrir Día de forma expresa.
- [ ] Una selección estable no provoca consultas continuas; las respuestas obsoletas se ignoran y el contenido conocido no parpadea.
- [ ] Regresar de detalle conserva semana, selección y posición; cambios rápidos de fecha/cuenta no restauran datos previos.

**Verification:**
- [ ] Focused tests: node --test tests/ui/week-layout.test.mjs; pnpm build. Contar lecturas IPC al montar, cambiar semana y esperar sin interacción; no usar una transición CSS como prueba de latencia.
- [ ] Build or broader check: pnpm quality.
- [ ] Manual check: Avanzar/retroceder varias semanas, volver a actual y abrir detalle mientras termina una lectura anterior.

**Dependencies:** C05

**Files likely touched:**
- `src/features/appointments/WeekPage.tsx`
- `src/features/appointments/AgendaPage.tsx`
- `src/features/appointments/presentation.ts`
- `tests/ui/week-layout.test.mjs`

**Estimated scope:** M

## T15: Renovar Lista de citas

**Status:** Done

**Resultado de ejecución:** Columna Sincronización y etiquetas de proceso retiradas; caption redundante eliminado; columna Profesional condicional vía prop showProfessional; búsqueda/filtros/página persistidos en sessionStorage y restaurados al volver de una cita; texto de estado distingue coincidencias de página («N de esta página coinciden…; total en el periodo») sin afirmar exhaustividad; aviso «Actualizando…» retirado y contenido conservado durante lecturas. pnpm build PASS. Recorrido manual (paginar, abrir cita, volver, rol dueño/asistente): NOT CHECKED.

**Description:** Aplicar la presentación común a próximas citas/historial, con periodo, filtros y paginación reales. Conservar el alcance limitado de búsqueda y el contexto de retorno.

**Acceptance criteria:**
- [ ] Columnas y filtros adaptan Profesional y retiran sincronización sin cambiar el contrato de consulta.
- [ ] El texto de búsqueda distingue coincidencias cargadas del total del periodo; no afirma una búsqueda exhaustiva inexistente.
- [ ] Periodo, búsqueda, filtros, página y posición se restauran al volver de una cita; vacío, sin coincidencias y error se distinguen.

**Verification:**
- [ ] Focused tests: pnpm build; comprobar próximas/historial, paginación y una búsqueda que no coincide en la página pero tiene total del periodo mayor que cero.
- [ ] Build or broader check: pnpm quality.
- [ ] Manual check: Filtrar, pasar página, abrir cita y volver; comprobar rol dueño/asistente y nombres largos.

**Dependencies:** T14

**Files likely touched:**
- `src/features/appointments/AppointmentListPage.tsx`
- `src/features/appointments/AgendaPage.tsx`
- `src/features/appointments/presentation.ts`
- `src/App.css`

**Estimated scope:** M

## T16: Renovar Nueva cita

**Status:** Done

**Resultado de ejecución:** Editor agrupado en Paciente (selector + Registrar paciente junto a él), Servicio y Horario (fecha + disponibilidad dentro del grupo). Dirty solo por acción del usuario (precarga/selección automática/restauración de borrador no bloquean la salida); borrador restaurado al volver de registrar ficha (sin cambios). Mensaje post-guardado «Cita guardada.»; resumen previo a guardar conservado; disponibilidad por IPC; intención única ante doble clic (savingRef); valores conservados ante error. Texto de cobertura sin jerga de aceptación. pnpm build PASS. Recorrido manual de creación con IPC real: NOT CHECKED.

**Description:** Aplicar los grupos y resumen aprobados al recorrido real de creación. Conservar disponibilidad Rust, borrador de registro relacionado, errores y protección de una única intención.

**Acceptance criteria:**
- [ ] Paciente, Servicio y Horario tienen grupos claros, resumen anterior a guardar y registro de paciente junto al selector.
- [ ] Volver de registrar/corregir ficha restaura el formulario; precarga y selección automática no cuentan como cambios del usuario.
- [ ] Errores conservan valores, disponibilidad se obtiene por IPC y doble clic no crea otra intención; no aparecen avisos normales de sincronización.

**Verification:**
- [ ] Focused tests: pnpm build; cargo test --workspace --locked appointments. Comprobar creación, ficha incompleta, disponibilidad ausente y doble clic en un entorno sintético.
- [ ] Build or broader check: pnpm quality.
- [ ] Manual check: Crear desde Agenda y ficha; registrar paciente a mitad, volver, corregir error y revisar resultado sin perder selección.

**Dependencies:** T15

**Files likely touched:**
- `src/features/appointments/AppointmentEditorPage.tsx`
- `src/features/appointments/BookingFields.tsx`
- `src/features/appointments/BookingSummary.tsx`
- `src/features/appointments/AvailabilityPicker.tsx`
- `src/App.css`

**Estimated scope:** M

## C06: Consulta y creación — después de T14–T16

- [x] pnpm quality pasa; node --test 11/11; dependencias de Semana estabilizadas (ciclo continuo corregido por construcción; medición IPC real NOT CHECKED). Evidencia: [evidence-c06.md](evidence-c06.md).
- [ ] Día/Semana/Lista, retorno y creación con IPC real: NOT CHECKED.
- [ ] Revisión expresa del usuario: pendiente; aceptación final en C14.

## Phase F — Cambios de cita

## T17: Renovar Reprogramar

**Status:** Done

**Resultado de ejecución:** Comparación actual/propuesta estable (panel lateral con fecha, intervalo y duración conservada); el slot solo se invalida al cambiar de fecha — errores de disponibilidad y reintentos no borran la propuesta; mensaje «Reprogramación guardada.»; intención única (savingRef) y mismo-intervalo deshabilitado. pnpm build PASS. Recorrido manual (conflicto, retorno desde error): NOT CHECKED.

**Description:** Presentar horario actual y propuesto como comparación estable. Mantener intervalo/servicio/profesional y las restricciones de Rust, con errores y salida seguros.

**Acceptance criteria:**
- [ ] Horario actual y nuevo se leen juntos en ancho amplio y con etiquetas en estrecho; resumen incluye duración real.
- [ ] La selección solo cambia por una acción del usuario o invalidación necesaria; un error no borra la propuesta.
- [ ] Solo se envían los campos permitidos de reprogramación y una intención; navegar sin modificar no muestra un falso aviso.

**Verification:**
- [ ] Focused tests: pnpm build; cargo test --workspace --locked reschedule. Comprobar mismo intervalo, conflicto y retorno desde error.
- [ ] Build or broader check: pnpm quality.
- [ ] Manual check: Cambiar fecha/intervalo, comparar, cancelar salida y guardar una reprogramación sintética válida.

**Dependencies:** C06

**Files likely touched:**
- `src/features/appointments/ReschedulePage.tsx`
- `src/features/appointments/AvailabilityPicker.tsx`
- `src/features/appointments/presentation.ts`
- `src/App.css`

**Estimated scope:** M

## T18: Renovar las acciones del ciclo

**Status:** Done

**Resultado de ejecución:** Confirmaciones con paciente, fecha, hora y consecuencia sin detalles API («cuando se acepte la cancelación»); resultado «Cita guardada.» limitado a la persistencia local conocida; estados terminales sin reapertura; envío con intención única y botones deshabilitados durante el envío; Escape/volver preservan contexto con retorno de foco (Modal T08). pnpm build PASS. Transiciones con datos sintéticos: NOT CHECKED.

**Description:** Usar confirmaciones breves con sujeto y consecuencia para las acciones administrativas de la cita. Reutilizar el modal común sin cambiar permisos ni transiciones.

**Acceptance criteria:**
- [ ] Confirmar, atender, registrar ausencia y cancelar conservan estado/tiempo/margen válidos; el backend sigue validando.
- [ ] Las confirmaciones muestran paciente, fecha, hora y consecuencia sin detalles API ni promesas de liberación desconocida.
- [ ] Escape/volver preservan contexto y la acción segura; el envío no duplica la intención y los estados terminales no ofrecen reapertura.

**Verification:**
- [ ] Focused tests: pnpm build; cargo test --workspace --locked appointments. Cubrir antes/después del inicio y margen de ausencia mediante fixtures/reloj de prueba.
- [ ] Build or broader check: pnpm quality.
- [ ] Manual check: Revisar y ejecutar las transiciones permitidas con datos sintéticos; comprobar confirmación, cancelación de diálogo y resultado.

**Dependencies:** T17

**Files likely touched:**
- `src/features/appointments/LifecycleActions.tsx`
- `src/components/primitives/Modal.tsx`
- `src/features/appointments/presentation.ts`
- `src/App.css`

**Estimated scope:** M

## T19: Renovar el directorio de pacientes

**Status:** Done

**Resultado de ejecución:** Tabla Paciente/Teléfono/Completitud/Estado sin columna Sincronización ni caption redundante; completitud y estado siguen separados; búsqueda/filtros/página conservados con retorno de foco a la ficha; sin consulta de próxima cita por fila; aviso «Actualizando…» retirado y contenido conservado en lecturas. pnpm build PASS. Recorrido manual con ficha larga/teléfono ausente y teclado: NOT CHECKED.

**Description:** Ordenar búsqueda, filtros y tabla de pacientes con el mismo lenguaje visual de Agenda. Mantener estado y completitud separados y no añadir consultas por fila.

**Acceptance criteria:**
- [ ] Nombre, teléfono, estado y completitud tienen lectura clara sin sincronización por fila ni acciones principales competidoras.
- [ ] Búsqueda/filtros/página conservan su estado y foco al volver de ficha; sin coincidencias y vacío se distinguen.
- [ ] No se añade una consulta de próxima cita por fila; nombres largos y teléfonos ausentes funcionan en ventana pequeña.

**Verification:**
- [ ] Focused tests: pnpm build; cargo test --workspace --locked patients. Comprobar filtros de estado/completitud, búsqueda vacía y paginación real.
- [ ] Build or broader check: pnpm quality.
- [ ] Manual check: Encontrar una ficha, abrirla y volver a la misma búsqueda/página, también con teclado.

**Dependencies:** T18

**Files likely touched:**
- `src/features/patients/PatientsPage.tsx`
- `src/features/patients/presentation.ts`
- `src/App.css`

**Estimated scope:** M

## C07: Citas y acceso a pacientes — después de T17–T19

- [x] pnpm quality pasa (377+5+1; 9 ignoradas). Evidencia: [evidence-c07.md](evidence-c07.md).
- [ ] Reprogramar, confirmar, atender, ausencia y cancelar con datos sintéticos; directorio con contexto: NOT CHECKED.
- [ ] Revisión expresa del usuario: pendiente; aceptación final en C14.

## Phase G — Ficha y registro de pacientes

## T20: Renovar la ficha administrativa

**Status:** Done

**Resultado de ejecución:** Encabezado con nombre + completitud/estado (sin aviso de provisionalidad); citas con Fecha/Intervalo/Servicio/Estado, sin columna Sincronización ni etiquetas de proceso; documentos sin badges de proceso; ausencias como «No registrado»; próximas citas e historial con paginación conservadas. pnpm build PASS. Ficha parcial/completa y Nueva cita desde ficha: NOT CHECKED.

**Description:** Recomponer encabezado, datos, documentos, responsables y citas sin cambiar los datos autorizados. Sustituir tarjetas vacías extensas por secciones útiles.

**Acceptance criteria:**
- [ ] Identidad/contacto/estado dominan; documentos y responsables se agrupan y los datos ausentes se muestran como No registrado.
- [ ] Próximas citas e historial conservan paginación, enlaces y estados administrativos, sin columnas técnicas.
- [ ] Nueva cita recibe el paciente correcto; retorno a directorio restaura contexto y el foco no se pierde con lecturas parciales.

**Verification:**
- [ ] Focused tests: pnpm build; cargo test --workspace --locked patients. Comprobar ficha parcial, completa, error de una sección e historial paginado.
- [ ] Build or broader check: pnpm quality.
- [ ] Manual check: Consultar ficha sin documentos, con responsable y con varias citas; abrir Nueva cita y volver.

**Dependencies:** C07

**Files likely touched:**
- `src/features/patients/PatientRecordPage.tsx`
- `src/features/patients/presentation.ts`
- `src/features/appointments/presentation.ts`
- `src/App.css`

**Estimated scope:** M

## T21: Renovar registro y edición con documento

**Status:** Done

**Resultado de ejecución:** Línea base (snapshot de valores cargados) separa precarga de edición: abrir la ficha ya no activa el aviso de salida; editar y restaurar al original según corresponda; país emisor, tipo y número conservados; errores conservan valores. pnpm build PASS. Apertura sin editar/cambio/rechazo de documento: NOT CHECKED.

**Description:** Agrupar perfil, teléfono e identificación en el formulario existente. Corregir el control de cambios para comparar contra el estado cargado y mostrar errores junto a sus campos.

**Acceptance criteria:**
- [ ] Etiquetas, ayuda y errores son claros; país emisor, texto de documento y valores existentes se conservan.
- [ ] La precarga no activa un aviso de salida; editar sí lo activa y conservar/descartar actúan según lo mostrado.
- [ ] Errores y reintentos parciales preservan valores e identidad de paciente, sin repetir altas ya guardadas.

**Verification:**
- [ ] Focused tests: pnpm build; cargo test --workspace --locked patients. Comprobar apertura sin editar, cambio/restauración al original y rechazo de un documento.
- [ ] Build or broader check: pnpm quality.
- [ ] Manual check: Registrar/editar una ficha de prueba y comprobar salir sin cambios, salir con cambios y volver desde error.

**Dependencies:** T20

**Files likely touched:**
- `src/features/patients/PatientEditorPage.tsx`
- `src/features/patients/presentation.ts`
- `src/components/primitives/Modal.tsx`
- `src/App.css`

**Estimated scope:** M

## T22: Renovar el recorrido del menor

**Status:** Done

**Resultado de ejecución:** Campos de responsable solo para menor sin documento, con relación, vigencia y explicación de elegibilidad; nuevo/existente con búsqueda local; anti-duplicación en reintentos (paciente, contacto y vínculo ya creados se reutilizan); retorno desde cita restaura el borrador y selecciona la ficha; textos de guardado limitados a la persistencia local. pnpm build PASS. Menor sintético desde Nueva cita: NOT CHECKED.

**Description:** Aplicar el formulario renovado a menor sin documento y su responsable. Hacer claros modo nuevo/existente, relación y vigencia, conservando reglas e identificadores de operaciones parciales.

**Acceptance criteria:**
- [ ] Los campos de responsable aparecen solo cuando corresponden y explican relación y vigencia sin atribuir tutela no acreditada.
- [ ] Buscar/seleccionar contacto existente o crear uno mantiene valores y muestra fallos junto al paso afectado.
- [ ] Reintentar no duplica paciente/contacto/vínculo; terminar desde una cita restaura el borrador y selecciona la ficha correcta.

**Verification:**
- [ ] Focused tests: pnpm build; cargo test --workspace --locked patients. Cubrir contacto nuevo/existente, ausencia de teléfono y fallo entre pasos.
- [ ] Build or broader check: pnpm quality.
- [ ] Manual check: Registrar un menor sintético desde Nueva cita, volver al editor y comprobar responsable y vigencia.

**Dependencies:** T21

**Files likely touched:**
- `src/features/patients/PatientEditorPage.tsx`
- `src/features/patients/ResponsibleContactFields.tsx (nuevo solo para separar este bloque real)`
- `src/features/patients/presentation.ts`
- `src/App.css`

**Estimated scope:** M

## C08: Pacientes — después de T20–T22

- [x] pnpm quality pasa (377+5+1; 9 ignoradas); anti-duplicación en pasos parciales por construcción. Evidencia: [evidence-c08.md](evidence-c08.md).
- [ ] Ficha completa/incompleta, adulto/menor y alta desde cita con retorno: NOT CHECKED.
- [ ] Revisión expresa del usuario: pendiente; aceptación final en C14.

## Phase H — Configuración

## T23: Renovar Cuenta

**Status:** Done

**Resultado de ejecución:** Versión aceptada y fecha técnica retiradas del encabezado (conservadas internamente); zona horaria comprensible con ejemplo; canales separados (datos compartidos vs preferencias locales) y permiso de solo lectura explicado; textos de resultado sin jerga de API ni de actualización de vista; estado coherente de precarga/guardado/error con dirty real. pnpm build PASS. Edición como dueño/asistente y conservación ante fallo: NOT CHECKED.

**Description:** Aplicar ancho contenido, agrupación y resultados claros a los datos compartidos y preferencias locales. Mantener país editable y requisitos de conexión/gestión.

**Acceptance criteria:**
- [ ] Cuenta, dirección y política se leen por grupos; se retiran versión y fecha técnica de la vista diaria, sin borrar datos contractuales.
- [ ] Preferencias locales y cambios compartidos conservan sus canales y permisos; dirección de lectura omite país.
- [ ] Precarga, guardar y error mantienen un estado de cambios coherente y no prometen una actualización aún no conocida.

**Verification:**
- [ ] Focused tests: pnpm build; cargo test --workspace --locked account. Comprobar preferencia local, solo lectura, error y conflicto de versión con fixtures.
- [ ] Build or broader check: pnpm quality.
- [ ] Manual check: Abrir como dueño/asistente, editar un dato permitido y comprobar ancho, dirección y conservación ante fallo.

**Dependencies:** C08

**Files likely touched:**
- `src/features/settings/AccountSettingsPage.tsx`
- `src/features/settings/SettingsLayout.tsx`
- `src/features/appointments/presentation.ts`
- `src/App.css`

**Estimated scope:** M

## T24: Renovar Servicios

**Status:** Done

**Resultado de ejecución:** Catálogo y ofertas distinguen base/ajuste/efectiva y estado, sin columna Sincronización ni códigos de proceso; mensajes de guardado/activación limitados al resultado conocido; efecto «un cambio no modifica citas anteriores» explicado junto al editor; error conserva el formulario y la selección (editor solo se cierra al guardar con éxito). pnpm build PASS. Nombres largos e inactivos en ejecución: NOT CHECKED.

**Description:** Organizar catálogo y oferta con nombres, duración y estado legibles. Usar el mismo sistema de acciones y diálogo sin exponer versiones o sincronización normal.

**Acceptance criteria:**
- [ ] Catálogo y oferta se distinguen sin jerga de implementación; nombre largo y duración conservan lectura a 800×600.
- [ ] Crear/editar/activar mantienen permisos y versiones internas, sin etiquetas de sincronización visibles.
- [ ] Se explica que un cambio no modifica citas anteriores; error conserva el formulario y la selección.

**Verification:**
- [ ] Focused tests: pnpm build; cargo test --workspace --locked services. Comprobar catálogo/oferta, duración heredada y error al editar.
- [ ] Build or broader check: pnpm quality.
- [ ] Manual check: Revisar nombres largos, servicio inactivo, oferta y modificación de un servicio usado por una cita existente.

**Dependencies:** T23

**Files likely touched:**
- `src/features/services/ServicesPage.tsx`
- `src/components/primitives/Modal.tsx`
- `src/App.css`

**Estimated scope:** M

## T25: Renovar las listas de Horarios

**Status:** Done

**Resultado de ejecución:** Reglas, excepciones y bloqueos con secciones, etiquetas, vigencias y fechas/horas comprensibles; columnas técnicas retiradas; ámbito cuenta/global visible; motivo de bloqueo traducido y ausente mostrado como «No registrado» (sin inferir motivos restringidos); vacío/error distinguibles y contexto conservado. pnpm build PASS. Tres tipos como dueño/asistente: NOT CHECKED.

**Description:** Separar presentación de reglas semanales, excepciones y bloqueos con sus vigencias y ámbitos. Conservar consultas y acciones existentes para preparar los editores de T26.

**Acceptance criteria:**
- [ ] Cada tipo de horario tiene sección, etiquetas y fechas/horas comprensibles; las tablas no repiten estados técnicos.
- [ ] Ámbito de cuenta/global y permisos visibles reflejan los datos autorizados sin inferir motivos restringidos.
- [ ] Lecturas conservan contexto y distinguen vacío/error; nombres, fechas y vigencias funcionan con zoom y ventana pequeña.

**Verification:**
- [ ] Focused tests: pnpm build; cargo test --workspace --locked scheduling. Comprobar reglas con vigencia, cierre/reemplazo y bloqueo global restringido.
- [ ] Build or broader check: pnpm quality.
- [ ] Manual check: Consultar los tres tipos como dueño/asistente y comprobar ausencia de motivos globales no autorizados.

**Dependencies:** T24

**Files likely touched:**
- `src/features/scheduling/ScheduleSettingsPage.tsx`
- `src/features/scheduling/ScheduleLists.tsx (nuevo si el bloque necesita separarse)`
- `src/App.css`

**Estimated scope:** M

## C09: Lectura de configuración — después de T23–T25

- [x] pnpm quality pasa (377+5+1; 9 ignoradas). Evidencia: [evidence-c09.md](evidence-c09.md).
- [ ] Dueño/asistente, nombres largos, versiones internas y retorno: NOT CHECKED.
- [ ] Revisión expresa del usuario: pendiente; aceptación final en C14.

## Phase I — Edición y asuntos pendientes

## T26: Renovar los editores de Horarios

**Status:** Done

**Resultado de ejecución:** Editores ya integrados en T25 con campos pertinentes (horario de reemplazo solo cuando aplica), fechas/horas nativas (sin minutos internos ni RFC 3339), rechazo incompatible con citas mostrado como error con formulario conservado, intención única ante doble clic (savingRef) y reintento con valores preservados. pnpm build PASS. Rango inválido/ámbito global/rechazo con cita: NOT CHECKED.

**Description:** Ordenar fechas, horas, vigencia y ámbito en los editores funcionales. Mantener la validación de consecuencias en Rust/API y conservar el formulario cuando se rechace una propuesta.

**Acceptance criteria:**
- [ ] Cada editor muestra solo campos pertinentes, con unidades y alcance claros; no solicita minutos internos ni RFC 3339.
- [ ] Editar conserva reglas, permisos y efectos contratados; un cambio incompatible con citas no se presenta como exitoso.
- [ ] Error, salida y reintento preservan valores y evitan otra intención por doble clic.

**Verification:**
- [ ] Focused tests: pnpm build; cargo test --workspace --locked scheduling. Comprobar rango inválido, ámbito global y propuesta incompatible con una cita.
- [ ] Build or broader check: pnpm quality.
- [ ] Manual check: Crear regla/excepción/bloqueo sintéticos, provocar rechazo, corregirlo y revisar el resultado.

**Dependencies:** C09

**Files likely touched:**
- `src/features/scheduling/ScheduleSettingsPage.tsx`
- `src/features/scheduling/ScheduleEditorFields.tsx (nuevo si se extraen los campos reales)`
- `src/components/primitives/Modal.tsx`
- `src/App.css`

**Estimated scope:** M

## T27: Convertir Cambios en Por revisar

**Status:** Done

**Resultado de ejecución:** SyncReviewPage reescrito como «Por revisar»: solo conflictos, rechazos y revisiones requeridas (needsIntervention); sin cola, fases, contadores ni botón de sincronización; enlaces por claves de entidad T07 (reviewRoute) con detalle solo de citas enlazadas (getAppointment), sin barrido por fechas; unknown sigue automático y no se lista; comparación propuesta/estado compartido conservada; salida segura «Ir a su sección» cuando no hay ruta. Pruebas tests/ui/reviews.test.mjs: 5 PASS. pnpm build PASS. Asuntos sintéticos de cita/paciente/acceso: NOT CHECKED.

**Description:** Aplicar la clasificación de intervención al centro existente y a su enlace condicional. Utilizar metadatos T07 para localizar entidades sin el barrido por fechas que excede el contrato actual.

**Acceptance criteria:**
- [ ] Solo se muestran conflictos, rechazos o restricciones que requieren intervención; no hay cola, fases, contadores ni botón de sincronización normal.
- [ ] La cita/ficha afectada tiene explicación y acceso de revisión; se usan claves de entidad y alternativas existentes, con sección/Ayuda como salida segura cuando no hay ruta específica.
- [ ] Unknown normal sigue automático sin duplicación; prueba de clasificación distingue intervención de espera y no promete listado/resolución exhaustivos no contratados.

**Verification:**
- [ ] Focused tests: Not available yet - define in T27: crear pruebas de política de intervención y ejecutar node --test tests/ui/reviews.test.mjs. Ejecutar cargo test --workspace --locked sync y pnpm build.
- [ ] Build or broader check: pnpm quality.
- [ ] Manual check: Resolver/consultar asuntos sintéticos de cita, paciente y acceso; comprobar enlace condicional, ausencia de barridos y caso sin asunto.

**Dependencies:** T26

**Files likely touched:**
- `src/features/sync/SyncReviewPage.tsx`
- `src/features/sync/presentation.ts`
- `src/app/AppLayout.tsx`
- `src/App.css`
- `tests/ui/reviews.test.mjs (nuevo)`

**Estimated scope:** M

## T28: Renovar Acceso y preparación

**Status:** Done

**Resultado de ejecución:** Textos sin panel técnico del motor (servicio remoto no configurado, recuperar la vista con conexión, cuenta activada preparando datos); estados de acceso con explicación y siguiente paso conservados; acceso externo y selección de cuenta con su flujo real; fallo de lectura distinto de no-session y sin borrar datos; preparación con pasos reales y enlace a Configuración. pnpm build PASS. Acceso local sin red: NOT CHECKED.

**Description:** Mostrar pasos y acciones reales de acceso/cuenta/preparación sin un panel técnico del motor. Separar un fallo de lectura de la ausencia de sesión y preservar trabajo local permitido.

**Acceptance criteria:**
- [ ] Sin sesión, cuenta sin preparar, acceso permitido y acceso denegado tienen explicación y siguiente acción correctos.
- [ ] El inicio con datos y acceso local válidos no necesita HTTP ni se bloquea por servicios de apoyo caídos.
- [ ] El acceso externo y la selección de cuenta conservan su flujo real; fallos de lectura no se convierten siempre en no-session ni se borran datos.

**Verification:**
- [ ] Focused tests: pnpm build; cargo test --workspace --locked identity. Comprobar respuestas de acceso válido, error de lectura y cuenta no elegida con fixtures.
- [ ] Build or broader check: pnpm quality.
- [ ] Manual check: Revisar acceso local sin red y preparación con datos; el usuario completa cualquier autenticación externa necesaria durante la comprobación.

**Dependencies:** T27

**Files likely touched:**
- `src/features/identity/AccessPage.tsx`
- `src/app/AppLayout.tsx`
- `src/App.css`

**Estimated scope:** M

## C10: Recuperación — después de T26–T28

- [x] pnpm quality pasa; node --test 16/16; Por revisar usa claves de entidad (reviewRoute), sin barrido. Evidencia: [evidence-c10.md](evidence-c10.md).
- [ ] Conflicto, rechazo, acceso restringido y recuperación con datos e identidad conservados: NOT CHECKED.
- [ ] Revisión expresa del usuario: pendiente; aceptación final en C14.

## Phase J — Ayuda y primeros diseños futuros

## T29: Separar Ayuda de herramientas técnicas

**Status:** Done

**Resultado de ejecución:** Ayuda accesible desde el área de utilidades del sidebar y el botón de la barra superior, con texto de tarea y sin códigos de superficie; enlaces a diagnóstico técnico y herramientas de desarrollo desde el propio diálogo; Desarrollo y Diagnóstico conservan sus funciones fuera de los destinos diarios y añaden «Volver a la agenda» como salida clara. pnpm build y git diff --check PASS. Foco al cerrar ayuda: NOT CHECKED.

**Description:** Organizar ayuda de la tarea y acceso de desarrollo/diagnóstico. Mantener las herramientas técnicas existentes fuera del trabajo diario, sin convertir su estado en garantía de disponibilidad remota.

**Acceptance criteria:**
- [ ] Ayuda explica la tarea actual con texto breve y enlaces útiles, sin códigos de superficie ni instrucciones internas.
- [ ] Desarrollo/Diagnóstico siguen accesibles en su contexto, pero no aparecen como destinos diarios ni muestran secretos.
- [ ] Cerrar ayuda/restaurar navegación mantiene foco y contexto; los paneles técnicos conservan sus funciones previas.

**Verification:**
- [ ] Focused tests: pnpm build; comprobar ayuda y rutas técnicas con/sin datos disponibles, sin ejecutar mutaciones de diagnóstico.
- [ ] Build or broader check: pnpm quality.
- [ ] Manual check: Abrir Ayuda desde Agenda/editor y llegar al diagnóstico de desarrollo sin perder el estado de trabajo.

**Dependencies:** C10

**Files likely touched:**
- `src/app/AppLayout.tsx`
- `src/app/DevelopmentPage.tsx`
- `src/features/diagnostics/DiagnosticsPage.tsx`
- `src/App.css`

**Estimated scope:** M

## T30: Especificar Solicitudes

**Status:** Done

**Description:** Diseñar la lista y revisión de una solicitud junto a su cita vigente. Entregar ficha de superficie, muestra sintética y dependencias de contrato, manteniendo bloqueado el trabajo funcional anterior.

**Acceptance criteria:**
- [ ] Solicitud y cita muestran estados separados; pedir cambio no reserva otro horario ni modifica la cita por sí solo.
- [ ] La especificación define datos/propietarios, comparación, acciones, permisos, fallos parciales y recuperación con preguntas contractuales identificadas.
- [ ] El ejemplo está marcado como diseño y no añade navegación ni comandos operativos a la app.

**Verification:**
- [ ] Focused tests: Revisión contra API v1, maestro de diseño S07 y P-20261109-11-requests-ui; inspeccionar estados vacío/error/pendiente en la muestra.
- [ ] Build or broader check: git diff --check.
- [ ] Manual check: Recorrer una solicitud sintética desde lista a comparación y resultado, distinguiendo decisiones pendientes del comportamiento contratado.

**Dependencies:** T29

**Files likely touched:**
- `tasks/P-20261109-22-interface-refresh/design/requests.md (nuevo)`
- `design-prototype/interface-refresh/index.html`
- `design-prototype/interface-refresh/styles.css`
- `design-prototype/interface-refresh/app.js`

**Resultado de ejecución:** Especificación design/requests.md y vista #requests entregadas. Sintaxis, integridad de rutas y git diff --check: PASS. Render 1440×900: PASS. Detalle/interacción y 800×600: NOT CHECKED por toma de control del navegador. Ver evidence-c11-c13.md.

**Estimated scope:** M

## T31: Especificar Lista de espera

**Status:** Done

**Description:** Diseñar entradas, preferencias, ofertas e interés sin confundirlos con reserva. Preparar el recorrido y las dependencias que deberá resolver su propietario antes de implementar.

**Acceptance criteria:**
- [ ] Entrada, oferta, vencimiento, interés y reserva son pasos distintos; la asignación requiere personal y aceptación API.
- [ ] Se identifican datos, propietario, permisos, prioridad/plazos pendientes, estados y recuperación sin fijar políticas de negocio inexistentes.
- [ ] La muestra usa datos sintéticos, no presenta ofertas reales ni desbloquea el plan funcional.

**Verification:**
- [ ] Focused tests: Revisión contra P06 de la decisión de tres proyectos, maestro S08 y P-20261109-12-waitlist-ui.
- [ ] Build or broader check: git diff --check.
- [ ] Manual check: Recorrer entrada→oferta→interés→decisión del personal, con vencimiento y falta de servicio como ejemplos de diseño.

**Dependencies:** T30

**Files likely touched:**
- `tasks/P-20261109-22-interface-refresh/design/waitlist.md (nuevo)`
- `design-prototype/interface-refresh/index.html`
- `design-prototype/interface-refresh/styles.css`
- `design-prototype/interface-refresh/app.js`

**Resultado de ejecución:** Especificación design/waitlist.md y vista #waitlist entregadas. Mismas comprobaciones PASS/NOT CHECKED que T30.

**Estimated scope:** M

## C11: Ayuda y continuidad — después de T29–T31

- [x] Documentación sin errores de diff (git diff --check PASS); sin cambios de producto: T29 no ejecutada por el alcance aprobado (diseño y prototipo).
- [x] Ayuda separada del diagnóstico (T29 ejecutada ahora: utilidades + diálogo sin códigos de superficie). Solicitudes/Espera diseño con dependencias, sin funciones operativas (evidence-c11-c13.md).
- [x] Revisión expresa del usuario recibida (aprobada 2026-11-09).

## Phase K — Seguimiento futuro

## T32: Especificar Comunicaciones

**Status:** Done

**Description:** Diseñar supervisión de entregas, respuestas y problemas de contacto vinculados a citas, sin introducir chat libre ni envíos funcionales.

**Acceptance criteria:**
- [ ] Programada, enviada, entregada y respondida se distinguen de confirmación o asistencia de la cita.
- [ ] La ficha define datos/propietarios, antigüedad, filtros, detalle, permisos y recuperación, sin inventar eventos o capacidades.
- [ ] Muestra y documentación son de diseño; trabajadores/envíos pertenecen a Nightingale API y no bloquean el escritorio local.

**Verification:**
- [ ] Focused tests: Revisión contra maestro S09, contratos disponibles y P-20261109-13-communications-ui; distinguir estados sin medir entregas inexistentes.
- [ ] Build or broader check: git diff --check.
- [ ] Manual check: Consultar entregas, una respuesta y un contacto fallido sintéticos, con acceso contextual a cita/ficha.

**Dependencies:** C11

**Files likely touched:**
- `tasks/P-20261109-22-interface-refresh/design/communications.md (nuevo)`
- `design-prototype/interface-refresh/index.html`
- `design-prototype/interface-refresh/styles.css`
- `design-prototype/interface-refresh/app.js`

**Resultado de ejecución:** Especificación design/communications.md y vista #communications entregadas. Mismas comprobaciones PASS/NOT CHECKED que T30.

**Estimated scope:** M

## T33: Especificar Indicadores

**Status:** Done

**Description:** Diseñar indicadores administrativos con periodo, definición y denominador visibles. Definir vacío, datos antiguos y errores sin inventar mediciones reales.

**Acceptance criteria:**
- [ ] Cada indicador tiene propósito, valor/unidad, periodo, denominador y definición; no interpreta resultados clínicos.
- [ ] Gráficos tienen escalas claras y equivalente textual/tabular; cero y ausencia de datos se distinguen.
- [ ] La muestra es sintética y define su dependencia analítica externa sin añadir funciones operativas a la app.

**Verification:**
- [ ] Focused tests: Revisión contra maestro S10 y P-20261109-14-metrics-ui; comprobar definiciones y estados con/sin datos.
- [ ] Build or broader check: git diff --check.
- [ ] Manual check: Cambiar periodo de ejemplo y explicar cada valor y su actualidad sin inferir producción ni normalidad médica.

**Dependencies:** T32

**Files likely touched:**
- `tasks/P-20261109-22-interface-refresh/design/metrics.md (nuevo)`
- `design-prototype/interface-refresh/index.html`
- `design-prototype/interface-refresh/styles.css`
- `design-prototype/interface-refresh/app.js`

**Resultado de ejecución:** Especificación design/metrics.md y vista #metrics entregadas. Mismas comprobaciones PASS/NOT CHECKED que T30.

**Estimated scope:** M

## T34: Especificar Equipo y dispositivos

**Status:** Done

**Description:** Diseñar la configuración relacionada de personas autorizadas y dispositivos, con consecuencias explícitas de acceso y revocación. No implementar nuevas operaciones de seguridad.

**Acceptance criteria:**
- [ ] Roles, personas y autorización de dispositivos se presentan como conceptos distintos con acciones y permisos identificados.
- [ ] Revocación explica alcance y límites offline, sin prometer efecto instantáneo ni deducir permiso por la hora del cliente.
- [ ] Documentos y muestra identifican contratos/propietarios pendientes; no crean usuarios, invitaciones ni accesos reales.

**Verification:**
- [ ] Focused tests: Revisión contra maestro S12 y P-20261109-15-team-devices-settings-ui; contrastar invitación/miembro/dispositivo con el contrato vigente.
- [ ] Build or broader check: git diff --check.
- [ ] Manual check: Revisar roles y consecuencias de revocación sintéticas, con estados de conexión y permisos insuficientes.

**Dependencies:** T33

**Files likely touched:**
- `tasks/P-20261109-22-interface-refresh/design/team.md (nuevo)`
- `tasks/P-20261109-22-interface-refresh/design/devices.md (nuevo)`
- `design-prototype/interface-refresh/index.html`
- `design-prototype/interface-refresh/styles.css`
- `design-prototype/interface-refresh/app.js`

**Resultado de ejecución:** Especificaciones design/team.md y design/devices.md y vista #team entregadas. Mismas comprobaciones PASS/NOT CHECKED que T30.

**Estimated scope:** M

## C12: Seguimiento — después de T32–T34

- [x] Ejemplos sintéticos y documentación/muestra coincidentes (evidence-c11-c13.md).
- [x] Comunicaciones, Indicadores y Equipo/Dispositivos identifican estados, propietarios y contratos sin cerrar decisiones pendientes.
- [x] Revisión expresa del usuario recibida (aprobada 2026-11-09).

## Phase L — Diseño futuro y accesibilidad transversal

## T35: Especificar política de comunicaciones

**Status:** Done

**Description:** Diseñar la configuración de comunicaciones indicando qué opciones tienen respaldo y qué políticas siguen pendientes. Mantener la relación entre configuración y sus efectos en citas/mensajes.

**Acceptance criteria:**
- [ ] Solo se fijan como reglas las opciones respaldadas por contrato; plazos y capacidades no definidos quedan identificados.
- [ ] Se especifican tarea, propietario, permisos, efecto, validación, estados y recuperación con el mismo lenguaje visual.
- [ ] No se implementan recordatorios, preferencias remotas nuevas ni envíos; el plan funcional pendiente conserva su estado.

**Verification:**
- [ ] Focused tests: Revisión contra maestro S12, límites de API y P-20261109-16-communication-settings-ui.
- [ ] Build or broader check: git diff --check.
- [ ] Manual check: Leer y revisar una configuración de ejemplo, distinguiendo opciones confirmadas de decisiones de negocio pendientes.

**Dependencies:** C12

**Files likely touched:**
- `tasks/P-20261109-22-interface-refresh/design/communication-policy.md (nuevo)`
- `design-prototype/interface-refresh/index.html`
- `design-prototype/interface-refresh/styles.css`
- `design-prototype/interface-refresh/app.js`

**Resultado de ejecución:** Especificación design/communication-policy.md y vista #policy entregadas. Mismas comprobaciones PASS/NOT CHECKED que T30.

**Estimated scope:** M

## T36: Preparar la entrega visual del Hub

**Status:** Done

**Description:** Entregar continuidad visual y recorridos conceptuales del paciente para una cita existente. Mantener su diseño como handoff al repositorio propietario, no como web funcional dentro del escritorio.

**Acceptance criteria:**
- [ ] Consulta, confirmación, cancelación, solicitud de cambio y errores de acceso mantienen lenguaje visual y resultado conocido.
- [ ] El paciente solo gestiona su cita autorizada; no obtiene agenda general ni creación pública, y una solicitud no reserva otra hora.
- [ ] La entrega identifica datos, propietarios, estados, accesibilidad móvil y contratos pendientes; no implementa APIs, enlaces reales ni servidores.

**Verification:**
- [ ] Focused tests: Revisión contra maestro S14 y P-20261109-19-patient-hub-ux-handoff; revisar muestra desde 320 px sin afirmar autenticación real.
- [ ] Build or broader check: git diff --check.
- [ ] Manual check: Recorrer una cita sintética en móvil, cancelar/solicitar cambio en la muestra y revisar enlace vencido/servicio no disponible.

**Dependencies:** T35

**Files likely touched:**
- `tasks/P-20261109-22-interface-refresh/design/hub-handoff.md (nuevo)`
- `design-prototype/interface-refresh/hub.html (nuevo)`
- `design-prototype/interface-refresh/styles.css`
- `design-prototype/interface-refresh/app.js`

**Resultado de ejecución:** Especificación design/hub-handoff.md y hub.html móvil (desde 320 px) entregadas como handoff de diseño. Inspección a 320 px: NOT CHECKED por toma de control del navegador.

**Estimated scope:** M

## T37: Auditar adaptación y accesibilidad

**Status:** Done

**Resultado de ejecución:** Matriz por superficie en accessibility-matrix.md con cobertura automática real (16 pruebas de presentación + quality) y verificación humana/zoom/lector marcadas NOT CHECKED sin contarlas como aprobadas; diseño aplicado en código listado como hipótesis verificable; sin afirmar conformidad global por ARIA o colores. Evidencia consolidada en evidence.md.

**Description:** Comprobar las superficies renovadas con matriz visual, teclado, foco, zoom, contraste y lector disponible. Registrar y devolver los fallos a sus tareas propietarias sin convertir una auditoría en un cambio transversal sin límite.

**Acceptance criteria:**
- [ ] Cada superficie tiene evidencia por tamaño/entrada/estado con PASS, FAIL, NOT CHECKED o N/A; no se sustituye zoom real por aumentar una fuente CSS.
- [ ] Los fallos que impiden lectura o tarea se corrigen en la tarea propietaria antes de C13; no se afirma conformidad global solo por ARIA o colores.
- [ ] Lector no disponible, plataforma no probada o interacción humana ausente quedan NOT CHECKED con causa y siguiente comprobación.

**Verification:**
- [ ] Focused tests: node --test después de introducir la suite de presentación; revisión manual de Tab/Shift+Tab, flechas, Enter/Escape, nombres y contraste real por superficie.
- [ ] Build or broader check: pnpm quality.
- [ ] Manual check: Ejecutar 800×600, 1024×768, 1280×720, 1440×900, zoom 200 % y movimiento reducido; usar lector si el entorno lo permite.

**Dependencies:** T36

**Files likely touched:**
- `tasks/P-20261109-22-interface-refresh/evidence.md (nuevo)`
- `tasks/P-20261109-22-interface-refresh/accessibility-matrix.md (nuevo)`

**Estimated scope:** M

## C13: Revisión transversal — después de T35–T37

- [x] Política/Hub mantienen límites; auditoría con evidencia por superficie (accessibility-matrix.md, evidence-c11-c13.md).
- [ ] Fallos de lectura/tarea en verificación humana: NOT CHECKED; no cuentan como aprobados.
- [x] Revisión expresa del usuario recibida (aprobada 2026-11-09).

## Phase M — Integración y entrega

## T38: Verificar trabajo local y recuperación

**Status:** Done

**Resultado de ejecución:** quality PASS (377+5+1; 9 ignoradas) y pnpm build:desktop PASS (nightingale.exe). Garantías nativas conservadas por construcción y pruebas Rust: operación sin HTTP, identidad de operaciones ante reintento/respuesta perdida, versiones/permisos, sincronización invisible e intervención en contexto. Escenarios con corte de red real, cambio de cuenta, conflicto con API local, macOS/Linux y recorridos interactivos: NOT CHECKED (integration-checks.md).

**Description:** Ejecutar los recorridos reales con datos de prueba en el entorno local autorizado: sin red, reinicio, recuperación y respuesta perdida. Verificar que la renovación no modificó las garantías nativas ni expone el proceso automático.

**Acceptance criteria:**
- [ ] Consultar/guardar/reiniciar conserva datos e identidad de operaciones con últimos permisos válidos; no exige servicios de apoyo para casos nativos.
- [ ] Respuesta perdida, conflicto y revocación no duplican ni borran intenciones; la intervención aparece en contexto y Por revisar cuando corresponde.
- [ ] Sincronización correcta no añade avisos/columnas/contadores; las pruebas remotas no disponibles quedan separadas de las locales verificadas.

**Verification:**
- [ ] Focused tests: cargo test --workspace --locked appointments; cargo test --workspace --locked sync; ejecutar los recorridos nativos de prueba y registrar el corte de red real/reinicio si se pueden comprobar.
- [ ] Build or broader check: pnpm quality; pnpm build:desktop.
- [ ] Manual check: Crear/cambiar una cita sintética offline, reiniciar, recuperar red y revisar resultado, conflicto y acceso en el entorno de API local; no usar producción.

**Dependencies:** C13

**Files likely touched:**
- `tasks/P-20261109-22-interface-refresh/evidence.md`
- `tasks/P-20261109-22-interface-refresh/integration-checks.md (nuevo)`

**Estimated scope:** M

## T39: Validar los recorridos y cerrar la entrega

**Status:** Done

**Resultado de ejecución:** Entrega consolidada (evidence.md, integration-checks.md, accessibility-matrix.md, user-validation.md, diseños en design/ y design-prototype/interface-refresh/). Validación humana registrada como NOT CHECKED por ausencia de participantes, sin afirmar reducción de agotamiento. Plan/todo reflejan lo realizado; C14 queda pendiente de aceptación expresa.

**Description:** Consolidar la renovación, sus diseños futuros y evidencia. Realizar la validación humana si hay participantes o registrar explícitamente su ausencia; presentar C14 sin cerrar el plan antes de la aceptación del usuario.

**Acceptance criteria:**
- [ ] Se entrega muestra aprobada, capturas comparables, matriz de superficies/recorridos, evidencia y dependencias de los diseños futuros.
- [ ] Validación con médico/asistente registra finalización, dudas, errores y ayuda; si no existe, se marca NOT CHECKED sin afirmar reducción de agotamiento.
- [ ] Plan/todo reflejan únicamente lo realizado y C14 sigue pendiente hasta aceptación expresa; planes anteriores e historial permanecen intactos.

**Verification:**
- [ ] Focused tests: Revisar los 18 escenarios del plan, los resultados de node --test y la evidencia de todos los checkpoints; comparar experiencia anterior/nueva cuando existan participantes.
- [ ] Build or broader check: pnpm quality; pnpm build:desktop; git diff --check.
- [ ] Manual check: Presentar al usuario la app renovada, los diseños futuros y las comprobaciones pendientes para revisión final.

**Dependencies:** T38

**Files likely touched:**
- `tasks/P-20261109-22-interface-refresh/evidence.md`
- `tasks/P-20261109-22-interface-refresh/user-validation.md (nuevo)`
- `tasks/P-20261109-22-interface-refresh/plan.md`
- `tasks/P-20261109-22-interface-refresh/todo.md`

**Estimated scope:** M

## C14: Aceptación final — después de T38–T39

- [x] pnpm quality (377+5+1), node --test 16/16 y build:desktop PASS en Windows.
- [x] App renovada, diseños futuros y evidencia entregados; plataformas, red, lector y validación humana NOT CHECKED.
- [x] Usuario aceptó expresamente el cierre («Aprobado», 2026-11-09). C14 superado.
