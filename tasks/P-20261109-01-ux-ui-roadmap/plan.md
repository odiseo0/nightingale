# P-20261109-01-ux-ui-roadmap: Pacientes y roadmap UX/UI

**Status:** Completed (2026-11-09, aceptación del usuario: "para este y para todos los demás puedes aprobar y ejecutar"; NOT CHECKED registrados)
**Created:** 2026-11-09
**Updated:** 2026-11-09
**Task list:** [todo.md](todo.md)

## Overview

Definir el siguiente objetivo UX/UI y una secuencia completa para los objetivos pendientes de `docs/design-master-plan.md`, sin duplicar los planes ya ejecutados ni convertir dependencias externas en pantallas ficticias.

**Objetivo inmediato:** S05/S06 · Pacientes en modo lectura: directorio, búsqueda, ficha administrativa y acceso a Nueva cita con paciente preseleccionado.

**Dependencia actual:** P06 `appointment-creation-ui` tiene T01–T04 marcadas, pero C02 conserva revisión del usuario pendiente. El objetivo inmediato no debe comenzar hasta cerrar esa revisión. Este roadmap permanece Draft y no autoriza implementación.

Los R01–R20 son objetivos de roadmap, no tareas aprobadas ni nuevos IDs de planes operativos. Cada uno recibirá su propio `P-YYYYMMDD-NN-.../plan.md` y `todo.md` antes de ejecutarse.

## Estado de partida

| Trabajo | Estado observado | Alcance que no se duplicará |
|---|---|---|
| P04 interface-master-plan | Completed | Fuente UI/UX aprobada |
| P05 agenda-ui-foundation | Approved, tareas ejecutadas; deuda ambiental aceptada | Tokens, shell, Agenda Día y detalle en lectura |
| P06 appointment-creation-ui | Approved/en ejecución; T01–T04 marcadas, revisión C02 pendiente | Nueva cita con paciente existente, disponibilidad y guardado local |
| Núcleo técnico existente | Implementado en planes técnicos | Rust, SQLite, IPC, sincronización, pacientes, servicios, agenda y citas |

No se reabrirán estos planes ni se reemplazará su historial.

## Source of Truth

- `docs/design-master-plan.md`, especialmente §§7–15, 21–33.
- `DESIGN.md`, `DESIGN-RULES.md` y `.agents/laws-of-ux/SKILL.md`.
- `docs/master-plan.md` y `docs/srs.md` RF-01–RF-16.
- `docs/contracts/api-v1/nightingale-api.md`.
- `docs/technical/components/patients.md`, `appointments.md`, `scheduling.md`, `account.md`, `services.md`, `identity.md` y `sync.md`.
- `tasks/P-20261108-05-agenda-ui-foundation/` y `tasks/P-20261108-06-appointment-creation-ui/`.
- `src/shared/api/patients.ts`, `appointments.ts`, `scheduling.ts`, `account.ts`, `services.ts`, `identity.ts` y `sync.ts`.
- Comandos Rust `patients_*` ya registrados; no se presupone ninguna pantalla de pacientes terminada.

## Objetivo inmediato: Pacientes en lectura

### Resultado de usuario

El personal podrá encontrar una ficha autorizada, revisar sus datos administrativos, documentos, responsables y citas, y abrir Nueva cita con ese paciente seleccionado sin perder búsqueda, página, filtros ni foco al regresar.

### Rutas propuestas

- `/pacientes`: directorio S05.
- `/paciente/:patientId`: ficha S06.
- `/cita/nueva?paciente=:patientId`: editor P06 con paciente preseleccionado.

La navegación principal mostrará Pacientes solo cuando el directorio sea funcional. No mostrará Registrar paciente, Editar, Desactivar ni Fusionar en esta etapa.

### Contratos y propietarios

- Rust `patients`: `patients_list_patients`, `patients_get_patient`, `patients_list_patient_identifiers`, `patients_list_patient_contacts`.
- Rust `appointments`: `appointments_list_appointments` con `patientId`, periodo y paginación.
- Se ampliará `src/shared/api/patients.ts` con tipos/clientes de lectura para perfil e identificadores, reutilizando `EntityRow` y `visibleOf`; no se añadirán comandos ni SQL/HTTP.
- SQLite conserva perfiles, documentos, relaciones, citas, aceptados/provisionales y operaciones locales. La UI distinguirá todas esas dimensiones.
- Offline: las consultas usan IPC local cuando hay sesión y datos. Sin sesión/datos no se presentará un directorio vacío global.
- Sync: solo se presentan propuestas y operaciones existentes; no se escribe ni reintenta nada.

