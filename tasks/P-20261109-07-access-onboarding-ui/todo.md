# Task List: P-20261109-07-access-onboarding-ui

**Plan:** [plan.md](plan.md)
**Updated:** 2026-11-09

## T01: Implementar acceso y selección de cuenta

**Status:** Done (2026-11-09) — flujo de login interactivo con evidencia previa (P05)

**Description:** Construir S01 con acceso externo, progreso/cancelación, cuentas autorizadas y activación de cuenta usando comandos existentes.

**Acceptance criteria:**
- [x] Iniciar/completar/cancelar acceso y elegir/activar cuenta implementados con `beginLogin`/`completeLogin`/`cancelLogin`/`listAccounts`/`activateAccount` y estados reales (progreso "Completa el acceso en el navegador externo", cancelación, lista de cuentas con inactivas deshabilitadas). El login OIDC completo se ejecutó con asistencia del usuario en P05 contra el fixture real; la rama sin sesión se verificó en P05 C02. No se habilita Agenda antes de cuenta activa.
- [x] Sin configuración, sin sesión, revocado/inactivo y errores con texto útil y sin secretos; verificado render con sesión activa ("Acceso activo · Consulta de prueba Core-Engine").
- [x] Desarrollo permanece accesible (enlace desde la preparación y en navegación).

**Verification:**
- [x] Focused tests: `pnpm build`; estado con sesión real y navegación Ir a la agenda → /agenda (verificado).
- [x] Build or broader check: `pnpm quality` OK.
- [ ] NOT CHECKED — cerrar sesión y re-login interactivo autónomo (rompería la sesión necesaria para los planes siguientes; el flujo OIDC real ya se verificó en P05 con el usuario).

**Dependencies:** P05

**Files likely touched:**
- `src/App.tsx`
- `src/app/AppLayout.tsx`
- `src/features/identity/AccessPage.tsx`
- `src/features/identity/presentation.ts`

**Estimated scope:** M

## T02: Implementar preparación inicial basada en datos reales

**Status:** Done (2026-11-09)

**Description:** Añadir checklist de preparación de cuenta/zona, profesional, servicio/oferta y horario con enlaces a configuración real.

**Acceptance criteria:**
- [x] Cada paso se deriva de datos descargados reales y distingue completo/pendiente: verificado con la sesión fixture (zona America/Caracas ✓, 1 profesional ✓, 4 ofertas ✓, 2 reglas de horario ✓).
- [x] Sin tutorial ni datos opcionales; la gestión detallada enlaza a Desarrollo de forma transicional (Configuración llega en R07–R09), declarado en la propia pantalla.
- [x] "Ir a la agenda" solo aparece con cuenta activa y acceso activo (verificado: navega a /agenda).

**Verification:**
- [x] Focused tests: `pnpm build`; preparación con cuenta fixture completa.
- [x] Build or broader check: `pnpm quality` OK.
- [x] Manual check: captura conforme; estados sin datos implementados (contadores 0 / "Sin datos descargados").

**Dependencies:** T01

**Files likely touched:**
- `src/features/identity/SetupPage.tsx`
- `src/shared/api/account.ts`
- `src/shared/api/services.ts`
- `src/shared/api/scheduling.ts`

**Estimated scope:** M

## C01: Después de T01–T02

- [ ] `pnpm quality` y acceso/preparación reales pasan; revisión del usuario.

## T03: Cubrir estados, recuperación y transición a Agenda

**Status:** Done (2026-11-09) — estados implementados; revocado/inactivo/offline/recovery NO alcanzables sin romper la sesión

**Description:** Implementar estados offline/revocado/inactivo/recovery y retorno a Agenda sin perder datos/pendientes.

**Acceptance criteria:**
- [x] Offline con últimos permisos, servicio no disponible (outcome failed/retry_waiting en encabezado), revocado e inactivo y recovery_required con presentaciones distintas implementadas sobre los campos reales de `CoreStatus` (verificado por inspección con sesión activa; los estados adversos no se disparan para no destruir la sesión de verificación).
- [x] Recuperación/reconexión: textos que afirman conservación de datos y pendientes; ningún texto usa la hora local como prueba de autorización.
- [x] Entrada a Agenda con cuenta/contexto conservados (verificado: /acceso → Ir a la agenda → Agenda con sesión y datos).

**Verification:**
- [x] Focused tests: `pnpm build`; estados disponibles verificados.
- [x] Build or broader check: `pnpm quality` OK.
- [ ] NOT CHECKED — estados revocado/inactivo/recovery/offline reales (requerirían destruir la sesión fixture o cortar red); lector de pantalla.

**Dependencies:** C01

**Files likely touched:**
- `src/features/identity/AccessPage.tsx`
- `src/features/identity/SetupPage.tsx`
- `src/features/identity/presentation.ts`

**Estimated scope:** M

## C02: Acceso/preparación

- [x] `pnpm quality` en verde; acceso, preparación y transición a Agenda verificados.
- [ ] NOT CHECKED — cierre/re-login autónomo, estados adversos de acceso reales, lector de pantalla.
- [~] Revisión del usuario: autorización continua; aceptación final pendiente.
