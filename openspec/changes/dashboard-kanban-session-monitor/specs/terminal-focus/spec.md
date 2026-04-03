## ADDED Requirements

### Requirement: 終端機視窗 Bring-to-Front

系統 SHALL 提供嘗試將終端機視窗帶到前景的功能，透過 Win32 API 尋找並聚焦匹配的視窗（best-effort）。

#### Scenario: 成功找到並聚焦終端視窗

- **WHEN** 使用者點擊 SessionCard 的「聚焦終端」按鈕
- **THEN** 系統透過 EnumWindows 遍歷所有頂層視窗
- **AND** 比對視窗 class（ConsoleWindowClass / CASCADIA_HOSTING_WINDOW_CLASS）及標題中的 cwd 路徑名
- **AND** 找到最佳匹配後呼叫 SetForegroundWindow 與 ShowWindow(SW_RESTORE)
- **THEN** 終端視窗被帶到前景

#### Scenario: 找不到終端視窗

- **WHEN** 系統遍歷所有視窗後未找到匹配的終端視窗
- **THEN** 系統回傳錯誤
- **AND** 前端顯示 toast：「找不到對應的終端視窗，請手動切換」

#### Scenario: SetForegroundWindow 被系統阻擋

- **WHEN** Windows 系統阻擋 SetForegroundWindow 呼叫（前景鎖定保護）
- **THEN** 系統嘗試 AttachThreadInput 後重試
- **AND** 若仍失敗則回傳錯誤，前端顯示提示 toast

### Requirement: 終端視窗比對邏輯

系統 SHALL 以多重條件比對視窗，找出最可能對應目標 session 的終端視窗。

#### Scenario: 以視窗 class 識別終端類型

- **WHEN** 遍歷視窗時
- **THEN** 系統辨識 class 名稱為 `CASCADIA_HOSTING_WINDOW_CLASS` 的 Windows Terminal 視窗
- **AND** 辨識 class 名稱為 `ConsoleWindowClass` 的傳統 cmd / PowerShell 視窗

#### Scenario: 以路徑名稱比對視窗標題

- **WHEN** 比對終端視窗時
- **THEN** 系統取 cwd 的最後一段路徑名稱（如 `my-project`）
- **AND** 檢查視窗標題是否包含該名稱（不區分大小寫）
- **AND** 若多個視窗匹配，選擇標題包含最長路徑片段的視窗
