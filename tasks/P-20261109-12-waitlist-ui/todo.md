# Task List: P-20261109-12-waitlist-ui

**Plan:** [plan.md](plan.md)
**Updated:** 2026-11-09

Plan Blocked: requiere dominio/contrato/política real.

## T01: Implementar entradas En espera y compatibilidad

**Status:** Blocked

**Description:** Construir lista de entradas reales con paciente, servicio, preferencias y compatibilidad sin prioridad inventada.

**Acceptance criteria:**
- [ ] Entradas muestran datos reales y no implican orden de asignación.
- [ ] Candidatos compatibles se derivan del contrato, no de filtros visuales arbitrarios.
- [ ] Vacío/offline/antigüedad se distinguen y no hay reserva desde la lista.

**Verification:**
- [ ] Focused tests: `pnpm build` y consultas del contrato disponible.
- [ ] Build or broader check: `pnpm quality`.
- [ ] Manual check: lista, preferencias, teclado y viewports.

**Dependencies:** Dominio/contrato/política de lista de espera

**Files likely touched:**
- `src/features/waitlist/WaitlistPage.tsx`
- Cliente contractual definido por el plan técnico

**Estimated scope:** M

## T02: Implementar Ofertas y revisión de candidatos/horario

**Status:** Blocked

**Description:** Añadir vista Ofertas con intervalo, vencimiento y resultado, y revisión de candidato/horario.

**Acceptance criteria:**
- [ ] Ofertas separan intervalo, candidato, vencimiento y respuesta conocida.
- [ ] Aceptar oferta expresa interés y no crea cita; revisar horario usa disponibilidad real.
- [ ] Oferta vencida/cambiada no se presenta como vigente.

**Verification:**
- [ ] Focused tests: `pnpm build`; oferta/respuesta/vencimiento reales.
- [ ] Build or broader check: `pnpm quality`.
- [ ] Manual check: comparación, teclado y retorno; revisión C01.

**Dependencies:** T01

**Files likely touched:**
- `src/features/waitlist/OffersPage.tsx`
- `src/features/waitlist/OfferDetail.tsx`

**Estimated scope:** M

## C01: Después de T01–T02

- [ ] `pnpm quality` y ofertas reales pasan; revisión del usuario.

## T03: Implementar asignación por personal y estados de reserva

**Status:** Blocked

**Description:** Solicitar asignación desde el personal y presentar validación/provisionalidad/conflictos.

**Acceptance criteria:**
- [ ] Asignar solicita caso nativo de cita y API valida oferta/disponibilidad.
- [ ] Éxito se presenta primero como operación pendiente, no reserva aceptada.
- [ ] Conflicto conserva oferta/propuesta y ofrece revisión sin forzar.

**Verification:**
- [ ] Focused tests: `pnpm build`; asignación real y conflicto disponible.
- [ ] Build or broader check: `pnpm quality`.
- [ ] Manual check: offline, accesibilidad y NOT CHECKED registrados; revisión C02.

**Dependencies:** C01

**Files likely touched:**
- `src/features/waitlist/OfferDetail.tsx`
- `src/features/appointments/AppointmentEditorPage.tsx`

**Estimated scope:** M

## C02: Después de T03

- [ ] `pnpm quality` y asignación real pasan con evidencia disponible.
- [ ] Accesibilidad/offline/NOT CHECKED registrados; revisión del usuario.
