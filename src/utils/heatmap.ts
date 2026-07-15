import type { HeatmapCell } from "@/types";

export function toLocalDateString(date: Date): string {
  return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, "0")}-${String(date.getDate()).padStart(2, "0")}`;
}

export function buildYearHeatmapData(
  cells: HeatmapCell[],
  currentDate = new Date(),
): { rangeStart: Date; today: Date; data: [string, number][] } {
  const today = new Date(currentDate);
  today.setHours(0, 0, 0, 0);
  const rangeStart = new Date(today);
  rangeStart.setDate(rangeStart.getDate() - 364);
  const counts = new Map(cells.map((cell) => [cell.date, cell.count]));
  const data: [string, number][] = [];
  for (let offset = 0; offset <= 364; offset++) {
    const date = new Date(rangeStart);
    date.setDate(rangeStart.getDate() + offset);
    const dateString = toLocalDateString(date);
    data.push([dateString, counts.get(dateString) || 0]);
  }
  return { rangeStart, today, data };
}

