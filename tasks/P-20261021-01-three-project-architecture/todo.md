# Task List: P-20261021-01-three-project-architecture

**Plan:** [plan.md](plan.md)
**Updated:** 2026-10-21

## T01: Registrar la decisión y las especificaciones externas

**Status:** Done

**Description:** Registrar el acuerdo de tres proyectos y definir brevemente el contrato y los límites de Nightingale API y Nightingale Patient Hub. Marcar el ADR de acceso directo como sustituido sin borrar su contenido histórico.

**Acceptance criteria:**
- [x] Las especificaciones separan operación local, aceptación compartida y capacidades del paciente.
- [x] El Hub llama a Nightingale API; los workers pertenecen a API y no bloquean la sincronización.
- [x] La decisión incluye el diagrama, límites de seguridad y asuntos pendientes, sin afirmar implementación.

**Verification:**
- [x] Focused tests: enlaces comprobados y diagrama revisado estructuralmente, sin render visual; estado histórico anotado en ambos ADR anteriores.
- [x] Build or broader check: `git diff --check`; compilación de producto no aplicable.
- [x] Manual check: revisión textual de creación offline, cancelación del Hub y reintento tras respuesta perdida; no ejecución de servicios.

**Dependencies:** None
**Files likely touched:** `docs/decisions/transactional-sync-three-projects.md`, `docs/decisions/native-desktop-cloud-boundaries.md`, `docs/decisions/desktop-foundation.md`, `docs/specifications/nightingale-api.md`, `docs/specifications/nightingale-patient-hub.md`.
**Estimated scope:** M

## T02: Actualizar la arquitectura normativa

**Status:** Done

**Description:** Actualizar PMD, DAT y SRS con la nueva aceptación remota, los tres proyectos y los flujos de sincronización y Hub. Mantener requisitos locales, permisos offline y alcance funcional.

**Acceptance criteria:**
- [x] Ninguna regla activa exige acceso directo del escritorio a PostgreSQL o prohíbe la API acordada.
- [x] Diagramas, flujos, propiedad, seguridad y compatibilidad usan la misma arquitectura.
- [x] Estados de negocio, sincronización y aceptación local/remota siguen separados.

**Verification:**
- [x] Focused tests: búsquedas de acceso directo y repositorios anteriores, con revisión de resultados; las reglas antiguas solo se conservan en historial marcado.
- [x] Build or broader check: enlaces locales y `git diff --check`; compilación no aplicable.
- [x] Manual check: flujos documentados contrastados con ADR y especificaciones; sin pruebas de ejecución.

**Dependencies:** T01
**Files likely touched:** `docs/master-plan.md`, `docs/technical-architecture.md`, `docs/srs.md`.
**Estimated scope:** M

## T03: Alinear las guías de trabajo

**Status:** Done

**Description:** Actualizar instrucciones y guías para que el equipo use los tres proyectos y no añada servicios cloud al repositorio del escritorio. Comprobar la entrega documental completa.

**Acceptance criteria:**
- [x] Reglas, contribución, README y desarrollo remiten al ADR vigente y a las especificaciones.
- [x] La base actual sigue descrita como diagnóstico y metadatos, sin requisitos cloud nuevos.
- [x] No cambia código, configuración de producto, dependencias ni planes históricos.

**Verification:**
- [x] Focused tests: referencias antiguas revisadas y 62 enlaces/destinos y anclas locales comprobados en 15 archivos Markdown.
- [x] Build or broader check: `git diff --check` pasa; alcance revisado con Git; producto y plan cerrado sin cambios. Compilación no aplicable.
- [x] Manual check: recorrido documental README → arquitectura → especificaciones y límites offline/de seguridad revisados.

**Dependencies:** T02
**Files likely touched:** `AGENTS.md`, `README.md`, `CONTRIBUTING.md`, `docs/operations/development.md`, `tasks/README.md`.
**Estimated scope:** M

## C01: Después de T01–T03

- [x] Verificación documental completa y alcance del cambio comprobado; Mermaid revisado estructuralmente, sin render visual.
- [x] Diagrama y especificaciones disponibles para revisión.
- [ ] Revisión del usuario completada.
