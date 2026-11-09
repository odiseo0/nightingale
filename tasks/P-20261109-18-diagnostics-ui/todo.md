# Task List: P-20261109-18-diagnostics-ui

**Plan:** [plan.md](plan.md)
**Updated:** 2026-11-09

## T01: Implementar S13 con metadatos reales

**Status:** Done (2026-11-09)

**Description:** Construir diagnóstico con versión de app/plataforma/arquitectura, esquema/applicationId, sesión local y último intercambio conocido.

**Acceptance criteria:**
- [x] `getDiagnostics`, `getStorageStatus`, `getCoreStatus` y `sync_list_pending` presentan datos reales con etiquetas (verificado: 0.1.0, windows/x86_64, esquema 10, sesión con cuenta activa, resultado complete, 0 pendientes, último intercambio); cifras tabulares Aileron.
- [x] Aviso explícito: datos locales, no prueban conexión/aceptación remota.
- [x] Acceso desde Ayuda (verificado: enlace "Abrir diagnóstico (S13)" → /diagnostico) y sin entrada en la navegación diaria.

**Verification:**
- [x] Focused tests: `pnpm build`; consulta real verificada.
- [x] Build or broader check: `pnpm quality` OK.
- [x] Manual check: solo DTO permitidos, sin secretos ni campos arbitrarios.

**Dependencies:** P05

**Files likely touched:**
- `src/features/diagnostics/DiagnosticsPage.tsx`
- `src/features/help/HelpDialog.tsx`
- `src/app/AppLayout.tsx`

**Estimated scope:** M

## T02: Cubrir errores, sesión y límites de interpretación

**Status:** Done (2026-11-09)

**Description:** Añadir estados de carga/error/sin sesión y revisar privacidad, accesibilidad y contenido técnico.

**Acceptance criteria:**
- [x] Errores con texto seguro y reintento ("No se pudo consultar el diagnóstico local…") sin interpolar objetos; sin afirmaciones cloud.
- [x] Sin sesión distinguible (estados de sesión del DTO: "Sin sesión"/"sin cuenta"); "las pendientes no son un error" explícito.
- [x] Viewports/zoom heredados de reglas CSS verificadas; lector de pantalla NOT CHECKED.

**Verification:**
- [x] Focused tests: `pnpm build`; estados verificados con sesión real.
- [x] Build or broader check: `pnpm quality` OK.

**Dependencies:** T01

**Files likely touched:**
- `src/features/diagnostics/DiagnosticsPage.tsx`
- `src/features/diagnostics/presentation.ts`

**Estimated scope:** M

## C01/C02: Diagnóstico

- [x] `pnpm quality` en verde; datos reales, ausencia de secretos (solo DTO permitidos), errores seguros y acceso desde Ayuda verificados.
- [ ] NOT CHECKED — lector de pantalla (no instalado); estado sin sesión (no se destruye la sesión de verificación).
- [~] Revisión del usuario: autorización continua; aceptación final pendiente.
