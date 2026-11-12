# Nightingale · Prototipo de diseño

Prototipo independiente de la aplicación de escritorio y del recorrido del paciente. Contiene 52 rutas en el catálogo. Las capturas de `previews/` son históricas, del 2026-10-22; la ampliación registra sus capturas en las carpetas de evidencia [C01–C08](../tasks/P-20261111-01-prototype-completion/) y la hoja de vistas final en [ampliacion-contact-sheet.png](previews/ampliacion-contact-sheet.png). No modifica la aplicación Tauri.

Ampliación [P-20261111-01](../tasks/P-20261111-01-prototype-completion/plan.md): **T01–T21 terminadas; C01–C07 aprobadas por el usuario; C08 y C09 pendientes de su revisión.** La carga de una ruta no acredita que su recorrido esté terminado; los recorridos se verifican por bloque en la evidencia enlazada.

## Abrir

Abre `index.html` en un navegador. Los archivos, las fuentes Montserrat/Mulish y los iconos Phosphor son locales: el prototipo no necesita paquetes ni conexión a servicios.

También puedes servir solo esta carpeta desde la raíz del repositorio:

```powershell
python -m http.server 8766 --bind 127.0.0.1 --directory design-prototype
```

Después abre `http://127.0.0.1:8766/`. Detén el servidor con Ctrl+C cuando termines. Este comando requiere Python disponible en PATH. La revisión C02 utilizó un servidor temporal de archivos con Node local; no añade archivos ni dependencias al prototipo.

El enlace **Ver todas las pantallas**, al pie, abre el índice. Para empezar por él, usa `index.html#/pantallas`. La fecha del ejemplo es el jueves 22 de octubre de 2026, en Caracas.

## Iteración de diseño · 2026-11-11

Por petición del usuario, Semana muestra separadores cada hora y mantiene la escala de 96 px/hora y las duraciones reales. Al abrir una cita aparece un modal sobre la pantalla de origen; cierra con el botón, Escape o clic fuera y conserva búsqueda, foco y desplazamiento. Reprogramación, confirmación y cancelación siguen disponibles según estado y permiso.

**Colores propuestos**, al pie, explica la propuesta pendiente de revisión: azul para Programada, verde para Confirmada/Entregado, ámbar para Ausencia/datos por completar/solicitudes por revisar, rosa tenue para Cancelada, neutros para Atendida y datos habituales. Cada estado mantiene su rótulo. La propuesta usa los mismos roles en etiquetas, calendario y modal; no cambia aún la paleta aprobada en DESIGN.md. Las fichas completas quedan neutras para destacar las que necesitan trabajo.

[Capturas, comprobaciones y límites de esta iteración](../tasks/P-20261111-01-prototype-completion/evidence/design-iteration/verification.md). Esta evidencia sustituye la presentación del detalle lateral de las capturas anteriores; no cierra las tareas pendientes ni acredita correcciones de los otros hallazgos de revisión.

La revisión de tipografía e identidad añade nombres y horarios con más peso, datos de apoyo en gris legible e iconos Phosphor junto a etiquetas de tabla y detalle. Las iniciales usan seis fondos suaves, estables por identificador de persona: no cambian al filtrar, editar el nombre o recargar. Esos colores ayudan a reconocer personas y pueden repetirse; los estados de cita mantienen su propia paleta y rótulos. No se añade persistencia ni un campo de base de datos. [Evidencia de tipografía e identidad](../tasks/P-20261111-01-prototype-completion/evidence/typography-identity/verification.md).

## Recorridos

