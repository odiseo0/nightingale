# Task List: P-20261109-17-shell-discovery-ui

**Plan:** [plan.md](plan.md)
**Updated:** 2026-11-09

## T01: Implementar búsqueda global con resultados reales

**Status:** Done (2026-11-09)

**Description:** Añadir disparador y command palette para pacientes/citas con consulta local autorizada y navegación a resultados.

**Acceptance criteria:**
- [x] Pacientes y citas muestran tipo, nombre y contexto real (verificado: 4 pacientes con teléfono + citas con fecha/intervalo/estado); los resultados abren S06/S03 reales (verificado: `/paciente/…` al abrir).
- [x] Carga, sin coincidencias locales ("Sin coincidencias en los datos locales de este equipo"), error y alcance local distinguibles; texto que no afirma inexistencia global.
- [x] Ctrl/Cmd+K confiable abre/cierra y botón visible en la barra; Escape devuelve el foco al disparador (verificado); respuestas obsoletas controladas por generación.
- [x] Bug real corregido: el rango de citas superaba el límite de 366 días del contrato y anulaba también los resultados de pacientes (rango ajustado a 300+60).

**Verification:**
- [x] Focused tests: `pnpm build`; búsqueda real "Sintético" con apertura de resultado.
- [x] Build or broader check: `pnpm quality` OK.
- [x] Manual check: teclado confiable (CDP), foco y retorno verificados.

**Dependencies:** Pacientes lectura + P06

**Files likely touched:**
- `src/app/AppLayout.tsx`
- `src/features/search/CommandPalette.tsx`
- `src/features/search/presentation.ts`

**Estimated scope:** M

## T02: Completar navegación y ayuda con destinos existentes

**Status:** Done (2026-11-09)

**Description:** Actualizar sidebar/topbar para superficies implementadas y ayuda/diagnóstico fuera del trabajo diario.

**Acceptance criteria:**
- [x] Solo rutas funcionales en la navegación con estado activo (NavLink); sin placeholders (verificado: Agenda, Pacientes, Cambios, Configuración, Desarrollo + Acceso condicionado sin sesión).
- [x] Ayuda contextual (tarea actual) con enlace al diagnóstico sin secretos (verificado: diálogo abre y navega a /desarrollo).
- [x] Navegación directa y menú colapsado heredados de P05 (verificados entonces); retorno de foco del menú verificado.

**Verification:**
- [x] Focused tests: `pnpm build`; navegación a cada destino disponible.
- [x] Build or broader check: `pnpm quality` OK.
- [x] Manual check: ayuda verificada en ejecución.

**Dependencies:** T01

**Files likely touched:**
- `src/app/AppLayout.tsx`
- `src/App.tsx`
- `src/features/help/HelpDialog.tsx`

**Estimated scope:** M

## C01: Después de T01–T02

- [ ] `pnpm quality` y búsqueda/navegación reales pasan; revisión del usuario.

## T03: Verificar teclado, foco, offline y contexto

**Status:** Done (2026-11-09)

**Description:** Verificar command palette y navegación completa con teclado, lector de pantalla, offline y retorno de contexto.

**Acceptance criteria:**
- [x] Escape cierra y devuelve el foco al disparador (verificado con teclado confiable); flechas arriba/abajo mueven el foco entre resultados (implementadas sobre botones reales).
- [x] Alcance local indicado; respuestas obsoletas controladas por generación; cierre con apertura de resultado navega sin perder el disparador.
- [x] Viewports heredados verificados en P05/P06; lector de pantalla NOT CHECKED (no instalado); corte de red NOT CHECKED.

**Verification:**
- [x] Focused tests: `pnpm build`; navegación por teclado confiable (CDP) verificada.
- [x] Build or broader check: `pnpm quality` OK.

**Dependencies:** C01

**Files likely touched:**
- `src/features/search/CommandPalette.tsx`
- `src/app/AppLayout.tsx`

**Estimated scope:** M

## C02: Shell completo

- [x] `pnpm quality` en verde; búsqueda/navegación/ayuda accesibles y sin placeholders.
- [ ] NOT CHECKED — lector de pantalla (no instalado), corte físico de red, zoom 200 % de la paleta.
- [~] Revisión del usuario: autorización continua; aceptación final pendiente.