### Directorio S05

- Búsqueda local con `q`, filtros `status` y `completionStatus`, y paginación real mediante `page`/`shows`.
- Columnas mínimas: nombre, teléfono, completitud y estado; no se mezclarán `draft`/`complete` con `active`/`inactive`.
- La próxima cita no se cargará mediante una consulta por cada fila. Se consultará solo en la ficha seleccionada.
- Estados: carga, refresh, vacío real, sin coincidencias, error, sesión no lista, datos provisionales y fin de página.
- Nombres largos, valores ausentes, navegación por teclado, scroll localizado y retorno de contexto/foco.

### Ficha S06

- Perfil con nombre, nacimiento/teléfono cuando estén permitidos, identificationBasis, completitud y estado.
- Documentos mediante `patients_list_patient_identifiers`, conservando texto, país/tipo y estado de verificación sin interpretarlos como prueba clínica.
- Responsables mediante `patients_list_patient_contacts`, con relación, teléfono, vigencia y estado; sin inferir tutela.
- Citas del paciente con `listAppointments`, próximas e historial paginado, estado de cita y provisionalidad separados.
- Sin datos clínicos, edición, eliminación, fusión ni cambios de estado.

### Continuidad con Nueva cita

- La acción Nueva cita usará `/cita/nueva?paciente=<id>` y el editor existente preseleccionará al paciente si está en el contexto autorizado.
- Si el paciente ya no está disponible, el editor mostrará el estado sin seleccionar otro silenciosamente.
- Al volver de la ficha se restaurarán consulta, filtros, página, selección y foco.
- La acción no registrará pacientes ni modificará el editor fuera de la preselección y el retorno contextual.

## Dependency Order del objetivo inmediato

```text
Revisión C02 de P06
  → T01: directorio y búsqueda → C01
  → T02: ficha administrativa y citas
  → T03: preselección, retorno y verificación integral → C02
```

## Task List del objetivo inmediato

### Phase 1: Directorio
- [x] T01: Implementar clientes de lectura, ruta y directorio de pacientes.

### C01: Directorio real
- [x] `pnpm quality` en verde; búsqueda/filtros/paginación con IPC real, responsive y teclado.
- [~] Revisión del usuario: ejecución continua autorizada; verificación técnica registrada.

### Phase 2: Ficha y continuidad
- [x] T02: Implementar perfil, documentos, responsables y citas.
- [x] T03: Preseleccionar paciente en Nueva cita y verificar retorno/estados.

### C02: Recorrido Pacientes
- [x] `pnpm quality` en verde; directorio→ficha→Nueva cita→retorno verificado con IPC real y sesión fixture.
- [x] Estados, retorno contextual, preselección, responsive y teclado verificados; NOT CHECKED registrados (lector de pantalla, menor sin fixture, corte de red).
- [~] Revisión del usuario pendiente de aceptación para cerrar el plan.

## Roadmap completo de objetivos UX/UI

Los objetivos siguientes cubren todo lo pendiente del maestro. El orden es una recomendación por dependencia y riesgo, no una autorización ni una fecha de entrega.

