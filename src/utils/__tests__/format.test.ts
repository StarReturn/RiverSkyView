import { describe, it, expect } from "vitest";
import {
  formatRelativeTime,
  formatDate,
  formatDateTime,
  formatFileSize,
  truncatePath,
} from "../format";

describe("format - formatRelativeTime", () => {
  it("returns 暂无活动 for null", () => {
    expect(formatRelativeTime(null)).toBe("暂无活动");
  });

  it("returns 刚刚 for very recent time", () => {
    const now = new Date().toISOString();
    expect(formatRelativeTime(now)).toBe("刚刚");
  });

  it("returns minutes ago", () => {
    const tenMinAgo = new Date(Date.now() - 10 * 60000).toISOString();
    expect(formatRelativeTime(tenMinAgo)).toContain("分钟前");
  });

  it("returns hours ago", () => {
    const twoHoursAgo = new Date(Date.now() - 2 * 3600000).toISOString();
    expect(formatRelativeTime(twoHoursAgo)).toContain("小时前");
  });

  it("returns days ago", () => {
    const threeDaysAgo = new Date(Date.now() - 3 * 86400000).toISOString();
    expect(formatRelativeTime(threeDaysAgo)).toContain("天前");
  });
});

describe("format - formatDate", () => {
  it("formats date correctly", () => {
    expect(formatDate("2026-07-15T10:00:00+08:00")).toBe("2026-07-15");
  });

  it("returns empty string for null", () => {
    expect(formatDate(null)).toBe("");
  });
});

describe("format - formatDateTime", () => {
  it("formats datetime correctly", () => {
    const result = formatDateTime("2026-07-15T14:32:00+08:00");
    expect(result).toContain("2026-07-15");
    expect(result).toContain("14:32");
  });

  it("returns empty string for null", () => {
    expect(formatDateTime(null)).toBe("");
  });
});

describe("format - formatFileSize", () => {
  it("formats bytes", () => {
    expect(formatFileSize(500)).toBe("500 B");
  });

  it("formats kilobytes", () => {
    expect(formatFileSize(1024)).toBe("1.0 KB");
  });

  it("formats megabytes", () => {
    expect(formatFileSize(1024 * 1024 * 5)).toBe("5.0 MB");
  });

  it("formats gigabytes", () => {
    expect(formatFileSize(1024 * 1024 * 1024 * 2)).toBe("2.0 GB");
  });

  it("returns empty for null", () => {
    expect(formatFileSize(null)).toBe("");
  });
});

describe("format - truncatePath", () => {
  it("returns short paths unchanged", () => {
    expect(truncatePath("D:\\Projects\\Test", 50)).toBe("D:\\Projects\\Test");
  });

  it("truncates long paths", () => {
    const longPath = "D:\\Very\\Long\\Path\\That\\Exceeds\\The\\Maximum\\Allowed\\Length\\For\\Display";
    const result = truncatePath(longPath, 30);
    expect(result.length).toBeLessThanOrEqual(33);
    expect(result).toContain("...");
  });
});
