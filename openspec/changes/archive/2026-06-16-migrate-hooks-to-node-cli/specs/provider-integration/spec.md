## ADDED Requirements

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
