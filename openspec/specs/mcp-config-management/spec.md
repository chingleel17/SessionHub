# mcp-config-management

## ADDED Requirements

### Requirement: MCP 設定範圍（scope）

系統 SHALL 支援兩種 MCP 設定範圍：global（跨專案的使用者層級）與 project（單一專案）。所有列出、新增、編輯、啟用/停用、刪除操作 SHALL 以 scope 參數化，global 與 project 使用同一套後端邏輯，僅設定檔路徑不同。MCP 管理介面 SHALL 以 Agents 頁的「MCP」頁籤呈現：global scope 自 sidebar 的全域 Agents 頁進入；project scope 自 ProjectView 的 Agents sub-tab 進入（MCP 頁籤內以「專案」「全域」scope 群組呈現，provider 分頁於群組之上共用，見 agents-config-view-ux）。系統 SHALL NOT 於 sidebar 提供獨立的 MCP 導覽項。

#### Scenario: 全域範圍管理

- **WHEN** 使用者自 sidebar 開啟全域 Agents 頁並切換到 MCP 頁籤
- **THEN** 顯示的是各平台的 global 設定檔內容（清單正常載入呈現，不得空白），操作寫入 global 設定檔

#### Scenario: 專案範圍管理

- **WHEN** 使用者在某專案 Agents sub-tab MCP 頁籤的專案群組操作
- **THEN** 顯示與寫入的是該專案根目錄下各平台的 project 設定檔，不影響 global 設定

#### Scenario: provider 分頁同步作用於兩群組

- **WHEN** 使用者於專案 MCP 頁籤將 provider 分頁切到 codex
- **THEN** 專案與全域兩個群組同時顯示 codex 的清單，無需分別切換

#### Scenario: 舊 sub-tab 狀態相容

- **WHEN** 使用者的介面狀態殘留舊版獨立 "mcp" sub-tab 識別
- **THEN** 系統將其視為 "agents" sub-tab 開啟，不得渲染空白內容

### Requirement: MCP 設定總覽

系統 SHALL 在指定 scope 下以平台分頁列出 claude、codex、opencode、copilot 四個 provider 的 MCP server 清單。每個平台 SHALL 顯示設定檔完整路徑與是否存在；清單中每個 server SHALL 顯示名稱、啟用狀態與設定摘要。摘要 SHALL 依優先序取值：設定中的 `description` 欄位 > `url` > 指令檔名（basename，不含完整路徑）加參數；摘要 MUST 以單行截斷顯示並於 tooltip 提供完整內容，不得因長路徑撐爆欄位。清單、工具列與相鄰區塊之間 SHALL 保有可辨識的垂直間距，邊框不得互相緊貼。

清單 SHALL 以表格呈現，欄位為名稱、狀態、摘要、操作。表頭（`th`）SHALL 一律置中；名稱、摘要欄內容 SHALL 靠左對齊（利於閱讀長字串），狀態欄內容 SHALL 置中對齊。表格欄寬 SHALL 為固定版面（`table-layout: fixed`）並個別指定寬度，不得沿用其他矩陣表格（如 agents 同步矩陣）的等分規則，以避免視窗縮小時表頭與內容錯位；操作欄 SHALL 有足夠寬度容納其固定按鈕組合，不得因內容換行撐高列高。

點擊清單中可編輯 server 所在的整列（列上非按鈕區域）SHALL 開啟該項目的編輯視窗，等同點擊「編輯」操作；操作欄按鈕的點擊事件 MUST 阻止冒泡，不得因此誤觸開啟編輯視窗。不可編輯（唯讀來源）的項目與操作進行中（busy）時，整列點擊 SHALL 停用。

操作欄 SHALL 以圖示按鈕（IconButton）呈現「編輯」「複製到其他工具」「刪除」三個操作；「啟用／停用」SHALL 以文字按鈕呈現於同一操作欄，依當前狀態呈現對應顏色語意（啟用中→可停用，停用中→可啟用），停用操作 SHALL 使用中性色而非危險色，以避免與刪除操作的危險語意混淆。

#### Scenario: 檢視各平台 MCP 清單

