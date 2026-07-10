# mcp-config-management

## ADDED Requirements

### Requirement: MCP 設定範圍（scope）

系統 SHALL 支援兩種 MCP 設定範圍：global（跨專案的使用者層級）與 project（單一專案）。所有列出、新增、編輯、啟用/停用、刪除操作 SHALL 以 scope 參數化，global 與 project 使用同一套後端邏輯，僅設定檔路徑不同。Global scope 自 sidebar footer 的 MCP 頁進入；project scope 自 ProjectView 的 "MCP" sub-tab 進入，作用於當前專案根目錄。

#### Scenario: 全域範圍管理

- **WHEN** 使用者自 sidebar 開啟 MCP 頁
- **THEN** 顯示的是各平台的 global 設定檔內容，操作寫入 global 設定檔

#### Scenario: 專案範圍管理

- **WHEN** 使用者在某專案的 MCP sub-tab 操作
- **THEN** 顯示與寫入的是該專案根目錄下各平台的 project 設定檔，不影響 global 設定

### Requirement: MCP 設定總覽

系統 SHALL 在指定 scope 下以平台分頁列出 claude、codex、opencode、copilot 四個 provider 的 MCP server 清單。每個平台 SHALL 顯示設定檔完整路徑與是否存在；清單中每個 server SHALL 顯示名稱、啟用狀態與設定摘要（remote 類型顯示 `url`，local 類型顯示 `command` 與 `args` 組合）。

#### Scenario: 檢視各平台 MCP 清單

- **WHEN** 使用者開啟 MCP 頁（任一 scope）並切換到任一平台分頁
- **THEN** 顯示該平台該 scope 設定檔中所有 MCP server（名稱、啟用狀態、摘要），並顯示設定檔路徑

#### Scenario: 設定檔不存在

- **WHEN** 某平台在該 scope 的設定檔尚不存在
- **THEN** 該平台顯示空清單與「設定檔不存在」提示，且不視為錯誤；其餘平台正常顯示

#### Scenario: 單一平台讀取失敗不影響其他平台

- **WHEN** 某平台設定檔存在但無法解析（格式損壞）
- **THEN** 該平台顯示錯誤訊息，其他平台清單仍正常載入

### Requirement: 各平台設定檔的讀寫位置與格式

後端 SHALL 依下列位置讀寫各平台的 MCP 設定，且寫入時 MUST 採 atomic write（temp 檔 + rename）並僅修改 MCP 區段、不得變動檔案其餘內容。

Global scope：

- claude：`%USERPROFILE%\.claude.json` 頂層 `mcpServers`（JSON，改寫時 MUST 保持既有鍵順序）
- codex：`<codexRoot>\config.toml` 的 `[mcp_servers.<name>]`（TOML，改寫時 MUST 保留既有註解與排版）
- opencode：`%USERPROFILE%\.config\opencode\opencode.json` 的 `mcp`（JSON）
- copilot：`<copilotRoot>\mcp-config.json` 的 `mcpServers`（JSON）

Project scope（`<project>` = 專案根目錄）：

- claude：`<project>\.mcp.json` 的 `mcpServers`（JSON）
- codex：`<project>\.codex\config.toml` 的 `[mcp_servers.<name>]`（TOML）
- opencode：`<project>\opencode.json` 的 `mcp`（JSON）
- copilot：讀取時優先 `<project>\.github\mcp.json`，若不存在但 `<project>\.mcp.json` 存在則讀後者；寫入一律回 `<project>\.github\mcp.json`（`mcpServers`，JSON）

codexRoot 與 copilotRoot SHALL 沿用 app-settings 既有的 root 解析（使用者自訂路徑優先，否則預設家目錄）。

#### Scenario: codex 設定改寫保留註解

- **WHEN** 使用者對 codex（任一 scope）新增或編輯一個 MCP server
- **THEN** 對應 `config.toml` 中使用者原有的註解與非 MCP 區段內容原樣保留

#### Scenario: claude 設定改寫不影響其他設定

- **WHEN** 使用者對 claude global 停用或編輯一個 MCP server
- **THEN** `.claude.json` 中 `mcpServers` 以外的內容（含鍵順序）不變

