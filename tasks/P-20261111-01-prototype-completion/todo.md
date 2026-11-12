# Task List: P-20261111-01-prototype-completion

**Plan:** [plan.md](plan.md)
**Updated:** 2026-11-12

Plan aprobado por el usuario el 2026-11-11. Son tareas de un prototipo independiente, sin integración de servicios. Los criterios comunes, la matriz de superficies, los límites de contrato y los rangos de revisión están en plan.md. Las comprobaciones históricas no cierran estas tareas.

## T01: Actualizar la tipografía

**Status:** Done
**Superficies:** S01–S14

**Description:** Aplicar Montserrat en encabezados y Mulish en lectura, sin alterar la composición del prototipo.

**Acceptance criteria:**
- [x] Fuentes locales con origen, licencia y pesos 400/500/600; sin solicitudes externas.
- [x] Agenda, ficha y editor conservan jerarquía, lectura y ajustes para nombres largos.

**Verification:**
- [x] Focused tests: `node --check design-prototype/app.js`; comprobar el recorrido indicado abajo.
- [x] Build or broader check: `git diff --check -- design-prototype tasks/P-20261111-01-prototype-completion`; no requiere compilación Tauri.
- [x] Manual check: Comparar Agenda, ficha y editor con sus capturas históricas a 1280×800 y 800×600.

**Evidence:** seis capturas T01 en `evidence/C01/`. Navegador a 1280×800 y 800×600: Agenda, ficha y editor sin desbordamiento horizontal del documento. Estilos calculados Montserrat/Mulish; pesos 400/500/600 y SHA-256 idéntico a los archivos locales de origen. El alcance no incluye aún las correcciones de campos o navegación de tareas posteriores.

**Dependencies:** Ninguna
**Files likely touched:**
- `design-prototype/styles.css`
- `design-prototype/assets/montserrat-latin-variable.woff2`
- `design-prototype/assets/mulish-latin-variable.woff2`
- `design-prototype/assets/fonts-LICENSE.txt`
**Estimated scope:** S

## T02: Completar el marco de navegación

**Status:** Done
**Superficies:** S02, S11, S12

**Description:** Mantener el marco original, usar Phosphor y hacer condicional el acceso a Por revisar.

**Acceptance criteria:**
- [x] No hay avisos de sincronización normal, desconexión compatible con trabajo local o reintentos automáticos.
- [x] Phosphor es el único conjunto de iconos; cuenta, ruta y foco se conservan al navegar.
- [x] El dueño con un profesional no ve selector Profesional; asistentes y casos con varios profesionales sí.

**Verification:**
- [x] Focused tests: `node --check design-prototype/app.js`; comprobar el recorrido indicado abajo.
- [x] Build or broader check: `git diff --check -- design-prototype tasks/P-20261111-01-prototype-completion`; no requiere compilación Tauri.
- [x] Manual check: Alternar dueño/asistente y escenarios en controles del prototipo; recorrer navegación con teclado.

**Evidence:** [verificación C01](evidence/C01/verification.md) y `checks.json`. 34 rutas a 1280×800 y 800×600; seis combinaciones de rol/escenario; retorno de detalle con fecha/filtro/foco; menú a 375×812 con Tab/Shift+Tab/Escape y fondo inert; salto al contenido. Se corrigió el desbordamiento de la etiqueta accesible de la tabla del asistente y se repitió su comprobación. Se incrustan SVG de Phosphor en `app.js` para conservar apertura directa por archivo: no se necesita `assets/icons.svg`. Se actualiza README para distinguir evidencia histórica y nueva.

**Dependencies:** T01
**Files likely touched:**
- `design-prototype/app.js`
- `design-prototype/styles.css`
- `design-prototype/assets/icons.svg`
- `design-prototype/assets/icons-LICENSE.txt`
**Estimated scope:** M

## C01: Base visual y navegación

- [x] Verificación y límites de T01, T02 registrados.
- [x] Revisión visual, teclado, estados y retorno del bloque.
- [x] Aceptación del usuario antes del siguiente bloque. «Apruebo C01; continúa con T03–T05» (2026-11-11).

## T03: Completar acceso y preparación

**Status:** Done
**Superficies:** S01

**Description:** Ampliar las dos vistas históricas con estados de acceso y pasos iniciales reales dentro de la simulación.

**Acceptance criteria:**
- [x] Se pueden explorar acceso externo en curso, equipo no autorizado, selección de cuenta, preparación y acceso denegado.
- [x] La preparación recoge cuenta/zona, servicio y horario; no marca pasos sin completar.
- [x] La pérdida de sesión o acceso no ofrece borrar datos ni inventa PIN, contraseña local o biometría.

