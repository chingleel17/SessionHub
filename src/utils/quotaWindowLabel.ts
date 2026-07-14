import type { MessageKey } from "../locales/zh-TW";

const WINDOW_KEY_MAP: Record<string, MessageKey> = {
  five_hour: "quota.window.fiveHour",
  primary: "quota.window.fiveHour",
  "5h": "quota.window.fiveHour",
  seven_day: "quota.window.sevenDay",
  secondary: "quota.window.sevenDay",
  weekly: "quota.window.sevenDay",
  "7d": "quota.window.sevenDay",
  seven_day_sonnet: "quota.window.sevenDaySonnet",
  seven_day_opus: "quota.window.sevenDayOpus",
  seven_day_fable: "quota.window.sevenDayFable",
};

// provider-aware 視窗標籤本地化：copilot 的 primary/secondary 語意是 Premium/Chat（非時間視窗），維持原字串
export function localizedWindowLabel(
  provider: string,
  windowKey: string,
  rawLabel: string,
  t: (key: MessageKey) => string,
): string {
  if (provider === "copilot") return rawLabel;
  const messageKey = WINDOW_KEY_MAP[windowKey];
  return messageKey ? t(messageKey) : rawLabel;
}
