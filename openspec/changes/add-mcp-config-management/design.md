# Design: add-mcp-config-management

## Context

四個 provider 的 MCP 設定分散且格式不一，且各有 global 與 project 兩種 scope。經實地驗證（2026-07-07，使用者機器上的實際檔案）與官方文件查證：

Global scope：

| Provider | 設定檔 | 區段 | 格式 | 原生 enabled 旗標 |
| --- | --- | --- | --- | --- |
| claude | `%USERPROFILE%\.claude.json` | 頂層 `mcpServers` | JSON | 無 |
| codex | `<codexRoot>\config.toml` | `[mcp_servers.<name>]` | TOML | 有（`enabled = false`） |
| opencode | `%USERPROFILE%\.config\opencode\opencode.json` | `mcp` | JSON | 有（`"enabled": false`） |
| copilot | `<copilotRoot>\mcp-config.json` | `mcpServers` | JSON | 無 |

Project scope（`<project>` = 專案根目錄）：

| Provider | 設定檔 | 區段 | 格式 | 依據 |
| --- | --- | --- | --- | --- |
| claude | `<project>\.mcp.json` | `mcpServers` | JSON | Claude Code 專案層 `.mcp.json` 慣例 |
| codex | `<project>\.codex\config.toml` | `[mcp_servers.<name>]` | TOML | Codex 在 trusted 專案讀取 `.codex/config.toml` |
| opencode | `<project>\opencode.json` | `mcp` | JSON | OpenCode 專案根目錄設定檔 |
| copilot | `<project>\.github\mcp.json` | `mcpServers` | JSON | Copilot CLI 讀取 `.github/mcp.json`（亦接受 `.mcp.json`）|

Project scope 的原生 enabled 旗標與停用策略與 global 相同（依 provider 分流，見 D4）。

限制：

- `.claude.json` 是 Claude Code 的主設定檔，內含大量非 MCP 內容（專案歷史、偏好），只允許動 `mcpServers` 區段，且不可打亂其餘內容。
- codex `config.toml` 可能含使用者手寫註解與排版，改寫時必須保留。
- 專案慣例：IPC 集中於 `App.tsx`、子元件純顯示、Rust command 回傳 `Result<T, String>`、文字全走 i18n。

## Goals / Non-Goals

**Goals**

- 檢視與維護四個平台的 MCP server，涵蓋 global 與 project 兩種 scope。
- 各平台、各 scope 獨立管理：新增、編輯（含改名）、啟用/停用、刪除。
- 寫入安全：atomic write、僅動目標區段、格式保留（TOML 註解、JSON 鍵順序）。
- 後端邏輯 scope 化：同一套讀寫程式以「設定檔路徑 + 區段鍵 + 格式」參數化，global/project 共用。

**Non-Goals**

- 不做跨平台同步或複製（與 Agents skills 的同步矩陣不同）。
- 不做 global↔project 之間的複製或繼承檢視（各 scope 各自獨立管理）。
- 不驗證 MCP server 本身可否連線／啟動，只管設定檔內容。
- 不提供修改 codex trust 狀態的功能（trust 由 codex CLI 自行判定；SessionHub 只讀取並提示現況，不寫入 `[projects.*]` 區塊）。

## Decisions

### D1: 設定內容以「自由 JSON 物件」呈現與編輯，不建結構化表單

各平台 schema 差異大（copilot 的 `command` 是字串 + `args` 陣列；opencode 的 `command` 是陣列；codex 另有 `env`、`startup_timeout_sec`、`http_headers` 等）。UI 以 name 欄位 + JSON textarea 編輯完整設定值，儲存前僅驗證「必須是合法 JSON 物件」。
替代方案：per-provider 表單——欄位漂移快、維護成本高，且擋不住新欄位，放棄。

### D2: codex TOML 使用 `toml_edit` 做格式保留編輯

`toml` crate 序列化會整檔重寫、丟失註解；`toml_edit`（DocumentMut）可只改 `[mcp_servers.*]` 節點並保留其餘原文。JSON 值與 TOML 值之間需要小型雙向轉換器（object→table、array→array、拒絕 null）。

### D3: `serde_json` 啟用 `preserve_order` feature

改寫 `.claude.json` 等檔案時若鍵被 BTreeMap 排序，diff 會爆炸且影響使用者檔案可讀性。`preserve_order` 讓 Map 改用 IndexMap、維持原始鍵順序，對既有程式無行為影響。

### D4: 停用策略依平台分流