**Verification:**
- [x] Focused tests: `node --check design-prototype/app.js`; comprobar el recorrido indicado abajo.
- [x] Build or broader check: `git diff --check -- design-prototype tasks/P-20261111-01-prototype-completion`; no requiere compilación Tauri.
- [x] Manual check: Recorrer primera preparación, regreso con acceso local permitido y acceso revocado con datos conservados.

**Evidence:** `evidence/C02/T03-checks.json`: 12 superficies a 1280×800 y 800×600, recorrido de acceso/preparación, error de intervalo con campos conservados, guardado válido, acceso local permitido y revocado, activación sin red y restricción del asistente. Acciones y resultados en memoria, sin autenticación o autorización reales.

**Dependencies:** T02 y C01
**Files likely touched:**
- `design-prototype/app.js`
- `design-prototype/styles.css`
**Estimated scope:** M

## T04: Completar las vistas de Agenda

**Status:** Done
**Superficies:** S02

**Description:** Afinar Día, Semana y Lista con filtros útiles, cobertura y navegación coherente.

**Acceptance criteria:**
- [x] Día, Semana y Lista conservan fecha, búsqueda y filtros al abrir/cerrar detalle; los filtros no generan huecos ficticios.
- [x] Semana usa 96 px/hora y mínimo 160 px/día, con desplazamiento propio y citas legibles de varias duraciones.
- [x] Vacío real, filtro sin coincidencias, fecha no descargada y error de lectura tienen estados distintos.

**Verification:**
- [x] Focused tests: `node --check design-prototype/app.js`; comprobar el recorrido indicado abajo.
- [x] Build or broader check: `git diff --check -- design-prototype tasks/P-20261111-01-prototype-completion`; no requiere compilación Tauri.
- [x] Manual check: Recorrer las tres vistas con nombres largos, bloqueos, citas consecutivas y distintas duraciones a 800×600 y 1280×800.

**Evidence:** `evidence/C02/T04-checks.json` y capturas Día/Semana. Intervalo disponible 10:30 antes/después de filtrar Confirmadas; día vacío conocido y fecha sin cobertura distintos; seis estados de lectura; semana con intervalos de 15/30/45/60 minutos; variante de varios profesionales sin eventos que se tapen; retorno desde Semana/Lista y carga de 8 a 16 filas conservando consulta y periodo. La muestra no calcula disponibilidad del producto ni acredita servicios.

**Dependencies:** T03
**Files likely touched:**
- `design-prototype/app.js`
- `design-prototype/styles.css`
**Estimated scope:** M

## T05: Completar el ciclo de la cita

**Status:** Done
**Superficies:** S03, S04

**Description:** Revisar detalle, alta, reprogramación, historia y confirmaciones de acciones.

**Acceptance criteria:**
- [x] Reprogramar solo modifica intervalo y conserva duración, paciente, profesional, servicio y responsable; compara horario anterior y nuevo.
- [x] Las acciones responden a estado, momento y permiso; estados terminales muestran consulta e historia.
- [x] Registro de paciente desde cita, errores y cambios sin guardar conservan borrador y contexto; doble envío no duplica el ejemplo.

**Verification:**
- [x] Focused tests: `node --check design-prototype/app.js`; comprobar el recorrido indicado abajo.
- [x] Build or broader check: `git diff --check -- design-prototype tasks/P-20261111-01-prototype-completion`; no requiere compilación Tauri.
- [x] Manual check: Crear desde hueco y paciente; registrar paciente; provocar solapamiento; reprogramar y comprobar terminales y cancelación.

**Evidence:** [verificación C02](evidence/C02/verification.md), `T05-checks.json` y `T05-from-patient.json`. Alta desde intervalo/ficha, doble clic sin duplicar, reprogramación de 45 minutos con historia conservada, responsable obligatorio, registro relacionado con retorno al borrador, permisos y terminales, cancelación local con ocupación conservada, protección al usar Atrás, bloqueo/conflicto con campos conservados. `context-layout-checks.json` acredita retorno exacto de desplazamiento/foco y orden de resumen/acciones a 800×600. Son resultados ficticios en memoria; integración y persistencia no comprobadas.

**Dependencies:** T04
**Files likely touched:**
- `design-prototype/app.js`
- `design-prototype/styles.css`
**Estimated scope:** M

## C02: Acceso, Agenda y citas

- [x] Verificación y límites de T03, T04, T05 registrados en [evidence/C02/verification.md](evidence/C02/verification.md).
- [x] Revisión visual, teclado, estados y retorno del bloque. 48 rutas × tres tamaños; revisión funcional centrada en T03–T05. Zoom al 200 % y lector de pantalla pendientes.
- [x] Aceptación del usuario: «Apruebo C02» (2026-11-11); continúa con T06–T07.

## T06: Completar el registro de pacientes

**Status:** Done
**Superficies:** S05, S06

**Description:** Ampliar directorio, alta y edición administrativa conforme a las formas de identificación.

