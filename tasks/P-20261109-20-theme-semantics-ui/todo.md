# Task List: P-20261109-20-theme-semantics-ui

**Plan:** [plan.md](plan.md)
**Updated:** 2026-11-09

Plan Blocked: no ejecutar sin decisión expresa del usuario y cambio aprobado de DESIGN.

## T01: Definir y aprobar roles de error/advertencia/crítico

**Status:** Blocked

**Description:** Definir significado, texto, iconos, fondos/bordes y contraste de estados semánticos sin alterar cita/sync/comunicación.

**Acceptance criteria:**
- [ ] Cada rol tiene significado, uso/no uso y presentación no dependiente de color.
- [ ] Pares texto/fondo/foco/controles cumplen contraste objetivo en componentes reales.
- [ ] DESIGN se actualiza por decisión expresa antes de código.

**Verification:**
- [ ] Focused tests: contraste de pares y revisión de estados.
- [ ] Build or broader check: `git diff --check` para DESIGN y documentos.
- [ ] Manual check: aprobación expresa del usuario.

**Dependencies:** Decisión del usuario

**Files likely touched:**
- `DESIGN.md`
- `DESIGN-RULES.md`
- `docs/design-master-plan.md`

**Estimated scope:** M

## T02: Definir y aprobar paleta dark/system

**Status:** Blocked

**Description:** Diseñar roles dark/system y comportamiento de preferencia sin inversión mecánica del tema claro.

**Acceptance criteria:**
- [ ] Superficies/texto/acento/bordes/estados tienen tokens propios y `color-scheme` definido.
- [ ] Preferencia/cambio conserva contenido/foco y no altera significado de estados.
- [ ] DESIGN y alcance de implementación quedan aprobados antes de tokens.

**Verification:**
- [ ] Focused tests: contraste y muestras de componentes reales.
- [ ] Build or broader check: `git diff --check` documental.
- [ ] Manual check: aprobación del usuario; C01.

**Dependencies:** T01

**Files likely touched:**
- `DESIGN.md`
- `docs/design-master-plan.md`

**Estimated scope:** M

## C01: Después de T01–T02

- [ ] Especificación visual aprobada por el usuario; revisión de roles completa.

## T03: Implementar tokens y verificar todas las superficies

**Status:** Blocked

**Description:** Implementar tokens/tema aprobado y revisar componentes/estados/viewports/lectores de pantalla sin cambios semánticos implícitos.

**Acceptance criteria:**
- [ ] Tokens se consumen centralmente sin colores locales incompatibles.
- [ ] Texto, foco, controles, selección, disabled y gráficos cumplen contraste en render real.
- [ ] Preferencia/cambio y reduced motion funcionan sin pérdida de contexto; NOT CHECKED se registran.

**Verification:**
- [ ] Focused tests: `pnpm build`; contraste y estados reales.
- [ ] Build or broader check: `pnpm quality`.
- [ ] Manual check: viewports, zoom, teclado/foco y lector de pantalla disponible; revisión C02.

**Dependencies:** C01

**Files likely touched:**
- `src/App.css`
- Superficies que presenten valores cromáticos locales, identificadas por auditoría

**Estimated scope:** M

## C02: Después de T03

- [ ] `pnpm quality` y ampliación visual verificada en superficies reales.
- [ ] Accesibilidad/NOT CHECKED registrados; revisión del usuario.