- codex / opencode：寫入原生 `enabled` 旗標。停用時設 `enabled = false`；啟用時**移除**該鍵（預設即啟用，保持檔案乾淨）。
- claude / copilot：無原生旗標。停用時把該 server 的設定值從 provider 設定檔移除，搬到 SessionHub 應用資料的暫存檔 `%APPDATA%\SessionHub\mcp-disabled.json`；啟用時原樣寫回並自暫存移除。清單 API 需合併暫存項目（標示 `enabled: false`）。
  替代方案：claude 用 `disabledMcpjsonServers`——語意不穩定且僅部分 scope 適用，放棄。
- 暫存檔結構以 provider + scope 為鍵，避免 global 與不同專案的同名 server 互相覆蓋：
  ```json
  { "<provider>::<scopeKey>": { "<name>": <原始設定值> } }
  ```
  `scopeKey` = global 用字面 `"global"`；project 用正規化（小寫、統一分隔符）後的 `project_cwd`。

### D5: scope 以 `McpScope` enum 參數化，一套讀寫邏輯共用

沿用 Agents 的 `AgentsScope` 模式，定義：

```rust
#[serde(tag = "kind", rename_all = "camelCase")]
enum McpScope { Global, Project { project_cwd: String } }
```

核心讀寫邏輯（列出、upsert、delete、toggle、JSON/TOML 存取、TOML↔JSON 轉換）只依賴「設定檔絕對路徑 + 區段鍵 + 格式（JSON/TOML）+ 是否有原生 enabled 旗標 + 是否用停用暫存」這組參數，與 scope 無關。scope 只影響「解析出哪個設定檔路徑」。這讓 project 支援幾乎等於「多一個路徑解析分支」，不需要複製整套邏輯。

路徑解析函式 `mcp_config_path(provider, scope) -> PathBuf`：
- global 分支沿用 `settings.rs` 的 `resolve_codex_root` / `resolve_copilot_root` / `default_opencode_config_root`；claude 取 `USERPROFILE\.claude.json`。
- project 分支以 `project_cwd` 為根，接上各 provider 的專案層相對路徑（見 Context 的 project scope 表）。

### D6: 後端 command 介面（scope 參數化）

新增 `src-tauri/src/mcp_config.rs`（邏輯 + 單元測試）與 `src-tauri/src/commands/mcp_config.rs`（Tauri 包裝，`spawn_blocking`）。四個 commands，皆帶 `scope: McpScope`：

- `list_mcp_configs(scope) -> Vec<McpProviderConfig>`：回傳該 scope 下四個平台；單一平台讀檔失敗記錄在該平台的 `error` 欄位，不影響其他平台。
- `upsert_mcp_server(scope, provider, name, original_name?, config_json)`：新增／編輯；`original_name` 與 `name` 不同時視為改名（移除舊鍵）。若項目目前在停用暫存中，更新暫存內容並保持停用。
- `delete_mcp_server(scope, provider, name)`：同時清除設定檔與停用暫存中的同名項目。
- `set_mcp_server_enabled(scope, provider, name, enabled)`：依 D4 分流。

型別（camelCase serde）：

```rust
struct McpServerEntry { name: String, enabled: bool, config_json: String }
struct McpProviderConfig { provider_id: String, config_path: String, config_exists: bool, servers: Vec<McpServerEntry>, error: Option<String> }
```

寫入沿用 `agents_config.rs` 的 `atomic_write_file`（改為 `pub(crate)`）。project scope 寫入時若設定檔或父目錄不存在，比照 global 建立；不擅自建立 `.codex` / `.github` 以外的目錄結構——僅在使用者實際新增 server 時才建檔。

### D7: 前端 `McpConfigView` 由 scope prop 驅動，global 頁與 project sub-tab 共用