**Acceptance criteria:**
- [x] Directorio muestra contacto permitido, próxima cita, actividad y completitud sin mezclar esos estados.
- [x] Registro distingue borrador, documento propio y menor sin documento; pide solo campos que exige el contrato.
- [x] Documentos conservan prefijos y ceros; país emisor/tipo/número y teléfono por código/número tienen etiquetas y validación clara.

**Verification:**
- [x] Focused tests: `node --check design-prototype/app.js`; comprobar el recorrido indicado abajo.
- [x] Build or broader check: `git diff --check -- design-prototype tasks/P-20261111-01-prototype-completion`; no requiere compilación Tauri.
- [x] Manual check: Registrar adulto y menor sin documento; probar ficha incompleta, nombres iguales, teléfono parcial y búsqueda local sin coincidencias.

**Evidence:** `evidence/C03/T06-checks.json` y cinco capturas. Directorio a 1280×800 y 800×600 (emulación táctil, sin desbordamiento del documento) con contacto/actividad/completitud/estado separados; alta de adulto con ceros preservados y de menor con responsable; pareja telefónica parcial rechazada conservando campos; borrador con requisitos faltantes y edición que completa la ficha; conflicto de completitud rechazado conservando el estado previo; homónimos y búsqueda vacía local. Resultados ficticios en memoria.

**Dependencies:** T05 y C02
**Files likely touched:**
- `design-prototype/app.js`
- `design-prototype/styles.css`
**Estimated scope:** M

## T07: Completar ficha y responsables

**Status:** Done
**Superficies:** S06

**Description:** Añadir consulta y edición de responsables, sus relaciones con el paciente y el historial administrativo.

**Acceptance criteria:**
- [x] Responsable muestra relación, vigencia, actividad y representación como datos distintos; no identifica al menor con el documento del adulto.
- [x] Ficha separa próximos encuentros e historial paginado y permite editar datos o iniciar Nueva cita según permiso.
- [x] Inactivar explica que no cancela citas; un conflicto de completitud conserva campos y estado anterior sin convertirlo en borrador.

**Verification:**
- [x] Focused tests: `node --check design-prototype/app.js`; comprobar el recorrido indicado abajo.
- [x] Build or broader check: `git diff --check -- design-prototype tasks/P-20261111-01-prototype-completion`; no requiere compilación Tauri.
- [x] Manual check: Añadir/editar un responsable desde ficha y cita; volver sin perder datos; comprobar inactivación y conflicto de completitud.

**Evidence:** `evidence/C03/T07-checks.json` y tres capturas. Ficha de menor con dos responsables (vigente y finalizado) mostrando parentesco, función, vigencia, representación y actividad separados; bloqueo de inactivación del único responsable vigente con la ficha completa conservada; alta de responsable desde ficha y desde el editor de cita con selección automática; historial paginado 5 de 7 → 7 de 7; inactivación de ficha sin cancelar citas. Resultados ficticios en memoria.

**Dependencies:** T06
**Files likely touched:**
- `design-prototype/app.js`
- `design-prototype/styles.css`
**Estimated scope:** M

## C03: Pacientes y responsables

- [x] Verificación y límites de T06, T07 registrados.
- [x] Revisión visual, teclado, estados y retorno del bloque.
- [x] Aceptación del usuario: preaprobación explícita «Apruebo C03 si necesita aprobación» (objetivo del 2026-11-11); continúa con T08–T10.

## T08: Completar cuenta y preferencias

**Status:** Done
**Superficies:** S12

**Description:** Separar datos compartidos de cuenta y preferencias locales.

**Acceptance criteria:**
- [x] Lectura y edición mantienen lugar, dirección, zona y alcance de cada dato; lectura termina en ciudad/estado.
- [x] Preferencias locales y cambios que requieren conexión/permiso tienen límites distintos.
- [x] Los cambios preservan datos históricos de citas; solo se presenta el tema claro definido.

**Verification:**
- [x] Focused tests: `node --check design-prototype/app.js`; comprobar el recorrido indicado abajo.
- [x] Build or broader check: `git diff --check -- design-prototype tasks/P-20261111-01-prototype-completion`; no requiere compilación Tauri.
- [x] Manual check: Editar cuenta con y sin permiso, revisar dirección incompleta y comparar una cita histórica antes/después del cambio simulado.

**Evidence:** `evidence/C04/T08-checks.json` y captura. Edición como dueña guardada; cita atendida conserva su lugar tras cambiar la cuenta; dirección incompleta rechazada con mensaje propio y campos conservados (se retiró `required` nativo para que el error del prototipo sea visible); asistente recibe solo lectura; sin conexión bloquea datos compartidos y permite preferencias locales, que persisten tras navegar. Resultados ficticios en memoria.

