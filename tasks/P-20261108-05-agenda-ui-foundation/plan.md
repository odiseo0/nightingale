# P-20261108-05-agenda-ui-foundation: Base visual y Agenda diaria

**Status:** Completed (2026-11-08, aceptación del usuario con NOT CHECKED registrados)
**Created:** 2026-11-08
**Updated:** 2026-11-08
**Task list:** [todo.md](todo.md)

## Overview

Construir el primer recorrido de la interfaz aprobada: **fundamentos visuales + shell de escritorio + Agenda de día y detalle de cita en modo lectura**, usando datos ficticios y comandos Tauri existentes.

Este es el primer bloque recomendado por el Plan Maestro de Diseño porque combina una base reutilizable con la tarea principal del usuario. Evita tanto crear un catálogo abstracto de componentes como migrar toda la aplicación antes de validar un recorrido real.

**No hay autorización de implementación mientras el plan permanezca Draft.** La aprobación solo permitirá ejecutar T01–T04 con revisión en C01/C02; no incluye creación/edición de citas, otras superficies ni cambios de dominio.

## Conclusión de prioridad

Se construirá primero este recorrido porque:

- El maestro propone Agenda de hoy como inicio habitual (§§5–10).
- Shell, tokens, fuentes e iconos son dependencias compartidas de todas las superficies (§§16–19 y 29).
- `appointments_get_booking_context`, `appointments_list_appointments`, `appointments_get_appointment` y `appointments_get_history` ya permiten un recorrido real de lectura.
- Un vertical slice permite validar tipografía, densidad, estados, navegación, responsive y accesibilidad antes de escalar.
- El acceso técnico actual se conserva en Desarrollo; no se retrabaja S01 antes de entregar la tarea diaria.
- Nueva cita requiere formularios, disponibilidad y mutaciones más complejas; pertenece a una etapa posterior.

No se construirá primero un catálogo completo de primitives, Patient Hub, Semana/Lista, Semana, Indicadores, Pacientes ni formularios de escritura.

## Source of Truth

- `docs/design-master-plan.md` §§5–10, 16–21, 23–31 y 33.
- `DESIGN.md`: tokens, Anek Latin, Aileron, Phosphor, estados y accesibilidad.
- `DESIGN-RULES.md`: tarea, claridad, estados y aplicación obligatoria de Laws of UX.
- `.agents/laws-of-ux/SKILL.md`: selección contextual y verificación.
- `docs/master-plan.md` §§4, 9.7, 13, 14, 22 y 26–30.
- `docs/contracts/api-v1/nightingale-api.md` §§4–5: cita, historia, estados y permisos.
- `src/App.tsx`, `src/App.css`: esqueleto actual.
- `src/features/appointments/AgendaPanel.tsx`: flujo técnico existente.
- `src/shared/api/appointments.ts`: contexto, lista, detalle e historia.
- `src/shared/api/account.ts` e `identity.ts`: cuenta y acceso actuales.
- Referencias oficiales para dependencias propuestas:
  - https://github.com/phosphor-icons/react — paquete actual `@phosphor-icons/react`, imports tree-shaken/CSR y peso Regular.
  - https://reactrouter.com/ — React Router, baseline actual compatible con Node 24, Vite 8 y React 19.

## Architecture Decisions

### Alcance funcional y nativo

- Propietario nativo: componentes Rust `appointments`, `account` e `identity`.
- Casos de uso: consultar contexto de reserva, citas de un día, detalle e historia; consultar cuenta/estado de acceso.
- Datos locales: filas aceptadas/provisionales, pacientes del contexto, historia aceptada y operaciones locales ya almacenadas en SQLite.
- Interfaz pública: clientes existentes de `src/shared/api/`; no se añaden comandos, SQL, HTTP, migraciones ni escrituras.
- Sin red: las lecturas usan IPC local cuando sesión, permisos y datos ya existen. La ausencia de sesión/datos no se presenta como agenda vacía.
- Sync: solo se presentan estados provisionales/operaciones existentes. No se sincroniza, reintenta ni acepta nada en este plan.
- Excepción cloud: ninguna nueva. Patient Hub queda fuera.

### Recorrido y rutas iniciales

Rutas propuestas para aprobación con este plan, sin fijar las rutas de futuras superficies:

- `/agenda`: S02 Día, con detalle lateral cuando quepa.
- `/cita/:appointmentId`: S03 en superficie propia cuando el ancho o acceso directo lo requiera.
- `/desarrollo`: esqueleto técnico actual, acceso/sincronización y diagnóstico fuera del trabajo diario.

La navegación mostrará solo destinos funcionales. No incluirá Pacientes, Semana, Lista, Nueva cita, búsqueda global ni otros placeholders. Esas opciones aparecerán cuando sus planes las implementen. Durante T02, `/agenda` conservará temporalmente el panel técnico existente; T03 lo sustituirá y T04 añadirá `/cita/:appointmentId`, de modo que ningún checkpoint introduzca una superficie vacía.

