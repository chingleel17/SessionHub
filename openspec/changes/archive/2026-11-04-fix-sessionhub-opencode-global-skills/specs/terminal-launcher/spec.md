# terminal-launcher

## MODIFIED Requirements

### Requirement: Windows 終端與 CLI 啟動環境一致性

系統 SHALL 讓一般終端啟動、多工具啟動及 session resume 共用相同的 Windows MSYS stackdump 緩解環境組態，避免任一啟動入口遺漏。此要求適用於所有由系統建立的 console 程序，包含 shell 啟動器所建立的終端與 herdr client console。shell 子程序 SHALL 繼承 SessionHub 程序環境，除既有 MSYS 合併語意及明確記錄的服務啟動例外外，不得為修正 skills 問題清除或覆寫使用者環境；herdr 既有 server／pane 的環境依其自身生命週期判定，不得誤稱由本次請求重新建立。

#### Scenario: 開啟一般終端
- **WHEN** `open_terminal` 在 Windows 啟動使用者設定的終端
- **THEN** 新程序套用 MSYS stackdump 緩解環境
- **AND** shell 新程序繼承 SessionHub 當下的其他環境變數

#### Scenario: 開啟或恢復 AI coding CLI
- **WHEN** `open_in_tool` 或 `resume_session_in_terminal` 在 Windows 啟動受支援的 AI coding CLI
- **THEN** 新建立的 console 程序套用與一般終端相同的 MSYS stackdump 緩解環境
- **AND** shell 模式的新程序遵循與一般終端相同的環境繼承規則

#### Scenario: 啟動參數維持不變
- **WHEN** 系統套用 MSYS stackdump 緩解環境
- **THEN** 各 terminal 與 provider 原有的命令、參數、工作目錄及 Windows console creation flags 維持不變

#### Scenario: herdr 模式的環境套用範圍

- **WHEN** 啟動器為 herdr 且系統建立 client console 程序
- **THEN** 該程序以 console creation flags 開啟獨立視窗
- **AND** 套用與 shell 啟動器相同的 MSYS stackdump 緩解環境

#### Scenario: herdr tab 建立不另開 console

- **WHEN** 啟動器為 herdr 且系統對既有 client console 建立新的 tab
- **THEN** 該次 tab 建立不產生新的 Windows console 程序，因此不套用 console creation flags

#### Scenario: herdr socket API 呼叫不顯示視窗

- **WHEN** 系統執行 herdr socket API 類指令以查詢狀態或操作 tab／pane
- **THEN** 該程序以隱藏視窗方式執行，不閃現 console 視窗

#### Scenario: 保留使用者工具設定

- **WHEN** shell 啟動器建立終端以執行 OpenCode
- **THEN** 系統不為修正技能列示而清除或覆寫使用者既有的 OpenCode 停用旗標
