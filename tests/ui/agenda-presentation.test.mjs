import { test } from "node:test";
import assert from "node:assert/strict";
import {
  formatAddressLine,
  formatTimeEnd,
  historyActionLabel,
  showsProfessionalColumn,
} from "../../src/features/appointments/presentation.ts";

test("Profesional se oculta solo para el dueño con un único profesional", () => {
  assert.equal(showsProfessionalColumn("owner", 1), false);
  assert.equal(showsProfessionalColumn("owner", 0), false);
  assert.equal(showsProfessionalColumn("owner", 2), true);
  assert.equal(showsProfessionalColumn("assistant", 1), true);
  assert.equal(showsProfessionalColumn("assistant", 2), true);
  assert.equal(showsProfessionalColumn(null, 1), true);
});

test("El fin de intervalo a medianoche se presenta como 24:00", () => {
  assert.equal(formatTimeEnd("UTC", "2026-11-10T00:00:00Z"), "24:00");
  assert.equal(formatTimeEnd("UTC", "2026-11-10T14:30:00Z"), "14:30");
});

test("La dirección de lectura termina en ciudad y estado, sin país", () => {
  assert.equal(
    formatAddressLine({ address: "Calle 5, Edificio Médico", city: "Barquisimeto", state: "Lara", }),
    "Calle 5, Edificio Médico, Barquisimeto, Lara",
  );
  assert.equal(formatAddressLine({ address: "", city: "Barquisimeto", state: "Lara" }), "Barquisimeto, Lara");
  assert.equal(formatAddressLine({ address: null, city: null, state: null }), "");
  assert.equal(formatAddressLine({ address: "Av. 20", city: null, state: "Lara" }), "Av. 20, Lara");
});

test("La actividad de historia se traduce a acciones comprensibles", () => {
  assert.equal(historyActionLabel.created, "Cita creada");
  assert.equal(historyActionLabel.rescheduled, "Cita reprogramada");
  assert.equal(historyActionLabel.no_show, "Ausencia registrada");
});
