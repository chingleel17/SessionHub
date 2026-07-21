## Why

SessionHub 目前支援 Claude、Codex、Copilot、OpenCode 四個 AI coding 工具，但使用者主力之一的 Google Antigravity（IDE 與 agy CLI）尚未納入。Antigravity 的 session、hook、quota 資料都在本機可讀取，補上這個 provider 能讓使用者在同一處統一管理，並掌握 Antigravity 的 5 小時／每週額度。

## What Changes

- 新增 Antigravity 為第五個 session provider，掃描 `~/.gemini/{antigravity,antigravity-cli,antigravity-ide}/brain` 三個目錄，產出 session 列表（僅 metadata 層級）。
- 新增 Antigravity hook 管理，支援全域 `~/.gemini/config/hooks.json` 與專案 `<repo>/.agents/hooks.json` 的讀寫，schema 近似 Claude Code。
- 新增 Antigravity quota adapter，透過本機 Language Server 的 Connect RPC 讀取即時額度（5 小時／每週視窗），底部狀態列顯示 Gemini 群組、Dashboard 顯示 Gemini 與 Claude/GPT 兩群組。
- 前端 `providerLabel.ts`、settings provider 開關、quota 顯示元件擴充以涵蓋 antigravity。
- Antigravity quota 依賴 IDE／agy 執行中的 Language Server；未執行時 adapter 優雅降級為「不可用」狀態，不影響其他功能。

## Capabilities

### New Capabilities

- `antigravity-provider`: 掃描三個 Antigravity brain 目錄，讀取每個 conversation 的 `*.metadata.json`（summary/updatedAt）與 `.system_generated/logs/transcript.jsonl`，映射為 `SessionInfo`，接進現有 provider 掃描迴圈與前端顯示。
- `antigravity-hook-scripts`: 全域與專案層級 `hooks.json` 的 CRUD，涵蓋 `PreToolUse`/`PostToolUse`/`PreInvocation`/`PostInvocation`/`Stop` 事件、matcher 正則與 command/timeout，複用現有 Claude hook UI 模式。
- `antigravity-quota`: 透過本機 Language Server（動態 port + CSRF）呼叫 `RetrieveUserQuotaSummary`，解析模型群組的 5 小時／每週 bucket，映射為 `QuotaWindow`，實作 `QuotaAdapter` trait 註冊進 `QuotaManager`。

### Modified Capabilities

- 無 requirement 層級的既有 spec 變更；本 change 只新增 capability，沿用現有 `session-list`、`provider-filter`、`provider-quota-monitoring`、`global-status-bar`、`dashboard` 的既有契約。

## Impact

- **Rust 後端**：新增 `src-tauri/src/sessions/antigravity.rs`、`src-tauri/src/quota/antigravity.rs`；修改 `sessions/mod.rs`（掃描迴圈）、`quota/mod.rs`（註冊 adapter）、hook 相關 command 模組、`types.rs`（provider 常數與 settings 欄位）。
- **前端**：`src/utils/providerLabel.ts`、`SettingsView.tsx`（provider／quota 開關）、hook 設定 UI、狀態列與 Dashboard quota 顯示元件。
- **資料來源（唯讀）**：`~/.gemini/{antigravity,antigravity-cli,antigravity-ide}/brain/`、`~/.gemini/config/hooks.json`、`<repo>/.agents/hooks.json`、本機 `language_server.exe` 的 127.0.0.1 RPC。
- **平台**：quota 的進程／port 探測（tasklist + netstat）目前為 Windows 實作；其他平台可後續補上或降級為不可用。
- **相依**：不需新增外部相依；沿用 `ureq`（HTTP）、`serde_json`、`chrono`。