**Dependencies:** T07 y C03
**Files likely touched:**
- `design-prototype/app.js`
- `design-prototype/styles.css`
**Estimated scope:** M

## T09: Completar catálogo y ofertas

**Status:** Done
**Superficies:** S12

**Description:** Representar el catálogo del profesional y la oferta de servicios de la cuenta sin confundir sus duraciones.

**Acceptance criteria:**
- [x] Hay consulta, alta/edición permitida y activación/inactivación de servicio u oferta, con alcance explícito.
- [x] Duración base y ajuste de oferta se distinguen; no aparecen versiones técnicas.
- [x] Antes de guardar se entiende que las citas existentes mantienen sus datos; errores conservan el formulario.

**Verification:**
- [x] Focused tests: `node --check design-prototype/app.js`; comprobar el recorrido indicado abajo.
- [x] Build or broader check: `git diff --check -- design-prototype tasks/P-20261111-01-prototype-completion`; no requiere compilación Tauri.
- [x] Manual check: Editar oferta y catálogo, desactivar servicio y verificar que el ejemplo de cita previa conserva su servicio y duración.

**Evidence:** `evidence/C04/T09-checks.json` y captura. Catálogo con duración base separada del ajuste de oferta (45 = ajuste sobre 30; 45 = base); validación propia con formulario conservado (se retiró min/max nativo); edición válida (ajuste 20); desactivación de oferta con confirmación; la oferta desactivada desaparece de la nueva cita; la cita a1 conserva «Consulta general · 09:00 – 09:30» pese al cambio de referencia; inactivación de catálogo en cascada y límites de dueña/conexión. Resultados ficticios en memoria.

**Dependencies:** T08
**Files likely touched:**
- `design-prototype/app.js`
- `design-prototype/styles.css`
**Estimated scope:** M

## T10: Completar horarios y bloqueos

**Status:** Done
**Superficies:** S12

**Description:** Sustituir cuadros ilustrativos por recorridos de edición de reglas semanales, excepciones y bloqueos.

**Acceptance criteria:**
- [x] Regla semanal tiene vigencia y tramos; excepción distingue cierre/reemplazo y bloqueo distingue alcance.
- [x] El cambio que afecta citas conserva formulario y muestra citas afectadas; no simula aceptación.
- [x] La vista de asistente no revela motivos globales restringidos; los intervalos usan horas legibles.

**Verification:**
- [x] Focused tests: `node --check design-prototype/app.js`; comprobar el recorrido indicado abajo.
- [x] Build or broader check: `git diff --check -- design-prototype tasks/P-20261111-01-prototype-completion`; no requiere compilación Tauri.
- [x] Manual check: Crear dos tramos, excepción y bloqueo; provocar cruce con cita y revisar la misma vista como asistente.

**Evidence:** `evidence/C04/T10-checks.json` y dos capturas. Regla con días, tramos dinámicos y vigencia; excepción de cierre semilla (26 oct); bloqueos con alcance; rechazo de bloqueo que cruza citas con lista de pacientes e intervalos y formulario conservado (dos intentos); asistente sin motivo global ni edición; Agenda, Semana y Nueva cita usan ventanas dinámicas (bandas «Fuera de horario», primer intervalo libre 10:30, validación con horas legibles). La creación exitosa de regla/excepción usa el mismo mecanismo simular-revertir-listar verificado en bloqueos. Resultados ficticios en memoria.

**Dependencies:** T09
**Files likely touched:**
- `design-prototype/app.js`
- `design-prototype/styles.css`
**Estimated scope:** M

## C04: Configuración operativa

- [ ] Verificación y límites de T08, T09, T10 registrados.
- [ ] Revisión visual, teclado, estados y retorno del bloque.
- [ ] Aceptación del usuario antes del siguiente bloque.

## T11: Completar solicitudes

**Status:** Done
**Superficies:** S07

**Description:** Ampliar lista/detalle con cita vigente, antigüedad y estados de conciliación ilustrados.

**Acceptance criteria:**
- [x] La petición y la cita actual permanecen juntas; elegir horario no altera la cita por sí solo.
- [x] Se distinguen solicitud resuelta/desactualizada y cambio aceptado cuya conciliación sigue fallida.
- [x] Las acciones sin contrato no producen éxitos simulados de servicio; su límite se explica en los controles de revisión del prototipo.

**Verification:**
- [x] Focused tests: `node --check design-prototype/app.js`; comprobar el recorrido indicado abajo.
- [x] Build or broader check: `git diff --check -- design-prototype tasks/P-20261111-01-prototype-completion`; no requiere compilación Tauri.
- [x] Manual check: Revisar una petición, abrir horario y volver; explorar conciliación parcial y solicitud ya modificada sin duplicar cambios.

