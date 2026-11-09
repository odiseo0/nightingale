# Task List: P-20261109-14-metrics-ui

**Plan:** [plan.md](plan.md)
**Updated:** 2026-11-09

Plan Blocked: requiere servicio/contrato analítico.

## T01: Implementar periodo e indicadores con denominadores

**Status:** Blocked

**Description:** Construir S10 con selector de periodo real y métricas con valor, unidad, numerador/denominador, actualización y definición.

**Acceptance criteria:**
- [ ] Métricas usan datos aceptados del servicio, sin cálculos locales con pendientes.
- [ ] Periodo/zona/actualización/definiciones son visibles y accesibles.
- [ ] Valores cero/ausentes se distinguen de datos indisponibles.

**Verification:**
- [ ] Focused tests: `pnpm build` y lectura del contrato analítico.
- [ ] Build or broader check: `pnpm quality`.
- [ ] Manual check: interpretación, teclado y viewports.

**Dependencies:** Servicio/contrato analítico y definiciones

**Files likely touched:**
- `src/features/metrics/MetricsPage.tsx`
- Cliente contractual definido por el plan técnico

**Estimated scope:** M

## T02: Implementar visualización y alternativa tabular

**Status:** Blocked

**Description:** Añadir gráfico sobrio con escala/etiquetas y tabla equivalente para citas/resultados del periodo.

**Acceptance criteria:**
- [ ] Gráfico conserva unidades, escala, periodo y color secundario sin ser único canal.
- [ ] Tabla/texto equivalente contiene los mismos valores y denominadores.
- [ ] No se muestran mejoras sin línea base ni datos clínicos inferidos.

**Verification:**
- [ ] Focused tests: `pnpm build`; equivalencia gráfico/tabla con datos ficticios identificados.
- [ ] Build or broader check: `pnpm quality`.
- [ ] Manual check: contraste, zoom, lector de pantalla disponible; revisión C01.

**Dependencies:** T01

**Files likely touched:**
- `src/features/metrics/MetricsChart.tsx`
- `src/features/metrics/MetricsTable.tsx`

**Estimated scope:** M

## C01: Después de T01–T02

- [ ] `pnpm quality` y datos/visualización reales pasan; revisión del usuario.

## T03: Cubrir sin datos/desactualizado/indisponible y accesibilidad

**Status:** Blocked

**Description:** Implementar estados de ausencia, antigüedad y fallo del servicio sin bloquear Agenda.

**Acceptance criteria:**
- [ ] Sin datos, desactualizado e indisponible tienen mensajes/acciones distintos.
- [ ] Última vista muestra antigüedad sin afirmar actualidad.
- [ ] Fallo analítico enlaza a Agenda y no borra contenido previo.

**Verification:**
- [ ] Focused tests: `pnpm build`; estados reales disponibles.
- [ ] Build or broader check: `pnpm quality`.
- [ ] Manual check: accesibilidad/offline y NOT CHECKED registrados; revisión C02.

**Dependencies:** C01

**Files likely touched:**
- `src/features/metrics/MetricsPage.tsx`
- `src/features/metrics/presentation.ts`

**Estimated scope:** M

## C02: Después de T03

- [ ] `pnpm quality` y estados analíticos pasan con evidencia disponible.
- [ ] Accesibilidad/offline/NOT CHECKED registrados; revisión del usuario.
