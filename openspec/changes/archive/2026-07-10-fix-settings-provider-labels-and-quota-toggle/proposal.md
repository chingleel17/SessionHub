## Why

設定頁的 provider integration 卡片將所有 provider 的設定檔位置統一標示為「設定 / plugin 路徑」，但 Claude Code 顯示的其實是 hooks 目錄（`.claude\hooks`），並非 plugin，文案與實際內容不符，容易誤導使用者。

此外，Provider Quota 監控關閉某個 provider（例如 OpenCode）後，Dashboard 的 Quota 卡片重新開啟時仍會顯示該 provider 的舊用量資料，即使 StatusBar 已正確不再顯示。原因是前端 Dashboard 從未依「啟用中 provider」清單過濾 quota 快照，後端也只在手動「重新整理」時才清除已停用 provider 的快取，導致關閉監控後舊資料殘留並在下次進入設定頁/Dashboard 時重新浮現。

## What Changes

- 將設定頁 provider integration 卡片的欄位標籤由統一的「設定 / plugin 路徑」改為依 provider 類型顯示對應語意的名稱：Claude Code 顯示「Hook 路徑」，其餘沿用「設定 / plugin 路徑」（或依實際安裝機制命名，如 Codex 的 hooks.json 亦屬 hook 設定）。
- 修正 Dashboard 的 `QuotaOverview` 未依使用者停用的 provider 清單過濾用量快照的問題，使其行為與 StatusBar 一致：使用者取消勾選的 provider 不應在 Dashboard 中出現。
- 修正儲存設定時未清除已停用 provider 的 quota 快取／DB snapshot 的問題，避免只有在手動「重新整理」時才清乾淨，導致重開頁面又看到舊資料。

## Capabilities

### Modified Capabilities
- `app-settings`: 設定頁 provider integration 卡片欄位標籤 SHALL 依 provider 類型顯示正確語意（Claude Code 為「Hook 路徑」而非「plugin 路徑」）
- `provider-quota-monitor`: Dashboard 與後端快取 SHALL 依使用者當前的 quota 啟用清單過濾/清除已停用 provider 的用量資料，不僅限於手動刷新時觸發

## Impact

- 前端：`src/components/SettingsView.tsx`（provider integration 欄位標籤）、`src/components/DashboardView.tsx`（quota 區塊渲染）、`src/components/QuotaOverview.tsx`（過濾邏輯）、`src/locales/zh-TW.ts` / `src/locales/en-US.ts`（新增/調整翻譯鍵）
- 後端：`src-tauri/src/commands/settings.rs`（`save_settings` 需觸發快取清除）、`src-tauri/src/commands/quota.rs`（抽出可重用的「移除已停用 provider 快取」邏輯，供 `save_settings` 與 `refresh_quota` 共用）
