# Task List: P-20261109-08-account-settings-ui

**Plan:** [plan.md](plan.md)
**Updated:** 2026-11-09

## T01: Implementar navegación de Configuración y vista de cuenta

**Status:** Done (2026-11-09)

**Description:** Añadir S12 con subnavegación y presentación de datos de cuenta, lugar, zona, política, versión y permisos.

**Acceptance criteria:**
- [x] `account_get_view` muestra datos reales (nombre, zona IANA, lugar completo, política, versión aceptada y última actualización); solo lectura cuando `canManage` es falso (fieldset deshabilitado con explicación); sin cuenta → estado propio.
- [x] Preferencias locales separadas (checkbox real `servicesShowInactive` vía `account_set_preference`); tema claro como único implementado, sin ofrecer dark.
- [x] Subnavegación `/configuracion/cuenta` con NavLink activo; navegación directa verificada; enlace "Configuración" en la barra lateral.

**Verification:**
- [x] Focused tests: `pnpm build`; lectura real verificada (versión 1 inicial, datos del fixture).
- [x] Build or broader check: `pnpm quality` OK.
- [x] Manual check: captura conforme; contenido largo con wrapping.

**Dependencies:** P05

**Files likely touched:**
- `src/App.tsx`
- `src/app/AppLayout.tsx`
- `src/features/settings/SettingsLayout.tsx`
- `src/features/settings/AccountSettingsPage.tsx`

**Estimated scope:** M

## C01: Cuenta visible

- [x] `pnpm quality` en verde; datos reales y preferencias separadas verificados; revisión del usuario: autorización continua.

## T02: Implementar edición de datos/política y preferencias locales

**Status:** Done (2026-11-09)

**Description:** Añadir formularios para datos de cuenta, política de ausencia y preferencias del dispositivo usando versiones/comandos existentes.

**Acceptance criteria:**
- [x] Cambios remotos envían solo patch + expectedVersion (verificado: mutación real del margen 15→20→15 con `updateAccountPolicy`); preferencias locales por canal separado (verificado toggle con mensaje "no requiere conexión").
- [x] Aceptación y refresh presentados por separado: mensaje real "Cambio aceptado por la API. Actualizando la vista local con el resultado."; `refreshStatus=failed` tendría texto que no presenta la vista como actualizada (mapeado, no disparado).
- [x] Doble envío controlado (ref de intención), datos conservados ante error, salida protegida.

**Verification:**
- [x] Focused tests: `pnpm build`; actualización real de política con la sesión fixture (versiones 1→2→3 verificadas).
- [x] Build or broader check: `pnpm quality` OK.
- [x] Bug corregido en verificación: el mensaje de resultado se sobrescribía con "Sin cambios" por lectura de estado obsoleta; se reconstruye localmente y se encadena la versión entre detalle y política.

**Dependencies:** C01

**Files likely touched:**
- `src/features/settings/AccountSettingsPage.tsx`
- `src/features/settings/PreferencesForm.tsx`
- `src/shared/api/account.ts`

**Estimated scope:** M

## T03: Cubrir versiones, refresh, errores y salida sin guardar

**Status:** Done (2026-11-09)

**Description:** Tratar conflictos, versiones, refresh fallido y cambios sin guardar con retorno contextual.

**Acceptance criteria:**
- [x] `VERSION_CONFLICT` y permisos con texto de recuperación mapeados (sin ejercitar destruyendo la sesión); campos conservados (verificado: tras "Seguir editando" el valor editado permanece).
- [x] Aceptación con refresh fallido: mensaje persistente implementado ("los datos locales se conservan y no se presentan como actualizados"); refreshStatus=failed no se simula.
- [x] Descartar/Seguir editando verificados en esta superficie (modal al navegar con cambios); atrás/adelante con el mismo patrón de blocker verificado en P06/R02.

**Verification:**
- [x] Focused tests: `pnpm build`; salida sin guardar verificada con IPC real.
- [x] Build or broader check: `pnpm quality` OK.
- [ ] NOT CHECKED — VERSION_CONFLICT real y refreshStatus=failed reales (no se simulan); lector de pantalla.

**Dependencies:** T02

**Files likely touched:**
- `src/features/settings/AccountSettingsPage.tsx`
- `src/features/settings/PreferencesForm.tsx`
- `src/features/settings/presentation.ts`

**Estimated scope:** M

## C02: Cuenta y preferencias

- [x] `pnpm quality` en verde; actualización remota real (política, versiones encadenadas) y preferencia local verificadas; salida protegida verificada.
- [ ] NOT CHECKED — VERSION_CONFLICT/refreshStatus=failed reales, solo lectura sin permiso (sin segundo rol), lector de pantalla.
- [~] Revisión del usuario: autorización continua; aceptación final pendiente.
