## Requirements

### Requirement: Provider integration 安裝與狀態管理

系統 SHALL 能檢測 Copilot、OpenCode、Codex 與 Claude 的 bridge integration 狀態，並允許使用者由 SessionHub 自動安裝、更新或重新安裝整合檔案；系統 SHALL 同時追蹤已安裝 integration 的版本號，並在版本落差時提示使用者更新。

安裝 Codex 或 Copilot integration 時，系統 SHALL 同時確保對應的 hook 腳本已安裝至 bundled 路徑，並在 hook 設定中以腳本路徑取代內嵌命令字串。

#### Scenario: 安裝 OpenCode integration

- **WHEN** 使用者在設定頁對 OpenCode 點擊「安裝整合」
- **THEN** 系統建立或更新 OpenCode plugin 檔案到偵測到的 plugin 設定位置
- **AND** 狀態更新為 `installed` 或顯示具體錯誤

#### Scenario: 安裝 Codex integration

- **WHEN** 使用者在設定頁對 Codex 點擊「安裝整合」
- **THEN** 系統先確保 Codex hook 腳本已安裝至 bundled 路徑
- **AND** 系統建立或更新 `~/.codex/hooks.json`，hook command 引用腳本路徑
- **AND** 狀態更新為 `installed` 或顯示具體錯誤

#### Scenario: 安裝 Copilot integration

- **WHEN** 使用者在設定頁對 Copilot CLI 點擊「安裝整合」
- **THEN** 系統先確保 Copilot hook 腳本已安裝至 bundled 路徑
- **AND** 系統建立或更新 `~/.copilot/settings.json`，hook command 引用腳本路徑（Windows 用 `.ps1`，Unix 用 `.sh`）
- **AND** 狀態更新為 `installed` 或顯示具體錯誤

#### Scenario: 安裝 Claude integration

- **WHEN** 使用者在設定頁對 Claude 點擊「安裝整合」
- **THEN** 系統讀取並 merge 更新 `~/.claude/settings.json` 的 `hooks.Stop` 陣列
- **AND** 狀態更新為 `installed` 或顯示具體錯誤

#### Scenario: provider 路徑不可寫入

- **WHEN** SessionHub 無法寫入 provider 設定目錄
- **THEN** 系統將該 provider 狀態標示為 `manual_required`
- **AND** 提供快速開啟或編輯設定檔案的入口

#### Scenario: 偵測到已安裝版本過舊

- **WHEN** 使用者進入設定頁或應用程式啟動
- **AND** 已安裝插件的 `integrationVersion` 低於程式內建的 `CURRENT_PLUGIN_VERSION`
- **THEN** 系統將整合狀態標示為 `outdated` 並顯示版本資訊（已安裝 v{N} / 最新 v{M}）
- **AND** 提供「更新插件」按鈕

#### Scenario: 重新安裝已是最新版的插件

- **WHEN** 使用者在設定頁點擊「重新安裝」且插件已是最新版
- **THEN** 系統仍執行寫入（覆蓋），並顯示 success toast

### Requirement: Bridge 事件格式標準化

系統 SHALL 以統一的本地 bridge 事件格式接收 provider 更新訊號，至少包含 provider、eventType、timestamp，並在可取得時包含 sessionId 與 cwd。

#### Scenario: OpenCode plugin 發送 session 更新

- **WHEN** OpenCode plugin 發出 `session.updated` 或等效事件
- **THEN** SessionHub 接收的 bridge record 以標準欄位格式保存
- **AND** 後續 refresh 流程不需直接解析 OpenCode 原始事件格式

#### Scenario: Copilot hook 發送 session 結束

- **WHEN** Copilot hook 發出 session 結束事件
- **THEN** SessionHub 接收的 bridge record 包含 provider=`copilot`
- **AND** 系統可依該事件重新驗證並更新對應 session 清單

#### Scenario: Codex hook 發送 session 更新

- **WHEN** Codex hook 發出 session 更新或等效事件
- **THEN** SessionHub 接收的 bridge record 包含 provider=`codex`
- **AND** 系統可依該事件重新驗證並更新對應 session 清單

### Requirement: Hook 命令以 Node 主軌、sh fallback 產生

各 provider（claude / codex / copilot）的整合安裝流程 SHALL 產生以 `node <script.js>` 為主軌、sh 腳本為 fallback 的 hook 命令；SHALL NOT 再產生 PowerShell 命令或 `powershell` / `commandWindows` 欄位。Rust 端 `render_*_hook_command` 系列函式 SHALL 改為輸出 node 主軌命令，並移除 PowerShell 命令產生分支。

#### Scenario: Copilot 安裝產生 node 主軌命令

- **WHEN** 使用者安裝或更新 Copilot 整合
- **THEN** Copilot 設定檔中各 hook 事件的主命令為 `node <script.js> --bridge-path ... --provider copilot`
- **AND** 設定檔不含 `powershell` 欄位

#### Scenario: 設定檔保留 sh fallback 命令

- **WHEN** 任一 provider 安裝或更新整合
- **THEN** 設定檔中各 hook 事件保留指向 `.sh` 腳本的 fallback 命令欄位

#### Scenario: 移除 PowerShell 命令產生

- **WHEN** 檢視 `render_*_hook_command` 系列函式的輸出
- **THEN** 不再產生任何 PowerShell 指令字串，亦不寫出 `.ps1` 腳本至 provider 目錄

### Requirement: Node hook 主軌相依與退路

主軌 hook 命令 SHALL 依賴執行環境的 Node.js；當 Node.js 不可用時，整合機制 SHALL 透過設定檔的 sh fallback 欄位提供退路，避免 hook 完全失效。

#### Scenario: 無 node 時退回 sh fallback

- **WHEN** provider 執行 hook 主軌 `node <script.js>` 但環境無 node
- **THEN** provider 依設定檔的 fallback 欄位改以 sh 腳本執行 hook（依各 provider hook schema 的雙欄機制）

### Requirement: Provider integration 腳本分離管理

系統 SHALL 以 provider-specific 的腳本或模板資產管理 Copilot、OpenCode 與 Codex 的 integration 內容，不得將三個 provider 的 hook / plugin 內容混寫在同一份受管理產物中。

#### Scenario: 管理 Copilot integration 內容
- **WHEN** 系統安裝或更新 Copilot integration
- **THEN** 系統僅寫入與 Copilot 相關的 hook 內容
- **AND** 不影響 OpenCode 或 Codex 的受管理腳本資產

#### Scenario: 管理 Codex integration 內容
- **WHEN** 系統安裝或更新 Codex integration
- **THEN** 系統僅寫入與 Codex 相關的 hook 內容
- **AND** 不重用混雜其他 provider 事件映射的模板內容
