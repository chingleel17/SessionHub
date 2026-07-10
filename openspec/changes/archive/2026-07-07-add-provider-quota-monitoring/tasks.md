## 1. Quota Core

- [x] 1.1 在 `src-tauri/src/types.rs` 定義 `QuotaSnapshot`、`QuotaWindow`、`LocalTokenUsage`、`ExtraCredits` struct 與 `QuotaStatus` enum（`ok` / `error` / `unsupported` / `no_auth`）；在 `src/types/index.ts` 新增對應前端型別
- [x] 1.2 在 `src-tauri/src/quota/` 新增模組，定義 `QuotaAdapter` trait 並實作四個內建 adapter：
  - `ClaudeAdapter`：讀 `~/.claude/.credentials.json` 取 OAuth token，呼叫 `https://api.anthropic.com/api/oauth/usage`（header: `anthropic-beta: oauth-2025-04-20`），解析 `five_hour` / `seven_day` / `seven_day_sonnet` / `seven_day_opus` / `extra_usage`
  - `CopilotAdapter`：spawn `gh auth token` 取 token、`gh api user` 取 username，呼叫 `https://api.github.com/users/{username}/settings/billing/ai_credit/usage`
  - `OpenCodeAdapter`：掃描 `{opencodeRoot}/sessions/` JSONL，計算本月 input/output tokens
  - `CodexAdapter`：掃描 `{codexRoot}/` JSONL，計算本月 input/output tokens
- [x] 1.3 在 `src-tauri/src/quota/cache.rs` 實作雙層快取：
  - 記憶體層：`Mutex<HashMap<String, QuotaSnapshot>>` 掛入 `AppState`
  - SQLite 層：`db.rs` 新增 `quota_snapshots` table（欄位：`provider TEXT PRIMARY KEY, snapshot_json TEXT, fetched_at TEXT`）；新增 `load_quota_snapshots_from_db()` / `save_quota_snapshot_to_db()` helper
  - app 啟動時從 DB 載入填入記憶體快取；每次 refresh 成功後同步寫 DB（失敗不覆蓋）

## 2. Refresh And Connector Flow

- [x] 2.1 在 `src-tauri/src/commands/quota.rs` 新增兩個 Tauri command：
  - `get_quota_snapshots() -> Result<Vec<QuotaSnapshot>, String>`：直接回傳記憶體快取，不觸發查詢
  - `refresh_quota(provider: Option<String>) -> Result<Vec<QuotaSnapshot>, String>`：`None` 表示全量刷新（所有 enabledProviders），`Some(key)` 表示單一 provider；非同步呼叫各 adapter，結果寫入記憶體與 SQLite
  - 在 `commands/mod.rs` pub use；在 `lib.rs` invoke_handler 登記
- [x] 2.2 在 `watcher.rs` 或 `lib.rs` run() 實作兩個流程：
  - **app startup**：啟動時若 `enable_quota_monitoring: true`，先從 DB 載入快照，再 spawn 背景執行緒做第一次完整 refresh
  - **主要觸發源：Hook 事件**（見 2.3）；定時器僅作為 fallback（每 `quota_refresh_interval` 分鐘對無法透過 hook 觸發的 provider 做補充 refresh，例如 app 長時間閒置時）
  - 不依賴定時器做主要刷新，settings 中的 `quota_refresh_interval` 僅作為 fallback 間隔而非主要刷新頻率
  - 實際決策：若 hook 事件正常觸發（provider 有活躍使用），定時器不應重複觸發；只在超過 `quota_refresh_interval` 時間且沒有任何 hook 觸發時才啟動補充 refresh
- [x] 2.3 在 `provider/bridge.rs` 的 `process_provider_bridge_event()` 中，收到任何 bridge event 後，以 per-provider 30 秒 debounce 排程呼叫 `refresh_quota(Some(provider))`；debounce 狀態存入新的 `last_quota_refresh_trigger: Arc<Mutex<HashMap<String, Instant>>>` 欄位（加入 `WatcherState`）
- [x] 2.4 確認 `QuotaAdapter` trait 定義完整（`provider_key()` + `fetch_snapshot()`），並在 `quota/mod.rs` 的 `QuotaManager` 中以 `Vec<Box<dyn QuotaAdapter>>` 持有所有內建 adapter；manager 根據 `enabled_providers` 過濾要執行的 adapter，不在清單內的 provider 不查詢

## 3. Settings And Provider Diagnostics

