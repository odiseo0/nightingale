import { test } from "node:test";
import assert from "node:assert/strict";
import {
  needsIntervention,
  reviewRoute,
} from "../../src/features/sync/presentation.ts";

test("Solo conflicto, rechazo y revisión requerida exigen intervención", () => {
  assert.equal(needsIntervention("conflict"), true);
  assert.equal(needsIntervention("rejected"), true);
  assert.equal(needsIntervention("review_required"), true);
  assert.equal(needsIntervention("pending"), false);
  assert.equal(needsIntervention("in_flight"), false);
  assert.equal(needsIntervention("retryable"), false);
  assert.equal(needsIntervention("unknown"), false);
  assert.equal(needsIntervention("accepted"), false);
});

test("El resultado desconocido sigue automático: no se lista ni se duplica", () => {
  // unknown se recupera con la misma identidad; la clasificación lo trata
  // como espera, no como intervención.
  assert.equal(needsIntervention("unknown"), false);
});

test("Las citas enlazan por su clave de entidad", () => {
  assert.equal(reviewRoute("appointment", "019a1234-0000-7000-8000-000000000010"), "/cita/019a1234-0000-7000-8000-000000000010");
  assert.equal(reviewRoute("appointment", { appointmentId: "a1" }), "/cita/a1");
});

test("Las fichas enlazan por su clave de entidad", () => {
  assert.equal(reviewRoute("patient_profile", "p1"), "/paciente/p1");
  assert.equal(reviewRoute("patient_profile", { patientId: "p2" }), "/paciente/p2");
});

test("Sin ruta específica no se inventa enlace", () => {
  assert.equal(reviewRoute("schedule_rule", "r1"), null);
  assert.equal(reviewRoute("appointment", ""), null);
  assert.equal(reviewRoute("appointment", null), null);
  assert.equal(reviewRoute("patient_identifier", { identifierId: "x" }), null);
});
