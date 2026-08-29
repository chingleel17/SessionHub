import { describe, expect, test } from "bun:test";

import {
  loadColumnWidths,
  normalizeColumnWidths,
  resizeAdjacentColumns,
  saveColumnWidths,
} from "../src/components/DashboardView";

const DEFAULT_WIDTHS = [25, 25, 25, 25];

describe("normalizeColumnWidths", () => {
  test("保留合法欄寬", () => {
    expect(normalizeColumnWidths([20, 30, 15, 35])).toEqual([20, 30, 15, 35]);
  });

  test.each([
    ["缺少欄位", [30, 30, 40]],
    ["包含非有限值", [25, Number.NaN, 25, 25]],
    ["低於最小值", [9, 31, 30, 30]],
    ["總和失衡", [20, 20, 20, 20]],
  ])("%s 時恢復平均欄寬", (_name, widths) => {
    expect(normalizeColumnWidths(widths)).toEqual(DEFAULT_WIDTHS);
  });

  test("修正容許範圍內的浮點誤差", () => {
    const normalized = normalizeColumnWidths([20, 30, 15, 34.9999]);

    expect(normalized.reduce((sum, width) => sum + width, 0)).toBe(100);
    expect(normalized.every((width) => width >= 10)).toBe(true);
  });
});

describe("resizeAdjacentColumns", () => {
  test.each([
    ["左側", -100, [10, 40, 25, 25]],
    ["右側", 100, [40, 10, 25, 25]],
  ])("拖到%s極限時限制相鄰欄位", (_name, deltaPercent, expected) => {
    const resized = resizeAdjacentColumns(DEFAULT_WIDTHS, 0, deltaPercent);

    expect(resized).toEqual(expected);
    expect(resized.every((width) => width >= 10)).toBe(true);
    expect(resized.reduce((sum, width) => sum + width, 0)).toBe(100);
  });
});

describe("column width persistence", () => {
  test("異常儲存值載入後恢復並持久化為平均欄寬", () => {
    const storage = new Map<string, string>([
      ["sessionhub.kanban.columnWidths", JSON.stringify([80, 5, 5, 10])],
    ]);
    Object.defineProperty(globalThis, "localStorage", {
      configurable: true,
      value: {
        getItem: (key: string) => storage.get(key) ?? null,
        setItem: (key: string, value: string) => storage.set(key, value),
      },
    });

    const loaded = loadColumnWidths();
    saveColumnWidths(loaded);

    expect(loaded).toEqual(DEFAULT_WIDTHS);
    expect(storage.get("sessionhub.kanban.columnWidths")).toBe(JSON.stringify(DEFAULT_WIDTHS));
  });
});