- [x] 3.1 在 `src-tauri/src/types.rs` 的 `AppSettings` struct 新增：
  - `enable_quota_monitoring: bool`（`#[serde(default = "default_true")]`）
  - `quota_refresh_interval: u32`（`#[serde(default = "default_quota_refresh_interval")]`，預設值 `30`）
  - 在 `src-tauri/src/types.rs` 新增 `fn default_quota_refresh_interval() -> u32 { 30 }`
  - 在 `src/types/index.ts` 的 `AppSettings` 新增 `enableQuotaMonitoring?: boolean` 與 `quotaRefreshInterval?: 5 | 15 | 30 | 60`
- [x] 3.2 在 `src/components/SettingsView.tsx` 新增 quota monitoring 控制區塊：
  - toggle 開關（`enable_quota_monitoring`）
  - 刷新間隔下拉選單（5 / 15 / 30 / 60 分鐘）
  - provider quota diagnostics 列表：顯示各 enabledProviders 的最後刷新時間、status、error_message（若有）
  - 新增翻譯 key 至 `src/locales/zh-TW.ts` 與 `src/locales/en-US.ts`
- [x] 3.3 在 `src/components/SettingsView.tsx` provider integration 卡片區塊加入：
  - quota source 標示（`remote_api` 顯示 API 來源名稱，`local_scan` 顯示「本地估算」）
  - 最後刷新時間與 error 訊息
  - 手動 refresh 按鈕（呼叫 `refresh_quota(Some(provider))`）
  - 所有新增文字透過 `t("key")` 取得

## 4. Dashboard And Status Bar UI

- [x] 4.1 在 `src/components/DashboardView.tsx` 加入 provider quota overview 區塊（位於 analytics panel 上方或下方，依視覺空間決定）：
  - 每個 enabledProvider 一張小卡，顯示 provider 名稱、各 window 的 utilization bar（百分比 + reset 倒數）、local_tokens（若 source 為 local_scan）
  - 右上角「刷新」按鈕（呼叫 `refresh_quota(None)`）並顯示最後刷新時間
  - `enable_quota_monitoring: false` 時不渲染此區塊
- [x] 4.2 在 `src/components/GlobalStatusBar.tsx`（或對應 status bar 元件）的右側加入精簡 quota 摘要：
  - 格式：`[provider縮寫] [最高 utilization%]`，例如 `Claude 5h:72%  Copilot:45%`
  - 僅顯示 `status: "ok"` 且有 windows 資料的 provider；`no_auth` / `error` / `local_scan` 不佔 status bar 空間
  - 不影響既有 session 活動計數（`▶ N 進行中` / `⏳ N 等待回應`）的顯示
- [x] 4.3 確保所有 quota 相關 UI 元件在以下狀態有合理 fallback：
  - `status: "no_auth"`：顯示「需要登入 [provider]」或「需要 gh CLI」
  - `status: "error"`：顯示錯誤摘要與上次成功時間（若有）
  - `status: "unsupported"`：顯示「不支援」，不顯示 utilization bar
  - 首次載入（快取為空）：顯示 skeleton / loading 狀態，不顯示空值
  - `enable_quota_monitoring: false`：整個 quota 區塊隱藏，不留空白

## 5. Validation

- [x] 5.1 在 `src-tauri/src/` 補上後端測試（`#[cfg(test)]` mod）：
  - `ClaudeAdapter::fetch_snapshot()` 在 credentials 不存在時回傳 `status: "no_auth"`
  - `CopilotAdapter::fetch_snapshot()` 在 gh CLI 不存在時回傳 `status: "no_auth"`
  - quota snapshot 快取：寫入後可從記憶體讀回；SQLite save → reload 後資料一致
  - `QuotaManager` 只對 enabledProviders 中的 provider 呼叫 adapter
- [x] 5.2 手動驗證前端流程：
  - 開啟設定頁，確認 quota monitoring toggle 與 refresh interval 下拉存在且可儲存
  - 開啟 Dashboard，確認 quota overview 區塊出現並顯示至少一個 provider 的資料（或正確的 no_auth / error 狀態）
  - 點擊手動刷新，確認 `fetched_at` 更新
  - 關閉 `enable_quota_monitoring`，確認 Dashboard 區塊與 status bar 摘要消失
- [x] 5.3 執行 `cd src-tauri && cargo test` 確認全部測試通過；執行 `bun run build` 確認前端型別無錯誤
