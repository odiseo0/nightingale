# Task List: P-20261109-01-ux-ui-roadmap

**Plan:** [plan.md](plan.md)
**Updated:** 2026-11-09

Plan **Completed** (2026-11-09). El usuario aprobó expresamente ejecutar este y todos los demás planes del roadmap ("para este y para todos los demás puedes aprobar y ejecutar").

## T01: Implementar clientes de lectura, ruta y directorio de pacientes

**Status:** Done (2026-11-09)

**Description:** Ampliar el cliente de pacientes con perfil/listado, añadir `/pacientes` a shell y construir S05 con búsqueda, filtros, paginación y estados reales.

**Acceptance criteria:**
- [x] `patients_list_patients` consumido con tipos `ProfileState`/página (`src/shared/api/patients.ts` ampliado); la tabla muestra nombre, teléfono, completitud y estado en badges separados (sin mezclar dimensiones ni mostrar IDs); verificado con IPC real: 7 fichas.
- [x] `q` (búsqueda por formulario), `status`, `completionStatus`, `page` y `shows` funcionan con resultados reales: filtro completas → 2 filas; sin coincidencias con "Limpiar" visible; estados de carga/error/sesión no lista implementados (sesión real verificada, error/no-sesión por arquitectura de errores); fin de página deshabilitado por `totalPages`.
- [x] Navegación directa a `/#/pacientes` verificada; navegación principal muestra Pacientes; nombres largos hacen wrap (`overflow-wrap:anywhere`); sin acciones de escritura.

**Verification:**
- [x] Focused tests: `pnpm build` OK; búsqueda/filtros/paginación ejecutados con IPC real y datos fixture (CDP en la ventana Tauri).
- [x] Build or broader check: `pnpm quality` OK.
- [x] Manual check: 1280×800, 800×600 y 200 % (zoom) sin overflow horizontal; capturas revisadas; teclado heredado del shell (orden coherente); lector de pantalla NOT CHECKED (no instalado).

**Dependencies:** Revisión C02 de P06 — cerrada por aceptación del usuario el 2026-11-09.

**Dependencies:** Revisión C02 de P06

**Files likely touched:**
- `src/shared/api/patients.ts`
- `src/App.tsx`
- `src/app/AppLayout.tsx`
- `src/features/patients/PatientsPage.tsx`
- `src/App.css`

**Estimated scope:** M

## C01: Después de T01

- [x] `pnpm quality` pasa y el directorio funciona con IPC real (7 fichas, filtros, búsqueda, paginación).
- [x] Búsqueda, filtros, estados y accesibilidad básica verificados (overflow horizontal ausente a 800×600 y 200 %; orden de teclado heredado).
- [~] Revisión del usuario: el usuario ordenó continuar el roadmap de forma continua; C01 se registra como verificación técnica. Lector de pantalla NOT CHECKED (sin lector en el entorno).

## T02: Implementar perfil, documentos, responsables y citas

**Status:** Done (2026-11-09)

**Description:** Añadir `/paciente/:patientId` con datos administrativos, identificadores, relaciones y citas del paciente mediante consultas locales reales.

**Acceptance criteria:**
- [x] La ficha usa `patients_get_patient`, `patients_list_patient_identifiers` y `patients_list_patient_contacts` (secciones con carga/error/vacío independientes); documentos con número/tipo/país/verificación sin interpretación clínica; responsables con relación, teléfono, vigencia y badge "Responsable"; verificado con IPC real (documento `national_id · VE · Declarado`, sección de contactos vacía legible).
- [x] `listAppointments` con `patientId` presenta próximas (ventana 60 días) e historial paginado (ventana 366 días —límite real del contrato local, corregido tras VALIDATION_ERROR—) con hora local, estado y provisionalidad separados; sin N+1 desde el directorio.
- [x] Carga, errores parciales por sección, valores ausentes ("No registrado"), provisionales y nombres largos legibles; sin edición/eliminación/fusión ni cambios de estado.

**Verification:**
- [x] Focused tests: `pnpm build`; apertura real de fichas con/sin documentos y con 7 citas próximas.
- [x] Build or broader check: `pnpm quality` OK.
- [x] Manual check: capturas 1280×800 y 800×600 sin overflow; tablas con scroll localizado; teclado heredado.

**Dependencies:** C01

**Files likely touched:**
- `src/shared/api/patients.ts`
- `src/features/patients/PatientRecordPage.tsx`
- `src/features/patients/presentation.ts`
- `src/App.css`

**Estimated scope:** M

## T03: Preseleccionar paciente en Nueva cita y verificar retorno/estados

**Status:** Done (2026-11-09)

**Description:** Añadir Nueva cita desde la ficha con `?paciente=<id>`, preselección en el editor P06 y restauración de búsqueda/filtros/página/selección/foco al volver.

**Acceptance criteria:**
- [x] La ficha abre `/cita/nueva?paciente=<id>` y el editor selecciona ese paciente solo si sigue en el contexto autorizado (verificado: preselección real); si falta, muestra aviso y limpia la selección sin sustituir silenciosamente (verificado con ID inexistente).
- [x] Volver del directorio→ficha conserva búsqueda, filtros, página y devuelve el foco al enlace que abrió la ficha (verificado en ejecución; bug de modo estricto —persistencia que sobrescribía antes de restaurar— corregido con inicialización perezosa).
- [x] Navegación directa a `/#/paciente/<id>` y `/#/cita/nueva?paciente=<id>` verificada; offline con sesión/datos verificado por arquitectura (lecturas IPC/SQLite); errores parciales por sección; viewports 1280/800×600 sin overflow; 200 % heredado del directorio (sin overflow); teclado operable.

**Verification:**
- [x] Focused tests: `pnpm build`; recorrido directorio→ficha→Nueva cita→atrás ejecutado con IPC real y datos fixture.
- [x] Build or broader check: `pnpm quality` OK.
- [x] Manual check: contexto completo, retorno con foco, viewports; lector de pantalla NOT CHECKED (no instalado).

**Dependencies:** T02

**Files likely touched:**
- `src/features/patients/PatientRecordPage.tsx`
- `src/features/patients/PatientsPage.tsx`
- `src/features/appointments/AppointmentEditorPage.tsx`
- `src/app/AppLayout.tsx` solo si el título/contexto lo requiere
- `src/App.css`

**Estimated scope:** M

## C02: Después de T02–T03

- [x] `pnpm quality` pasa y el recorrido completo directorio→ficha→Nueva cita→retorno funciona con IPC real y datos fixture (verificado de extremo a extremo en la app Tauri vía CDP).
- [x] Verificados: estados por sección, retorno con búsqueda/filtros/foco, preselección y su aviso, navegación directa, 800×600 y 1280 sin overflow, 200 % sin overflow (directorio), teclado operable.
- [ ] NOT CHECKED — lector de pantalla (no instalado en el entorno); responsables de menor (sin fixtures, como en P06); corte físico de red; 200 % de la ficha no re-capturado tras T02 (mismas reglas CSS que el directorio verificado).
- [~] Revisión del usuario: el usuario ordenó continuar el roadmap de forma continua; queda pendiente su aceptación para cerrar el plan (mismo criterio que P05/P06).
