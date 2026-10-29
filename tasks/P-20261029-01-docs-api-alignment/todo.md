# Task List: P-20261029-01-docs-api-alignment

**Plan:** [plan.md](plan.md)
**Updated:** 2026-10-29

El usuario autorizó los alcances de este plan el 2026-10-29 y pidió implementarlo, haciéndose cargo de los commits. Solo archivos de documentación y de planificación. Los JSON del contrato y `nightingale-api.md` son copia fija: no se editan.

## T01: Registrar `docs/contracts/` en `docs/README.md` como estado vigente del API

**Status:** Done

**Evidence (2026-10-29):** añadida línea de revisión 2026-10-29, fila de la tabla de versiones y sección «Contrato API v1 (`docs/contracts/`)» con origen, revisión `api-v1.1`, head `f572f9c`, conteos y regla de copia fija; registrada la eliminación de `docs/database-schema.md` y la fuente física en `nightingale-api`. Verificación: `grep` de términos clave en `docs/README.md` y `git diff --check` con código 0.

**Description:** Actualizar `docs/README.md` para que registre la carpeta `docs/contracts/` como la copia de consulta del estado actual de Nightingale API: procedencia (`nightingale-api`, exportador `scripts/export_contracts.py`), revisión `api-v1.1`, contrato v1, head `f572f9c9b3aa4cbd52284853a91e9a66435a1dff`, corte 2026-10-29, conteos del manifiesto y regla de no editar los JSON a mano. Registrar también la eliminación de `docs/database-schema.md` por decisión del usuario del 2026-10-29, con la fuente física viva en `nightingale-api/docs/database-schema.md` según el contrato (§5.6).

**Acceptance criteria:**
- [ ] `docs/README.md` contiene una entrada o sección que identifica `docs/contracts/` como estado vigente del API, con revisión, fecha de corte y regla de copia fija.
- [ ] `docs/README.md` registra la eliminación de `docs/database-schema.md` y la ubicación de la fuente propietaria.
- [ ] La tabla de «Versiones de origen» o su equivalente refleja la revisión del 2026-10-29 sin borrar el historial previo.

**Verification:**
- [ ] Focused tests: `rg -n 'api-v1.1|f572f9c|contracts' docs/README.md` devuelve las referencias nuevas.
- [ ] Build or broader check: `git diff --check` termina con código 0; comprobación manual de enlaces relativos desde `docs/README.md`.
- [ ] Manual check: una persona entiende desde `docs/README.md` qué es `docs/contracts/`, quién la genera y que no debe editarla.

**Dependencies:** None

**Files likely touched:**
- `docs/README.md`

**Estimated scope:** S

## T02: Eliminar `docs/contracts/README.md` y confirmar enlaces rotos

**Status:** Done

**Evidence (2026-10-29):** `docs/contracts/README.md` eliminado; quedan los siete artefactos de `api-v1/`. Búsqueda de referencias rotas sin coincidencias en `docs/`, `AGENTS.md` ni `tasks/` (las menciones restantes son el propio plan y la guía fija `nightingale-api.md`, cuya copia no se edita; la ubicación real queda registrada en `docs/README.md`). `git diff --check` con código 0.

**Description:** Eliminar `docs/contracts/README.md`, que pertenece al repositorio de API y cuyas instrucciones no aplican en el escritorio (rutas `../docs/rust-api-guide.md`, script `test-support/contracts/check.py`, destino de copia `docs/nightingale-api/`). Comprobar que ningún archivo del repositorio enlaza a ese README ni a las rutas inexistentes que mencionaba, y corregir cualquier referencia propia del escritorio si aparece.

**Acceptance criteria:**
- [ ] `docs/contracts/README.md` ya no existe en el árbol.
- [ ] Ningún archivo de `docs/`, `AGENTS.md` ni `tasks/` enlaza a `docs/contracts/README.md`.
- [ ] Los enlaces entre los artefactos restantes de `docs/contracts/api-v1/` (guía ↔ JSON, manifiesto) siguen válidos.

**Verification:**
- [ ] Focused tests: `rg -n 'contracts/README|rust-api-guide|api-v1\.md' docs AGENTS.md tasks` — sin coincidencias que esperen el README eliminado (la guía `nightingale-api.md` y `docs/specifications/nightingale-api.md` pueden coexistir; revisar que no se confundan).
- [ ] Build or broader check: `git diff --check`; `ls docs/contracts/api-v1/` muestra los siete artefactos restantes.
- [ ] Manual check: abrir `docs/contracts/api-v1/nightingale-api.md` y confirmar que sus enlaces relativos a los JSON resuelven.

