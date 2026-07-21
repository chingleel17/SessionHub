## 1. 設定頁 provider integration 標籤修正

- [x] 1.1 在 `src/locales/zh-TW.ts` 與 `src/locales/en-US.ts` 新增翻譯鍵 `settings.integrations.fields.hookPath`（zh-TW：「Hook 路徑」；en-US：對應英文）
- [x] 1.2 修改 `src/components/SettingsView.tsx` provider integration 欄位陣列（約第 517-528 行），依 `integration.provider === "claude"` 選用 `hookPath` 翻譯鍵，其餘 provider 沿用 `configPath`
- [x] 1.3 手動驗證：開啟設定頁，確認 Claude Code 卡片顯示「Hook 路徑」，Codex / Copilot / OpenCode 卡片仍顯示「設定 / plugin 路徑」

## 2. Dashboard Quota 卡片依啟用清單過濾（前端）

- [x] 2.1 在 `src/components/DashboardView.tsx` 渲染 `QuotaOverview` 前（約第 735 行），依 `settingsForm.quotaEnabledProviders`（或對應 props）過濾 `quotaSnapshots`，只保留啟用中的 provider
- [x] 2.2 確認 `enableQuotaMonitoring` 全域開關與 per-provider 過濾兩者皆滿足才渲染 `QuotaOverview`（開關關閉或過濾後為空陣列時不顯示卡片）
- [x] 2.3 檢查 `src/components/QuotaOverview.tsx` 的 `visible` 過濾邏輯（第 217-227 行附近），確認上游已過濾後此處不需重複處理，或視情況加上防禦性過濾以避免其他呼叫者遺漏
- [x] 2.4 手動驗證：取消勾選 OpenCode quota 監控並儲存，確認 Dashboard 立即不再顯示 OpenCode 用量卡片（重新整理頁面 / 重新進入 Dashboard 後仍不顯示）

## 3. 後端：儲存設定時清除已停用 provider 的 quota 快取

- [x] 3.1 在 `src-tauri/src/quota/cache.rs` 新增 `pub(crate)` 函式 `prune_disabled_provider_quota(conn, quota_cache, enabled_providers)`，將 `refresh_quota` 內「移除已停用 provider 快取」的邏輯抽成可重用的共用函式
- [x] 3.2 修改 `src-tauri/src/commands/settings.rs` 的 `save_settings` command 簽名，新增 `db_state: State<'_, DbState>` 與 `quota_cache: State<'_, QuotaCache>` 參數
- [x] 3.3 在 `save_settings` 成功寫入 settings.json 後，呼叫 `prune_disabled_provider_quota`，依最新 `settings.quota_enabled_providers` 清除已停用 provider 的快取與 DB snapshot
- [x] 3.4 確認 `lib.rs` 中 `save_settings` 的 Tauri command 註冊（自動由 `pub fn` 與 `State` 推斷）維持相容，前端 `invoke("save_settings", { settings })` 呼叫不需改動
- [x] 3.5 修改 `src-tauri/src/quota/cache.rs` 的 `load_cache_from_db`，載入時依當下 `AppSettings.quota_enabled_providers` 過濾，只載入啟用中 provider 的歷史 snapshot 進記憶體快取；更新 `lib.rs` 呼叫處傳入 `settings.quota_enabled_providers`
- [x] 3.5.1 修正 `src/components/StatusBar.tsx` 的 `activeQuotas` 過濾邏輯（約第 175 行）：`get_provider_quota`（local token usage / `ProviderQuota[]`）的資料來源未依 `quotaEnabledProviders` 過濾，只檢查是否有 token/cost，導致停用 provider 後其歷史用量條仍會顯示；已加入 `quotaEnabledProviders.includes(q.provider)` 條件
- [x] 3.6 手動驗證：停用 OpenCode 監控並儲存後直接重啟應用程式，確認 OpenCode quota 資料不會重新出現在 Dashboard 或 StatusBar

## 4. 回歸驗證

- [x] 4.1 驗證重新啟用某 provider 的 quota 監控後，下次刷新可正常顯示該 provider 的最新用量
- [x] 4.2 驗證 StatusBar 與 Dashboard 兩處對「已停用 provider」的顯示行為一致（兩者皆已依 `quotaEnabledProviders` 過濾）
- [x] 4.3 執行 `/verify` 或等效手動流程，確認設定頁儲存、Dashboard 顯示、StatusBar 顯示三者皆正常運作，無 console 錯誤
