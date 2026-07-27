## Why

Quota 面板底部的「本月（YYYY-MM）」本機估算用量與同一張卡片內其他數值口徑不一致：上方進度條來自 provider 官方 API 的實際額度，下方卻是 SessionHub 自行掃描本機 session 檔推估的 token 數，兩者並列易被誤讀為同一套數字。此外 Codex 的解析邏輯已因 rollout 檔案格式變更而失效（token 實際位於 `payload.info.last_token_usage`，程式碼讀取頂層 `usage`），導致長期顯示 `0k / 0k`。

該功能僅 codex 與 opencode 兩家具備，無法用於跨供應商比較；跨供應商的 token 總計屬於 Analytics 的職責（`get_analytics_data_internal` 已支援 day/week/month 分組）。Quota 面板應專注回答「還剩多少可用額度」，不混入「已用掉多少 token」。

## What Changes

- **BREAKING**（內部 IPC 契約）：移除 `QuotaSnapshot.local_tokens` 欄位與 `LocalTokenUsage` 型別（Rust 與 TypeScript 兩端）。既有 SQLite 快取中含該欄位的舊 JSON 需能繼續反序列化。
- 移除本機 token 掃描邏輯：`quota/codex.rs` 的 `count_monthly_tokens_from_jsonl`、`quota/opencode.rs` 的 `count_monthly_tokens_from_db` 與 `count_monthly_tokens_from_json`。
- 移除三處前端渲染：`QuotaOverview.tsx`、`TrayQuotaPanel.tsx`、`QuotaOverlay.tsx` 的本機估算區塊，以及 `.qo-local*` CSS、`quota.monitoring.localUsage` 翻譯鍵（zh-TW / en-US）、`.design-sync/previews/QuotaOverview.tsx` 的 mock 資料。
- 移除 `tray_icon.rs` tooltip 組裝中的 `local_tokens` 分支。
- **新增**「無額度資料」顯示狀態：OpenCode snapshot 的 `windows` 為 null 且 `local_tokens` 是其唯一內容，移除後卡片將完全空白。三處渲染點在 provider 無任何可顯示額度資料時，改顯示一行說明文字（新增翻譯鍵），避免使用者誤判為功能損壞。
- **保留** `source: "local_scan"` 標記及其翻譯鍵：opencode 與 antigravity 仍以此渲染資料來源徽章，與 `local_tokens` 是不同的概念，不可一併移除。

## Capabilities

### New Capabilities

（無）

### Modified Capabilities

- `provider-quota-monitoring`: 移除 snapshot 模型中的 `local_tokens` 欄位與 `LocalTokenUsage` 型別定義；改寫 OpenCode adapter 與 Codex adapter 的資料來源條款（不再掃描本機 token）；新增「provider 無可顯示額度資料時的呈現」要求。
- `tray-quota-widget`: 移除 tray tooltip 與 overlay 內容中關於 `LocalTokenUsage.period_label` 的條款；改寫「OpenCode Gateway 觸發上游 quota 更新」要求中的 local scan 場景；mini panel 顯示項目移除 local tokens。

## Impact

**後端** `src-tauri/src/`
- `types/quota.rs` — 移除 `LocalTokenUsage` struct 與 `QuotaSnapshot.local_tokens`
- `quota/codex.rs` — 移除本機掃描函式與其呼叫、`chrono::Datelike` 相關 import；測試 fixture 的 `"localTokens": null` 預期值
- `quota/opencode.rs` — 移除兩個掃描函式；adapter 回傳的 snapshot 結構
- `quota/claude.rs`、`quota/copilot.rs`、`quota/antigravity.rs` — 移除建構 snapshot 時的 `local_tokens: None`
- `tray_icon.rs` — tooltip 組裝分支與測試 fixture

**前端** `src/`
- `types/index.ts` — 移除 `LocalTokenUsage` 型別與 `QuotaSnapshot.localTokens`
- `components/QuotaOverview.tsx`、`components/TrayQuotaPanel.tsx`、`components/QuotaOverlay.tsx` — 移除渲染區塊、新增無資料提示
- `locales/zh-TW.ts`、`locales/en-US.ts` — 移除 `quota.monitoring.localUsage`、新增無額度資料提示鍵
- `App.css` — 移除 `.qo-local*` 規則
- `.design-sync/previews/QuotaOverview.tsx` — 移除 mock 的 `localTokens`

**相容性**：`QuotaSnapshot` 以 serde 反序列化 SQLite 中的舊快照 JSON，移除欄位後多餘的 `localTokens` 鍵會被忽略（serde 預設行為），不需 DB migration。