- Acceso: navegador externo, cuenta, autorización del equipo, preparación, acceso local, sesión vencida, revocación y servicio no disponible; todo mediante simulación.
- Agenda: Día, Semana y Lista de citas; fechas, búsqueda, filtros, carga de más filas y detalle con retorno al contexto anterior. Vacío conocido, filtro vacío y fecha sin cobertura tienen mensajes distintos.
- Citas: crear, comprobar un solapamiento, reprogramar, confirmar, registrar atención/ausencia y cancelar con confirmación.
- Pacientes: buscar, abrir ficha, registrar y editar. El alta distingue documento propio y menor sin documento, guarda borradores y conserva prefijos y ceros de documentos; el teléfono se valida como pareja código/número. El directorio separa contacto, próxima cita, actividad, completitud y estado. La ficha separa próximos encuentros e historial paginado, gestiona responsables (parentesco, función, vigencia, representación y actividad por separado) y permite inactivar sin cancelar citas. También se puede registrar un paciente o un responsable desde el editor de cita conservando sus campos.
- Solicitudes: revisar la petición junto a la cita actual y pasar al editor de horario; la solicitud se resuelve al guardar sin duplicar cambios.
- Lista de espera: añadir y editar entradas con preferencias, ofrecer horarios compatibles y asignar el intervalo con confirmación del personal; el orden no fija prioridad. Sin ofertas ni conversión: los pacientes no son leads.
- Comunicaciones: filtrar por estado, abrir el detalle con la cita relacionada y revisar el contacto cuando hay fallo o resultado desconocido; sin reenvío a ciegas, sin chat libre ni envíos reales.
- Indicadores: informe por periodos operables con unidades, numeradores/denominadores, actualización, definiciones, escala del gráfico y equivalente tabular; distinguen periodo en curso, sin datos y servicio no disponible.
- Por revisar: problemas derivados del estado real (conflicto, edición rechazada, conciliación pendiente, revocación por confirmar, acceso retirado) con decisión del usuario; las recuperaciones automáticas y los resultados por comprobar no crean avisos.
- Búsqueda y ayuda: Ctrl+K con resultados agrupados de pacientes y citas, búsqueda por hora/servicio/fecha y consulta restaurada al volver; ayuda breve por tarea con enlaces; diagnóstico aislado sin secretos.
- Configuración: cuenta, servicios, horarios y bloqueos, políticas de comunicación, personas e invitaciones, y dispositivos con revocación y reautorización.
- Patient Hub: consultar, confirmar, cancelar, solicitar cambio y revisar resultados; distinguen cita modificada, enlace vencido o revocado, resultado por comprobar y servicio no disponible. Las etapas de este recorrido son ejemplos independientes del escritorio; no simulan una integración entre servidores.

Los datos solo viven en memoria. Recargar la página restablece el ejemplo. El editor de cita protege cambios al salir mediante enlaces o Atrás y conserva los campos al registrar un paciente relacionado. Los formularios de configuración validan con errores propios que conservan los datos escritos; los cambios compartidos exigen conexión, rol de administración y, cuando corresponde, confirmación de sujeto y efecto.

**Probar un escenario** permite elegir funcionamiento normal, falta de conexión, servicio no disponible o conflicto, y comparar la vista de la dueña con la del asistente. Es un control del prototipo, fuera de la navegación del producto. La desconexión que permite trabajo local y los reintentos automáticos permanecen en silencio. El enlace «Por revisar» aparece solo con problemas pendientes —derivados del estado real del ejemplo— y cada problema también figura junto al dato afectado. Las variantes de profesionales son datos de revisión; no amplían el alcance de la primera versión ni prueban permisos reales.

El mismo control permite revisar los estados de lectura de Agenda, el permiso de edición de citas y la hora ficticia. El ejemplo usa un margen de ausencia de 15 minutos, conforme al valor inicial del contrato vigente; la ausencia siempre exige una acción del personal. Volver al escenario normal representa una aceptación conocida de los cambios locales del ejemplo: no llama a un servidor. Una cancelación local conserva la ocupación anterior hasta esa aceptación simulada.

## Índice de vistas y capturas

Cada ruta se añade después de `index.html#`. Todas las vistas están guardadas en los archivos compartidos `index.html`, `styles.css` y `app.js`; no requieren un servidor que resuelva rutas.