**Evidence:** `evidence/C05/T11-checks.json` y captura. Lista con cuatro estados (2 pendientes, desactualizada, conciliación pendiente) y antigüedad relativa; detalle con petición, estado y cita vigente sin modificar; al guardar un nuevo horario desde la solicitud esta pasa a «Resuelta» y el contador baja (2 → 1) sin duplicar cambios; conciliación parcial y desactualizada solo ofrecen «Ver cita», sin éxitos simulados. Resultados ficticios en memoria.

**Dependencies:** T10 y C04
**Files likely touched:**
- `design-prototype/app.js`
- `design-prototype/styles.css`
**Estimated scope:** M

## T12: Completar lista de espera

**Status:** Done (revisada 2026-11-12: sin ofertas, decisión del usuario «los pacientes no son leads»)
**Superficies:** S08

**Description:** Definir alta/edición de entradas, preferencias y horarios compatibles con asignación directa confirmada por el personal. La pestaña y el flujo de «Ofertas» (interés, vencimiento, respuesta) se eliminan por decisión directa del usuario.

**Acceptance criteria:**
- [x] La lista no usa conceptos comerciales (sin ofertas ni conversión); ordenar no implica prioridad sin política.
- [x] Los horarios compatibles se calculan sobre horario y ocupación conocida; la asignación requiere confirmación explícita del personal.
- [x] Un intervalo que ya no está disponible conserva la entrada (recheck defensivo antes de crear la cita).

**Verification:**
- [x] Focused tests: `node --check design-prototype/app.js`; comprobar el recorrido indicado abajo.
- [x] Build or broader check: `git diff --check -- design-prototype tasks/P-20261111-01-prototype-completion`; no requiere compilación Tauri.
- [x] Manual check: Registrar preferencias, consultar horarios compatibles y asignar con confirmación; comprobar que un intervalo ocupado conserva la entrada.

**Evidence:** [verificación C05](evidence/C05/verification.md) y `evidence/C05/T12-checks.json` con cuatro capturas (la iteración previa con ofertas se conserva como histórica: `T12-checks-v1-ofertas.json`, `v1-ofertas-*`). Lista sin pestaña Ofertas ni estados de interés; «Ofrecer horario» muestra horarios compatibles calculados y «Asignar este horario» exige confirmación del personal; la entrada pasa a «Asignada» y la cita conserva servicio y duración; se corrigió un defecto de truncado de horas en acciones con «:» (fecha e intervalo viajan como atributos de datos). Escape en diálogos; 800×600 con scroll propio. Resultados ficticios en memoria; política de prioridad pendiente.

**Dependencies:** T11
**Files likely touched:**
- `design-prototype/app.js`
- `design-prototype/styles.css`
**Estimated scope:** M

## C05: Solicitudes y espera

- [x] Verificación y límites de T11, T12 registrados en [evidence/C05/verification.md](evidence/C05/verification.md).
- [x] Revisión visual, teclado, estados y retorno del bloque. Zoom al 200 % y lector de pantalla pendientes.
- [x] Aceptación del usuario: «Apruebo C05, por favor quiero que continúes hasta el siguiente checkpoint» (2026-11-12); continúa con T13–T14.

## T13: Completar comunicaciones

**Status:** Done
**Superficies:** S09

**Description:** Añadir filtros, detalle de entrega y recorrido de revisión de contacto.

**Acceptance criteria:**
- [x] Fila/detalle relacionan paciente, cita, tipo/canal, fecha y resultado conocido; no hay chat libre.
- [x] Programado, enviado, entregado, respondido y fallo no cambian el estado de asistencia.
- [x] Resultado desconocido no ofrece reenvío ciego; revisar contacto devuelve al mensaje conservando contexto.

**Verification:**
- [x] Focused tests: `node --check design-prototype/app.js`; comprobar el recorrido indicado abajo.
- [x] Build or broader check: `git diff --check -- design-prototype tasks/P-20261111-01-prototype-completion`; no requiere compilación Tauri.
- [x] Manual check: Filtrar entregas, abrir detalle con fallo de teléfono, consultar paciente y volver; revisar timeout sin reenvío.

**Evidence:** [verificación C06](evidence/C06/verification.md) y `evidence/C06/T13-checks.json` con dos capturas. Seis mensajes con paciente, tipo · canal, cita enlazada, envío y resultado (Entregado, Respondido, Programado, Revisar teléfono, Resultado desconocido); filtro por estado con limpieza visible; detalle con cita relacionada y nota de que ningún estado cambia la cita; fallo y desconocido sin reenvío ciego, con «Revisar contacto» que muestra teléfono/canal y «Volver al mensaje» conserva el contexto; el detalle de cita muestra sus mensajes reales. Escape; 800×600 con scroll propio. Resultados ficticios en memoria.

**Dependencies:** T12 y C05
**Files likely touched:**
- `design-prototype/app.js`
- `design-prototype/styles.css`
**Estimated scope:** M

