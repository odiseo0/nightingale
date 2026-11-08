# Task List: P-20261108-04-interface-master-plan

**Plan:** [plan.md](plan.md)
**Updated:** 2026-11-08

Solo redacción e integración documental. Ninguna tarea ejecuta diseño visual o implementación.

## T01: Registrar fuentes, cobertura y plan de redacción

**Status:** Done

**Description:** Crear el plan aprobado, su lista de tareas y la cobertura normativa necesaria para redactar el maestro sin reemplazar planes existentes.

**Acceptance criteria:**
- [x] Existen `plan.md` y `todo.md` con el ID P-20261108-04-interface-master-plan.
- [x] La cobertura enumera 34 secciones y exige trazabilidad RF-01–RF-16/S01–S14.
- [x] El alcance limita los cambios a documentación y conserva el prototipo como referencia histórica.

**Verification:**
- [x] Focused tests: lectura de las fuentes y del inventario de planes de 2026-11-08; ID 04 libre.
- [x] Build or broader check: no aplica; solo se crearon archivos de planificación.
- [x] Manual check: el plan aprobado y su desglose coinciden en alcance y orden.

**Dependencies:** None

**Files likely touched:**
- `tasks/P-20261108-04-interface-master-plan/plan.md`
- `tasks/P-20261108-04-interface-master-plan/todo.md`

**Estimated scope:** S

**Evidence:** fuentes técnicas, SRS, contrato, diseño, skill UX, prototipo y frontend actual revisados antes de la redacción.

## T02: Redactar el plan maestro de interfaz

**Status:** Done

**Description:** Crear `docs/design-master-plan.md` en estado Draft con las 34 secciones, catálogo de superficies/componentes, estados, frontend, accesibilidad, verificación, trazabilidad y contrato para planes derivados.

**Acceptance criteria:**
- [x] Las 34 secciones están presentes y distinguen requisitos, propuestas y pendientes.
- [x] RF-01–RF-16 y S01–S14 tienen trazabilidad; los estados locales/remotos no se confunden.
- [x] El documento no contiene backlog, implementación ni decisiones visuales incompatibles con DESIGN.

**Verification:**
- [x] Focused tests: checklist de secciones, RF, superficies, componentes y estados.
- [x] Build or broader check: `git diff --check -- docs/design-master-plan.md`.
- [x] Manual check: lectura cruzada contra SRS, plan técnico §22, DESIGN y Laws of UX.

**Dependencies:** T01

**Files likely touched:**
- `docs/design-master-plan.md`

**Estimated scope:** M

**Evidence:** documento de 1.535 líneas con 34 secciones, S01–S14, RF-01–RF-16, estados, componentes, accesibilidad y contrato para planes derivados. Tres errores de numeración detectados y corregidos durante C01.

## C01: Después de T02

- [x] La cobertura completa y la precedencia documental son coherentes.
- [x] El maestro permanece en Draft y no resuelve decisiones abiertas por preferencia visual.
- [x] Revisión de cobertura y límites completada antes de integrar enlaces.

## T03: Integrar enlaces y validar documentación

**Status:** Done

**Description:** Añadir el maestro a los índices documentales como Draft y verificar enlaces, formato, coherencia y archivos modificados.

**Acceptance criteria:**
- [x] `docs/README.md` y `AGENTS.md` enlazan el maestro como Draft sin alterar la precedencia técnica.
- [x] Los enlaces Markdown locales nuevos resuelven a archivos existentes.
- [x] El diff está limitado al maestro, sus índices y este plan; los chequeos documentales pasan.

**Verification:**
- [x] Focused tests: comprobación directa de enlaces locales y búsqueda de contradicciones tipográficas/semánticas.
- [x] Build or broader check: `git diff --check` para archivos seguidos y `git diff --no-index --check /dev/null <archivo>` para los archivos nuevos.
- [x] Manual check: revisar diff y confirmar que código, prototipo, contratos y planes históricos no cambiaron.

**Dependencies:** T02 y C01

**Files likely touched:**
- `docs/README.md`
- `AGENTS.md`
- `tasks/P-20261108-04-interface-master-plan/plan.md`
- `tasks/P-20261108-04-interface-master-plan/todo.md`

**Estimated scope:** S

**Evidence:** 46 enlaces locales resueltos; 34 secciones principales; whitespace válido en los cuatro archivos nuevos/de planificación y en los dos índices. `git status --short` muestra solo el maestro, sus índices y este plan.

## C02: Revisión final

- [x] Presentar al usuario el borrador y las comprobaciones realizadas.
- [x] Registrar que cualquier diseño o implementación posterior requiere otro plan aprobado.
- [x] El usuario aprobó el contenido del maestro el 2026-11-08; su estado se actualizó explícitamente a Aprobado.