| Pantalla | Ruta | Captura |
|---|---|---|
| Acceso | `/acceso` | [Ver](previews/acceso.png) |
| Acceso en el navegador | `/acceso/navegador` | C02 |
| Seleccionar consulta | `/acceso/cuentas` | C02 |
| Autorizar equipo | `/acceso/equipo` | C02 |
| Preparación inicial pendiente | `/acceso/preparando` | C02 |
| Acceso local permitido | `/acceso/local` | C02 |
| Sesión remota vencida | `/acceso/sesion-vencida` | C02 |
| Acceso revocado | `/acceso/denegado` | [C02](../tasks/P-20261111-01-prototype-completion/evidence/C02/T03-acceso-revocado.jpg) |
| Conexión necesaria | `/acceso/conexion` | C02 |
| Servicio de acceso no disponible | `/acceso/servicio` | C02 |
| Preparación | `/preparacion` | [Ver](previews/preparacion.png) |
| Resultado de preparación | `/preparacion/lista` | [C02](../tasks/P-20261111-01-prototype-completion/evidence/C02/T03-preparacion-lista.jpg) |
| Agenda del día | `/agenda` | [Ver](previews/agenda.png) |
| Agenda semanal | `/agenda/semana` | [Ver](previews/agenda-semana.png) |
| Lista de citas | `/citas` | [Ver](previews/citas.png) |
| Detalle de cita | `/cita/a1` | [Ver](previews/cita-a1.png) |
| Nueva cita | `/cita/nueva` | [Ver](previews/cita-nueva.png) |
| Reprogramar | `/cita/a1/reprogramar` | [Ver](previews/cita-a1-reprogramar.png) |
| Reprogramar 45 minutos | `/cita/a16/reprogramar` | [C02](../tasks/P-20261111-01-prototype-completion/evidence/C02/C02-reprogramar.jpg) |
| Cita atendida | `/cita/a10` | C02 |
| Cita cancelada | `/cita/a11` | C02 |
| Ausencia registrada | `/cita/a19` | C02 |
| Pacientes | `/pacientes` | [Ver](previews/pacientes.png) |
| Ficha del paciente | `/paciente/p1` | [Ver](previews/paciente-p1.png) |
| Ficha de menor con responsable | `/paciente/p9` | [C03](../tasks/P-20261111-01-prototype-completion/evidence/C03/T06-registro-menor.png) |
| Ficha incompleta · borrador | `/paciente/p10` | [C03](../tasks/P-20261111-01-prototype-completion/evidence/C03/T06-ficha-borrador.png) |
| Pacientes homónimos | `/paciente/p11` | C03 |
| Registrar paciente | `/pacientes/nuevo` | [Ver](previews/pacientes-nuevo.png) |
| Solicitudes | `/solicitudes` | [Ver](previews/solicitudes.png) |
| Revisar solicitud | `/solicitud/r1` | [Ver](previews/solicitud-r1.png) |
| Lista de espera | `/lista-espera` | [C05](../tasks/P-20261111-01-prototype-completion/evidence/C05/T12-espera.png) |
| Comunicaciones | `/comunicaciones` | [C06](../tasks/P-20261111-01-prototype-completion/evidence/C06/T13-comunicaciones.png) |
| Indicadores | `/indicadores` | [C06](../tasks/P-20261111-01-prototype-completion/evidence/C06/T14-indicadores.png) |
| Cuenta | `/configuracion/cuenta` | [Ver](previews/configuracion-cuenta.png) |
| Servicios | `/configuracion/servicios` | [Ver](previews/configuracion-servicios.png) |
| Horarios | `/configuracion/horarios` | [Ver](previews/configuracion-horarios.png) |
| Política de comunicaciones | `/configuracion/comunicaciones` | [C07](../tasks/P-20261111-01-prototype-completion/evidence/C07/T17-politica-comunicaciones.png) |
| Usuarios y permisos | `/configuracion/equipo` | [C07](../tasks/P-20261111-01-prototype-completion/evidence/C07/T15-equipo.png) |
| Dispositivos | `/configuracion/dispositivos` | [C07](../tasks/P-20261111-01-prototype-completion/evidence/C07/T16-dispositivos.png) |
| Por revisar | `/actividad` | [C08](../tasks/P-20261111-01-prototype-completion/evidence/C08/T18-por-revisar.png) |
| Conflicto de horario | `/conflicto/a3` | [Ver](previews/conflicto-a3.png) |
| Diagnóstico | `/diagnostico` | [Ver](previews/diagnostico.png) |
| Hub: cita | `/hub/cita` | [Ver](previews/hub-cita.png) |
| Hub: confirmación | `/hub/confirmada` | [Ver](previews/hub-confirmada.png) |
| Hub: cancelar | `/hub/cancelar` | [Ver](previews/hub-cancelar.png) |
| Hub: cancelada | `/hub/cancelada` | [Ver](previews/hub-cancelada.png) |
| Hub: pedir cambio | `/hub/cambiar` | [Ver](previews/hub-cambiar.png) |
| Hub: solicitud recibida | `/hub/solicitud` | [Ver](previews/hub-solicitud.png) |
| Hub: cita modificada | `/hub/cita-modificada` | [C08](../tasks/P-20261111-01-prototype-completion/evidence/C08/T20-hub-cita-modificada-320.png) |
| Hub: enlace vencido o revocado | `/hub/enlace-vencido` | [Ver](previews/hub-enlace-vencido.png) |
| Hub: resultado por comprobar | `/hub/no-disponible` | [Ver](previews/hub-no-disponible.png) |
| Hub: servicio no disponible | `/hub/servicio` | C08 |

