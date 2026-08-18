## ADDED Requirements

### Requirement: Commands provider 集合與 roots 依啟用平台決定

Commands 掃描 SHALL 只處理 `AppSettings.enabledProviders` 中的平台，並為各 provider 掃描其 project/global command roots：Claude `.claude/commands`；Codex `.codex/prompts`；OpenCode `.opencode/command` 與 `.opencode/commands`；Copilot `.github/prompts` 與既有 `.copilot/prompts` fallback；Antigravity/Gemini `.gemini/commands`，以及各自 user root 下的對應目錄。Claude、Codex、OpenCode 使用 `.md`，Copilot 使用 `.prompt.md`，Gemini 使用 `.toml`；停用 provider SHALL 不掃描、不建立欄位。不同 provider 的同名 command SHALL 分別保留；同一 provider 多 root 同名 command SHALL 合併 locations，不得在沒有 CLI 證據時猜測 effective path。

#### Scenario: 啟用 Antigravity Commands
- **WHEN** `enabledProviders` 包含 `antigravity` 且專案存在 `.gemini/commands/review.toml`
- **THEN** Commands 掃描包含 Antigravity 的 `review` command 與其來源路徑

#### Scenario: 停用 Claude 不掃描 commands
- **WHEN** `enabledProviders` 不包含 `claude`
- **THEN** 系統不存取 project/global `.claude/commands`，介面不顯示 Claude 欄位

#### Scenario: OpenCode 同時掃 singular 與 plural roots
- **WHEN** `.opencode/command/a.md` 與 `.opencode/commands/b.md` 同時存在
- **THEN** OpenCode Commands root discovery 包含 `a` 與 `b`

### Requirement: OpenCode Commands effective 狀態與同步狀態分離

Commands 掃描 SHALL 使用 OpenCode resolved config JSON 取得指定 scope 的 effective commands：project scope 在專案根目錄查詢，global scope 在不含專案設定的中立工作目錄查詢。OpenCode effective 狀態 SHALL 與 provider root discovery、檔案同步狀態分離；Claude、Codex、Copilot、Antigravity Commands SHALL 不執行 CLI 狀態檢查，但仍依各自 roots 掃描。任何 provider 的檔案存在、複製、連結及雜湊結果 MUST NOT 用來產生「已載入／未安裝」等執行狀態。

#### Scenario: 有原生可解析查詢
- **WHEN** OpenCode resolved config 回傳某個自訂 command
- **THEN** 系統將該 command 的 OpenCode effective 狀態標示為 `available` 並記錄 `cli` 資料來源

#### Scenario: 同步一致但 CLI 未載入
- **WHEN** command 來源與 OpenCode 目標檔案內容一致，但成功的 OpenCode 查詢未回傳該 command
- **THEN** 同步狀態仍為一致
- **AND** 實際可用狀態不得顯示為 `available`

#### Scenario: 同名資源受 scope 覆寫
- **WHEN** project scope 的 command 覆寫同名 global command
- **THEN** project 查詢呈現 provider 實際採用的有效項目
- **AND** 結果保留可取得的來源 scope 或標示為 `effective`，不得猜測錯誤來源

### Requirement: 未支援的 Commands provider 跳過 CLI 檢查

Claude、Codex、Copilot、Antigravity 沒有已驗證的穩定 command 列舉能力，系統 SHALL 不為其啟動 CLI、不建立 CLI discovery 狀態，也不顯示 unknown/error/未驗證 badge；provider root discovery 仍照常執行。是否新增 CLI 支援 SHALL 由後續版本在完成實測與 parser 測試後明確加入固定能力矩陣，不得在執行時解析 help 或文字輸出猜測能力。

#### Scenario: Provider 沒有 command list 指令
- **WHEN** 使用者開啟 Commands 頁籤
- **THEN** 掃描到的 command 仍可預覽與同步
- **AND** 系統不為 Claude、Codex、Copilot 啟動 command 列舉指令或顯示執行狀態

#### Scenario: OpenCode 查詢失敗
- **WHEN** OpenCode resolved config 查詢失敗、逾時或回傳無效 JSON
- **THEN** Commands 清單仍顯示既有檔案與同步資料
- **AND** 群組層顯示非阻擋提示，各 command 不建立推測狀態