- `McpConfigView.tsx` 為純顯示元件，收 `scope: McpScope` 與資料/handlers props：provider 分頁（sub-tab-bar）、工具列（設定檔路徑、外部開啟、檔案總管顯示、重新整理、新增）、server 表格（名稱／狀態 pill／摘要／操作）。摘要欄從設定 JSON 摘出 `url` 或 `command + args`。同一元件同時服務 global 與 project，僅 scope 與資料來源不同。
- 編輯器用 dialog（`dialog-backdrop`/`dialog-card`）：name 輸入 + JSON textarea + 前端 JSON 驗證與重名檢查。
- 依專案慣例，queries/mutations 與 invoke 集中在 `App.tsx`；global 與 project 各自一組 query（query key 帶 scope），mutation 帶 scope 參數，成功後 invalidate 對應 scope 的 query。
- Global 入口：Sidebar footer 在 Agents 鈕旁新增 MCP 鈕（新 `PlugIcon`）；`activeView` 加 `"mcp-global"`（view 重設條件、標題列 title/subtitle 同步更新）。
- Project 入口：`ProjectView` 的 sub-tab 列（現有 sessions / plans / agents 等）新增 "mcp" 分頁，內容渲染 `McpConfigView` 並傳入 `{ kind: "project", projectCwd }`。project scope 的 MCP query 比照 agents 只在該 sub-tab 啟用時 fetch（`enabled` 條件），避免無謂 I/O。
- i18n：`mcp.*` 鍵補齊 zh-TW / en-US。

### D8: 停用中的 server 在清單中以獨立視覺樣式常駐顯示

`list_mcp_configs` 已將停用暫存項目合併進 `servers`（見 D4），故清單本身天生就會顯示停用項目，不需要額外的「顯示暫存」開關。UI 需求收斂為：表格中每列的啟用狀態 SHALL 用可辨識的 pill 呈現（例如「啟用中」/「已停用」兩種樣式），停用列 SHALL 允許照常編輯/刪除/重新啟用，與啟用列同等可操作，只是不參與 provider 實際載入。

### D9: codex 專案 trust 狀態改為主動偵測並顯示提示

`~/.codex/config.toml` 有 `[projects."<絕對路徑>"]` 區塊記錄每個專案的 `trust_level`（例如 `trust_level = "trusted"`）。新增唯讀輔助函式 `is_codex_project_trusted(project_cwd) -> bool`：讀取 global `~/.codex/config.toml`，比對正規化後的專案路徑作為 key，找不到區塊或 `trust_level` 不是 `"trusted"` 一律視為 untrusted。

此函式只在 project scope 的 codex 分頁使用，回應中額外帶一個 `codexTrusted: bool` 欄位（不影響 `McpProviderConfig` 既有結構，作為 sibling 欄位或另開一支輕量 command `check_codex_project_trust(project_cwd) -> bool`，避免污染四個泛用 CRUD command 的簽名）。前端在 codex 分頁頂端顯示提示 banner：untrusted 時提示「此專案尚未被 codex 信任，於此新增的 MCP 設定不會生效，請先於 codex CLI 執行信任該專案」；trusted 時不顯示。
替代方案：只顯示靜態說明文字、不偵測——使用者仍可能誤以為設定已生效，放棄。

## Risks / Trade-offs

- [改寫 `.claude.json` 損壞 Claude Code 設定] → atomic write（temp + rename）、只動 `mcpServers` 鍵、`preserve_order` 保序；解析失敗時整個操作中止並回傳錯誤，絕不寫入半成品。
- [codex TOML 轉換丟資料（datetime、巢狀表）] → `toml_edit` 只重寫被編輯的 server 節點；JSON→TOML 轉換拒絕 null 並回報錯誤。
- [停用暫存與外部手動編輯衝突（使用者手動把同名 server 加回）] → 啟用時若設定檔已存在同名項目，以還原值覆蓋；清單顯示以設定檔為準、暫存項目後綴合併，同名時設定檔優先。
- [CLI 工具讀檔瞬間撞上寫入] → atomic rename 保證讀到的是完整舊檔或完整新檔。
- [同名 server 在 global 與多個專案間共用停用暫存互相覆蓋] → 暫存鍵含 provider + scopeKey（見 D4），彼此隔離。
- [copilot 專案設定檔位置有 `.github/mcp.json` 與 `.mcp.json` 兩種慣例] → 本次固定寫 `.github/mcp.json`（官方文件主推、與既有 Agents 的 copilot 專案路徑慣例一致）；讀取時若 `.github/mcp.json` 不存在但 `.mcp.json` 存在則讀後者，寫入一律回 `.github/mcp.json`。
- [codex trust 路徑比對因大小寫或斜線分隔符不同導致誤判 untrusted] → 比對前對兩側路徑做正規化（統一小寫、統一 `\`/`/`），與現有 `agents_config.rs` 的專案路徑正規化手法一致。

## Migration Plan

純新增功能，無資料遷移。回滾即移除新頁面與 commands；`mcp-disabled.json` 為附加檔案，殘留無害。

## Open Questions

（無未決問題；先前兩項已收斂為 D8、D9 並落實於 spec。）
