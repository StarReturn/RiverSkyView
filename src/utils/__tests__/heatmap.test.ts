import { describe, expect, it } from "vitest";
import { buildYearHeatmapData } from "@/utils/heatmap";

describe("buildYearHeatmapData", () => {
  it("always returns a complete 365-day range for an empty project", () => {
    const result = buildYearHeatmapData([], new Date(2026, 6, 15));
    expect(result.data).toHaveLength(365);
    expect(result.data[result.data.length - 1]).toEqual(["2026-07-15", 0]);
  });

  it("fills missing dates with zero and preserves task counts", () => {
    const result = buildYearHeatmapData(
      [{ date: "2026-07-14", count: 3 }],
      new Date(2026, 6, 15),
    );
    expect(result.data[result.data.length - 2]).toEqual(["2026-07-14", 3]);
    expect(result.data[result.data.length - 1]).toEqual(["2026-07-15", 0]);
  });
});