## Criterios visuales

- Tema claro, Montserrat para encabezados y Mulish para lectura/controles, pesos 400/500/600, fondos fríos y azul para acciones y selección.
- Phosphor 2.1.10 regular, con trazados SVG locales y licencia; sin React en el navegador.
- Tablas y separadores finos, navegación compacta y detalle de cita en un modal con desplazamiento propio y fondo no interactivo.
- Texto oscuro explícito en etiquetas neutras. La confirmación combina verde oscuro con un rótulo; el color no lleva solo el significado.
- Fecha y horario anterior junto al nuevo durante la reprogramación. Las acciones que cambian la cita conservan una confirmación explícita cuando corresponde.
- Formularios con etiquetas permanentes, controles nativos, foco visible y reducción de movimiento.

La propuesta se basa en [DESIGN.md](../DESIGN.md), [DESIGN-RULES.md](../DESIGN-RULES.md), [Laws of UX](../.agents/laws-of-ux/SKILL.md) y la revisión del usuario de las referencias Attio/Linear. No copia marcas ni funciones de esos productos.

## Verificación y límites

La revisión histórica del 2026-10-22 comprobó las 34 rutas a 1280×720, 800×600 y 320×720, sin desbordamiento horizontal del documento. Las tablas y la semana pueden tener su propio desplazamiento horizontal. Se inspeccionaron visualmente Agenda, detalle, editor y Hub. `verification.json` conserva esa comprobación histórica; no acredita la ampliación. La ampliación se verifica por bloques en la evidencia C01–C08 del plan y se resume en `verification-2026-11-12.json` (52 rutas, cobertura del despachador, tamaños comprobados, límites y SHA-256 del código final).

La verificación de T01/T02 del 2026-11-11 se registra en el plan y su carpeta `evidence/C01/`: fuentes, rutas, variantes de rol, avisos, retorno y teclado. El usuario aprobó C01 y autorizó T03–T05.

[La verificación C02](../tasks/P-20261111-01-prototype-completion/evidence/C02/verification.md) registra acceso/preparación, Agenda y ciclo de cita. Las 48 rutas cargaron a 1280×800, 1024×768 y 800×600 sin desbordamiento horizontal del documento; esa prueba no verifica todas sus funciones. La semana usa 96 px/hora, días de al menos 160 px y desplazamiento propio. Hay recorridos de creación, reprogramación con duración conservada, responsables, permisos, terminales, conflicto y cambios sin guardar. El usuario aprobó C02 el 2026-11-11.

[La verificación T06](../tasks/P-20261111-01-prototype-completion/evidence/C03/T06-checks.json) registra el directorio y el registro de pacientes: alta de adulto y de menor, borrador con requisitos, pareja telefónica, prefijos y ceros, homónimos, búsqueda vacía local, conflicto de completitud y edición que completa una ficha. El catálogo pasa a 51 rutas con las fichas de ejemplo p9–p11.

[La verificación T07](../tasks/P-20261111-01-prototype-completion/evidence/C03/T07-checks.json) registra la ficha y los responsables: datos de vínculo separados, bloqueo de completitud al inactivar el único responsable vigente, alta desde ficha y desde cita, historial paginado e inactivación de ficha sin cancelar citas.

[La verificación T08](../tasks/P-20261111-01-prototype-completion/evidence/C04/T08-checks.json) registra la cuenta: datos compartidos con dirección hasta ciudad/estado, preferencias locales separadas, solo lectura para la asistente, bloqueo sin conexión y preservación del lugar de las citas históricas.

[La verificación T09](../tasks/P-20261111-01-prototype-completion/evidence/C04/T09-checks.json) registra el catálogo y la oferta: duración base separada del ajuste, activación por niveles con confirmación, límites de dueña y conexión, y citas existentes que conservan servicio y duración.

[La verificación T10](../tasks/P-20261111-01-prototype-completion/evidence/C04/T10-checks.json) registra horarios y bloqueos: reglas con vigencia y tramos, excepciones de cierre/reemplazo, bloqueos con alcance, rechazo de cambios que cruzan citas con lista de afectados, asistente sin motivos globales y Agenda/Semana/Nueva cita sobre ventanas dinámicas.