## T14: Completar indicadores

**Status:** Done
**Superficies:** S10

**Description:** Mantener un informe sobrio con periodo real de ejemplo, definiciones y estados de datos.

**Acceptance criteria:**
- [x] Cada dato tiene unidad, periodo/zona, numerador/denominador cuando aplica, actualización y definición.
- [x] Gráficos tienen escala, etiquetas y equivalente tabular; filtros cambian un conjunto de ejemplo coherente.
- [x] Sin datos, vista antigua y servicio indisponible se distinguen; Agenda sigue utilizable.

**Verification:**
- [x] Focused tests: `node --check design-prototype/app.js`; comprobar el recorrido indicado abajo.
- [x] Build or broader check: `git diff --check -- design-prototype tasks/P-20261111-01-prototype-completion`; no requiere compilación Tauri.
- [x] Manual check: Cambiar periodo y revisar equivalencia entre cifras/gráfico/tabla; probar datos vacíos, antiguos e indisponibles.

**Evidence:** [verificación C06](evidence/C06/verification.md) y `evidence/C06/T14-checks.json` con dos capturas. Cinco periodos con conjuntos de ejemplo coherentes (octubre conserva 64/90 %; semana pasada 22/90 % con equivalente tabular); unidad, numerador/denominador y actualización en cada dato; gráfico con escala y tabla «Tramo | Citas»; «Sin datos en este periodo» con retorno, aviso de periodo en curso y «No pudimos leer el informe» con reintento honesto mientras Agenda sigue utilizable. Escape; 800×600 con métricas 2×2. Números ficticios fijos por periodo.

**Dependencies:** T13
**Files likely touched:**
- `design-prototype/app.js`
- `design-prototype/styles.css`
**Estimated scope:** M

## C06: Seguimiento

- [x] Verificación y límites de T13, T14 registrados en [evidence/C06/verification.md](evidence/C06/verification.md).
- [x] Revisión visual, teclado, estados y retorno del bloque. Zoom al 200 % y lector de pantalla pendientes.
- [x] Aceptación del usuario: «Puedes continuar, apruebo C06» (2026-11-12); continúa con T15–T17.

## T15: Completar equipo e invitaciones

**Status:** Done
**Superficies:** S12

**Description:** Definir personas, invitaciones y detalle de permisos dentro de las capacidades conocidas.

**Acceptance criteria:**
- [x] Cada persona tiene identidad propia, rol y acceso; no se inventan roles personalizados.
- [x] Invitación, estado y revisión de permisos explican conexión y autorización; lo no contratado queda como diseño.
- [x] Retirar acceso muestra sujeto y efecto; no promete revocación instantánea de equipos sin red.

**Verification:**
- [x] Focused tests: `node --check design-prototype/app.js`; comprobar el recorrido indicado abajo.
- [x] Build or broader check: `git diff --check -- design-prototype tasks/P-20261111-01-prototype-completion`; no requiere compilación Tauri.
- [x] Manual check: Recorrer invitación y revisión de acceso como dueño/asistente; comprobar límites sin red y confirmación de alcance.

**Evidence:** [verificación C07](evidence/C07/verification.md) y `evidence/C07/T15-checks.json` con captura. Personas con correo, rol definido y estado; invitaciones pendiente/vencida con alta validada (error propio que conserva), retirada con confirmación de efecto y reinvitación precargada; detalle de persona con rol, alcance y salvedades; retirada de acceso con sujeto, efecto y salvedad offline, bloqueada sin conexión; asistente en solo lectura. Resultados ficticios en memoria.

**Dependencies:** T14 y C06
**Files likely touched:**
- `design-prototype/app.js`
- `design-prototype/styles.css`
**Estimated scope:** M

## T16: Completar dispositivos

**Status:** Done
**Superficies:** S12

**Description:** Añadir detalle de equipo, autorización y revisión o revocación permitida.

**Acceptance criteria:**
- [x] Lista y detalle muestran equipo/aplicación, último acceso conocido y autorización sin secretos.
- [x] Revocar identifica equipo y consecuencias, distingue el equipo actual y no promete efecto inmediato offline.
- [x] Los estados de solo lectura, acceso restringido y resultado por comprobar ofrecen un siguiente paso válido.

**Verification:**
- [x] Focused tests: `node --check design-prototype/app.js`; comprobar el recorrido indicado abajo.
- [x] Build or broader check: `git diff --check -- design-prototype tasks/P-20261111-01-prototype-completion`; no requiere compilación Tauri.
- [x] Manual check: Abrir equipo actual y recepción; revisar revocación, acceso restringido y resultado desconocido.