**Dependencies:** None

**Files likely touched:**
- `docs/contracts/README.md` (eliminado)

**Estimated scope:** XS

## T03: Añadir a `AGENTS.md` la remisión al contrato API

**Status:** Done

**Evidence (2026-10-29):** `AGENTS.md` gana en «Source of truth» el párrafo del contrato (rol de estado vigente, artefactos generados y fijos, actualización solo por nueva exportación) y en «Documentation» el enlace a la copia del contrato. Resto de reglas intacto.

**Description:** Actualizar `AGENTS.md` para que cualquier trabajo que interactúe con Nightingale API remita primero a `docs/contracts/api-v1/`: qué contiene la carpeta, que sus JSON son generados y copia fija (no editar a mano), que una actualización del contrato se recibe copiando una nueva exportación completa desde `nightingale-api` y que el esquema físico PostgreSQL no vive en este repositorio.

**Acceptance criteria:**
- [ ] `AGENTS.md` menciona `docs/contracts/` y su papel de contrato vigente del API.
- [ ] `AGENTS.md` deja claro que los artefactos del contrato no se editan a mano y que el esquema físico PostgreSQL pertenece a `nightingale-api`.
- [ ] El resto de reglas de `AGENTS.md` no se reescribe; solo se añade o ajusta lo necesario.

**Verification:**
- [ ] Focused tests: `rg -n 'contracts' AGENTS.md` devuelve la remisión nueva.
- [ ] Build or broader check: `git diff --check`; lectura completa del bloque modificado sin contradicciones con las reglas existentes.
- [ ] Manual check: un colaborador nuevo que lea `AGENTS.md` sabe dónde mirar antes de escribir código que llame al API.

**Dependencies:** None

**Files likely touched:**
- `AGENTS.md`

**Estimated scope:** XS

## C01: Después de T01–T03

- [x] Registro del contrato consistente en `docs/README.md` y `AGENTS.md`.
- [x] Sin referencias rotas a `docs/contracts/README.md`.
- [x] Revisión del usuario completada (aprobación del plan, 2026-10-29).

## T04: Actualizar `docs/master-plan.md` con el API implementado

**Status:** Done

**Evidence (2026-10-29):** línea de revisión 2026-10-29; §3 con párrafo de API implementada y copia fija; §5.2 con anotaciones de contrato v1 en «Identidad y acceso» y «Duración fija o variable de citas»; §8.3 con regla de copia fija; §29 punto 1 marcado como cerrado en el repositorio de API. Verificación: `grep` de `api-v1.1`/`docs/contracts` y `git diff --check` con código 0.

**Description:** Anotar en el Master Plan el hecho verificable de que Nightingale API está implementada y exportada, sin reabrir decisiones ni ordenar trabajo nuevo: referencia a `docs/contracts/api-v1/` como contrato vigente; en §5.2 y §29, anotar que el contrato v1 fija las formas HTTP de sincronización, los intervalos de cita (inicio/fin con duración de referencia) y el flujo de identidad de dispositivos, manteniendo como abiertos lo que el contrato no cierra (proveedor OIDC concreto, cifrado SQLite, cloud, cola, RPO/RTO). Dejar explícito que el consumo del contrato por el escritorio sigue pendiente.

**Acceptance criteria:**
- [ ] El PMD remite a `docs/contracts/api-v1/` y distingue «API implementada en su repositorio» de «integración del consumidor pendiente».
- [ ] Las decisiones que el contrato v1 fija aparecen anotadas; las que siguen abiertas conservan su estado.
- [ ] No se añaden tareas, backlog ni fechas al PMD.

**Verification:**
- [ ] Focused tests: `rg -n 'api-v1.1|contracts' docs/master-plan.md` devuelve las anotaciones.
- [ ] Build or broader check: `git diff --check`; comparar §5.2 y §29 contra `docs/contracts/api-v1/capabilities.json` y `nightingale-api.md` para no afirmar de más.
- [ ] Manual check: el §30 «Resumen normativo» sigue siendo fiel tras el cambio.

**Dependencies:** T01

**Files likely touched:**
- `docs/master-plan.md`

**Estimated scope:** M

## T05: Actualizar `docs/technical-architecture.md` con el API implementado

**Status:** Done