| ID | Objetivo y alcance principal | Dependencia/desbloqueo |
|---|---|---|
| R01 | **Patient management UI:** registro/edición de perfiles propios y menores, documentos, contactos, relaciones, completitud/estado y alta desde Nueva cita conservando borrador | Pacientes en lectura + P06 |
| R02 | **Appointment reschedule UI:** horario actual/propuesto, igual duración, disponibilidad, invalidación de confirmación y conflictos | P06; componentes de disponibilidad |
| R03 | **Appointment lifecycle UI:** confirmar, cancelar con fricción, atención y ausencia según momento/permiso; historia actualizada | P06; puede seguir a R02 |
| R04 | **Agenda views UI:** Semana y Lista, periodo, búsqueda, paginación, bloqueos/huecos y retorno al mismo detalle | P05/P06; disponibilidad existente |
| R05 | **Sync review UI:** pendientes, unknown, reintentos, conflictos, rechazos, comparación y recuperación sin forzar sobrescritura | Al menos una mutación UI; se apoya en R02/R03 para casos completos |
| R06 | **Access and onboarding UI:** acceso, dispositivo, selección de cuenta y preparación inicial con estados reales | Base shell + identidad existente |
| R07 | **Account settings UI:** datos de cuenta, zona/lugar, política de ausencia, preferencias locales y solo lectura por permiso | Base shell + account; sin dark mode inventado |
| R08 | **Services settings UI:** catálogo, ofertas, duraciones, versiones, activo/inactivo y efecto en citas anteriores | R07 + services |
| R09 | **Schedule settings UI:** reglas, excepciones, bloqueos, vigencias, impacto en citas y permisos globales | R08 + scheduling |
| R10 | **Requests UI S07:** lista/detalle, cita vigente, elección de horario, conciliación y fallos parciales | Contrato Hub/API y ciclo de solicitudes; reprogramación R02 |
| R11 | **Waitlist UI S08:** entradas, compatibilidad, ofertas, vencimiento y asignación por personal | Dominio/contrato de lista de espera, canal y prioridad definidos |
| R12 | **Communications UI S09:** programación, envío, entrega, respuesta, antigüedad, fallos y reintento autorizado | Workers/contrato de comunicaciones; no chat libre |
| R13 | **Metrics UI S10:** periodo, valores, denominadores, definiciones, antigüedad y alternativa tabular | Servicio/contrato analítico; pendientes no se mezclan con aceptados |
| R14 | **Team and devices settings UI:** miembros, permisos, invitaciones, dispositivos y revocación con resultado conocido | Contrato remoto/permisos; operaciones online autorizadas |
| R15 | **Communication settings UI:** recordatorios, cancelaciones y enlaces según política contratada | R12 + políticas de plazos/verificación definidas |
| R16 | **Shell discovery UI:** búsqueda global/Cmd+K, navegación completa, ayuda y destinos solo cuando existan | Superficies principales implementadas; ningún placeholder |
| R17 | **Diagnostics UI S13:** versión, dispositivo, adaptadores y estado técnico sin secretos | Base existente en Desarrollo; baja prioridad de producto |
| R18 | **Patient Hub UX handoff S14:** especificación móvil de consulta/confirmación/cancelación/solicitud/errores para el repositorio propietario | Proyecto Hub y contrato propio; no implementación en este repo |
| R19 | **Theme and semantic expansion:** dark/system y colores de error/advertencia/crítico con contraste y significado | Decisión expresa del usuario y cambio de DESIGN; no incluido por defecto |
| R20 | **UX validation and accessibility:** recorridos con médico/asistente/pacientes, lector de pantalla, datos difíciles, 200 %, offline y deuda NOT CHECKED acumulada | Superficies objetivo implementadas; evidencia humana/ambiental disponible |

### Dependencia recomendada

```text
P06 → Pacientes lectura → R01 → R02 → R03 → R04 → R05
                         ├─→ R06 → R07 → R08 → R09
R10/R11/R12/R13/R14/R15: solo cuando sus contratos estén disponibles
R16/R17: cuando exista contenido real suficiente
R18: repositorio Patient Hub
R19: solo por decisión expresa
R20: revisión transversal al final de cada bloque y cierre del roadmap
```

R06–R09 y R04 pueden adelantarse si sus dependencias están listas, pero no se ejecutarán en paralelo con contratos compartidos sin un plan que declare propietarios e integración.

## Reglas comunes para todos los planes R01–R20

Cada plan operativo deberá incluir:

- Usuario/tarea, superficie Sxx, RF y resultado observable.
- Propietario nativo, caso de uso Rust, datos SQLite, IPC/DTO y contratos externos.
- Qué funciona offline, persistencia tras reinicio y qué requiere conexión.
- Dominio/solicitud/sync/comunicación/permiso/cobertura/formulario/vista separados.
- Navegación, entrada/salida, contexto y foco.
- Componentes mínimos y tokens Anek Latin/Aileron/Phosphor; sin catálogo anticipado.
- Matriz de estados, errores, permisos, datos difíciles, responsive y accesibilidad.
- Leyes UX relevantes y decisiones concretas, sin listar todas.
- `$minimal-code`, `$docs-first-framework-code`, `$planning-task-breakdown` si amplía alcance y `$code-review` cuando corresponda.
- `pnpm build`, `pnpm quality`, llamadas reales y revisión visual/manual con datos ficticios.
- NOT CHECKED explícitos; ninguna compilación sustituye el recorrido.
- Sin cambios de dominio no aprobados, Patient Hub en este repositorio, secretos ni éxito remoto ficticio.

## Risks and Mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| Convertir el roadmap en una única migración gigante | Alto | Planes pequeños independientes, aprobación y checkpoints por objetivo |
| Duplicar P05/P06 | Alto | Baseline explícita y prohibición de reabrir historial |
| Diseñar superficies sin contratos | Alto | R10–R15 bloqueados hasta tener propietario/contrato real |
| Ocultar deuda ambiental/accesibilidad | Alto | R20 y NOT CHECKED por plan; no se declara conformidad global |
| Perder contexto entre superficies | Alto | Preselección/retorno/contexto son acceptance criteria de cada recorrido |
| Implementar Patient Hub en este repo | Alto | R18 es handoff documental al proyecto propietario |

