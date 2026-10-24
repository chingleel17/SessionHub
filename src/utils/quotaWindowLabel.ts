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
  if (messageKey) return t(messageKey);
  // 後端對非固定視窗產生 "30d" / "12h" 這類動態標籤，依語系換成對應單位
  const match = /^(\d+)([dh])$/.exec(rawLabel);
  if (!match) return rawLabel;
  const unit = t(match[2] === "d" ? "quota.unit.day" : "quota.unit.hour").trim();
  return `${match[1]}${unit}`;
}
