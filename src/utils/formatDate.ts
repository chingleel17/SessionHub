/**
 * 將 ISO 時間字串格式化為人類可讀的日期時間字串。
 *
 * @param iso - ISO 格式的時間字串（如 "2026-03-31T12:26:47.872Z"），可為 null 或 undefined
 * @param locale - BCP 47 語系標籤，例如 "zh-TW" 或 "en-US"
 * @returns 格式化後的日期時間字串，無效輸入回傳 "-"
 */
export function formatDateTime(
  iso: string | null | undefined,
  locale: string,
): string {
  if (!iso) return "-";

  const date = new Date(iso);
  if (isNaN(date.getTime())) return "-";

  return date.toLocaleString(locale, {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    hour12: false,
  });
}

/**
 * 將 ISO 時間字串格式化為僅含日期的字串（不含時分）。
 *
 * @param iso - ISO 格式的時間字串，可為 null 或 undefined
 * @param locale - BCP 47 語系標籤，例如 "zh-TW" 或 "en-US"
 * @returns 格式化後的日期字串（如 "2026/07/21"），無效輸入回傳 "-"
 */
export function formatDate(
  iso: string | null | undefined,
  locale: string,
): string {
  if (!iso) return "-";

  const date = new Date(iso);
  if (isNaN(date.getTime())) return "-";

  return date.toLocaleDateString(locale, {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
  });
}
