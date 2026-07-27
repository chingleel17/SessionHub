import type { QuotaSnapshot } from "../types";

/**
 * 判斷 snapshot 是否不含任何可渲染的額度內容。
 * 條件：無 windows（null 或空陣列）且無 extraCredits、resetCredits。
 * 以通用述詞判斷，不依 provider 名稱硬編，任何無額度來源的 provider 皆適用。
 */
export function hasNoQuotaContent(snapshot: QuotaSnapshot): boolean {
  const hasWindows = (snapshot.windows?.length ?? 0) > 0;
  return !hasWindows && !snapshot.extraCredits && !snapshot.resetCredits;
}
