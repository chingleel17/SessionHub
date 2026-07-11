# Proposal: add-mcp-config-management

## Why

使用者同時使用 claude / codex / opencode / copilot 四種 AI CLI，各平台的全域 MCP server 設定分散在四個不同位置、三種不同格式（JSON 與 TOML），目前只能手動開檔編輯，容易改錯格式或漏改。SessionHub 已具備 Agents（AGENTS.md / skills / commands）集中管理能力，MCP 設定是同一類需求的自然延伸。

## What Changes

- Agents 頁（全域與專案）新增「MCP」頁籤，以平台分頁呈現各 provider 的 MCP server 清單；不新增獨立的 sidebar MCP 導覽項。
- 專案 Agents sub-tab 採「單一共用頁籤列 + 專案/全域收折群組」版面（頁籤同步切換、群組帶計數，仿 VS Code 分組樣式），全域群組可完整操作。
- Skills / Commands 清單改為「名稱 + 描述」VS Code 式呈現（描述自 frontmatter 抽取）並支援搜尋；同步矩陣與報告移入「同步」modal。
- 每個平台、每個 scope 支援：列出、新增、編輯（含改名）、啟用/停用、刪除 MCP server；**不做跨平台同步**。
- 後端新增讀寫四個平台設定檔的能力（global / project 兩種 scope）：
  - claude：global = `%USERPROFILE%\.claude.json` 頂層 `mcpServers`；project = `<project>\.mcp.json` 的 `mcpServers`（JSON）
  - codex：global = `<codexRoot>\config.toml`；project = `<project>\.codex\config.toml`，皆為 `[mcp_servers.<name>]`（TOML，須保留既有註解與排版）
  - opencode：global = `%USERPROFILE%\.config\opencode\opencode.json`；project = `<project>\opencode.json`，皆為 `mcp` 鍵（JSON）
  - copilot：global = `<copilotRoot>\mcp-config.json`；project = `<project>\.github\mcp.json`，皆為 `mcpServers`（JSON）
- 停用策略依平台原生能力分流：codex / opencode 原生支援 `enabled` 旗標；claude / copilot 無原生旗標，停用時將該 server 設定搬移至 SessionHub 應用資料的暫存檔（以 provider + scope 為鍵），啟用時還原寫回。
- MCP 新增/編輯採類型導向表單（HTTP/SSE、npx、本機執行檔、自訂 JSON），依 provider 組裝原生 schema；摘要欄精簡呈現（description > url > 指令 basename，截斷 + tooltip）。

## Capabilities

### New Capabilities

- `mcp-config-management`: 跨平台 MCP server 設定（global 與 project 兩種 scope）的列出、新增、編輯、啟用/停用、刪除，含各平台設定檔格式的讀寫規則、停用暫存機制、類型導向編輯表單與摘要呈現規則。

### Modified Capabilities

- `agents-config-view-ux`: Agents 頁新增 MCP 頁籤；專案 Agents 分頁採單一頁籤列 + 專案/全域收折群組；Skills / Commands 清單改名稱+描述式呈現並支援搜尋；同步操作移入同步 modal。
- `agents-skills-sync`: 掃描結果新增 skill 描述（SKILL.md frontmatter `description`）。
- `agents-commands-sync`: 掃描結果新增 command 描述（.md frontmatter `description`）。

## Impact

- 前端：新增 `src/components/McpConfigView.tsx`（內嵌內容元件，scope 由 props 決定）；`src/components/AgentsConfigView.tsx`（新增 MCP 頁籤、收 MCP props）；`src/App.tsx`（global/project MCP queries/mutations、雙分區 props 組裝）；`src/components/ProjectView.tsx`（Agents sub-tab 雙分區容器）；`src/components/Sidebar.tsx`（不新增 MCP 導覽）；`src/components/Icons.tsx`；`src/types/index.ts`；`src/locales/zh-TW.ts`、`src/locales/en-US.ts`。
- 後端：新增 `src-tauri/src/mcp_config.rs` 與 `src-tauri/src/commands/mcp_config.rs`；`lib.rs` 註冊 4 個新 commands（list / upsert / delete / set-enabled）。
- 相依：新增 `toml_edit`（TOML 格式保留編輯）；`serde_json` 需啟用 `preserve_order` feature（避免改寫 `.claude.json` 時打亂鍵順序）。
- 風險：直接改寫使用者的 `.claude.json`（含大量非 MCP 設定）與 codex `config.toml`，寫入必須採 atomic write 且僅動目標區段。
