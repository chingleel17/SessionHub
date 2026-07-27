# terminal-launcher

## MODIFIED Requirements

### Requirement: Windows 終端與 CLI 啟動環境一致性

系統 SHALL 讓一般終端啟動、多工具啟動及 session resume 共用相同的 Windows 子程序環境組態，避免任一啟動入口遺漏。

此組態涵蓋兩個面向：

1. **MSYS stackdump 緩解** — 以合併語意設定 MSYS 選項，保留使用者既有值
2. **環境區塊完整性** — 完整傳遞 SessionHub 程序的環境區塊，除本需求明確允許的注入項目外不移除、清空或覆寫任何變數，使子程序中的工具（含 AI coding CLI）之路徑解析與資源探索行為與使用者手動開啟的終端一致

詳細的一致性契約與界線定義見 `terminal-child-env-parity`。

#### Scenario: 開啟一般終端
- **WHEN** `open_terminal` 在 Windows 啟動使用者設定的終端
- **THEN** 新程序套用 MSYS stackdump 緩解環境
- **AND** 新程序具備完整的環境區塊

#### Scenario: 開啟或恢復 AI coding CLI
- **WHEN** `open_in_tool` 或 `resume_session_in_terminal` 在 Windows 啟動受支援的 AI coding CLI
- **THEN** 新程序套用與一般終端相同的 MSYS stackdump 緩解環境
- **AND** 新程序具備與一般終端相同的環境區塊內容

#### Scenario: 啟動參數維持不變
- **WHEN** 系統套用 MSYS stackdump 緩解環境
- **THEN** 各 terminal 與 provider 原有的命令、參數、工作目錄及 Windows console creation flags 維持不變

#### Scenario: 不因環境一致性而變更使用者設定
- **WHEN** 系統為維持環境一致性而處理子程序環境
- **THEN** 使用者明確設定的 AI coding CLI 停用旗標維持原值，系統不清除亦不覆寫
