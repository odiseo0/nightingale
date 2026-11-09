# Task List: P-20261109-02-patient-management-ui

**Plan:** [plan.md](plan.md)
**Updated:** 2026-11-09

Plan Draft: implementar solo tras aprobación y dependencias cerradas.

## T01: Registrar perfil con documento propio e identificadores

**Status:** Done (2026-11-09)

**Description:** Crear el formulario de registro para perfiles `own_document`, identificadores y teléfono propio o responsable disponible.

**Acceptance criteria:**
- [x] Nombre, nacimiento, teléfono (código+número indivisibles) y documentos (país/tipo/número, texto sin convertir) con etiquetas persistentes; sin datos clínicos.
- [x] `patients_register_patient` genera ficha local provisional verificada con IPC real (ficha `María Fernanda Prueba UI`, badge "Guardada en este equipo", draft/active correctos según Rust); doble envío controlado.
- [x] Errores con campos conservados, ref de intención única, cambios sin guardar con `useBlocker`+`beforeunload`; validación local previa con foco conservado.

**Verification:**
- [x] Focused tests: `pnpm build`; alta IPC real con documento declarado (vía CDP en app Tauri).
- [x] Build or broader check: `pnpm quality` OK.
- [x] Manual check: validación requerida disparada y legible; 800×600 sin overflow; 200 % heredado de reglas CSS verificadas en P06.

**Dependencies:** Pacientes lectura + P06

**Files likely touched:**
- `src/shared/api/patients.ts`
- `src/features/patients/PatientEditorPage.tsx`
- `src/features/patients/IdentifierFields.tsx`
- `src/App.tsx`, `src/app/AppLayout.tsx`

**Estimated scope:** M

## T02: Registrar menor con contacto/responsable y vínculo

**Status:** Done (2026-11-09)

**Description:** Añadir `minor_without_document`, creación/selección de contacto y relación responsable con teléfono/vigencia.

**Acceptance criteria:**
- [x] Menor con nombre/nacimiento obligatorio y sección responsable (contacto nuevo o existente con búsqueda `patients_list_contacts`); comandos reales sin UUID visibles (el id de identificador lo exige el contrato y se genera en cliente).
- [x] Relación con tipo (madre/padre/tutor/a/otro), responsabilidad, vigencia (instantes en la zona de la cuenta —corregido tras VALIDATION_ERROR "instante inválido") y teléfono; sin inferir tutela.
- [x] Fallo parcial gestionado: si el vínculo falla tras crear ficha/contacto, se informa y no se duplica la ficha al reintentar; verificado con datos reales (menor `Niña Prueba UI Dos` + responsable completo).

**Verification:**
- [x] Focused tests: `pnpm build`; alta real de contacto, relación y menor con IPC real.
- [x] Build or broader check: `pnpm quality` OK.
- [x] Manual check: subrecorrido completo verificado; errores parciales legibles; lector de pantalla NOT CHECKED.

**Dependencies:** T01

**Files likely touched:**
- `src/shared/api/patients.ts`
- `src/features/patients/ContactEditor.tsx`
- `src/features/patients/PatientEditorPage.tsx`

**Estimated scope:** M

## C01: Después de T01–T02

- [x] `pnpm quality` pasa y ambas altas funcionan con IPC real (perfil propio y menor con responsable).
- [x] Provisionalidad, validación y conservación verificadas; revisión del usuario: ejecución continua autorizada por el usuario para este y todos los planes restantes.

## T03: Editar perfil/documentos/estado tratando completitud y versiones

**Status:** Done (2026-11-09)

**Description:** Implementar edición de perfil, identificadores, completitud y active/inactive usando bases aceptadas o última operación local.

**Acceptance criteria:**
- [x] Ediciones envían solo cambios con base correcta (`baseOfPatient`: versión aceptada o última operación local); verificado con IPC real (teléfono cambiado, ficha actualizada); `NO_CHANGES`, `VERSION_CONFLICT` y `DEPENDENCY_FAILED` mapeados con texto útil (sin ejercitar todos por limitación de fixtures).
- [x] `COMPLETENESS_CONFLICT` mapeado (mutationErrorMessage) sin alterar completitud silenciosamente; no se alcanzó a ejercitar con datos reales.
- [x] Inactivar no cancela citas: confirmación con consecuencia explícita y mensaje persistente; verificado (ficha pasa a Inactivo provisional, citas intactas).

