# Task List: P-20261109-15-team-devices-settings-ui

**Plan:** [plan.md](plan.md)
**Updated:** 2026-11-09

Plan Blocked: requiere contrato remoto y recorrido de permisos/revocación.

## T01: Implementar miembros e invitaciones reales

**Status:** Blocked

**Description:** Construir lista de miembros, roles/estado e invitación con correo mediante contrato remoto autorizado.

**Acceptance criteria:**
- [ ] Miembros muestran identidad/rol/estado permitidos sin compartir cuenta.
- [ ] Invitar usa operación remota real y muestra resultado conocido; no simula envío.
- [ ] Cambios de permiso solo con conexión/autorización y no se ofrecen en modo local ficticio.

**Verification:**
- [ ] Focused tests: `pnpm build` y operaciones reales disponibles.
- [ ] Build or broader check: `pnpm quality`.
- [ ] Manual check: roles, errores, teclado y retorno.

**Dependencies:** Contrato remoto de miembros/invitaciones y permisos

**Files likely touched:**
- `src/features/settings/TeamSettingsPage.tsx`
- Cliente contractual definido por el plan técnico

**Estimated scope:** M

## T02: Implementar dispositivos y revisión de acceso

**Status:** Blocked

**Description:** Añadir lista de dispositivos con aplicación, último acceso conocido y estado, y detalle de acceso autorizado.

**Acceptance criteria:**
- [ ] Dispositivo actual/otros se distinguen por datos reales, sin rutas sensibles ni tokens.
- [ ] Revisar acceso muestra alcance y último resultado conocido.
- [ ] Revocar solo con operación remota/permiso y confirmación de consecuencia.

**Verification:**
- [ ] Focused tests: `pnpm build`; lectura/revisión real de dispositivos.
- [ ] Build or broader check: `pnpm quality`.
- [ ] Manual check: diálogo, teclado/foco y viewports; revisión C01.

**Dependencies:** T01

**Files likely touched:**
- `src/features/settings/DevicesSettingsPage.tsx`
- `src/features/settings/DeviceAccessDialog.tsx`

**Estimated scope:** M

## C01: Después de T01–T02

- [ ] `pnpm quality` y operaciones remotas reales pasan; revisión del usuario.

## T03: Cubrir revocación, reconexión y resultados desconocidos

**Status:** Blocked

**Description:** Presentar revocación, reconexión y resultados desconocidos sin afirmar efecto instantáneo en equipos offline.

**Acceptance criteria:**
- [ ] Revocación muestra cuando fue aceptada y que un equipo offline la conocerá al reconectar.
- [ ] Resultado desconocido conserva identidad para comprobar antes de repetir.
- [ ] Rechazo por permiso conserva intención y no usa hora local como autorización.

**Verification:**
- [ ] Focused tests: `pnpm build`; revocación/reconexión/unknown con fixture disponible.
- [ ] Build or broader check: `pnpm quality`.
- [ ] Manual check: accesibilidad y NOT CHECKED registrados; revisión C02.

**Dependencies:** C01

**Files likely touched:**
- `src/features/settings/DevicesSettingsPage.tsx`
- `src/features/settings/DeviceAccessDialog.tsx`

**Estimated scope:** M

## C02: Después de T03

- [ ] `pnpm quality` y revocación real pasan con evidencia disponible.
- [ ] Accesibilidad/NOT CHECKED registrados; revisión del usuario.
