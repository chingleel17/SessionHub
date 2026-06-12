## Context

SessionHub 目前有三個 provider（Copilot、OpenCode、Codex），都遵循相同的 provider 模式：Rust 掃描模組 + ScanCache + ProviderCache + SessionInfo + SessionStats。Claude Code CLI 將 sessions 儲存在 `~/.claude/projects/<encoded-path>/<session-id>.jsonl`，每行為一個 JSON 訊息物件，包含 role、content 與 usage 資訊。

## Goals / Non-Goals

**Goals:**
- 新增 Claude provider 遵循現有的 provider 模式（Rust session 掃描、mtime-based 增量更新、ScanCache）
- 解析 Claude JSONL session 檔案取得：summary、session 時間戳、token 用量、model 名稱、工具呼叫
- 新增 Claude hook 整合（寫入 `~/.claude/settings.json` 的 `hooks` 欄位），讓 session 結束時通知 SessionHub bridge
- 新增 `provider_quota` SQLite 資料表，記錄各 provider 每月累計 token 用量，並允許使用者設定方案上限；在 Settings 與 Dashboard 顯示用量進度條

**Non-Goals:**
- 不呼叫 Anthropic API 取得即時帳號用量（無公開 API）
- 不支援 Claude Web 或 API key session（僅 Claude Code CLI）
- 不實作自動方案偵測（上限由使用者手動設定）

## Decisions

### D1: 遵循 Codex mtime-based 增量掃描模式

Claude JSONL 檔案以檔案修改時間作為 cache 失效鍵，與 Codex 相同。這樣可以重用 `session_mtimes` 表和現有掃描基礎設施，而非為 Claude 另建 cursor-based 機制。

**Alternative considered**: cursor-based（如 OpenCode）— 但 Claude JSONL 是單一獨立檔案而非目錄結構，mtime diff 更直接。

### D2: 目錄結構 — `~/.claude/projects/<encoded-path>/<uuid>.jsonl`

Claude Code CLI 以 URL-encoded 專案路徑作為目錄名，子目錄中每個 JSONL 檔案為一個 session。掃描策略：列舉 `claude_root/projects/` 下所有兩層深的 `.jsonl` 檔案；cwd 從 encoded 目錄名反推（URL decode）。

### D3: Hook 整合寫入 `~/.claude/settings.json`

Claude Code CLI 支援 `hooks` 設定（`stop` hook），在 agent 停止時執行任意命令。SessionHub 注入一個寫入 bridge JSON 檔案的 shell 命令作為 `stop` hook，與 Copilot hook 的 bridge 機制一致。

**Alternative considered**: 使用 MCP server — 過於複雜，且需要 Claude Code 版本相依。

### D4: 用量追蹤以 SQLite 本地累計為主

因為 Anthropic 未提供公開的訂閱用量 API，我們在 `provider_quota` 資料表中累計從 session stats 解析出的 token 用量（每月重置），並讓使用者在 Settings 中輸入方案上限（tokens/月）。Dashboard 顯示進度條（已用 / 上限）。

### D5: claude_root 預設 `~/.claude`，可在 Settings 覆寫

與其他 provider 的 `*_root` 設定一致，AppSettings 增加 `claude_root` 欄位。

## Risks / Trade-offs

- [Claude JSONL 格式可能隨版本改變] → 解析時做 defensive deserialization，parse error 僅影響單一 session 並記錄 `parse_error` 旗標
- [hook 注入可能覆蓋使用者現有 hook] → 採用 merge 策略（讀取現有 settings.json → 在 `hooks.stop` 陣列 append SessionHub entry），而非完整覆寫
- [月累計 token 計算不精確（部分舊 session 不在統計內）] → 僅計算 SessionHub 掃描到且有 stats 的 sessions；UI 明確標示「SessionHub 記錄的用量」而非「實際帳號用量」

## Migration Plan

1. DB migration：應用程式啟動時，若 `provider_quota` 表不存在則自動建立（與現有 DB init 模式相同）
2. Settings migration：`claude_root` 欄位加上 `#[serde(default)]`，舊設定讀入後自動填入預設值
3. 無 breaking change：Claude provider 預設為 disabled，使用者需在 Settings 中手動啟用並設定 root 路徑

### D6: 參考 ccusage 自行實作解析邏輯（不引入外部 binary）

[ccusage](https://github.com/ryoppippi/ccusage) 是一個同樣以 Rust 實作的 Claude token 用量工具，其解析邏輯已驗證正確。我們選擇**將其核心解析邏輯移植進 SessionHub**，而非以 shell exec 呼叫 ccusage binary。

原因：
- 避免外部 binary 依賴與安裝需求
- SessionHub 本身即 Rust，直接整合效率更高
- 僅需移植 token 解析與 dedup 邏輯（無需 CLI、pricing database 等）

關鍵移植項目：`UsageEntry` 結構體（含 `message.usage` 所有欄位）、`message.id` dedup、5 小時區間切割邏輯。

### D7: 真實 JSONL 格式（從本機驗證）

JSONL 每行頂層欄位：`type`、`uuid`、`parentUuid`、`isSidechain`、`message`、`requestId`、`timestamp`、`cwd`、`sessionId`、`version`、`gitBranch`

- `cwd` **直接在 entry 頂層**（不需從目錄名 decode）
- **無** `costUSD` 欄位（需自行從 token 計算）
- `message.usage` 包含：`input_tokens`、`output_tokens`、`cache_creation_input_tokens`、`cache_read_input_tokens`、`speed`、`service_tier`、`cache_creation.ephemeral_1h_input_tokens`、`cache_creation.ephemeral_5m_input_tokens`
- 同一 `message.id` 會在 JSONL 中多次出現（isSidechain replay），需以 `message.id` dedup

### D8: 背景執行以 Tauri 系統匣實現

使用者在 Settings 啟用後，關閉視窗改為 hide（保持 Rust 後端執行），讓 file watcher 與 bridge 事件處理繼續運作。`tauri-plugin-tray` 已包含在 Tauri 2 的 `core:tray:default` ACL 中，不需新增外部 crate。

Claude 的 5 小時用量窗口機制意味著用量統計需即時更新——背景執行是讓此功能有意義的前提。

## Open Questions

- Claude Code CLI 的 `settings.json` hooks 格式是否跨版本穩定？（目前依 Claude Code v2.1.x 驗證）
- 模型定價表是否要硬編碼或抓取 ccusage 的 pricing JSON？（初版硬編碼常用模型，後續再看）
- Claude Pro vs Max plan 的 5 小時 token 上限數值為何？（讓使用者自填，後續可內建常用方案預設）
