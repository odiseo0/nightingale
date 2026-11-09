# Task List: P-20261108-06-appointment-creation-ui

**Plan:** [plan.md](plan.md)
**Updated:** 2026-11-08

Plan **Approved — implementación completa, pendiente de aceptación del usuario** (2026-11-08): dependencia de C02 de P05 resuelta con sesión fixture real. Trabajo secuencial; solo frontend y clientes de comandos existentes.

## T01: Añadir Nueva cita, ruta y carga de contexto

**Status:** Done (2026-11-08)

**Description:** Incorporar la acción Nueva cita en Agenda, la ruta `/cita/nueva`, el título de shell y el editor que carga el contexto real con estados de sesión/datos ausentes.

**Acceptance criteria:**
- [x] Nueva cita abre `/cita/nueva` con navegación directa y retorno (verificado con hash directo y botones en app real); la acción solo se renderiza cuando el contexto es utilizable (`canCreate`: pacientes, profesional y ofertas visibles).
- [x] El editor distingue carga, sesión no lista y contexto vacío (sin pacientes/profesional/ofertas) sin mostrar controles inútiles; contexto listo verificado con la sesión fixture real (7 pacientes, 4 ofertas).
- [x] Shell/tokens reutilizados; sin placeholders ni dependencias nuevas (solo código con comandos existentes).

**Verification:**
- [x] Focused tests: `pnpm build` OK; navegación Agenda→Nueva cita y estados con `getBookingContext` ejecutados en la ventana Tauri real vía CDP.
- [x] Build or broader check: `pnpm quality` OK.
- [x] Manual check: navegación directa, título de shell ("Nueva cita") y layout verificados en 800×600, 1024×768 y grande con capturas; revisión del usuario sustituida por autorización expresa de ejecución continua.

**Dependencies:** C02 de `P-20261108-05-agenda-ui-foundation`

**Files likely touched:**
- `src/App.tsx`
- `src/app/AppLayout.tsx`
- `src/features/appointments/AgendaPage.tsx`
- `src/features/appointments/AppointmentEditorPage.tsx`
- `src/App.css`

**Estimated scope:** M

## T02: Implementar campos y selección de responsables

**Status:** Done (2026-11-08) — flujo responsable parcialmente verificado (ver NOT CHECKED en C02)

**Description:** Construir los campos de paciente, profesional/oferta, fecha y responsable usando datos del contexto y un cliente tipado para `patients_list_patient_contacts`.

**Acceptance criteria:**
- [x] Paciente, oferta y fecha con etiquetas persistentes, valores reales y selección comprensible sin UUID; las opciones únicas se muestran como texto contextual y se seleccionan automáticamente.
- [x] El responsable aparece solo para `identificationBasis=minor_without_document` y consulta vínculos reales con nombre, relación, teléfono y vigencia (`src/shared/api/patients.ts`); no se crean contactos ni se decide elegibilidad en React.
- [x] Cambiar paciente/profesional/oferta invalida solo lo afectado (slot o responsable) con guardia de generación; el resto del borrador se conserva (verificado en ejecución).

**Verification:**
- [x] Focused tests: `pnpm build`; `patients_list_patient_contacts` ejecutado con IPC real (paciente fixture: 0 vínculos activos, sin error).
- [x] Build or broader check: `pnpm quality` OK.
- [x] Manual check: teclado (orden de tabulación verificado con eventos confiables), etiquetas persistentes y estados; fichas incompletas marcadas "(ficha incompleta)" con aviso contextual.

**Dependencies:** T01

**Files likely touched:**
- `src/shared/api/patients.ts`
- `src/features/appointments/AppointmentEditorPage.tsx`
- `src/features/appointments/BookingFields.tsx`
- `src/features/appointments/presentation.ts`
- `src/App.css`

**Estimated scope:** M

## C01: Después de T01–T02

- [x] `pnpm quality` pasa en el entregable.
- [x] Nueva cita y todos sus campos funcionan con contexto/contactos reales (IPC real en app Tauri: contexto con 7 pacientes/4 ofertas; contactos sin error; campos con datos reales y teclado operable).
- [~] Revisión del usuario: no aplica como pausa; autorización expresa de ejecución continua sin revisiones intermedias.

## T03: Implementar intervalos disponibles y resumen

**Status:** Done (2026-11-08)

**Description:** Consultar disponibilidad local por día y duración efectiva, presentar cobertura e intervalos seleccionables y mantener un resumen actualizado antes de guardar.

**Acceptance criteria:**
- [x] `getLocalAvailability` se ejecuta con profesional/oferta/fecha/duración efectiva reales; intervalos presentados en la zona de la cuenta (formato h23 con fin de intervalo en 24:00); sin cálculos de disponibilidad en React.
- [x] Cobertura (dentro de ventana / sin ventana / insuficiente — esta última registrada como texto), carga, error con reintento y ausencia de intervalos se diferencian; cambiar oferta/fecha invalida solo el slot y conserva el resto del borrador (verificado: el paciente seleccionado se conserva al cambiar fecha).
- [x] El resumen muestra paciente, profesional, servicio, fecha, intervalo, duración, responsable si aplica y zona; Guardar permanece deshabilitado hasta completar lo esencial.