#### Scenario: 專案設定寫入獨立於全域

- **WHEN** 使用者在某專案的 project scope 新增一個 MCP server
- **THEN** 只有該專案的 project 設定檔被建立/修改，global 設定檔不變

#### Scenario: copilot 專案設定檔位置解析

- **WHEN** 某專案同時存在 `.github\mcp.json` 與 `.mcp.json`
- **THEN** 讀取以 `.github\mcp.json` 為準；且任何寫入都落在 `.github\mcp.json`

### Requirement: 新增與編輯 MCP server

系統 SHALL 支援在任一平台新增與編輯 MCP server。編輯器 SHALL 提供名稱欄位與設定內容（JSON）編輯區；儲存前 MUST 驗證名稱非空白、設定內容為合法 JSON 物件，驗證失敗 MUST 阻止儲存並顯示錯誤。編輯時允許改名：改名 SHALL 移除舊名稱項目並以新名稱寫入。新增時若名稱與既有項目重複 MUST 阻止並提示。codex 平台 SHALL 將 JSON 設定值轉換為對應 TOML 結構寫入；設定值含 `null` 時 MUST 回報錯誤（TOML 不支援）。

#### Scenario: 新增 server 且設定檔不存在

- **WHEN** 使用者在設定檔尚不存在的平台新增 MCP server 並儲存
- **THEN** 系統建立設定檔（含必要的父目錄與區段結構）並寫入該 server

#### Scenario: 無效 JSON 被拒絕

- **WHEN** 使用者輸入的設定內容不是合法 JSON 物件（語法錯誤、或為陣列/字串）
- **THEN** 儲存被阻止並顯示可理解的錯誤訊息，設定檔不被寫入

#### Scenario: 改名

- **WHEN** 使用者編輯既有 server 並修改名稱後儲存
- **THEN** 設定檔中舊名稱項目被移除，新名稱項目寫入相同（或已編輯的）設定值

### Requirement: 啟用與停用 MCP server

系統 SHALL 支援逐一啟用/停用 MCP server，策略依平台原生能力分流：

- codex 與 opencode：停用 SHALL 在該 server 設定寫入 `enabled = false`（TOML）／`"enabled": false`（JSON）；啟用 SHALL 移除該旗標。清單的啟用狀態 SHALL 以「`enabled` 不為 false」判定。
- claude 與 copilot：無原生旗標。停用 SHALL 將該 server 自設定檔移除並將原始設定值暫存至 `%APPDATA%\SessionHub\mcp-disabled.json`（結構 `{"<provider>::<scopeKey>": {"<name>": <設定值>}}`，`scopeKey` 為 `"global"` 或正規化後的專案路徑，確保 global 與各專案的同名 server 互不覆蓋）；啟用 SHALL 將暫存值原樣寫回設定檔並自暫存移除。清單 SHALL 合併顯示對應 scope 暫存中的停用項目（`enabled: false`）。

#### Scenario: opencode 原生停用

- **WHEN** 使用者停用 opencode 的某個 server
- **THEN** `opencode.json` 中該 server 增加 `"enabled": false`，其餘欄位不變；再次啟用時該鍵被移除

#### Scenario: claude 停用搬移至暫存

- **WHEN** 使用者停用 claude 的某個 server
- **THEN** 該 server 自 `.claude.json` 的 `mcpServers` 移除、完整設定值存入 `mcp-disabled.json`，且清單仍顯示該項目並標示為停用

#### Scenario: claude 啟用還原

- **WHEN** 使用者啟用一個處於停用暫存中的 claude server
- **THEN** 暫存的設定值原樣寫回 `.claude.json` 的 `mcpServers`，暫存檔中該項目被移除

#### Scenario: 編輯停用中的項目

- **WHEN** 使用者編輯一個目前停用（位於暫存）的 claude/copilot server 並儲存
- **THEN** 更新暫存中的設定值且項目保持停用，不寫回 provider 設定檔

#### Scenario: 停用項目在清單中常駐可見

- **WHEN** 清單中存在已停用的 MCP server（無論是 codex/opencode 的 `enabled: false` 或 claude/copilot 的暫存項目）
- **THEN** 該項目 SHALL 與啟用中項目並列顯示於同一清單，以獨立的「已停用」樣式標示，且可照常編輯、刪除、重新啟用

