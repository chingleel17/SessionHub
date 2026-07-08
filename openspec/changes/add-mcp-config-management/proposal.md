# Proposal: add-mcp-config-management

## Why

使用者同時使用 claude / codex / opencode / copilot 四種 AI CLI，各平台的全域 MCP server 設定分散在四個不同位置、三種不同格式（JSON 與 TOML），目前只能手動開檔編輯，容易改錯格式或漏改。SessionHub 已具備 Agents（AGENTS.md / skills / commands）集中管理能力，MCP 設定是同一類需求的自然延伸。

## What Changes

- 新增「MCP」全域頁面（與 Agents 頁同層級，sidebar footer 進入），以平台分頁呈現各 provider 的全域 MCP server 清單。
- 專案頁（ProjectView）新增「MCP」sub-tab，管理該專案的專案層 MCP 設定（比照 Agents 的 global/project 雙 scope 模式）。
- 每個平台、每個 scope 支援：列出、新增、編輯（含改名）、啟用/停用、刪除 MCP server；**不做跨平台同步**。
- 後端新增讀寫四個平台設定檔的能力（global / project 兩種 scope）：
  - claude：global = `%USERPROFILE%\.claude.json` 頂層 `mcpServers`；project = `<project>\.mcp.json` 的 `mcpServers`（JSON）
  - codex：global = `<codexRoot>\config.toml`；project = `<project>\.codex\config.toml`，皆為 `[mcp_servers.<name>]`（TOML，須保留既有註解與排版）
  - opencode：global = `%USERPROFILE%\.config\opencode\opencode.json`；project = `<project>\opencode.json`，皆為 `mcp` 鍵（JSON）
  - copilot：global = `<copilotRoot>\mcp-config.json`；project = `<project>\.github\mcp.json`，皆為 `mcpServers`（JSON）
- 停用策略依平台原生能力分流：codex / opencode 原生支援 `enabled` 旗標；claude / copilot 無原生旗標，停用時將該 server 設定搬移至 SessionHub 應用資料的暫存檔（以 provider + scope 為鍵），啟用時還原寫回。

## Capabilities

### New Capabilities

- `mcp-config-management`: 跨平台 MCP server 設定（global 與 project 兩種 scope）的列出、新增、編輯、啟用/停用、刪除，含各平台設定檔格式的讀寫規則與停用暫存機制。

### Modified Capabilities

<!-- 無既有 spec 的需求層級變更：sidebar 導覽與 view routing 的擴充屬於實作細節，納入 mcp-config-management spec 的 UI 需求中。 -->

## Impact

- 前端：新增 `src/components/McpConfigView.tsx`（scope 由 props 決定）；`src/App.tsx`（新 activeView `"mcp-global"`、queries/mutations、標題列）；`src/components/ProjectView.tsx`（新增 "mcp" sub-tab）；`src/components/Sidebar.tsx`（footer 導覽鈕）；`src/components/Icons.tsx`（新 icon）；`src/types/index.ts`；`src/locales/zh-TW.ts`、`src/locales/en-US.ts`。
- 後端：新增 `src-tauri/src/mcp_config.rs` 與 `src-tauri/src/commands/mcp_config.rs`；`lib.rs` 註冊 4 個新 commands（list / upsert / delete / set-enabled）。
- 相依：新增 `toml_edit`（TOML 格式保留編輯）；`serde_json` 需啟用 `preserve_order` feature（避免改寫 `.claude.json` 時打亂鍵順序）。
- 風險：直接改寫使用者的 `.claude.json`（含大量非 MCP 設定）與 codex `config.toml`，寫入必須採 atomic write 且僅動目標區段。
