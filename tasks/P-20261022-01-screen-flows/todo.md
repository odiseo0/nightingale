# Task List: P-20261022-01-screen-flows

**Plan:** [plan.md](plan.md)
**Updated:** 2026-10-22

Este es un plan de revisión de diseño en estado Draft. No contiene tareas de implementación aprobadas. Los checks siguientes se completan al revisar los recorridos, no al redactar la propuesta.

## T01: Validar el inicio y la navegación

**Status:** Todo

**Description:** Revisar Agenda como inicio y la ubicación de citas, pacientes, seguimiento y configuración usando la propuesta y la muestra con datos ficticios.

**Acceptance criteria:**
- [ ] Se acuerda qué abre un equipo preparado y qué muestra la primera instalación.
- [ ] Se puede ubicar una cita de hoy, una pasada y un paciente sin destinos duplicados.
- [ ] Se revisa el contenido de cada vista de Agenda y el acceso a servicios/horarios.

**Verification:**
- [ ] Focused tests: recorrido de escritorio de S01–S02/S05–S06 con datos ficticios; sin pruebas automáticas de negocio.
- [ ] Build or broader check: `git diff --check`; compilación no aplicable a documentos.
- [ ] Manual check: identificar la siguiente cita y volver desde un detalle conservando el contexto; revisión del usuario.

**Dependencies:** None
**Files likely touched:** `plan.md`, `todo.md` en este plan.
**Estimated scope:** S

## T02: Validar el recorrido de una cita

**Status:** Todo

**Description:** Revisar S03–S04 y el paso de registrar paciente para crear, cambiar o cancelar una cita, separando los resultados local y compartido.

**Acceptance criteria:**
- [ ] Cada acción muestra paciente, horario, consecuencia y resultado conocido.
- [ ] Reprogramar conserva visibles el horario anterior y la propuesta; un error conserva el formulario.
- [ ] Atención/ausencia y confirmación respetan permisos y transiciones; las reglas abiertas quedan identificadas.

**Verification:**
- [ ] Focused tests: recorrido crear, reprogramar y cancelar sobre ejemplos; contrastar SRS RF-01–04/08/10.
- [ ] Build or broader check: `git diff --check`; no ejecutar `pnpm quality` o `pnpm build:desktop` por cambios solo documentales.
- [ ] Manual check: revisión de texto largo, nombres iguales y uso de teclado en el prototipo cuando corresponda; revisión del usuario.

**Dependencies:** T01
**Files likely touched:** `plan.md`, `todo.md` en este plan.
**Estimated scope:** S

## C01: Después de T01–T02

- [ ] Inicio, navegación y recorrido de cita revisados por el usuario.
- [ ] Pruebas con médico/asistente registradas, o señaladas como pendientes sin afirmar validación de uso real.

## T03: Validar pendientes y recuperación

**Status:** Todo

**Description:** Revisar S07–S12 y sus mensajes ante falta de red, conflicto, respuesta perdida, permiso cambiado o dato no descargado.

**Acceptance criteria:**
- [ ] Se distinguen estados de cita, solicitud, sincronización y comunicación sin éxitos remotos inventados.
- [ ] Se conserva la intención local y existe un siguiente paso claro ante cada fallo.
- [ ] Los datos antiguos, ausentes y vacíos se distinguen en agenda, solicitudes, comunicaciones e indicadores.

**Verification:**
- [ ] Focused tests: revisar los escenarios de RF-13–16 contra las tablas y recorridos del plan; no son pruebas de red/SQLite/API.
- [ ] Build or broader check: `git diff --check` y contraste con ADR y especificaciones vigentes.
- [ ] Manual check: el usuario explica qué quedó guardado, qué falta aceptar y qué puede hacer en cada ejemplo.

**Dependencies:** T02 y C01
**Files likely touched:** `plan.md`, `todo.md` en este plan.
**Estimated scope:** M

## T04: Validar el recorrido externo del paciente

**Status:** Todo

**Description:** Revisar S14 y su relación con S07, sin añadir una agenda pública, reservas nuevas o reglas clínicas al Hub.

**Acceptance criteria:**
- [ ] Confirmar/cancelar y solicitar cambio tienen resultados y límites claros, con fallo parcial identificado.
- [ ] Se definen las superficies de enlace inválido, servicio no disponible y versión cambiada sin deducir políticas abiertas.
- [ ] Se revisan requisitos móviles, teclado, foco y contenido mínimo sin dar por probada la accesibilidad del producto.

**Verification:**
- [ ] Focused tests: lectura de recorridos válidos y fallidos contra la especificación del Hub.
- [ ] Build or broader check: `git diff --check`; no implementar ni compilar el repositorio externo.
- [ ] Manual check: revisión del recorrido con datos ficticios y aceptación del usuario.

**Dependencies:** T03
**Files likely touched:** `plan.md`, `todo.md` en este plan.
**Estimated scope:** S

## C02: Después de T03–T04

- [ ] Requisitos vigentes y propuestas UX quedan diferenciados.
- [ ] La revisión identifica decisiones de dominio/seguridad pendientes sin resolverlas por diseño.
- [ ] El usuario acepta la definición; la implementación conserva su aprobación y planificación propias.

## T05: Crear el prototipo visual independiente

**Status:** Done

**Description:** Solicitud directa posterior del usuario: generar todas las pantallas en `design-prototype/`, con navegación e interacciones de ejemplo, datos ficticios y el sistema visual del repositorio. No se usa OpenDesign ni se integra funcionalidad en el producto.

**Acceptance criteria:**
- [x] Las 14 superficies de la definición y sus variantes principales tienen rutas accesibles desde un índice de pantallas.
- [x] Agenda, citas, pacientes y solicitudes se pueden recorrer; los estados normales de sincronización no aparecen en las filas y los chips neutros tienen texto oscuro.
- [x] Se verifican navegación, formularios, estados excepcionales y presentación a distintos tamaños; se documentan los límites del prototipo.

**Verification:**
- [x] Focused tests: sintaxis, recursos y rutas; recorridos de ejemplo en navegador.
- [x] Build or broader check: comprobación directa de archivos nuevos y `git diff --check`. No hay cambios en la aplicación que requieran compilación Tauri.
- [x] Manual check: revisión visual de Agenda, detalle, editor y Hub; no sustituye la revisión del usuario ni pruebas con personal médico.

**Dependencies:** propuesta existente y revisión del usuario sobre sincronización, contraste y referencias visuales.
**Files likely touched:** `design-prototype/index.html`, `design-prototype/styles.css`, `design-prototype/app.js`, `design-prototype/README.md`, recursos locales del prototipo.
**Estimated scope:** M

**Evidence:** 34 rutas y capturas en `design-prototype/`; ver `README.md` y `verification.json` de esa carpeta. Navegación a 1280×720, 800×600 y 320×720 sin desbordamiento del documento. Los recorridos principales y los estados excepcionales se verificaron en la vista previa local; sin llamadas a servicios ni integración en Tauri. Se detectaron y corrigieron un hueco que no se actualizaba, desbordamientos en anchos pequeños y mensajes que no correspondían a citas nuevas.
