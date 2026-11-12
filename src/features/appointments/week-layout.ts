/** Geometría comprobable de la vista Semana (96 px/hora, columnas sin
 * solapamiento, rango visible que incluye los eventos conocidos). La
 * presentación consume estos cálculos; ninguna medida visual cambia
 * instantes ni duración reales. */

export const WEEK_HOUR_HEIGHT = 96; // px por hora; 48 px por media hora
export const WEEK_DAY_MIN_WIDTH = 160; // px mínimos por columna de día
export const WEEK_BASE_START_MINUTE = 7 * 60;
export const WEEK_BASE_END_MINUTE = 20 * 60;

export interface MinuteRange {
  startMinute: number;
  endMinute: number;
}

/** Intervalo visible: el horario base ampliado a horas enteras para incluir
 * los eventos conocidos, así una cita de madrugada aparece en su hora real
 * y no comprimida bajo otra. */
export function visibleMinuteRange(
  events: { startMin: number; endMin: number }[],
  baseStart: number = WEEK_BASE_START_MINUTE,
  baseEnd: number = WEEK_BASE_END_MINUTE,
): MinuteRange {
  let start = baseStart;
  let end = baseEnd;
  for (const event of events) {
    if (event.startMin < start) start = Math.floor(event.startMin / 60) * 60;
    if (event.endMin > end) end = Math.ceil(event.endMin / 60) * 60;
  }
  return { startMinute: Math.max(0, start), endMinute: Math.min(24 * 60, end) };
}

/** Caja vertical en px de un evento dentro del rango visible. */
export function eventBox(
  startMin: number,
  endMin: number,
  range: MinuteRange,
): { top: number; height: number } {
  const top = (Math.max(startMin, range.startMinute) - range.startMinute) * (WEEK_HOUR_HEIGHT / 60);
  const bottom = (Math.min(endMin, range.endMinute) - range.startMinute) * (WEEK_HOUR_HEIGHT / 60);
  return { top, height: Math.max(0, bottom - top) };
}

export interface Columned<T> {
  item: T;
  column: number;
  columns: number;
}

/** Distribuye eventos que se solapan en columnas internas para que no se
 * tapen unos a otros. */
export function assignColumns<T extends { startMin: number; endMin: number }>(
  items: T[],
): Columned<T>[] {
  const sorted = [...items].sort((a, b) => a.startMin - b.startMin || a.endMin - b.endMin);
  const placed: Columned<T>[] = [];
  let cluster: T[] = [];
  let clusterEnd = -1;

  const flush = () => {
    if (!cluster.length) return;
    const columnEnds: number[] = [];
    let total = 0;
    for (const item of cluster) {
      let column = columnEnds.findIndex((end) => end <= item.startMin);
      if (column === -1) {
        columnEnds.push(item.endMin);
        column = columnEnds.length - 1;
      } else {
        columnEnds[column] = item.endMin;
      }
      total = Math.max(total, column + 1);
      placed.push({ item, column, columns: 0 });
    }
    for (let i = placed.length - cluster.length; i < placed.length; i += 1) {
      placed[i].columns = total;
    }
    cluster = [];
  };

  for (const item of sorted) {
    if (cluster.length && item.startMin >= clusterEnd) flush();
    cluster.push(item);
    clusterEnd = Math.max(clusterEnd, item.endMin);
  }
  flush();
  return placed;
}