**Evidence:** [verificación C07](evidence/C07/verification.md) y `evidence/C07/T16-checks.json` con dos capturas. Cuatro equipos con sistema, aplicación, último acceso y estado (Actual/Autorizado/Por confirmar/Revocado); detalle con autorización y sin secretos; revocación con consecuencias y salvedad offline, bloqueada sin conexión; «Comprobar de nuevo» confirma y «Autorizar de nuevo» reactiva; asistente en solo lectura; 800×600 sin desbordamiento. Estados ficticios en memoria.

**Dependencies:** T15
**Files likely touched:**
- `design-prototype/app.js`
- `design-prototype/styles.css`
**Estimated scope:** M

## T17: Completar políticas de comunicación

**Status:** Done
**Superficies:** S12

**Description:** Ordenar recordatorios, cancelación y enlaces por propósito sin fijar reglas abiertas.

**Acceptance criteria:**
- [x] Se distinguen políticas de recordatorio, acciones del paciente y enlaces; solo capacidades contratadas son editables.
- [x] Los valores sin política aprobada son ejemplos de revisión y no se presentan como regla del producto.
- [x] Guardar o fallar conserva el formulario y no afirma que se enviaron mensajes.

**Verification:**
- [x] Focused tests: `node --check design-prototype/app.js`; comprobar el recorrido indicado abajo.
- [x] Build or broader check: `git diff --check -- design-prototype tasks/P-20261111-01-prototype-completion`; no requiere compilación Tauri.
- [x] Manual check: Revisar controles permitidos/pendientes, guardar ejemplo y provocar error sin perder datos ni prometer entrega.

**Evidence:** [verificación C07](evidence/C07/verification.md) y `evidence/C07/T17-checks.json` con captura. Tres secciones separadas (Recordatorios editable; Acciones del paciente y Enlaces de cita como diseño pendiente); «Ejemplo de revisión · sin política aprobada» en la anticipación; error sin conexión y como asistente con formulario conservado (48 horas); guardado con aviso «No se envían mensajes por este cambio».

**Dependencies:** T16
**Files likely touched:**
- `design-prototype/app.js`
- `design-prototype/styles.css`
**Estimated scope:** M

## C07: Equipo, dispositivos y políticas

- [x] Verificación y límites de T15, T16, T17 registrados en [evidence/C07/verification.md](evidence/C07/verification.md).
- [x] Revisión visual, teclado, estados y retorno del bloque. Zoom al 200 % y lector de pantalla pendientes.
- [x] Aceptación del usuario: «C05, C06 y C07 Aprobados, por favor continúa con T18, T19, T20, T21» (2026-11-12); continúa con T18–T21.

## T18: Completar asuntos por revisar

**Status:** Done
**Superficies:** S11

**Description:** Convertir el centro en una lista de conflictos, rechazos y restricciones que requieren una decisión.

**Acceptance criteria:**
- [x] El enlace aparece solo con intervención pendiente y cada problema también figura junto al dato afectado.
- [x] La comparación conserva propuesta y estado compartido; no añade forzar, descartar o sobrescribir sin contrato.
- [x] Recuperación automática y resultado desconocido no crean ruido diario ni permiten una segunda intención.

**Verification:**
- [x] Focused tests: `node --check design-prototype/app.js`; comprobar el recorrido indicado abajo.
- [x] Build or broader check: `git diff --check -- design-prototype tasks/P-20261111-01-prototype-completion`; no requiere compilación Tauri.
- [x] Manual check: Recorrer conflicto de horario, edición rechazada y permiso revocado; comprobar normal/offline silenciosos y recuperación sin duplicados.

**Evidence:** [verificación C08](evidence/C08/verification.md) y `evidence/C08/T18-checks.json` con captura. Centro derivado del estado real con contador en navegación (edición rechazada, conciliación de r4, revocación de T16; conflicto con el escenario); «Marcar revisado» archiva el asunto del centro y del aviso de Horarios; sin ítems de recuperación automática ni resultado desconocido.

**Dependencies:** T17 y C07
**Files likely touched:**
- `design-prototype/app.js`
- `design-prototype/styles.css`
**Estimated scope:** M

## T19: Completar búsqueda y ayuda

**Status:** Done
**Superficies:** S01–S13

**Description:** Mejorar acceso a pacientes/citas, ayuda por tarea y diagnóstico separado.

**Acceptance criteria:**
- [x] Búsqueda visible y Ctrl+K/Cmd+K tienen resultados agrupados, teclado y cuenta/contexto permitidos.
- [x] Sin coincidencias locales no se presenta como ausencia global; volver restaura búsqueda y foco.
- [x] Ayuda explica la tarea; diagnóstico contiene solo ejemplos técnicos sin secretos y fuera del trabajo diario.

**Verification:**
- [x] Focused tests: `node --check design-prototype/app.js`; comprobar el recorrido indicado abajo.
- [x] Build or broader check: `git diff --check -- design-prototype tasks/P-20261111-01-prototype-completion`; no requiere compilación Tauri.
- [x] Manual check: Buscar homónimos y citas, abrir resultado y volver con teclado; revisar ayuda y diagnóstico sin datos sensibles.