- **WHEN** 使用者開啟 MCP 頁（任一 scope）並切換到任一平台分頁
- **THEN** 顯示該平台該 scope 設定檔中所有 MCP server（名稱、啟用狀態、摘要），並顯示設定檔路徑

#### Scenario: 設定檔不存在

- **WHEN** 某平台在該 scope 的設定檔尚不存在
- **THEN** 該平台顯示空清單與「設定檔不存在」提示，且不視為錯誤；其餘平台正常顯示

#### Scenario: 單一平台讀取失敗不影響其他平台

- **WHEN** 某平台設定檔存在但無法解析（格式損壞）
- **THEN** 該平台顯示錯誤訊息，其他平台清單仍正常載入

#### Scenario: 點擊列開啟編輯

- **WHEN** 使用者點擊某個可編輯 server 所在列的非按鈕區域
- **THEN** 開啟該 server 的編輯視窗，內容與點擊「編輯」按鈕相同

#### Scenario: 點擊操作欄按鈕不觸發列點擊

- **WHEN** 使用者點擊列內的「編輯」「複製到其他工具」「刪除」或「啟用／停用」按鈕
- **THEN** 僅觸發該按鈕自身的操作，不重複開啟編輯視窗

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

系統 SHALL 支援在任一平台新增與編輯 MCP server。編輯 dialog SHALL 使用不透明樣式呈現（見 ui-primitives 的 `dialog-card--solid`），並提供名稱欄位與「類型」下拉選單（採用共用 Select 元件），依所選類型顯示對應欄位（見 design D12）：

- **HTTP/SSE**：URL（必填）與 Headers（選填，結構化 key-value 列表，見下方 Headers 遮蔽需求）
- **npx 套件**：套件名稱（必填）、額外參數（選填）、環境變數（選填 key-value 清單）
- **本機執行檔**：執行檔路徑（必填）、參數（選填）、環境變數（選填 key-value 清單）
- **自訂 JSON**：完整 JSON 編輯區（設定值原樣寫入），提供「自動解析」操作（見下方需求）

儲存時系統 SHALL 依當前 provider 將表單欄位組裝為該平台原生 schema（如 opencode 的 `type: "local"/"remote"` 與 `command` 陣列格式、codex 的 `http_headers`）。編輯既有項目時 SHALL 反解析設定值帶入對應類型的表單（`url` → HTTP/SSE；`command` 為 npx → npx 套件；其餘 `command` → 本機執行檔；無法對應者以自訂 JSON 呈現原始內容）。

儲存前 MUST 驗證：名稱非空白；各類型必填欄位非空；自訂 JSON 為合法 JSON 物件；驗證失敗 MUST 阻止儲存並顯示錯誤。編輯時允許改名：改名 SHALL 移除舊名稱項目並以新名稱寫入。新增時若名稱與既有項目重複 MUST 阻止並提示。codex 平台 SHALL 將 JSON 設定值轉換為對應 TOML 結構寫入；設定值含 `null` 時 MUST 回報錯誤（TOML 不支援）。

#### Scenario: 以 HTTP 類型新增

- **WHEN** 使用者選擇 HTTP/SSE 類型、填入 URL 並儲存於 opencode
- **THEN** 寫入 `{"type": "remote", "url": "<url>", "enabled": true}` 形式的設定（依 opencode schema）

#### Scenario: 以 npx 類型新增

- **WHEN** 使用者選擇 npx 套件類型、填入套件名稱並儲存於 claude
- **THEN** 寫入 `{"command": "npx", "args": ["-y", "<pkg>"]}` 形式的設定

#### Scenario: 新增 server 且設定檔不存在

- **WHEN** 使用者在設定檔尚不存在的平台新增 MCP server 並儲存
- **THEN** 系統建立設定檔（含必要的父目錄與區段結構）並寫入該 server

#### Scenario: 無效自訂 JSON 被拒絕

- **WHEN** 使用者以自訂 JSON 類型輸入不合法內容（語法錯誤、或為陣列/字串）
- **THEN** 儲存被阻止並顯示可理解的錯誤訊息，設定檔不被寫入

