# P-20261021-01-three-project-architecture: Documentar los tres proyectos

**Status:** In progress
**Created:** 2026-10-21
**Updated:** 2026-10-21
**Task list:** [todo.md](todo.md)

## Overview

Actualizar solo documentación para separar Nightingale, Nightingale Patient Hub y Nightingale API. Conservar la operación local Rust/SQLite y sustituir el acceso directo del escritorio a PostgreSQL por una API transaccional. Añadir un diagrama y especificaciones breves de los dos proyectos externos.

El usuario pidió esta actualización y revisó el alcance en la conversación. Después confirmó que el Hub llama a Nightingale API para guardar cambios de citas y que recordatorios, integración con n8n y analítica pertenecen a Nightingale API como módulos y workers. Esa autorización cubre esta ejecución documental; no autoriza implementación de producto, creación de repositorios, publicación ni despliegue.

## Source of Truth

- Solicitud y respuestas del usuario del 2026-10-21, posteriores al ADR de acceso directo.
- `AGENTS.md`, PMD §§4–5, 8–14, 17, 19, 23–25 y 29; DAT §§6–14, 17 y 24–26; SRS §§7–13.
- `tasks/README.md` y la habilidad `.agents/planning-task-breakdown/SKILL.md`.
- [Plan de la base de escritorio](../P-20261019-01-desktop-foundation/plan.md), completado: conservar su historial.
- Código actual: diagnóstico Tauri y metadatos SQLite. Ningún servicio o función de negocio se declara implementado por esta revisión.

## Architecture Decisions

- Propietario de los casos de uso locales: escritorio Rust; datos locales: SQLite, vistas recibidas y operaciones pendientes. Interfaz: comandos Tauri específicos.
- El trabajo local usa datos ya guardados y conserva cambios tras reiniciar sin red. Una caída de API impide confirmar cambios compartidos, no guardar operaciones locales permitidas.
- Nightingale API posee la aceptación remota: permisos, versiones, idempotencia, transacciones, historial y Change Log. PostgreSQL operativo queda privado; el escritorio no recibe sus credenciales.
- Patient Hub posee sus accesos y reglas del paciente. Llama a Nightingale API para consultar y modificar el estado compartido de citas; no escribe directamente en PostgreSQL operativo.
- La API procesa recordatorios y analítica en workers separados del recorrido transaccional. n8n sigue siendo una herramienta externa.
- Contratos HTTP y migraciones operativas tendrán propietario en Nightingale API; los del Hub, en Patient Hub. Las especificaciones iniciales viven aquí hasta su traslado explícito a los repositorios futuros, sin dos fuentes editables.
- Esta revisión no fija proveedor, rutas HTTP finales, librería web del Hub, duración de sesiones, política temporal de revocación ni objetivos numéricos sin evidencia.

## Dependency Order

T01 fija propiedad y contratos conceptuales → T02 armoniza documentos normativos → T03 actualiza reglas de trabajo y verifica el conjunto → C01 revisión de la entrega.

## Task List

- [x] T01: Registrar la decisión y las especificaciones externas.
- [x] T02: Actualizar la arquitectura normativa.
- [x] T03: Alinear las guías de trabajo.

### C01: Revisión documental final

- [x] Enlaces y referencias vigentes comprobados; bloques Mermaid revisados estructuralmente, sin render visual.
- [x] `git diff --check` pasa; código, dependencias y planes cerrados no cambian durante esta revisión.
- [ ] El usuario revisa el diagrama, la propiedad de datos y las dos especificaciones.

## Risks and Mitigations

| Riesgo | Impacto | Tratamiento |
|---|---|---|
| Mantener reglas activas de acceso directo | Alto | Buscar contradicciones en todos los documentos activos y marcar el ADR anterior como sustituido. |
| Duplicar reglas o escrituras de citas entre API y Hub | Alto | Separar política del paciente de aceptación transaccional; un único escritor operativo. |
| Prometer seguridad o rapidez no verificadas | Alto | Documentar límites, pruebas futuras y métricas pendientes; sin afirmar resultados de ejecución. |
| Reemplazar cambios previos del usuario | Alto | Editar los archivos actuales, conservar el historial y comprobar el alcance del diff. |

## Open Questions

Ninguna que bloquee esta revisión. Las decisiones de implementación quedan enumeradas en cada especificación. No se ejecutan pruebas de red, rendimiento, revocación o recuperación porque los servicios aún no existen. No corresponde compilar el producto por un cambio solo documental.

## Evidence

Entrega documental preparada el 2026-10-21. Verificación puntual: 15 archivos Markdown, 62 enlaces/destinos y anclas locales, nueve bloques Mermaid revisados estructuralmente; el diagrama general coincide en DAT y ADR. No se ha ejecutado un render visual de Mermaid. `git diff --check` pasa. La revisión Git confirma sin cambios respecto a HEAD el código, manifiestos, lockfiles y plan de la base, rutas que estaban limpias al inicio. Las demás ediciones previas del usuario se conservaron como base del trabajo. T01–T03 están terminadas; C01 sigue pendiente solo de revisión del usuario, sin declarar pruebas de producto o servicios nuevos.
