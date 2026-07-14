# Proposal: 狀態列 Quota 彈出面板、Dashboard 全部平台模式與 Codex 重置額度

## Why

1. **Codex 重置額度不可見**：Codex（ChatGPT 訂閱）提供「手動重置額度」（reset credits），有可用次數、每筆有效期限與狀態，但 SessionHub 目前只抓 rate limit windows，使用者必須另外用 [codex-reset-checker](https://github.com/doggy8088/codex-reset-checker) 之類的外部工具查詢。
2. **狀態列 quota chip 只能 hover**：目前狀態列的 quota chip 僅以原生 tooltip 顯示摘要，點擊無反應；tooltip 資訊量有限且無法互動，使用者想看完整額度資訊必須切換到 Dashboard。
3. **Dashboard QuotaOverview 一次只能看一個平台**：各 provider 分 tab 顯示，無法一眼掌握所有平台的額度狀況。

## What Changes

- **Codex 重置額度查詢**：Codex quota adapter 額外呼叫 `GET https://chatgpt.com/backend-api/wham/rate-limit-reset-credits`（沿用 auth.json 的 access_token / account_id），取得 `available_count`（可用次數）與 `credits[]`（每筆含 `granted_at` / `expires_at` / `status`），寫入 QuotaSnapshot 新欄位 `reset_credits`。
- **重置額度顯示**：
  - 狀態列 Codex chip 的 hover tooltip 增列重置額度摘要（可用次數、最近到期時間）。
  - Dashboard QuotaOverview 的 Codex 面板顯示重置額度區塊（可用次數、每筆期限倒數、是否可用）。
- **狀態列 quota 彈出面板**：點擊狀態列 quota 區域彈出一個錨定於狀態列上方的浮動面板，內容重用 Dashboard 的 QuotaOverview（tabs、視窗用量條、重置倒數、刷新按鈕），點擊面板外或再次點擊 chip 即關閉。
- **QuotaOverview「全部」tab**：在既有 provider tabs 之前新增「全部」tab，選取時同時垂直列出所有可見 provider 的面板；選擇會透過 localStorage 記憶（Dashboard 與彈出面板共用元件、各自記憶選取）。

## Capabilities

### New Capabilities

- `statusbar-quota-popup`: 點擊狀態列 quota 區域彈出浮動額度面板，重用 QuotaOverview 顯示完整額度資訊，支援點外關閉。
- `quota-overview-all-tab`: Dashboard QuotaOverview 新增「全部」tab，一次顯示所有可見 provider 的額度面板。

### Modified Capabilities

- `provider-quota-monitoring`: QuotaSnapshot 資料模型新增 `reset_credits` 欄位；Codex adapter 規格新增 rate-limit-reset-credits API 查詢（失敗時不影響既有 usage 查詢結果）。
- `global-status-bar`: quota chip 由純顯示改為可點擊（開關彈出面板）；Codex chip tooltip 增列重置額度摘要。

## Impact

- **Rust 後端**：
  - `src-tauri/src/types.rs` — 新增 `ResetCredits` / `ResetCreditEntry` struct，`QuotaSnapshot` 加 `reset_credits: Option<ResetCredits>`（serde camelCase）。
  - `src-tauri/src/quota/codex.rs` — 新增 reset-credits API 呼叫與解析；其餘 adapter 填 `None`。
- **前端**：
  - `src/types/index.ts` — 對應 TypeScript 型別。
  - `src/components/QuotaOverview.tsx` — 「全部」tab 模式、Codex 重置額度區塊。
  - `src/components/StatusBar.tsx` — quota 區域點擊事件、彈出面板容器、Codex tooltip 擴充。
  - `src/App.tsx` — 若彈出面板需要 refresh 回呼則由 App 下傳（既有 props 已具備 snapshots 與 refresh handler，預期不需新增 IPC）。
  - `src/App.css` 與 i18n locale 檔 — 面板樣式（遵循 sessionhub-minimal-ui token）與新增文案 key。
- **相依性**：無新增外部套件；沿用 `ureq` 與既有 Codex 憑證讀取邏輯。
- **與進行中變更的關係**：`tray-quota-widget`（系統匣 overlay）為獨立變更；本變更的彈出面板位於應用視窗內的狀態列，不與其衝突。
