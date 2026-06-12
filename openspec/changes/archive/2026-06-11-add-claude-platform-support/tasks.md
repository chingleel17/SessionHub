## 1. Rust 基礎型別與常數

- [x] 1.1 在 `src-tauri/src/types.rs` 加入 `CLAUDE_PROVIDER` 常數與 `CLAUDE_HOOK_FILE_NAME`（`settings.json`）
- [x] 1.2 在 `AppSettings` struct 加入 `claude_root: String`（`#[serde(default)]`）與 `claude_quota_reset_day: u8`（預設 1）
- [x] 1.3 在 `ScanCache` struct 加入 `claude: Mutex<Option<ProviderCache>>` 欄位
- [x] 1.4 新增 Claude JSONL 解析型別：`ClaudeEntry`（含 `type`、`uuid`、`parentUuid`、`isSidechain`、`message`、`requestId`、`timestamp`、`cwd`、`sessionId`）
- [x] 1.5 新增 `ClaudeMessage`（含 `model`、`id`、`usage: ClaudeUsage`）
- [x] 1.6 新增 `ClaudeUsage`（含 `input_tokens`、`output_tokens`、`cache_creation_input_tokens`、`cache_read_input_tokens`、`speed`、`service_tier`、`cache_creation: Option<ClaudeCacheCreation>`）
- [x] 1.7 新增 `ClaudeCacheCreation`（含 `ephemeral_1h_input_tokens`、`ephemeral_5m_input_tokens`）
- [x] 1.8 在 `settings.rs` 加入 `default_claude_root()` 與 `resolve_claude_root()` 函式

## 2. Claude Session 掃描模組

- [x] 2.1 新增 `src-tauri/src/sessions/claude.rs`，實作 `collect_claude_session_files()` 遞迴列舉 `<claude_root>/projects/**/*.jsonl`
- [x] 2.2 實作 `parse_claude_session_file()`：讀取 JSONL、取 `created_at`（第一個 user entry timestamp）、`updated_at`（最後一個 entry）、`cwd`（頂層欄位）、`sessionId`
- [x] 2.3 實作 `scan_claude_sessions()` 主函式，套用 mtime-based 增量掃描（與 Codex 模式相同）
- [x] 2.4 在 `src-tauri/src/sessions/mod.rs` 加入 `pub mod claude`

## 3. Claude Session 統計解析（ccusage 移植）

- [x] 3.1 在 `src-tauri/src/stats.rs` 新增 `compute_claude_stats()` 函式：僅處理 `type=assistant && isSidechain=false` 的 entry
- [x] 3.2 實作 `message.id` dedup 邏輯（HashMap，同一 id 保留 token 最多的那筆）
- [x] 3.3 累計 `input_tokens`、`output_tokens`、`cache_creation_input_tokens`（含 1h + 5m 細分）、`cache_read_input_tokens`
- [x] 3.4 計算 session 成本：建立簡易模型定價表（claude-sonnet-4-x、claude-haiku-4-x、claude-opus-4-x），1h cache 費率 2x，5m cache 費率 1x，`speed=fast` 套 1.3x
- [x] 3.5 在 stats cache 失效邏輯中加入 Claude provider 分支（以 JSONL mtime 為 cache key）

## 4. Claude 5 小時用量區間

- [x] 4.1 實作 `build_claude_session_blocks()` 函式：將 session 中的 assistant messages 依時間排序，以 5 小時間隔為分界切割成區間
- [x] 4.2 標示最後一個區間是否 `is_active`（距現在 < 5 小時）
- [x] 4.3 解析 `is_api_error_message=true` 的 entry，嘗試從錯誤文字提取 `usage_limit_reset_time`（`|<unix_seconds>` 格式）
- [x] 4.4 新增 `get_claude_usage_blocks` Tauri command，回傳各 session block 的 token 統計與活躍狀態

## 5. Provider Quota 資料庫

- [x] 5.1 在 `src-tauri/src/db.rs` 新增 `provider_quota` 表（`provider, billing_period, input_tokens, output_tokens, cache_creation_tokens, cache_read_tokens, cost_usd`）
- [x] 5.2 新增 `provider_quota_settings` 表（`provider, monthly_limit_tokens, monthly_limit_usd, reset_day`）
- [x] 5.3 實作 `upsert_provider_quota()` — 依 `(provider, billing_period)` 累加用量，`billing_period` 由 `reset_day` 決定
- [x] 5.4 實作 `get_provider_quota()` — 回傳各 provider 當期累計與上限設定