#### Scenario: 編輯既有項目帶入結構化表單

- **WHEN** 使用者編輯一個設定值為 `{"command": "npx", "args": ["-y", "foo"]}` 的既有 server
- **THEN** dialog 以 npx 套件類型開啟，套件名稱欄位帶入 `foo`

#### Scenario: 改名

- **WHEN** 使用者編輯既有 server 並修改名稱後儲存
- **THEN** 設定檔中舊名稱項目被移除，新名稱項目寫入相同（或已編輯的）設定值

### Requirement: HTTP Headers 顯示遮蔽

HTTP/SSE 類型的 Headers 欄位 SHALL 以結構化 key-value 列表呈現，支援新增、刪除單一列。每列的值輸入框 SHALL 預設以密碼欄位（`type="password"`）遮蔽內容，並提供逐列的顯示/隱藏切換（眼睛圖示）。此遮蔽僅為畫面顯示層級，避免螢幕共享或側錄時外洩憑證；系統 SHALL NOT 對寫入設定檔的內容做任何加密處理，值仍以明文寫入 provider 設定檔（MCP 協定本身即以明文儲存 header 值，此為既有限制而非本需求引入的行為）。

儲存前，若任一列的值非空但名稱（key）為空白，MUST 阻止儲存並提示錯誤；名稱與值皆為空白的列 SHALL 於組裝設定時忽略、不寫入。切換某列的顯示/隱藏狀態 SHALL NOT 清除當前的測試連線結果或表單錯誤訊息（純顯示操作，非資料變更）。

#### Scenario: 新增 Header 列

- **WHEN** 使用者在 HTTP/SSE 類型的編輯表單點擊「新增 Header」
- **THEN** 新增一列空白的名稱/值輸入框，值欄位預設為遮蔽狀態

#### Scenario: 顯示/隱藏 Header 值

- **WHEN** 使用者點擊某列的顯示/隱藏切換
- **THEN** 該列的值欄位在明文與遮蔽兩種顯示間切換，其餘列不受影響，且不影響已顯示的測試連線結果

#### Scenario: 空白名稱阻止儲存

- **WHEN** 使用者填入某列的值但未填名稱，並嘗試儲存
- **THEN** 儲存被阻止並顯示錯誤，提示名稱不可為空白

### Requirement: 自訂 JSON 自動解析

自訂 JSON 類型的編輯表單 SHALL 提供「自動解析」操作，將使用者貼上的原生 MCP server JSON 解析回結構化表單（HTTP/SSE、npx 套件、本機執行檔三者之一），沿用與「編輯既有項目反解析」相同的解析邏輯。

自動解析 SHALL 支援以下輸入型態：

- 單一 server 的設定物件（如 `{"command": "npx", "args": [...]}`）
- 整份或部分 provider 設定檔，外層包著已知的 MCP 區段鍵（`mcpServers`、`mcp_servers`、`mcp`）；即使該區段鍵旁存在其他同層鍵（如 `$schema`、其他非 MCP 設定）亦 SHALL 正確解開外殼取出區段內容，不因此判定為無法解析
- 解開區段後，若名稱欄位尚為空白，SHALL 以區段內該 server 的鍵名稱自動帶入名稱欄位

自動解析在下列情況 SHALL 停留於自訂 JSON 模式並顯示對應原因，不切換表單類型、不遺失使用者已輸入的內容：

- **多個 server**：解開的區段內含有一個以上的 server，無法判斷要解析哪一個
- **含未支援欄位**：解析出合法的 `url` 或 `command` 結構，但物件中含有結構化表單無法承載的欄位（如 `tools`）；為避免儲存時靜默丟失該欄位，SHALL 保留原始輸入為自訂 JSON，並於錯誤訊息中列出造成無法轉換的欄位名稱
- **完全無法辨識**：JSON 語法錯誤，或內容不符合任何已知的 MCP server 結構

#### Scenario: 解析單一 server 設定

