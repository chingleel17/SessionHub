import { describe, expect, test } from "bun:test";

import { zhTwMessages } from "../src/locales/zh-TW";
import { quotaPlanLabel } from "../src/utils/quotaPlanLabel";

const t = (key: keyof typeof zhTwMessages, params?: Record<string, string | number>) => {
  const raw: string = zhTwMessages[key];
  return params ? Object.entries(params).reduce<string>((text, [name, value]) => text.replace(`{${name}}`, String(value)), raw) : raw;
};

describe("quotaPlanLabel", () => {
  test("Copilot 只將已知內部類型映射為 Pro 與 Pro+", () => {
    const result = quotaPlanLabel("copilot", "individual", t);
    expect(result.label).toBe("Copilot Pro");
    expect(result.title).toContain("individual");
    expect(quotaPlanLabel("copilot", "individual_pro", t).label).toBe("Copilot Pro+");
    expect(quotaPlanLabel("copilot", "individual_unknown", t).label).toBe("個人方案");
  });

  test("Codex ProLite 顯示推定方案並保留原始分類", () => {
    const result = quotaPlanLabel("codex", "self_serve_business_prolite", t);
    expect(result.label).toBe("Business Premium");
    expect(result.title).toContain("self_serve_business_prolite");
    expect(result.title).toContain("內部類型");
    expect(quotaPlanLabel("codex", "self_serve_business_usage_based", t).label).toBe("商業帳戶");
  });

  test("Claude 已取得的方案名稱照實顯示", () => {
    expect(quotaPlanLabel("claude", "max", t).label).toBe("max");
  });
});