### Diseño y componentes

- Tokens semánticos centralizados en `App.css`; no se copian valores hexadecimales en features.
- Anek Latin/Aileron locales con licencias y fallback; Phosphor Regular mediante `@phosphor-icons/react` e imports específicos compatibles con tree-shaking.
- React Router para rutas reales y contexto de retorno; la versión exacta quedará fijada en lockfile y se comprobará contra documentación oficial antes de código.
- Componentes mínimos por necesidad: navegación, botones, feedback, tabla y badges. No se crea una librería anticipada ni se adopta Radix por defecto.
- El prototipo sirve como referencia histórica de densidad/composición, no como código ni fuente tipográfica.

### Datos y estados

- `BookingContext.patients` se usará para resolver nombre/contacto sin mostrar `patientId` como identidad principal.
- Fechas/horas se formatearán en la zona de la cuenta; no se cortarán strings UTC como si fueran hora local.
- Las lecturas conservarán contenido durante refresh, ignorarán respuestas obsoletas y limpiarán timers.
- Se distinguirán carga, actualización, vacío local, filtro sin coincidencias, error, sesión no lista, provisional y operaciones locales.
- El DTO de lista no expone cobertura descargada. Por tanto, no se afirmará que una fecha fue descargada ni que está vacía globalmente; la cobertura explícita queda como limitación registrada, no se inventa un comando Rust.
- Detalle será solo lectura: sin acciones de transición, reprogramación, cancelación o comunicación simulada.

## Dependency Order

```text
T01: recursos y tokens
  → T02: shell y rutas → C01: revisión de base
  → T03: Agenda diaria real
  → T04: detalle real y adaptación → C02: recorrido y revisión final
```

## Task List

### Phase 1: Base compartida
- [x] T01: Incorporar tokens, fuentes locales e iconos Phosphor.
- [x] T02: Construir shell y rutas conservando Desarrollo.

### C01: Base visual y navegación
- [x] `pnpm quality` en verde; shell usable con teclado y en viewports objetivo.
- [~] Fuentes, iconos, rutas y acceso al esqueleto verificados; revisión del usuario sustituida por autorización expresa de ejecución continua.

### Phase 2: Recorrido de lectura
- [x] T03: Implementar Agenda de día con datos Tauri reales.
- [x] T04: Implementar detalle de cita e historia con retorno de contexto.

### C02: Recorrido completo de lectura
- [x] `pnpm quality` en verde; recorrido y estados verificados en ejecución dentro de lo observable.
- [x] Verificaciones pendientes registradas como NOT CHECKED (sin convertir a PASS): datos ficticios con IPC real (sin sesión local), 800×600 exacto, texto 200 %, lector de pantalla y lectura offline con sesión. El usuario aprobó el cierre del plan con esta deuda de verificación registrada el 2026-11-08.

## UX laws aplicadas

| Ley | Decisión concreta |
|---|---|
| Cognitive Load / Selective Attention | Agenda como inicio y una sola tarea; sin navegación/acciones placeholder |
| Jakob / Mental Model | Sidebar, fecha, tabla y detalle contextual familiares |
| Proximity / Chunking | Controles, tabla y detalle agrupados por relación; etiquetas junto a valores |
| Working Memory | Fecha/selección conservadas y detalle junto a la lista cuando quepa |
| Fitts | Controles con etiqueta, foco visible y objetivos adecuados |
| Doherty | Carga/refresh sin vaciar contenido; la latencia se medirá, no se inferirá de CSS |

No se aplican límites arbitrarios de opciones ni se declara conformidad UX por elegir estas leyes.

## Risks and Mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| Crear demasiadas abstracciones visuales | Medio | Solo componentes consumidos por shell/Agenda; `$minimal-code` antes de cada cambio |
| Mostrar una agenda vacía cuando falten datos o sesión | Alto | Estados explícitos y texto limitado a vista local; cobertura no disponible queda registrada |
| Presentar hora UTC como local | Alto | Formatear con zona de cuenta y revisar fechas límite/DST en escenarios disponibles |
| Romper acceso técnico durante migración | Alto | Conservar `/desarrollo` y el flujo actual hasta planes específicos |
| Validar solo compilación o prototipo | Alto | Recorrido Tauri real, viewports, teclado/foco y revisión del usuario |
| Mezclar estado de cita y sincronización | Alto | Badges y operaciones separados; sin éxito remoto inventado |

## Open Questions

- La navegación/rutas definitivas de las demás superficies se decidirá en sus planes; esta propuesta solo fija las rutas iniciales al aprobarse.
- La cobertura de datos por fecha requiere una fuente contractual si se quiere distinguir “no descargado” de “sin citas” con certeza; fuera de este plan UI-only.
- Validación con médico/asistente y auditoría completa de accesibilidad no se afirman; se registran resultados y pendientes en C02.
- Dark mode, colores de error/advertencia y acciones de escritura permanecen fuera de alcance.
