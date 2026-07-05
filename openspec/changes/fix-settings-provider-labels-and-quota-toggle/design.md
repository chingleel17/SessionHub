## Context

這個 change 修正兩個獨立但都在「設定頁 / provider 整合」範疇內的問題：

1. `SettingsView.tsx` 用同一個翻譯鍵 `settings.integrations.fields.configPath`（「設定 / plugin 路徑」）顯示所有 provider 的設定檔位置，但 Claude Code 實際顯示的是 hooks 目錄路徑，語意不符。
2. `quota.rs` 內已經有「移除已停用 provider 快取」的邏輯（`remove_provider_from_cache_and_db`），但只寫在 `refresh_quota` command 且僅在全量刷新（`provider.is_none()`）時觸發；`settings.rs` 的 `save_settings` command 完全沒有拿到 `DbState` / `QuotaCache`，儲存設定時不會清快取。前端 `DashboardView.tsx` 渲染 `QuotaOverview` 時只檢查全域開關 `enableQuotaMonitoring`，未依 `quotaEnabledProviders` 過濾 `quotaSnapshots`，`QuotaOverview.tsx` 本身也沒有這層過濾（只濾掉 `status === "unsupported"`）。

## Goals / Non-Goals

**Goals:**
- Claude Code 的 provider integration 卡片顯示「Hook 路徑」而非「設定 / plugin 路徑」，其他 provider 維持現有標籤。
- Dashboard 的 Quota 卡片與 StatusBar 一樣，只顯示使用者目前啟用的 provider。
- 使用者取消勾選某 provider 的 quota 監控並儲存後，該 provider 的舊資料立即從快取與 DB 清除，不必等到手動「重新整理」。

**Non-Goals:**
- 不重新設計 `provider-quota-monitor` spec 中舊有的「5 小時區間 / 月累計 / 訂閱上限」需求，這些維持現狀。
- 不變更 provider integration 卡片的整體版面或其他欄位（bridge 路徑、最後事件時間等）。
- 不新增 per-provider 的 quota 資料保留策略（例如「停用但保留歷史」選項）——停用即清除。

## Decisions

### 1. 標籤依 provider 類型決定，而非新增每 provider 專屬翻譯鍵矩陣
在 `SettingsView.tsx` 組出 provider integration 欄位陣列時（約第 517-528 行），依 `integration.provider === "claude"` 決定 label 使用新翻譯鍵 `settings.integrations.fields.hookPath`（「Hook 路徑」），其餘 provider 沿用既有的 `settings.integrations.fields.configPath`（「設定 / plugin 路徑」）。

**替代方案考量**：曾考慮為每個 provider 建立獨立翻譯鍵（`configPath.claude` / `configPath.codex` / ...），但目前只有 Claude Code 的語意不符（其餘 provider 的路徑本來就是設定檔或 plugin 檔），沒有必要為尚未出現問題的 provider 增加維護成本。之後若 Codex/Copilot/OpenCode 的整合機制改變，再各自新增對應鍵值。

### 2. 前端在 Dashboard 渲染前依 `quotaEnabledProviders` 過濾，而非只改後端
`DashboardView.tsx` 第 735 行的渲染條件與傳給 `QuotaOverview` 的 `snapshots` 都要加上過濾：`quotaSnapshots.filter(s => quotaEnabledProviders.includes(s.provider))`。即使之後補齊後端清除邏輯，前端過濾仍是必要的防線——後端清除是非同步/最佳努力（`let _ = remove_provider_from_cache_and_db(...)`失敗會被忽略），前端過濾能保證 UI 一致性不依賴後端清除是否成功。

### 3. 後端把「移除已停用 provider 快取」抽成可從 `save_settings` 呼叫的共用邏輯
`save_settings` command 目前簽名是 `pub fn save_settings(settings: AppSettings) -> Result<(), String>`，沒有注入 `State<DbState>` / `State<QuotaCache>`。需要：
- 修改 command 簽名為 `pub fn save_settings(db_state: State<'_, DbState>, quota_cache: State<'_, QuotaCache>, settings: AppSettings) -> Result<(), String>`，Tauri 會自動注入這兩個 managed state（`lib.rs` 已經 `.manage(...)` 過，其餘 command 如 `get_quota_snapshots` 已是這種寫法）。
- 在 `save_settings_internal` 寫入 `settings.json` 成功後，讀取舊快取中所有 snapshot，比對新的 `settings.quota_enabled_providers`，對不在清單中的 provider 呼叫既有的 `remove_provider_from_cache_and_db`（複用 `refresh_quota` 裡已經寫好的邏輯，抽成 `quota.rs` 內的 pub(crate) helper 函式，兩處呼叫同一份實作）。

**替代方案考量**：曾考慮改成「`get_quota_snapshots` 查詢時即時過濾，不動快取/DB」，但這樣資料庫會無限累積已停用 provider 的歷史 snapshot row，且無法解決「應用程式啟動時 `load_cache_from_db` 整批載入」這個路徑（見下一項），所以仍選擇在儲存設定當下主動清除。

### 4. 應用程式啟動載入快取時也要過濾
`lib.rs` 啟動時的 `load_cache_from_db` 目前會把 DB 內所有 provider 的歷史 snapshot 不加篩選地載入記憶體快取。修改為載入時讀取當下 `AppSettings.quota_enabled_providers`，只載入清單內的 provider 資料，避免「使用者關閉監控 → 從未觸發過清除路徑（例如關閉後直接重啟應用程式）→ 啟動時又把舊資料撈回快取」的邊界情況。

## Risks / Trade-offs

- [Risk] 前端過濾與後端清除同時修改，可能出現「後端還沒清，前端先濾掉」造成的短暫不一致（例如過渡版本、快取延遲）→ Mitigation：前端過濾是主要防線，後端清除是輔助，兩者獨立生效即可，沒有時序依賴。
- [Risk] `save_settings` 新增 `State` 參數屬於 command 簽名變更，需確認前端 `invoke("save_settings", { settings })` 呼叫不受影響（Tauri 的 `State` 參數由框架自動注入，不需前端傳遞，呼叫端程式碼不用改）→ Mitigation：實作後以 `/verify` 走一次設定頁儲存流程確認無破壞性影響。
- [Trade-off] 停用某 provider 即清除其快取與 DB snapshot，代表使用者重新啟用該 provider 監控時需要重新累積資料（無法保留歷史），但這符合本次「停用即不追蹤」的預期行為，且訂閱本身用量也是即時查詢而非本地累積為主。

## Open Questions

無（範疇明確，若後續有多平台 hook 路徑語意調整需求，屆時再開新 change）。