- **WHEN** 使用者於自訂 JSON 貼上 `{"type": "remote", "url": "https://x/mcp", "headers": {"Authorization": "Bearer x"}}` 並點擊自動解析
- **THEN** 表單切換為 HTTP/SSE 類型，URL 帶入該值，Headers 列表帶入一列 `Authorization`（值為遮蔽狀態）

#### Scenario: 解析整份設定檔（含其他同層鍵）

- **WHEN** 使用者貼上 `{"$schema": "...", "mcp": {"my-server": {"type": "local", "command": [...]}}}` 並點擊自動解析
- **THEN** 系統解開 `mcp` 區段取出 `my-server` 的設定，表單切換為對應類型，且名稱欄位（原為空白）帶入 `my-server`

#### Scenario: 偵測到多個 server

- **WHEN** 使用者貼上的區段內含兩個以上的 server 並點擊自動解析
- **THEN** 表單停留於自訂 JSON，顯示提示說明偵測到多個 server、僅支援單一 server，不視為格式錯誤

#### Scenario: 含表單無法承載的欄位

- **WHEN** 使用者貼上 `{"type": "remote", "url": "...", "tools": ["*"]}` 並點擊自動解析
- **THEN** 表單停留於自訂 JSON（原始內容不變），顯示提示列出 `tools` 為無法轉換的欄位，不視為格式錯誤

#### Scenario: 完全無法辨識

- **WHEN** 使用者貼上的內容非合法 JSON，或不含 `url`／`command` 等已知欄位
- **THEN** 表單停留於自訂 JSON，顯示通用的「無法自動解析」錯誤

### Requirement: 測試 HTTP/SSE 連線

HTTP/SSE 類型的編輯表單 SHALL 提供「測試連線」操作，位於對話框底部與「取消」「儲存」同一列（左側）。測試 SHALL 使用表單當下（尚未儲存）的 URL 與 Headers 值，不依賴已儲存的設定。

測試前 MUST 驗證 URL 非空白、Headers 無空白名稱帶值的列，驗證失敗 SHALL 顯示對應錯誤且不發出請求。測試結果 SHALL 分類顯示於按鈕旁：連線正常、認證失敗（401/403）、連線成功但回應非預期（含狀態碼）、無法連線（含錯誤訊息）。修改 URL 或任一 Header 列、或重新開啟編輯視窗時，SHALL 清除前次測試結果。

系統 SHALL NOT 對 npx 套件、本機執行檔、自訂 JSON 類型提供測試連線操作（子行程握手為不同機制，不在此範圍）。

#### Scenario: 測試連線成功

- **WHEN** 使用者於 HTTP/SSE 表單填入可連線的 URL 並點擊測試連線
- **THEN** 顯示「連線正常」

#### Scenario: 測試連線認證失敗

- **WHEN** 目標端點回傳 401 或 403
- **THEN** 顯示「認證失敗」

#### Scenario: 修改欄位後清除舊結果

- **WHEN** 使用者於測試連線後修改 URL 或任一 Header
- **THEN** 先前顯示的測試結果被清除

#### Scenario: 非 HTTP 類型不提供測試

- **WHEN** 使用者將表單類型切換為 npx 套件、本機執行檔或自訂 JSON
- **THEN** 不顯示測試連線操作

### Requirement: 複製 MCP server 到其他工具

系統 SHALL 支援將同一 scope 下、某平台的 MCP server 複製到另一個已啟用的平台。複製操作 SHALL 開啟對話框，供使用者選擇目標平台（排除來源平台自身，僅列出使用者已啟用的平台）與可編輯的名稱。

複製邏輯 SHALL 重用既有的反解析（設定值 → 結構化表單）與組裝（結構化表單 → 目標平台原生 schema）邏輯，確保跨平台的 schema 差異（如 opencode 的 `command` 陣列格式 vs. claude/codex/copilot 的字串 + `args`、codex 的 `http_headers` vs. 其他平台的 `headers`）被正確轉換。來源設定值若無法反解析為結構化表單（自訂/未知格式），SHALL 阻止複製並提示需手動於目標平台新增，不做逐欄位語意轉換。

儲存前 MUST 驗證：目標名稱非空白；目標平台不存在同名 server。系統 SHALL NOT 支援跨 scope（全域與專案間）複製。

