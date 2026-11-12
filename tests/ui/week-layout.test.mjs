import { test } from "node:test";
import assert from "node:assert/strict";
import {
  assignColumns,
  eventBox,
  visibleMinuteRange,
  WEEK_BASE_END_MINUTE,
  WEEK_BASE_START_MINUTE,
  WEEK_HOUR_HEIGHT,
} from "../../src/features/appointments/week-layout.ts";

test("El rango visible cubre el horario base", () => {
  const range = visibleMinuteRange([]);
  assert.equal(range.startMinute, WEEK_BASE_START_MINUTE);
  assert.equal(range.endMinute, WEEK_BASE_END_MINUTE);
});

test("Una cita de madrugada amplía el rango a su hora real", () => {
  const range = visibleMinuteRange([{ startMin: 60, endMin: 90 }]);
  assert.equal(range.startMinute, 60);
  assert.equal(range.endMinute, WEEK_BASE_END_MINUTE);
});

test("Una cita nocturna amplía el fin hasta la hora entera siguiente", () => {
  const range = visibleMinuteRange([{ startMin: 21 * 60 + 15, endMin: 21 * 60 + 45 }]);
  assert.equal(range.startMinute, WEEK_BASE_START_MINUTE);
  assert.equal(range.endMinute, 22 * 60);
});

test("El rango nunca sale del día civil", () => {
  const range = visibleMinuteRange([{ startMin: -30, endMin: 25 * 60 }]);
  assert.equal(range.startMinute, 0);
  assert.equal(range.endMinute, 24 * 60);
});

test("La escala es 96 px por hora y 48 px por media hora", () => {
  const range = visibleMinuteRange([]);
  const halfHour = eventBox(9 * 60, 9 * 60 + 30, range);
  assert.equal(halfHour.height, WEEK_HOUR_HEIGHT / 2);
  const twoHours = eventBox(8 * 60, 10 * 60, range);
  assert.equal(twoHours.height, 2 * WEEK_HOUR_HEIGHT);
  assert.equal(twoHours.top, WEEK_HOUR_HEIGHT); // 8:00 está una hora tras 7:00
});

test("Los eventos solapados se distribuyen en columnas sin taparse", () => {
  const placed = assignColumns([
    { key: "a", startMin: 9 * 60, endMin: 10 * 60 },
    { key: "b", startMin: 9 * 60 + 30, endMin: 11 * 60 },
    { key: "c", startMin: 10 * 60 + 30, endMin: 11 * 60 + 30 },
  ]);
  const byKey = new Map(placed.map((p) => [p.item.key, p]));
  assert.equal(byKey.get("a").column, 0);
  assert.equal(byKey.get("b").column, 1);
  // c no solapa con a, así que reutiliza la primera columna.
  assert.equal(byKey.get("c").column, 0);
  assert.equal(byKey.get("a").columns, 2);
  assert.equal(byKey.get("b").columns, 2);
  assert.equal(byKey.get("c").columns, 2);
});

test("Los eventos consecutivos ocupan una sola columna", () => {
  const placed = assignColumns([
    { key: "a", startMin: 9 * 60, endMin: 10 * 60 },
    { key: "b", startMin: 10 * 60, endMin: 11 * 60 },
  ]);
  assert.equal(placed[0].column, 0);
  assert.equal(placed[1].column, 0);
  assert.equal(placed[0].columns, 1);
});