### Requirement: 刪除 MCP server

系統 SHALL 支援刪除 MCP server，執行前 MUST 經確認對話框。刪除 SHALL 同時移除 provider 設定檔中的項目與停用暫存中的同名項目；項目不存在時刪除 SHALL 視為成功（冪等）。

#### Scenario: 刪除已啟用項目

- **WHEN** 使用者確認刪除某平台的一個 MCP server
- **THEN** 該項目自設定檔移除且不再出現在清單

#### Scenario: 刪除停用中項目

- **WHEN** 使用者確認刪除一個處於停用暫存的 server
- **THEN** 暫存檔中該項目被移除且不再出現在清單

### Requirement: codex 專案信任狀態提示

系統 SHALL 在 project scope 的 codex 分頁偵測並提示該專案是否已被 codex CLI 信任。後端 SHALL 提供讀取 `~/.codex/config.toml` 中 `[projects."<專案路徑>"]` 區塊的 `trust_level` 欄位之能力；比對專案路徑前 MUST 正規化（統一大小寫、統一路徑分隔符）。找不到對應區塊或 `trust_level` 不等於 `"trusted"` 時 SHALL 視為未信任（untrusted）。前端 SHALL 在 codex 專案分頁頂端顯示提示：未信任時顯示警示文字，說明此專案尚未被 codex 信任、於此新增的 MCP 設定不會生效，並提示使用者需先於 codex CLI 信任該專案；已信任時不顯示提示。此偵測為唯讀，系統 SHALL NOT 修改 `[projects.*]` 區塊或以其他方式變更專案的信任狀態。

#### Scenario: 專案未被 codex 信任

- **WHEN** 使用者開啟某專案的 codex MCP 分頁，且該專案在 `~/.codex/config.toml` 中無 `[projects."<路徑>"]` 區塊或 `trust_level` 不是 `"trusted"`
- **THEN** 分頁頂端顯示提示：此專案尚未被 codex 信任，於此設定的 MCP server 不會生效

#### Scenario: 專案已被 codex 信任

- **WHEN** 使用者開啟某專案的 codex MCP 分頁，且該專案的 `trust_level` 為 `"trusted"`
- **THEN** 不顯示信任狀態提示

#### Scenario: global codex 分頁不受影響

- **WHEN** 使用者開啟 global scope 的 codex MCP 分頁
- **THEN** 不進行也不顯示 trust 狀態偵測（trust 僅與專案層設定相關）

### Requirement: Tauri command 介面

後端 SHALL 提供四個 Tauri commands，皆帶 `scope` 參數（`{ kind: "global" }` 或 `{ kind: "project", projectCwd }`）、回傳 `Result<T, String>` 並以 `spawn_blocking` 執行檔案 I/O：

- `list_mcp_configs(scope) -> Vec<McpProviderConfig>`：回傳四個平台的 `{ providerId, configPath, configExists, servers: [{ name, enabled, configJson }], error? }`
- `upsert_mcp_server(scope, provider, name, originalName?, configJson)`
- `delete_mcp_server(scope, provider, name)`
- `set_mcp_server_enabled(scope, provider, name, enabled)`
- `check_codex_project_trust(projectCwd) -> bool`：回傳該專案是否已被 codex 信任，僅 project scope 的 codex 分頁使用

前端 IPC 呼叫 SHALL 集中於 `App.tsx`，`McpConfigView` 為純顯示元件，並可由 scope prop 同時服務 global 頁與 project sub-tab；所有操作成功／失敗 SHALL 以 toast 回饋，介面文字 SHALL 全部經 i18n（zh-TW 與 en-US）。

#### Scenario: 未知 provider 被拒絕

- **WHEN** command 收到四個平台以外的 provider 識別字
- **THEN** 回傳描述性錯誤，不進行任何檔案操作

#### Scenario: 操作後清單刷新

- **WHEN** 任一 upsert / delete / toggle 操作成功
- **THEN** 前端使 `mcp-configs` query 失效並重新載入清單，顯示成功 toast
