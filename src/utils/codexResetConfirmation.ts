import type { MessageKey } from "../locales/zh-TW";
import type { ConfirmDialogState } from "../types";

type Translate = (key: MessageKey) => string;

export function createCodexResetCreditConfirmation(
  t: Translate,
  onConfirm: () => void,
): ConfirmDialogState {
  return {
    title: t("quota.resetCredits.confirmTitle"),
    message: t("quota.resetCredits.confirmMessage"),
    actionLabel: t("quota.resetCredits.use"),
    tone: "danger",
    onConfirm,
  };
}
