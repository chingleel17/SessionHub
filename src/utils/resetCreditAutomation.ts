import type { MessageKey } from "../locales/zh-TW";
import type { AppSettings, ResetCreditAutomationStatus } from "../types";

const messages: Record<string, MessageKey> = {
  disabled: "resetCredit.state.disabled", ready: "resetCredit.state.ready",
  paused: "resetCredit.state.paused", running: "resetCredit.state.running",
  reset: "quota.resetCredits.outcome.reset", nothing_to_reset: "quota.resetCredits.outcome.nothing_to_reset",
  no_credit: "quota.resetCredits.outcome.no_credit", already_redeemed: "quota.resetCredits.outcome.already_redeemed",
  no_auth: "resetCredit.reason.noAuth", account_changed: "resetCredit.reason.accountChanged",
  monitoring_disabled: "resetCredit.reason.monitoring", unknown: "resetCredit.reason.unknown",
  transient_error: "resetCredit.reason.network", invalid_response: "resetCredit.reason.invalidResponse",
  unsupported: "resetCredit.reason.unsupported", rate_limited: "resetCredit.reason.rateLimited",
  inconsistent: "resetCredit.reason.inconsistent", unknown_expiry: "resetCredit.reason.unknownExpiry",
  selection_unverified: "resetCredit.reason.selection", missed: "resetCredit.reason.missed",
  storage_error: "resetCredit.reason.storage", refresh_required: "resetCredit.reason.refresh",
  request_mismatch: "resetCredit.reason.request", invalid_request: "resetCredit.reason.request",
  interrupted: "resetCredit.reason.unknown", batch_processed: "resetCredit.reason.processed",
  notification_denied: "resetCredit.reason.notificationDenied", notification_failed: "resetCredit.reason.notificationFailed",
  dispatched: "resetCredit.notice.submitted", claimed: "resetCredit.notice.claimed",
  checked_empty: "resetCredit.notice.empty", waiting: "resetCredit.notice.waiting",
  cancelled: "resetCredit.state.disabled", failed: "resetCredit.reason.failed",
};

export function resetCreditMessage(t: (key: MessageKey) => string, code: string): string {
  return t(messages[code] ?? "resetCredit.reason.failed");
}

export function validResetCreditSettings(settings: AppSettings): boolean {
  const time = settings.codexResetCreditReminderTime ?? "09:00";
  const days = settings.codexResetCreditReminderDaysBefore ?? 0;
  return /^([01]\d|2[0-3]):[0-5]\d$/.test(time) && Number.isInteger(days) && days >= 0 && days <= 30;
}

export function newerResetCreditStatus(current: ResetCreditAutomationStatus | undefined, incoming: ResetCreditAutomationStatus): ResetCreditAutomationStatus {
  return current && current.generation > incoming.generation ? current : incoming;
}

/** 其他功能的自動保存不得順便啟用尚未儲存的重置券設定。 */
export function resetCreditSettingsForSave(form: AppSettings, saved: AppSettings | undefined, includeDraft: boolean) {
  const source = includeDraft ? form : saved;
  return {
    codexResetCreditReminderEnabled: source?.codexResetCreditReminderEnabled ?? false,
    codexResetCreditReminderTime: source?.codexResetCreditReminderTime ?? "09:00",
    codexResetCreditReminderDaysBefore: source?.codexResetCreditReminderDaysBefore ?? 0,
    codexResetCreditAutoUseEnabled: source?.codexResetCreditAutoUseEnabled ?? false,
  };
}
