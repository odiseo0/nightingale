# Task List: P-20261109-09-services-settings-ui

**Plan:** [plan.md](plan.md)
**Updated:** 2026-11-09

## T01: Implementar catálogo y ofertas con datos reales

**Status:** Done (2026-11-09)

**Description:** Construir listas de servicios, ofertas y perfiles con nombres, duraciones, versiones y estados aceptados/provisionales.

**Acceptance criteria:**
- [x] Catálogo/oferta distinguidos con base/ajuste/efectiva (verificado: oferta real 20/25/25) y versiones disponibles en el DTO; estados de servicio y de oferta en badges separados.
- [x] Activo/inactivo, propuesta local, aceptada y operaciones no mezclados (SyncCell con "Guardada en este equipo" + badges por operación); `showInactive` real verificado (oferta desactivada ocultada y reaparece al activar el checkbox —preferencia compartida con Cuenta).
- [x] Solo lectura por `canManage`; vacíos y error con reintento; filas provisionales visibles (bug corregido: las filas solo-provisionales no se renderizaban) y acciones deshabilitadas mientras hay propuesta pendiente.
- [x] Refresco periódico añadido tras detectarse estado obsoleto tras aceptación externa.

**Verification:**
- [x] Focused tests: `pnpm build`; lectura real (4 servicios, 4 ofertas + perfil profesional).
- [x] Build or broader check: `pnpm quality` OK.
- [x] Manual check: captura conforme.

**Dependencies:** R07

**Files likely touched:**
- `src/features/services/ServicesPage.tsx`
- `src/features/services/presentation.ts`
- `src/shared/api/services.ts`
- `src/App.css`

**Estimated scope:** M

## C01: Después de T01

- [ ] `pnpm quality` y lectura real pasan; revisión del usuario.

## T02: Implementar creación/edición de servicios/ofertas

**Status:** Done (2026-11-09)

**Description:** Añadir formularios para crear/editar servicios, ofrecer servicios y ajustar duración/perfil profesional usando comandos existentes.

**Acceptance criteria:**
- [x] Formularios con profesional real (único → contextual) y minutos válidos sin UUID; verificado: creación real de "Consulta de control" (20 min), oferta con ajuste 25 (base/ajuste/efectiva 20/25/25 verificados por IPC tras aceptación); edición de nombre profesional implementada con versión.
- [x] Aviso antes de guardar: "Cambiar un servicio no modifica las citas anteriores: conservan sus datos guardados."
- [x] Resultados provisionales con mensaje separado de aceptación; errores por código; ref de intención única.

**Verification:**
- [x] Focused tests: `pnpm build`; crear/ofrecer con IPC real (datos persistentes en el fixture: 2 servicios "Consulta de control" y 1 oferta ajustada —quedan registrados como datos de prueba).
- [x] Build or broader check: `pnpm quality` OK.
- [x] Bug corregido en verificación: la página no refrescaba tras aceptaciones externas (poll 10 s añadido).

**Dependencies:** C01

**Files likely touched:**
- `src/features/services/ServiceEditor.tsx`
- `src/features/services/OfferEditor.tsx`
- `src/features/services/ServicesPage.tsx`
- `src/shared/api/services.ts`

**Estimated scope:** M

## T03: Implementar activo/inactivo y tratamiento de versiones/conflictos

**Status:** Done (2026-11-09)

**Description:** Añadir cambios de estado y manejo de expectedVersion, dependencias, conflictos y refresh de vistas.

**Acceptance criteria:**
- [x] Activar/desactivar servicio y oferta con base (`expectedVersion` de la fila aceptada) verificado con IPC real: oferta desactivada → oculta por filtro → reactivada → "active" aceptado. Con propuesta pendiente las acciones se bloquean según DTO (`isProvisional`).
- [x] `VERSION_CONFLICT`/`DEPENDENCY_FAILED`/permisos con texto de recuperación (mapeo compartido `mutationErrorMessage`); formulario conservado ante error.
- [x] Recibo sin descarga no retira provisional ni se presenta como aceptada: la fila conserva badges de operación hasta la descarga (verificado en el ciclo pendiente→sync→aceptado).

**Verification:**
- [x] Focused tests: `pnpm build`; estados reales verificados con IPC.
- [x] Build or broader check: `pnpm quality` OK.
- [ ] NOT CHECKED — VERSION_CONFLICT/DEPENDENCY_FAILED reales (no se simulan); lector de pantalla.

**Dependencies:** T02

**Files likely touched:**
- `src/features/services/ServicesPage.tsx`
- `src/features/services/ServiceEditor.tsx`
- `src/features/services/OfferEditor.tsx`
- `src/features/services/presentation.ts`

**Estimated scope:** M

## C02: Servicios configurables

- [x] `pnpm quality` en verde; operaciones locales reales (crear servicio, oferta con ajuste, activar/desactivar) con efecto explicado y aceptación separada.
- [ ] NOT CHECKED — conflictos de versión/dependencia reales, lector de pantalla.
- [~] Revisión del usuario: autorización continua; aceptación final pendiente.
