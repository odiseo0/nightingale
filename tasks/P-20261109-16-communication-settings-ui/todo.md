# Task List: P-20261109-16-communication-settings-ui

**Plan:** [plan.md](plan.md)
**Updated:** 2026-11-09

Plan Blocked: requiere R12 y políticas concretas.

## T01: Implementar política de recordatorios contratada

**Status:** Blocked

**Description:** Construir formulario de recordatorios con cantidad/anticipación e inclusión de enlace solo según contrato.

**Acceptance criteria:**
- [ ] Opciones proceden de la política real, sin valores de ejemplo como verdad.
- [ ] Se explica efecto en futuros avisos sin alterar mensajes anteriores.
- [ ] Campos/ayuda/estados son accesibles y no ofrecen canales no contratados.

**Verification:**
- [ ] Focused tests: `pnpm build` y lectura/edición del contrato disponible.
- [ ] Build or broader check: `pnpm quality`.
- [ ] Manual check: formulario, teclado y comprensión del efecto.

**Dependencies:** R12 y política real de recordatorios

**Files likely touched:**
- `src/features/settings/CommunicationSettingsPage.tsx`
- Cliente contractual definido por el plan técnico

**Estimated scope:** M

## T02: Implementar política de cancelación y enlaces

**Status:** Blocked

**Description:** Añadir opciones de cancelación, vencimiento y verificación de enlaces solo cuando estén definidas.

**Acceptance criteria:**
- [ ] Acciones del paciente reflejan política real y no crean nuevas capacidades del Hub.
- [ ] Vencimiento/verificación solo se muestran si el contrato los incluye.
- [ ] Efecto en futuros enlaces se explica sin cambiar citas existentes.

**Verification:**
- [ ] Focused tests: `pnpm build`; políticas admitidas/rechazadas.
- [ ] Build or broader check: `pnpm quality`.
- [ ] Manual check: opciones, teclado y viewports; revisión C01.

**Dependencies:** T01

**Files likely touched:**
- `src/features/settings/CommunicationSettingsPage.tsx`
- `src/features/settings/presentation.ts`

**Estimated scope:** M

## C01: Después de T01–T02

- [ ] `pnpm quality` y políticas reales pasan; revisión del usuario.

## T03: Implementar guardado remoto, refresh y errores

**Status:** Blocked

**Description:** Guardar política con conexión/permiso y presentar aceptación, actualización de vista y errores de forma separada.

**Acceptance criteria:**
- [ ] Guardado usa contrato remoto y no se presenta como preferencia local.
- [ ] Aceptación con refresh pendiente/fallido no se muestra como vista actualizada.
- [ ] Errores conservan campos y no repiten operación ciegamente.

**Verification:**
- [ ] Focused tests: `pnpm build`; guardado/refresh/conflicto disponible.
- [ ] Build or broader check: `pnpm quality`.
- [ ] Manual check: accesibilidad y NOT CHECKED registrados; revisión C02.

**Dependencies:** C01

**Files likely touched:**
- `src/features/settings/CommunicationSettingsPage.tsx`
- `src/features/settings/presentation.ts`

**Estimated scope:** M

## C02: Después de T03

- [ ] `pnpm quality` y política guardada pasan con evidencia disponible.
- [ ] Accesibilidad/NOT CHECKED registrados; revisión del usuario.