## 6. Provider Quota Tauri Commands

- [x] 6.1 新增 `get_provider_quota` command（回傳各 provider 當期 tokens、cost_usd、limit、reset_date）
- [x] 6.2 新增 `set_provider_quota_settings` command（設定 monthly_limit_tokens / monthly_limit_usd / reset_day）
- [x] 6.3 在 `tauri::generate_handler!` 中註冊新 commands

## 7. Claude Hook Integration

- [x] 7.1 在 `src-tauri/src/provider/mod.rs` 加入 `detect_claude_integration_status()` （讀取 `~/.claude/settings.json` 偵測 `hooks.Stop` 是否含 SessionHub entry）
- [x] 7.2 實作 `install_claude_hook()`：讀取現有 `settings.json` → merge 至 `hooks.Stop` 陣列 → 寫回（不覆蓋其他設定）
- [x] 7.3 在 `commands/provider.rs` 加入 Claude install 分支
- [x] 7.4 在 `AppSettings.provider_integrations` 加入 Claude 整合狀態欄位

## 8. Watcher 與 Bridge 事件

- [x] 8.1 在 `src-tauri/src/watcher.rs` 加入 Claude file watcher（監聽 `claude_root/projects/` 目錄變更）
- [x] 8.2 在 bridge 事件處理加入 `provider=claude` 的 targeted refresh 路由

## 9. 背景執行（系統匣）

- [x] 9.1 在 `tauri.conf.json` 加入 `trayIcon` 設定與 icon（重用現有 `icons/32x32.png`）
- [x] 9.2 在 `src-tauri/src/lib.rs` 加入 `on_window_event` 監聽 `CloseRequested`，改為 `window.hide()` 而非真正關閉
- [x] 9.3 建立系統匣選單：「顯示視窗」、「退出 SessionHub」
- [x] 9.4 加入 `capabilities/tray.json`（或在現有 capabilities 中加入 `core:tray:default` 權限）
- [x] 9.5 在 Settings 加入「關閉時最小化至系統匣」開關（預設 off，使用者手動啟用）

## 10. 前端 StatusBar 三欄重設計

- [x] 10.1 修改 `src/components/StatusBar.tsx`：加入中欄（idle / done 計數，原有 active / waiting 保留）
- [x] 10.2 新增右欄 Provider Quota 區塊：各啟用 provider 的當期 token 進度條（有上限時）或純數字（無上限時）
- [x] 10.3 實作右欄 hover tooltip，顯示 input / output / cache tokens 與 cost_usd 細分
- [x] 10.4 在 `src/App.tsx` 加入 `get_provider_quota` invoke，定期更新並傳入 StatusBar

## 11. 前端 Settings 頁面 — Claude 設定區塊

- [x] 11.1 在 SettingsView 加入 Claude 設定區塊：root 路徑、啟用開關、integration 安裝按鈕
- [x] 11.2 加入月 token 上限輸入、月費用上限輸入、重置日設定（1–28）
- [x] 11.3 顯示當期用量（input / output / cache tokens 分開）與估算成本
- [x] 11.4 加入「此為 SessionHub 記錄的用量，非實際帳號用量」說明文字
- [x] 11.5 加入「關閉時最小化至系統匣」開關（呼叫設定 API 儲存）

## 12. 國際化與 UI 文字

- [x] 12.1 加入 Claude provider 設定頁相關 i18n key
- [x] 12.2 加入 Quota Monitor 相關 key（用量顯示、上限設定、警示、免責說明）
- [x] 12.3 加入系統匣相關 key（選單項目）
- [x] 12.4 確認 provider filter、session list badge 正確顯示 `claude`

## 13. 整合測試與驗證

- [ ] 13.1 驗證：啟用 Claude provider 後正確列出 `~/.claude/projects/` 下的 sessions（含子目錄）
- [ ] 13.2 驗證：token 統計正確執行 message.id dedup，不重複計算
- [ ] 13.3 驗證：安裝 Claude hook 後 `~/.claude/settings.json` 正確 merge（不破壞現有設定）
- [ ] 13.4 驗證：5 小時區間切割與活躍狀態標示正確
- [ ] 13.5 驗證：視窗關閉後系統匣圖示出現，bridge 事件仍可接收
- [ ] 13.6 驗證：Settings 設定月上限後 StatusBar 右欄進度條正確顯示
  （需執行應用程式手動驗證，非程式碼審查範疇）
