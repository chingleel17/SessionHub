import { describe, expect, test } from "bun:test";
import { withTimeout } from "../src/utils/withTimeout";

describe("withTimeout", () => {
  test("未完成的掃描會結束等待，且後續重試能成功", async () => {
    await expect(withTimeout(new Promise<never>(() => {}), 5, "scan timeout"))
      .rejects.toThrow("scan timeout");
    await expect(withTimeout(Promise.resolve("retried"), 100, "scan timeout"))
      .resolves.toBe("retried");
  });

  test("保留掃描錯誤", async () => {
    const error = new Error("scan failed");
    await expect(withTimeout(Promise.reject(error), 100, "scan timeout"))
      .rejects.toBe(error);
  });

  test("逾時後的舊結果不會覆蓋失敗狀態", async () => {
    let complete: (value: string) => void = () => {};
    const request = new Promise<string>((resolve) => { complete = resolve; });
    const result = withTimeout(request, 5, "scan timeout");
    await expect(result).rejects.toThrow("scan timeout");
    complete("late result");
    await expect(result).rejects.toThrow("scan timeout");
  });
});
