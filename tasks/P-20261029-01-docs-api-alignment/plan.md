# P-20261029-01-docs-api-alignment: Alinear la documentación con el contrato API v1

**Status:** Completed
**Created:** 2026-10-29
**Updated:** 2026-10-29
**Task list:** [todo.md](todo.md)

## Overview

Poner la documentación del escritorio al día con el estado real de Nightingale API: el contrato v1 (revisión `api-v1.1`) está exportado y copiado en `docs/contracts/api-v1/`, y la API está implementada en su repositorio propio. El resultado es que cualquier colaborador o agente que abra este repositorio encuentre: (1) `docs/contracts/` registrado como el estado vigente del API, (2) `AGENTS.md` remitiendo a esa carpeta para cualquier trabajo que interactúe con el API, (3) Master Plan y arquitectura técnica reflejando que el API existe y que sus formas HTTP ya están fijadas, y (4) los planes abiertos cerrados con constancia de la decisión del usuario.

Solo se tocan archivos de documentación y de planificación. No se modifica código de producto, no se editan los JSON generados del contrato y no se reabren planes cerrados.

El usuario autorizó este trabajo y sus alcances en la conversación del 2026-10-29: registrar el contrato en `docs/README.md`, eliminar el `README.md` de `docs/contracts/`, actualizar `AGENTS.md`, `docs/master-plan.md` y `docs/technical-architecture.md`, dejar eliminado `docs/database-schema.md` y dar por completados los planes que no aparecen como `Completed`.

## Source of Truth

- `docs/contracts/api-v1/nightingale-api.md` (corte 2026-10-29): propósito, estado, decisiones, modelo de datos, endpoints y flujos del API.
- `docs/contracts/api-v1/manifest.json`: huellas y conteos del contrato (46 rutas, 70 esquemas, 42 acciones).
- `docs/master-plan.md` (PMD 0.3, revisión 2026-10-23), `docs/technical-architecture.md` (DAT 0.3, revisión 2026-10-23), `docs/srs.md` (0.5): normativa vigente del escritorio.
- `docs/README.md`: registro de importaciones y versiones.
- `AGENTS.md`: reglas del proyecto.
- `tasks/README.md`: convenciones de planes, tareas y checkpoints.
- `tasks/P-20261021-01-three-project-architecture/` (In progress, C01 pendiente de revisión) y `tasks/P-20261022-01-screen-flows/` (Draft): planes que el usuario cierra con esta decisión.
- `tasks/P-20261023-01-database-schema/`: plan histórico cuyo documento fuente (`docs/database-schema.md`) el usuario decide eliminar de este repositorio.

## Architecture Decisions

- `docs/contracts/` se registra como **copia de consulta fija**, no como fuente editable: los JSON se regeneran en `nightingale-api` y se copian completos; no se corrigen a mano. El `README.md` que acompañaba la copia se elimina porque sus instrucciones (rutas `../docs/rust-api-guide.md`, script `test-support/contracts/check.py`) pertenecen al repositorio de API y no aplican aquí.
- `AGENTS.md` gana una regla de remisión: cualquier cambio que interactúe con Nightingale API deberá leer `docs/contracts/api-v1/` primero y respetar que sus artefactos son generados.
- Master Plan y DAT no se reescriben: se anotan las decisiones que el contrato v1 ya fija (formas HTTP, intervalos de cita, flujo de dispositivos) y se mantiene la distinción entre «API implementada en su repositorio» y «consumo del escritorio aún pendiente». Esto respeta la regla de que la especificación solo cambia cuando el usuario lo pide, como en este caso.
- `docs/database-schema.md` queda eliminado de este repositorio: el escritorio no interactúa con PostgreSQL y la fuente editable del modelo físico vive en `nightingale-api/docs/database-schema.md` (según el propio contrato, §5.6). El registro de esta decisión va en `docs/README.md` y en este plan, sin editar el plan histórico P-20261023-01.
- Los planes P-20261021-01 y P-20261022-01 se cierran como `Completed` con nota de la decisión del usuario del 2026-10-29. En P-20261022-01 se deja constancia de que T01–T04 (validación de recorridos) quedan cerradas sin ejecución por decisión expresa, conservando T05 (prototipo) como la única tarea ejecutada.

## Dependency Order

T01 → T02 → T03 (registro del contrato, limpieza de la copia, regla en AGENTS.md) son independientes entre sí en contenido, pero se ordenan así porque T01 define cómo se nombra la carpeta en el resto. C01 separa esta fase de la de documentos normativos. T04 y T05 son independientes entre sí; T06 cierra planes y depende de que la documentación ya refleje el estado final.

## Task List

### Phase 1: Registro del contrato y reglas de remisión
- [x] T01: Registrar `docs/contracts/` en `docs/README.md` como estado vigente del API
- [x] T02: Eliminar `docs/contracts/README.md` y confirmar enlaces rotos
- [x] T03: Añadir a `AGENTS.md` la remisión al contrato API

### C01: Después de T01–T03
- [x] Registro del contrato consistente en README y AGENTS.md
- [x] Sin referencias rotas a `docs/contracts/README.md`
- [x] Revisión del usuario completada (aprobación del plan, 2026-10-29)

### Phase 2: Documentos normativos y cierre de planes
- [x] T04: Actualizar `docs/master-plan.md` con el API implementado
- [x] T05: Actualizar `docs/technical-architecture.md` con el API implementado
- [x] T06: Cerrar P-20261021-01 y P-20261022-01 como Completed

### C02: Después de T04–T06
- [x] Master Plan y DAT distinguen API implementada de consumo del escritorio pendiente
- [x] Decisiones fijadas por el contrato v1 anotadas sin reabrir decisiones abiertas
- [x] Planes cerrados con constancia de la decisión del 2026-10-29
- [x] Revisión del usuario completada (aceptada el 2026-10-29; commits a cargo del usuario)

## Risks and Mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| Afirmar en PMD/DAT más de lo que el contrato acredita (el consumo Rust y la producción no están verificados) | Alto | Usar el lenguaje del propio contrato: «API implementada; integración del consumidor pendiente; sin acreditación de despliegue». |
| Editar los JSON generados o la guía `nightingale-api.md` como si fueran fuente del escritorio | Alto | Regla explícita en AGENTS.md y README: copia fija, no editable; actualización solo copiando una nueva exportación de `nightingale-api`. |
| Cerrar P-20261022-01 borrando la evidencia de que T01–T04 no se ejecutaron | Medio | Nota de cierre en `plan.md` y `todo.md` que conserva el estado histórico de las tareas. |
| `docs/database-schema.md` reaparece en un commit futuro | Bajo | Registro en `docs/README.md` de que fue eliminado por decisión del 2026-10-29 y de dónde vive la fuente física. |

## Open Questions

- ¿Se commitea `docs/contracts/` (hoy sin seguimiento en Git) como parte de este plan o lo dejas para un commit tuyo? Por defecto el plan **no** crea commits; solo deja el árbol listo.
