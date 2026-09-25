import { describe, expect, test } from "bun:test";

import { createDefaultAnalyticsQuery } from "../src/utils/analyticsTimeZone";

describe("createDefaultAnalyticsQuery", () => {
  test("uses the latest seven calendar days in the selected time zone", () => {
    const query = createDefaultAnalyticsQuery(
      "Asia/Taipei",
      ["codex"],
      new Date("2026-09-24T02:00:00.000Z"),
    );

    expect(query.startDate).toBe("2026-09-18");
    expect(query.endDate).toBe("2026-09-24");
  });
});
