import { describe, expect, test } from "bun:test";

import { createCodexResetCreditConfirmation } from "../src/utils/codexResetConfirmation";

describe("Codex reset confirmation", () => {
  test("does not run the consume action before confirmation", () => {
    let consumeRequests = 0;
    const dialog = createCodexResetCreditConfirmation(
      (key) => key,
      () => { consumeRequests += 1; },
    );

    expect(consumeRequests).toBe(0);
    expect(dialog.tone).toBe("danger");
    expect(dialog.title).toBe("quota.resetCredits.confirmTitle");

    dialog.onConfirm();

    expect(consumeRequests).toBe(1);
  });
});