#### Scenario: 複製 npx 類型 server

- **WHEN** 使用者將 opencode 的一個 npx 類型 server 複製到 claude
- **THEN** claude 設定檔中新增 `{"command": "npx", "args": ["-y", "<pkg>"]}` 形式的項目（`command` 由陣列轉為字串 + `args`）

#### Scenario: 複製 HTTP 類型 server 至 codex

- **WHEN** 使用者將一個帶有 `headers` 的 HTTP 類型 server 複製到 codex
- **THEN** codex 設定檔中新增的項目使用 `http_headers` 鍵，而非 `headers`

#### Scenario: 目標平台已有同名項目

- **WHEN** 目標平台已存在與複製名稱相同的 server
- **THEN** 複製被阻止並提示名稱重複

#### Scenario: 無法辨識格式的來源被阻止複製

- **WHEN** 使用者嘗試複製一個設定值為自訂/未知格式的 server
- **THEN** 複製被阻止並提示此格式無法自動轉換，需手動於目標平台新增

#### Scenario: 沒有可複製目標時停用操作

- **WHEN** 當前 scope 下僅有一個平台被啟用（無其他可複製目標）
- **THEN** 「複製到其他工具」操作呈現停用狀態

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

後端 SHALL 提供六個 Tauri commands。前五個皆帶 `scope` 參數（`{ kind: "global" }` 或 `{ kind: "project", projectCwd }`）、回傳 `Result<T, String>` 並以 `spawn_blocking` 執行檔案 I/O：

- `list_mcp_configs(scope) -> Vec<McpProviderConfig>`：回傳四個平台的 `{ providerId, configPath, configExists, servers: [{ name, enabled, configJson }], error? }`
- `upsert_mcp_server(scope, provider, name, originalName?, configJson)`
- `delete_mcp_server(scope, provider, name)`
- `set_mcp_server_enabled(scope, provider, name, enabled)`
- `check_codex_project_trust(projectCwd) -> bool`：回傳該專案是否已被 codex 信任，僅 project scope 的 codex 分頁使用
- `test_mcp_http_server(url, headers) -> McpConnectionTestResult`：不帶 `scope`（不涉及任何設定檔讀寫），對指定 URL 送出 MCP JSON-RPC `initialize` 請求以驗證 HTTP/SSE 類型連線是否可用，逾時 5 秒。回傳值為以 `kind` 欄位區分的聯集：`{ kind: "ok" }`（2xx 且回應為合法 JSON-RPC 結果）、`{ kind: "unauthorized" }`（401/403）、`{ kind: "unexpectedResponse", status }`（其他非預期狀態碼）、`{ kind: "connectionFailed", message }`（DNS/TCP/TLS 等傳輸層錯誤）

前端 IPC 呼叫 SHALL 集中於 `App.tsx`（含 `test_mcp_http_server`，不得由子元件直接 `invoke()`），`McpConfigView` 為純顯示的內嵌內容元件（不自帶卡片外框），由 `AgentsConfigView` 的 MCP 頁籤容器渲染，並可由 scope prop 同時服務全域 Agents 頁與專案分區／全域分區；所有操作成功／失敗 SHALL 以 toast 回饋，介面文字 SHALL 全部經 i18n（zh-TW 與 en-US）。測試連線例外：其分類結果（成功／認證失敗／回應非預期／連線失敗）SHALL 直接顯示於編輯視窗內，不透過 toast；僅 IPC 呼叫本身的例外（非分類結果）才視為失敗。

#### Scenario: 未知 provider 被拒絕

- **WHEN** command 收到四個平台以外的 provider 識別字
- **THEN** 回傳描述性錯誤，不進行任何檔案操作

#### Scenario: 操作後清單刷新

- **WHEN** 任一 upsert / delete / toggle 操作成功
- **THEN** 前端使 `mcp-configs` query 失效並重新載入清單，顯示成功 toast

#### Scenario: 測試連線不影響清單快取

- **WHEN** 使用者觸發測試連線
- **THEN** 不進行任何 query invalidation，也不寫入任何設定檔
