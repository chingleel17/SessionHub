## ADDED Requirements

### Requirement: Windows 工具程序樹抑制 MSYS stackdump

系統 SHALL 在 Windows 上由 SessionHub 啟動可能使用 Bash 的終端或 AI coding CLI 時，為子程序環境加入 `MSYS` 的空 `error_start:` 選項，使該程序及其後代程序不在目前工作目錄產生 `bash.exe.stackdump`。

#### Scenario: 從 SessionHub 啟動工具
- **WHEN** 使用者在 Windows 上由 SessionHub 開啟 terminal、Claude、Codex、Copilot、OpenCode、Gemini 或恢復既有 session
- **THEN** 啟動的根程序環境包含 `MSYS` 的 `error_start:` 選項
- **AND** 該環境由後續啟動的 Bash 子程序繼承

#### Scenario: 不相關的原生工具
- **WHEN** 使用者由 SessionHub 開啟 Explorer 或 VS Code
- **THEN** 系統不為該程序新增 MSYS stackdump 緩解選項

#### Scenario: 非 Windows 平台
- **WHEN** 相同啟動邏輯在非 Windows 平台編譯或執行
- **THEN** 系統不變更子程序的 `MSYS` 環境

### Requirement: 保留既有 MSYS 選項

系統 SHALL 在加入 stackdump 緩解選項時保留既有 `MSYS` 選項，且 SHALL 以不區分大小寫的方式辨識既有 `error_start` token，避免覆寫使用者指定的除錯器或重複加入選項。

#### Scenario: 尚未設定 MSYS
- **WHEN** 父程序環境沒有 `MSYS` 變數
- **THEN** 子程序收到 `MSYS=error_start:`

#### Scenario: 已有其他 MSYS 選項
- **WHEN** 父程序的 `MSYS` 為 `winsymlinks:nativestrict`
- **THEN** 子程序的 `MSYS` 同時保留 `winsymlinks:nativestrict` 並包含 `error_start:`

#### Scenario: 已有 error_start 選項
- **WHEN** 父程序的 `MSYS` 已包含 `error_start:` 或 `error_start:<debugger-path>`
- **THEN** 系統保留原值且不加入第二個 `error_start` token

### Requirement: 緩解措施不處理既有 dump

系統 SHALL 僅調整新啟動程序的環境，且 SHALL NOT 掃描或刪除磁碟上既有的 `bash.exe.stackdump`。

#### Scenario: 專案已有 stackdump
- **WHEN** 使用者啟動工具前，專案內已存在 `bash.exe.stackdump`
- **THEN** 系統不修改或刪除該檔案
