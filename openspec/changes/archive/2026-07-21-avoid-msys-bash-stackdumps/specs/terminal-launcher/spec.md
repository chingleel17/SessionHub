## ADDED Requirements

### Requirement: Windows 終端與 CLI 啟動環境一致性

系統 SHALL 讓一般終端啟動、多工具啟動及 session resume 共用相同的 Windows MSYS stackdump 緩解環境組態，避免任一啟動入口遺漏。

#### Scenario: 開啟一般終端
- **WHEN** `open_terminal` 在 Windows 啟動使用者設定的終端
- **THEN** 新程序套用 MSYS stackdump 緩解環境

#### Scenario: 開啟或恢復 AI coding CLI
- **WHEN** `open_in_tool` 或 `resume_session_in_terminal` 在 Windows 啟動受支援的 AI coding CLI
- **THEN** 新程序套用與一般終端相同的 MSYS stackdump 緩解環境

#### Scenario: 啟動參數維持不變
- **WHEN** 系統套用 MSYS stackdump 緩解環境
- **THEN** 各 terminal 與 provider 原有的命令、參數、工作目錄及 Windows console creation flags 維持不變
