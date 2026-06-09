## Why

SessionHub 目前支援 Copilot、OpenCode、Codex 三個 AI coding 平台，但缺少對 Claude Code CLI（Anthropic 官方 CLI）的支援。隨著 Claude Code 使用量增加，使用者需要在同一個介面管理所有平台的 sessions，並且能夠一眼看出各平台訂閱方案的剩餘用量。

## What Changes

- 新增 **Claude** 作為第四個 provider，支援讀取 `~/.claude/projects/` 目錄下的 JSONL session 檔案
- 新增 Claude session 統計解析，從 JSONL messages 中提取 token 用量、模型名稱、工具呼叫次數等
- 新增 Claude hook/bridge 整合，讓 Claude Code CLI hook 能通知 SessionHub session 狀態變更
- 新增 **Provider Quota Monitor**：各平台訂閱用量追蹤機制，在 Settings 與 Dashboard 顯示剩餘用量資訊（目前以 Claude Pro/Max plan 為主要目標，其他平台預留擴充點）

## Capabilities

### New Capabilities

- `claude-provider`: Claude Code CLI session 掃描、解析、統計（讀取 `~/.claude/projects/**/*.jsonl`，包含 session 列表、token 統計、模型資訊）
- `claude-hook-integration`: Claude Code 的 hook 整合（`.claude/settings.json` hook 設定，bridge 事件寫入），讓 SessionHub 在 session 結束時收到通知
- `provider-quota-monitor`: 跨平台訂閱用量監控，記錄各平台月用量累計並顯示距離方案上限的剩餘量；支援手動設定方案上限值

### Modified Capabilities

- `provider-integration`: 新增 Claude provider 的整合安裝狀態偵測與自動寫入 hook 設定

## Impact

- `src-tauri/src/sessions/`: 新增 `claude.rs` session 掃描模組
- `src-tauri/src/provider/`: `mod.rs`、`bridge.rs` 加入 Claude provider 常數與整合狀態
- `src-tauri/src/types.rs`: 新增 Claude JSONL 解析型別、Quota 相關型別
- `src-tauri/src/db.rs`: 新增 `provider_quota` 資料表（月累計 tokens / 設定上限）
- `src-tauri/src/lib.rs` / `commands/`: 新增 quota 相關 Tauri commands
- `src/`: Settings 頁面新增 Claude 設定區塊與 Quota 顯示元件；Dashboard 加入用量摘要
- 新增 i18n 翻譯鍵（中文）
- 依賴：無新外部 crate，沿用既有 `rusqlite`、`serde_json`、`notify`
