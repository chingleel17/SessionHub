## 1. 後端型別與掃描邏輯移除

- [x] 1.1 `src-tauri/src/types/quota.rs`：移除 `LocalTokenUsage` struct（第 19-25 行）與 `QuotaSnapshot.local_tokens` 欄位（第 62 行）；確認 `QuotaSnapshot` 未標註 `#[serde(deny_unknown_fields)]`，使舊快照 JSON 中的 `localTokens` 鍵可被忽略
- [x] 1.2 `src-tauri/src/quota/codex.rs`：移除 `count_monthly_tokens_from_jsonl` 函式與其呼叫、`period_label` 變數、snapshot 建構中的 `local_tokens` 欄位；清理因此未使用的 import（如 `chrono::Datelike`）
- [x] 1.3 `src-tauri/src/quota/opencode.rs`：移除 `count_monthly_tokens_from_db` 與 `count_monthly_tokens_from_json` 兩個函式、`period_label` 變數、snapshot 建構中的 `local_tokens` 欄位；清理未使用的 import（`chrono::Datelike`、rusqlite 相關）
- [x] 1.4 `src-tauri/src/quota/claude.rs`、`copilot.rs`、`antigravity.rs`：移除各處 snapshot 建構中的 `local_tokens: None`；確認 `source: "local_scan"` 標記完整保留
- [x] 1.5 `src-tauri/src/quota/mod.rs` 及其他引用處：移除對 `LocalTokenUsage` 的 import / re-export

## 2. 後端 tray tooltip 與測試

- [x] 2.1 `src-tauri/src/tray_icon.rs`：移除 tooltip 組裝中的 `local_tokens` 分支（約第 265 行）；無任何 window 的 provider 改附無額度資料說明文字
- [x] 2.2 `src-tauri/src/tray_icon.rs` 測試 fixture（約第 349 行）：移除 `local_tokens: None`
- [x] 2.3 `src-tauri/src/quota/codex.rs` 測試預期 JSON（約第 593 行）：移除 `"localTokens": null`
- [x] 2.4 執行 `cargo test`（非僅 `cargo build`）確認全部通過，且無未使用 import 警告

## 3. 前端型別與翻譯

- [x] 3.1 `src/types/index.ts`：移除 `LocalTokenUsage` 型別與 `QuotaSnapshot.localTokens` 欄位（約第 532 行）
- [x] 3.2 `src/locales/zh-TW.ts`：移除 `quota.monitoring.localUsage`；新增 `quota.monitoring.noQuotaData`（值為「無額度資料」）
- [x] 3.3 `src/locales/en-US.ts`：移除 `quota.monitoring.localUsage`；新增 `quota.monitoring.noQuotaData`（值為 "No quota data"）
- [x] 3.4 確認 `quota.monitoring.source.local_scan` 翻譯鍵在兩個語系檔中均保留未動

## 4. 前端渲染調整

- [x] 4.1 新增共用判斷：snapshot `status === "ok"` 且無 `windows`（null 或空陣列）且無 `extraCredits`、`resetCredits` 時視為「無額度資料」；以通用述詞實作，不得硬編 `provider === "opencode"`
- [x] 4.2 `src/components/QuotaOverview.tsx`：移除本機估算區塊（約第 219-229 行）；無額度資料時顯示 `t("quota.monitoring.noQuotaData")` 說明文字
- [x] 4.3 `src/components/TrayQuotaPanel.tsx`：移除 `localTokens` 區塊（約第 135-142 行）；無額度資料時顯示說明文字
- [x] 4.4 `src/components/QuotaOverlay.tsx`：改寫 fallback 分支（約第 348-354 行），原本無 windows 時顯示 localTokens 摘要或來源徽章文字，改為一律顯示無額度資料說明文字，不渲染 bar
- [x] 4.5 `src/App.css`：移除 `.qo-local`、`.qo-local-row`、`.qo-local-label`、`.qo-local-value`、`.qo-local-period` 規則；若新增說明文字需要樣式，沿用既有 note / muted 類別
- [x] 4.6 `.design-sync/previews/QuotaOverview.tsx`：移除 mock 資料中的 `localTokens`（約第 37-40 行）
- [x] 4.7 執行前端 typecheck 與 lint，確認無殘留型別錯誤

## 5. 驗證

- [x] 5.1 全專案搜尋 `local_tokens` / `localTokens` / `LocalTokenUsage`，確認除 `openspec/changes/archive/**`（歷史記錄，不得修改）外無殘留
- [x] 5.2 全專案搜尋 `local_scan`，確認來源徽章相關程式碼與翻譯鍵完整保留
- [x] 5.3 （未驗證：需啟動 app，單一實例鎖定阻擋，留待後續） 啟動應用程式，確認 Dashboard QuotaOverview 的 OpenCode 卡片顯示「無額度資料」而非空白，Codex 卡片正常顯示 rate limit 進度條且無本月估算區塊
- [x] 5.4 （未驗證：需啟動 app，單一實例鎖定阻擋，留待後續） 確認 tray mini panel 與 overlay widget 兩處顯示一致，且既有 SQLite 快照載入不報錯