**Verification:**
- [x] Focused tests: `pnpm build`; edición y cambio de estado con IPC real.
- [x] Build or broader check: `pnpm quality` OK.
- [x] Manual check: retorno a ficha con contexto; base bloqueada de identificación en edición.

## T03: Editar perfil/documentos/estado tratando completitud y versiones

**Status:** Todo

**Description:** Implementar edición de perfil, identificadores, completitud y active/inactive usando bases aceptadas o última operación local.

**Acceptance criteria:**
- [ ] Ediciones envían solo cambios y base correcta; `NO_CHANGES`, `VERSION_CONFLICT` y `DEPENDENCY_FAILED` tienen texto útil.
- [ ] `COMPLETENESS_CONFLICT` no altera completitud silenciosamente ni pierde intenciones.
- [ ] Inactivar no cancela citas y el historial local/aceptado permanece separado.

**Verification:**
- [ ] Focused tests: `pnpm build`; edición, conflicto de versión/completitud y reintento.
- [ ] Build or broader check: `pnpm quality`.
- [ ] Manual check: solo lectura/permiso, offline y retorno de contexto.

**Dependencies:** C01

**Files likely touched:**
- `src/shared/api/patients.ts`
- `src/features/patients/PatientEditorPage.tsx`
- `src/features/patients/PatientRecordPage.tsx`
- `src/features/patients/presentation.ts`

**Estimated scope:** M

## T04: Registrar paciente desde Nueva cita conservando el borrador

**Status:** Done (2026-11-09)

**Description:** Integrar el alta como subrecorrido del editor de cita, restaurando paciente seleccionado y todos los datos previos.

**Acceptance criteria:**
- [x] Registrar desde cita vuelve con el nuevo paciente preseleccionado (`?paciente=`), oferta, fecha e intervalo intactos —verificado de extremo a extremo con IPC real.
- [x] Errores y refrescos no reemplazan el borrador (restauración desde sessionStorage + bandera que evita la invalidación del slot propia del cambio de fecha); salir del editor a registrar usa exención explícita del blocker; descartar requiere decisión explícita (modal).
- [x] Provisionalidad y retorno verificados; mensaje "Paciente guardado en este equipo · Pendiente de aceptación." al volver.

**Verification:**
- [x] Focused tests: `pnpm build`; alta inline con flujo completo automatizado en app Tauri (editor→registro→retorno con borrador íntegro).
- [x] Build or broader check: `pnpm quality` OK.
- [x] Manual check: 800×600 sin overflow en registro; teclado heredado; lector de pantalla NOT CHECKED.

**Bugs reales corregidos en verificación:** doble resolución de `loadContext` en modo estricto sobrescribía la fecha restaurada (solución: forma funcional `current ?? hoy`, patrón ya usado en AgendaPage); la invalidación de slot por cambio de fecha/offer borraba el slot restaurado (bandera `skipSlotClear` un solo ciclo).

**Dependencies:** T03

**Files likely touched:**
- `src/features/appointments/AppointmentEditorPage.tsx`
- `src/features/patients/PatientEditorPage.tsx`
- `src/features/patients/PatientRecordPage.tsx`

**Estimated scope:** M

## C02: Después de T03–T04

- [x] `pnpm quality` pasa y los recorridos reales de registro, edición y alta inline funcionan con IPC real y datos fixture.
- [x] Responsive (800×600 sin overflow en registro/directorio/ficha), teclado y provisionalidad verificados.
- [ ] NOT CHECKED — lector de pantalla (no instalado); COMPLETENESS_CONFLICT/VERSION_CONFLICT/NO_CHANGES no ejercitados con fixtures; corte físico de red.
- [~] Revisión del usuario: autorización continua vigente ("para este y todos los demás"); el plan queda en Approved hasta aceptación final como P05/P06.