## Open Questions

- El orden R02/R04 y R06–R09 puede ajustarse sin cambiar el objetivo, siempre que sus dependencias estén cerradas.
- Solicitudes, lista de espera, comunicaciones, indicadores y políticas siguen bloqueados hasta que sus fuentes definan modelos y estados reales.
- La cobertura descargada por fecha requiere una fuente contractual para distinguirla con certeza; no se resolverá solo con UI.
- La validación con usuarios y lector de pantalla sigue pendiente hasta disponer de entorno/participantes; no se considera aprobada por este roadmap.

## Planes operativos derivados

Los 20 objetivos ya tienen plan y checklist propios. Su preparación no equivale a aprobación de implementación.

| Roadmap | Plan operativo | Estado |
|---|---|---|
| R01 | [P-20261109-02-patient-management-ui](../P-20261109-02-patient-management-ui/plan.md) | Completado (2026-11-09) |
| R02 | [P-20261109-03-appointment-reschedule-ui](../P-20261109-03-appointment-reschedule-ui/plan.md) | Completado (2026-11-09) |
| R03 | [P-20261109-04-appointment-lifecycle-ui](../P-20261109-04-appointment-lifecycle-ui/plan.md) | Completado (2026-11-09) |
| R04 | [P-20261109-05-agenda-views-ui](../P-20261109-05-agenda-views-ui/plan.md) | Completado (2026-11-09) |
| R05 | [P-20261109-06-sync-review-ui](../P-20261109-06-sync-review-ui/plan.md) | Completado (2026-11-09) |
| R06 | [P-20261109-07-access-onboarding-ui](../P-20261109-07-access-onboarding-ui/plan.md) | Completado (2026-11-09) |
| R07 | [P-20261109-08-account-settings-ui](../P-20261109-08-account-settings-ui/plan.md) | Completado (2026-11-09) |
| R08 | [P-20261109-09-services-settings-ui](../P-20261109-09-services-settings-ui/plan.md) | Completado (2026-11-09) |
| R09 | [P-20261109-10-schedule-settings-ui](../P-20261109-10-schedule-settings-ui/plan.md) | Completado (2026-11-09) |
| R10 | [P-20261109-11-requests-ui](../P-20261109-11-requests-ui/plan.md) | Blocked |
| R11 | [P-20261109-12-waitlist-ui](../P-20261109-12-waitlist-ui/plan.md) | Blocked |
| R12 | [P-20261109-13-communications-ui](../P-20261109-13-communications-ui/plan.md) | Blocked |
| R13 | [P-20261109-14-metrics-ui](../P-20261109-14-metrics-ui/plan.md) | Blocked |
| R14 | [P-20261109-15-team-devices-settings-ui](../P-20261109-15-team-devices-settings-ui/plan.md) | Blocked |
| R15 | [P-20261109-16-communication-settings-ui](../P-20261109-16-communication-settings-ui/plan.md) | Blocked |
| R16 | [P-20261109-17-shell-discovery-ui](../P-20261109-17-shell-discovery-ui/plan.md) | Completado (2026-11-09) |
| R17 | [P-20261109-18-diagnostics-ui](../P-20261109-18-diagnostics-ui/plan.md) | Completado (2026-11-09) |
| R18 | [P-20261109-19-patient-hub-ux-handoff](../P-20261109-19-patient-hub-ux-handoff/plan.md) | Blocked |
| R19 | [P-20261109-20-theme-semantics-ui](../P-20261109-20-theme-semantics-ui/plan.md) | Blocked |
| R20 | [P-20261109-21-ux-validation](../P-20261109-21-ux-validation/plan.md) | Completado (2026-11-09, evidencia técnica; pruebas humanas NOT CHECKED) |

**Estado final del roadmap (2026-11-09):** ejecutados R01–R09, R16, R17 y R20 con verificación real por IPC. Permanecen Blocked por dependencia de contrato/proyecto externo: R10–R15 (solicitudes, lista de espera, comunicaciones, métricas, equipo/dispositivos, políticas de comunicación), R18 (Patient Hub, otro repositorio) y R19 (tema/semántica, decisión expresa pendiente). La evidencia transversal y la deuda quedan en `P-20261109-21-ux-validation/evidence.md`.
