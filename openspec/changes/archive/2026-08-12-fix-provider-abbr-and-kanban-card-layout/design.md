## Context

見 proposal.md - Why。實作已完成並以 Playwright 對實際渲染結果（真實字型、CSS 變數、`npm run dev` 頁面）驗證，本文件記錄關鍵技術決策供日後參考與維護。

## Goals / Non-Goals

**Goals:**
- Provider 縮寫代碼收斂為單一函式 `getProviderAbbr()`（`src/utils/providerLabel.ts`）
- ProjectCard 標頭在任意寬度下不溢出容器，且專案名稱恆可讀

**Non-Goals:**
- 不變更 provider 標籤的顏色系統（`.provider-tag--*` 樣式維持不變）
- 不變更 `ProviderIcon` 元件與 `provider-tag` 兩套 CSS class 命名的整合（維持既有兩套並存）
- 不處理 Copilot 縮寫代碼命名爭議以外的品牌顯示問題

## Decisions

### 縮寫代碼統一為 `CP`（而非 `GH`）

`copilot` 在部分舊定義中為 `GH`（可能源自 GitHub），但使用者回報與螢幕截圖中看到的是 `CP` 樣式的標籤，且 `ProviderIcon.tsx`、`DashboardView.tsx`、`SessionCard.tsx` 原本已用 `CP`。選擇 `CP` 而非 `GH`，理由：
- `CP` 直接對應 Copilot 名稱，`GH` 對應 GitHub 品牌但容易與其他 GitHub 相關功能混淆
- 影響面較小：只有 `QuotaOverlay.tsx`、`StatusBar.tsx` 兩處需要從 `GH` 改為 `CP`，其餘三處元件已是 `CP`

**替代方案**：統一為 `GH` — 理由是原始 `PROVIDER_ABBR`（Quota 相關兩檔）定義較完整（含 antigravity），但會讓多數既有畫面（Dashboard、SessionCard）的顯示改變，且與使用者回報的「CP 是對的」認知衝突，故不採用。

### ProjectCard 標頭空間分配優先序：專案名稱 > 分支名稱 > provider 標籤

以實測（Playwright 於真實 App 注入卡片並量測 `getBoundingClientRect`）得出的真實卡片寬度（約 258px）驗證：
1. 專案名稱：`min-width: 5ch`（CSS floor），任何情況下不可低於此寬度
2. 分支名稱：`min-width: 0`，空間不足時優先壓縮至完全消失（截斷為 0 寬度）
3. Provider 標籤：固定寬度不縮放（`flex-shrink: 0`），但顯示數量上限為 2 個，超過以 `+N` 摘要呈現並在 `title` 中列出完整清單

**替代方案 1**：讓 `.kanban-project-providers` 使用 `overflow: hidden` 自動裁切多餘標籤 — 曾短暫採用後放棄，因為裁切是無提示的資訊遺失（使用者看不出還有其他 provider），比原始 bug（擠壓專案名稱）更差。

**替代方案 2**：分支名稱給固定 `max-width: 40%` 上限 — 以數學驗證會在窄卡片（如 220px）與長分支名同時發生時仍導致專案名稱寬度為負值（溢出），故改為 `min-width: 0` 的彈性壓縮，讓瀏覽器依實際內容動態分配。

### 驗證方式：Playwright 注入真實 App，而非純 CSS 靜態模型

初版以 Python 手算 flex 寬度分配，但固定元素寬度（count、date、goto、tag 寬度）為推測值，不可信。改為啟動 `npm run dev`，用 Playwright 在真實頁面的 `.kanban-column` 內注入測試卡片 HTML，量測實際渲染寬度（含真實字型、`--color-*` CSS 變數），才發現「3 個以上 provider 標籤即會裁切」是先前模型未能預測的真實邊界情況。

## Risks / Trade-offs

- [Risk] `+N` 摘要在 provider 數量固定為 2 上限，若日後新增更多 provider 導致同專案常態使用 3+ 工具，多數卡片會顯示 `+N`，摘要資訊density 下降 → Mitigation: 上限值集中定義於 `PROVIDERS_PER_PROJECT_CARD` 常數，未來可依實際卡片寬度或使用回饋調整，不需改動排版邏輯
- [Risk] Copilot 縮寫代碼從 `GH` 改為 `CP` 是本次變更中使用者未明確要求的可見變更（僅限 Quota Overlay 與狀態列兩處）→ Mitigation: 已於 proposal 與交付說明中明確標示，日後如需改回 `GH` 只需調整 `PROVIDER_ABBR` 常數一處

## Migration Plan

無資料遷移需求，純前端顯示邏輯調整，隨版本發布即生效，無需回滾腳本。若需回滾，還原對應 commit 即可。