**Evidence (2026-10-29):** línea de revisión 2026-10-29; §7.3 con estado implementado y límite del consumo; §9.1 y §14 con formas HTTP fijadas por el contrato v1; §10.2 con intervalos de cita del contrato; §23 etapa 1 y §24 con el contrato de sincronización cerrado. Verificación: `grep` de `docs/contracts`/`api-v1.1` y `git diff --check` con código 0.

**Description:** Anotar en el DAT que las formas HTTP del contrato de sincronización ya están definidas y exportadas (§14 y §24 dejan de presentarlas como «por acordar»), que las citas usan intervalos inicio/fin conforme al contrato v1 (§15.3), y que Nightingale API está implementada con su evidencia registrada en su repositorio (144 pruebas, migración `20261028_01`, 28 tablas, con los límites que el propio contrato declara). Mantener la distinción entre API implementada y consumo del escritorio sin verificación de extremo a extremo.

**Acceptance criteria:**
- [ ] El DAT remite a `docs/contracts/api-v1/` donde antes decía que las formas HTTP se acordarían después.
- [ ] §15.3 y las decisiones pendientes (§24) reflejan lo que el contrato v1 fija sin contradecir el PMD.
- [ ] Las cifras de evidencia citadas coinciden con las registradas en `docs/contracts/api-v1/nightingale-api.md` §§3 y 12.

**Verification:**
- [ ] Focused tests: `rg -n 'contracts' docs/technical-architecture.md` devuelve las remisiones.
- [ ] Build or broader check: `git diff --check`; cotejo de cifras y estados contra la guía del contrato.
- [ ] Manual check: §23 (etapas) y §26 (criterios de aceptación) no prometen como hecho lo que sigue pendiente.

**Dependencies:** T01

**Files likely touched:**
- `docs/technical-architecture.md`

**Estimated scope:** M

## T06: Cerrar P-20261021-01 y P-20261022-01 como Completed

**Status:** Done

**Evidence (2026-10-29):** ambos `plan.md` con `Status: Completed` y nota de cierre fechada; P-20261021-01 con C01 marcada como revisada por el usuario; P-20261022-01 con nota de que T01–T04/C01/C02 quedan cerradas sin ejecución por decisión expresa (estado `Todo` conservado). Verificación: `grep 'Status'` en ambos planes y `git diff --check` con código 0.

**Description:** Actualizar el estado de los dos planes que no aparecen como `Completed` con constancia de la decisión del usuario del 2026-10-29. En P-20261021-01: marcar C01 como revisado por el usuario y el plan como `Completed`. En P-20261022-01: marcar el plan como `Completed` con nota de que T01–T04 quedan cerradas sin ejecución por decisión expresa (T05, el prototipo, ya está Done), sin reescribir sus criterios ni borrar historial. No se tocan los planes ya `Completed` ni sus evidencias.

**Acceptance criteria:**
- [ ] `tasks/P-20261021-01-three-project-architecture/plan.md` y `todo.md` registran C01 completado y estado `Completed`.
- [ ] `tasks/P-20261022-01-screen-flows/plan.md` y `todo.md` registran estado `Completed` con nota de cierre sin ejecución de T01–T04.
- [ ] Ningún otro plan o tarea histórica se modifica.

**Verification:**
- [ ] Focused tests: `rg -n 'Status' tasks/P-20261021-01-three-project-architecture/plan.md tasks/P-20261022-01-screen-flows/plan.md` muestra `Completed`.
- [ ] Build or broader check: `git diff --check`; `rg -n '\*\*Status:\*\* (Draft|In progress)' tasks/` sin coincidencias tras el cierre.
- [ ] Manual check: los planes cerrados conservan sus evidencias y la nota histórica es legible.

**Dependencies:** T04, T05

**Files likely touched:**
- `tasks/P-20261021-01-three-project-architecture/plan.md`
- `tasks/P-20261021-01-three-project-architecture/todo.md`
- `tasks/P-20261022-01-screen-flows/plan.md`
- `tasks/P-20261022-01-screen-flows/todo.md`

**Estimated scope:** S

## C02: Después de T04–T06

- [x] Master Plan y DAT distinguen API implementada de consumo del escritorio pendiente.
- [x] Decisiones fijadas por el contrato v1 anotadas sin reabrir decisiones abiertas.
- [x] Planes cerrados con constancia de la decisión del 2026-10-29.
- [x] Revisión del usuario completada (aceptada el 2026-10-29; los commits quedan a cargo del usuario).