[La verificación T11](../tasks/P-20261111-01-prototype-completion/evidence/C05/T11-checks.json) registra solicitudes: petición y cita vigente juntas, antigüedad relativa, estados de resuelta/desactualizada/conciliación pendiente, y resolución al guardar el nuevo horario sin duplicar cambios. [La verificación C05](../tasks/P-20261111-01-prototype-completion/evidence/C05/verification.md) registra además la lista de espera revisada por el usuario: sin ofertas, con alta/edición de entradas y asignación directa confirmada por el personal.

[La verificación C06](../tasks/P-20261111-01-prototype-completion/evidence/C06/verification.md) registra comunicaciones (filtros, detalle con cita relacionada, fallo y resultado desconocido sin reenvío ciego, revisión de contacto con retorno) e indicadores (periodos operables, unidades y definiciones, escala y equivalente tabular, estados distintos). [La verificación C07](../tasks/P-20261111-01-prototype-completion/evidence/C07/verification.md) registra equipo e invitaciones, dispositivos con revocación y reautorización, y políticas de comunicación por propósito. [La verificación C08](../tasks/P-20261111-01-prototype-completion/evidence/C08/verification.md) registra el centro Por revisar derivado del estado real, la búsqueda agrupada de pacientes y citas con Ctrl+K, la ayuda por tarea y las nuevas pantallas del Hub (cita modificada, resultado por comprobar separado del servicio no disponible), incluido 320 px.

Se probaron creación, solapamiento con conservación del formulario, reprogramación, aviso de cambios sin guardar, cancelación, búsqueda/registro de pacientes, petición del Hub y escenarios de desconexión/conflicto. Las pruebas verifican el prototipo, no la arquitectura distribuida. No se afirma conformidad completa de accesibilidad; faltan revisión con lector de pantalla y pruebas con usuarios reales.

No hay autenticación, SQLite, sincronización, WhatsApp ni servicios externos. No se introducen datos reales. La muestra aplica las bases de identificación, permisos y duración descritas en el contrato vigente, sin acreditar su aplicación en el producto. Incluye citas de 15/30/45/60 minutos. Las políticas pendientes, como la prioridad de espera o los plazos de cancelación, siguen pendientes y el prototipo las muestra como diseño o ejemplo de revisión, no como regla. Zoom al 200 %, lector de pantalla y pruebas con usuarios siguen sin comprobarse.

Referencias de los controles nativos: [diálogo modal](https://developer.mozilla.org/en-US/docs/Web/API/HTMLDialogElement/showModal), [campo de fecha](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/input/date), [fuentes variables](https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/At-rules/@font-face/font-weight#setting_a_range_for_a_variable_font), [foco](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement/focus) y [fondo no interactivo del menú](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement/inert). Montserrat y Mulish conservan origen y licencia en `assets/fonts-LICENSE.txt`; [Phosphor 2.1.10](https://github.com/phosphor-icons/react/tree/v2.1.10) en `assets/icons-LICENSE.txt`. Los recursos de Inter se conservan como antecedentes y no se cargan.

Referencias C02: [columnas CSS Grid](https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/grid-template-columns), [posición sticky](https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/position#sticky_positioning), [scrollTop](https://developer.mozilla.org/en-US/docs/Web/API/Element/scrollTop), [replaceState](https://developer.mozilla.org/en-US/docs/Web/API/History/replaceState) y [hashchange](https://developer.mozilla.org/en-US/docs/Web/API/Window/hashchange_event). `replaceState` permite conservar la ruta del borrador sin provocar un segundo `hashchange`.

## Iteración de controles y calendario — 2026-11-12

Desplegables compartidos con chevrón, lista separada y hover/foco; modal de cita con fecha/hora y acciones como prioridad; calendario de selección de fecha; definición de horarios por días, vigencia y tramos con duración. Se conservan los controles nativos, los valores de los formularios y las reglas de las citas. [Recorrido, capturas, fuentes y límites](../tasks/P-20261111-01-prototype-completion/evidence/controls-calendar/verification.md). La paleta sigue pendiente de revisión del usuario.

## Reglas tipográficas — 2026-11-12

La petición de 45–75 caracteres por línea, escala de tamaños de 4px e interlineado según función queda registrada en DESIGN.md y DESIGN-RULES.md y aplicada al prototipo. Los párrafos usan 58ch tras medir el texto de Mulish; móvil y superficies estrechas permiten líneas más cortas. [Medidas y revisión visual](../tasks/P-20261111-01-prototype-completion/evidence/type-rhythm/verification.md).
