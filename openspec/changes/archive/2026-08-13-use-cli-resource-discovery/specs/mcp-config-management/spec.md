## ADDED Requirements

### Requirement: MCP provider 分頁依啟用平台與既有 adapter 交集顯示

MCP 管理 SHALL 以 `AppSettings.enabledProviders` 篩選 provider；只有同時啟用且已有 MCP 設定 adapter 的 Claude、Codex、OpenCode、Copilot 可顯示分頁及執行設定 I/O。停用 provider SHALL 不讀取設定檔、不執行 CLI、不顯示分頁。Antigravity 未有本 change 定義的 MCP 設定 adapter，因此即使啟用也 SHALL NOT 建立空白或推測性分頁。

#### Scenario: 只啟用 OpenCode
- **WHEN** `enabledProviders` 只包含 `opencode`
- **THEN** MCP 頁只顯示 OpenCode，且不讀取 Claude、Codex、Copilot 設定

#### Scenario: 啟用 Antigravity 不建立空白 MCP 分頁
- **WHEN** `enabledProviders` 包含 `antigravity`
- **THEN** 系統不因 provider 已啟用便建立無 adapter 的 Antigravity MCP 分頁

### Requirement: Copilot 與 Codex MCP 合併 CLI effective 狀態

MCP 設定總覽 SHALL 保留現有四家設定檔的編輯資料，但只對 Copilot 與 Codex 執行已驗證的 JSON CLI 查詢並合併 effective/configured 狀態。project scope SHALL 在專案工作目錄查詢合併結果，global scope SHALL 在中立工作目錄查詢。OpenCode 與 Claude MCP SHALL 不執行 CLI 狀態查詢、不解析文字輸出，也不顯示 runtime 連線或未驗證狀態。

#### Scenario: Copilot CLI 確認 effective server
- **WHEN** Copilot MCP JSON 清單在指定 scope 回傳某個 server
- **THEN** 清單保留設定編輯資料並顯示該 server 為 CLI effective/configured

#### Scenario: Codex 專案與全域結果隔離
- **WHEN** Codex MCP JSON 在專案工作目錄與中立工作目錄回傳不同清單
- **THEN** 專案群組與全域群組分別顯示各自查詢所得的 effective/configured 狀態

#### Scenario: CLI 回傳非目前設定檔來源的 server
- **WHEN** Copilot 或 Codex CLI 從合併設定中回傳 SessionHub 目前設定檔解析未發現的 server
- **THEN** 清單仍顯示該 server、執行狀態及可取得的來源資訊
- **AND** 若該來源不可由 SessionHub 安全寫入，編輯與刪除操作 SHALL 停用

### Requirement: 未支援或失敗的 MCP CLI 檢查不影響設定管理

OpenCode 與 Claude MCP SHALL 完全跳過 CLI 檢查並只顯示設定檔資料。Copilot 或 Codex CLI 不存在、逾時或 JSON 無法解析時，系統 SHALL 繼續回傳該 provider 的設定檔清單，不建立 unknown/error per-server 狀態，只在 provider 群組顯示非阻擋提示；其他 provider 與 scope SHALL 不受影響。CLI 輸出中的環境變數值、認證標頭及 token MUST NOT 回傳至前端或寫入記錄。

#### Scenario: 未支援 provider 不啟動 MCP CLI
- **WHEN** 使用者開啟 OpenCode 或 Claude MCP 分頁
- **THEN** 系統不啟動其 MCP list 指令，清單維持設定檔管理模式

#### Scenario: Copilot CLI 解析失敗
- **WHEN** Copilot MCP 指令回傳無效 JSON
- **THEN** Copilot 清單仍顯示設定檔中的 servers，provider 群組顯示無法更新 CLI 狀態
- **AND** Codex 與其他設定檔清單不受影響

#### Scenario: CLI 輸出包含機密欄位
- **WHEN** provider 查詢輸出包含 header 或環境變數值
- **THEN** 後端只回傳 server 名稱、狀態、scope、來源與經清理的錯誤摘要
- **AND** 不回傳或記錄機密值