**Verification:**
- [x] Focused tests: `pnpm build`; evaluaciones reales ejecutadas (domingo sin regla → sin intervalos; lunes con reglas → intervalos; tras crear citas, la ocupación se refleja en los intervalos devueltos por Rust).
- [x] Build or broader check: `pnpm quality` OK.
- [x] Manual check: resumen actualizado en vivo, cobertura legible, captura 800×600 sin overflow tras corrección de selects.

**Dependencies:** C01

**Files likely touched:**
- `src/features/appointments/AppointmentEditorPage.tsx`
- `src/features/appointments/AvailabilityPicker.tsx`
- `src/features/appointments/BookingSummary.tsx`
- `src/features/appointments/presentation.ts`
- `src/App.css`

**Estimated scope:** M

## T04: Guardar cita local, tratar errores y retornar con contexto

**Status:** Done (2026-11-08)

**Description:** Implementar envío con `createAppointment`, protección de doble envío y de cambios sin guardar, mensajes seguros y retorno a Agenda con selección/contexto y estado provisional.

**Acceptance criteria:**
- [x] Guardar usa el inicio del slot con la duración efectiva de la oferta, muestra progreso, evita una segunda intención (ref de intención única; verificado: doble clic creó exactamente una operación) y conserva todos los campos ante errores (verificado con SLOT_UNAVAILABLE y ficha incompleta).
- [x] Éxito presentado como "Guardada en este equipo · Pendiente de aceptación."; Agenda restaura fecha/filtros/selección (la fila nueva queda expandida) y el mensaje persistente; no se afirma aceptación remota. Persistencia tras reinicio verificada (las citas provisionales se aceptaron contra la API fixture y sobrevivieron al reinicio de la app).
- [x] Salir con cambios muestra Seguir editando/Descartar (API oficial `useBlocker` en forma función + `beforeunload`); verificado en enlace, atrás del navegador y ambas decisiones.

**Verification:**
- [x] Focused tests: `pnpm build`; creación local real verificada de extremo a extremo en la app Tauri (CDP): selección → disponibilidad → guardado → retorno → 1 operación nueva en SQLite con los instantes exactos.
- [x] Build or broader check: `pnpm quality` OK.
- [x] Manual check: doble clic, descarte, retorno con contexto/foco, 800×600/1024×768/grande y 200 % (zoom por estilo) sin overflow; lector de pantalla NOT CHECKED (no instalado).

**Dependencies:** T03

**Files likely touched:**
- `src/features/appointments/AppointmentEditorPage.tsx`
- `src/features/appointments/AgendaPage.tsx`
- `src/features/appointments/presentation.ts`
- `src/app/AppLayout.tsx` solo para título/contexto de navegación
- `src/App.css`

**Estimated scope:** M

## C02: Después de T03–T04

- [x] `pnpm quality` pasa (build + fmt + clippy + tests Rust) y Nueva cita funciona con IPC real: flujo completo Agenda→Nueva cita→selección→disponibilidad→guardado→Agenda ejecutado en la app Tauri con sesión fixture (1 operación local creada por guardado; doble clic sin duplicar).
- [x] Verificado: persistencia tras reinicio (9 citas visibles, aceptadas y descargadas), solapamiento rechazado por Rust con formulario conservado, duplicación evitada, errores legibles (SLOT_UNAVAILABLE, ficha incompleta), retorno con fecha/mensaje/selección, bloqueo de navegación (enlace y atrás), teclado (orden de tabulación con eventos confiables), 800×600 y 1024×768 sin overflow, texto al 200 % (emulado con zoom) sin overflow, contraste/tokens DS en capturas.
- [ ] NOT CHECKED — lector de pantalla: no hay lector instalado en el entorno.
- [ ] NOT CHECKED — responsable obligatorio de menor sin documento: los fixtures no incluyen ningún paciente con `identificationBasis=minor_without_document` (los vínculos activos del paciente consultado son 0); el código del flujo queda compilado pero no ejercitado con datos reales.
- [ ] NOT CHECKED — códigos de error no disparables con los fixtures: PERMISSION_DENIED, AUTHENTICATION_REQUIRED, INSUFFICIENT_COVERAGE, DEPENDENCY_FAILED, NO_CHANGES, VERSION_CONFLICT (mapeados con texto seguro, sin ejercitar).
- [ ] NOT CHECKED — corte físico de red durante creación/lectura (la creación es local por arquitectura y la sincronización se verificó contra la API real; el corte no se ejecutó).
- [x] Revisión del usuario: aceptada el 2026-11-09 (el usuario, informado de los NOT CHECKED, ordenó continuar con el roadmap). Plan cerrado como Completed; los NOT CHECKED quedan como deuda registrada (R20 del roadmap los recoge).