**Evidence:** [verificación C08](evidence/C08/verification.md) y `evidence/C08/T19-checks.json` con dos capturas. Resultados agrupados Pacientes/Citas (H08 cerrado): «Ana María» distingue homónimas por teléfono y «09:00» devuelve citas; mensaje de sin coincidencias local; Ctrl+K restaura la consulta tras abrir un resultado; ayuda por tarea con enlaces (H09 cerrado); diagnóstico sin secretos.

**Dependencies:** T18
**Files likely touched:**
- `design-prototype/app.js`
- `design-prototype/styles.css`
**Estimated scope:** M

## T20: Completar la muestra Patient Hub

**Status:** Done
**Superficies:** S14

**Description:** Ampliar la entrega visual móvil de la cita existente y sus resultados; mantenerla como muestra sin servicio.

**Acceptance criteria:**
- [x] Confirmar, cancelar y pedir cambio tienen confirmación/resultados claros; pedir cambio conserva horario y no ofrece agenda pública.
- [x] Se distinguen resultado por comprobar, cita modificada, enlace vencido/revocado y servicio indisponible.
- [x] Desde 320 px conserva una columna, acciones táctiles, foco y texto legible; no requiere registro o instalación inventados.

**Verification:**
- [x] Focused tests: `node --check design-prototype/app.js`; comprobar el recorrido indicado abajo.
- [x] Build or broader check: `git diff --check -- design-prototype tasks/P-20261111-01-prototype-completion`; no requiere compilación Tauri.
- [x] Manual check: Recorrer móvil con teclado las tres acciones y sus errores; verificar que GET/navegación no confirma ni cancela.

**Evidence:** [verificación C08](evidence/C08/verification.md) y `evidence/C08/T20-checks.json` con captura 320 px. Confirmar/cancelar/pedir cambio con resultados claros y horario conservado; «resultado por comprobar» alcanzado por flujo real con servicio caído; `hub/servicio` separado del resultado por comprobar; `hub/cita-modificada` con horario anterior; una columna legible a 320×720.

**Dependencies:** T19
**Files likely touched:**
- `design-prototype/app.js`
- `design-prototype/styles.css`
**Estimated scope:** M

## C08: Recuperación, ayuda y Hub

- [x] Verificación y límites de T18, T19, T20 registrados en [evidence/C08/verification.md](evidence/C08/verification.md).
- [x] Revisión visual, teclado, estados y retorno del bloque. Zoom al 200 % y lector de pantalla pendientes.
- [ ] Aceptación del usuario.

## T21: Cerrar cobertura y evidencia

**Status:** Done
**Superficies:** S01–S14

**Description:** Actualizar catálogo y registro de revisión con las rutas y variantes realmente terminadas.

**Acceptance criteria:**
- [x] Catálogo y README enumeran superficies, variantes y estados; no quedan enlaces rotos o controles que aparenten operar sin hacerlo.
- [x] La matriz de cobertura registra pruebas de tamaños, teclado, contexto y escenarios con sus límites.
- [x] verification.json y una hoja de vistas previa representan el código final; la evidencia histórica se identifica como histórica.

**Verification:**
- [x] Focused tests: `node --check design-prototype/app.js`; comprobar el recorrido indicado abajo.
- [x] Build or broader check: `git diff --check -- design-prototype tasks/P-20261111-01-prototype-completion`; no requiere compilación Tauri.
- [x] Manual check: Recorrer cada entrada del catálogo y estados; revisar escritorio 800×600/1024×768/1280×800, Hub 320×720 y texto ampliado al 200 %.

**Evidence:** [verificación C09](evidence/C09/verification.md) y `evidence/C09/T21-checks.json`. Catálogo cerrado en 52 rutas (sin ofertas; con `hub/cita-modificada` y `hub/servicio`); cobertura estática del despachador 52/52 con Node; README actualizado con índice y capturas por bloque; `verification-2026-11-12.json` con SHA-256 del código final; hoja de vistas `previews/ampliacion-contact-sheet.png`. Zoom al 200 %, lector de pantalla y 1024×768 del código final: no comprobados (declarados).

**Dependencies:** T20 y C08
**Files likely touched:**
- `design-prototype/app.js`
- `design-prototype/README.md`
- `design-prototype/verification.json`
- `design-prototype/previews/ampliacion-contact-sheet.png`
**Estimated scope:** M

## C09: Revisión completa

- [x] Verificación y límites de T21 registrados en [evidence/C09/verification.md](evidence/C09/verification.md).
- [x] Revisión visual, teclado, estados y retorno del bloque. Zoom al 200 % y lector de pantalla pendientes.
- [ ] Aceptación del usuario.


