## Purpose

定義 SessionHub 在 shell 與 herdr 終端啟動器下的終端聚焦、視窗比對與 tab 識別碼定位行為。

## Requirements

### Requirement: 終端機視窗 Bring-to-Front

在 shell 啟動器模式下，系統 SHALL 提供嘗試將終端機視窗帶到前景的功能，透過 Win32 API 尋找並聚焦匹配的視窗（best-effort）。

#### Scenario: 成功找到並聚焦終端視窗

- **WHEN** 啟動器為 shell 且使用者點擊 SessionCard 的「聚焦終端」按鈕
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

在 shell 啟動器模式下，系統 SHALL 以多重條件比對視窗，找出最可能對應目標 session 的終端視窗。此比對邏輯不適用於 herdr 啟動器，因 herdr 以單一視窗承載多個 pane，視窗標題無法區分個別 session。

#### Scenario: 以視窗 class 識別終端類型

- **WHEN** 啟動器為 shell 且遍歷視窗時
- **THEN** 系統辨識 class 名稱為 `CASCADIA_HOSTING_WINDOW_CLASS` 的 Windows Terminal 視窗
- **AND** 辨識 class 名稱為 `ConsoleWindowClass` 的傳統 cmd / PowerShell 視窗

#### Scenario: 以路徑名稱比對視窗標題

- **WHEN** 啟動器為 shell 且比對終端視窗時
- **THEN** 系統取 cwd 的最後一段路徑名稱（如 `my-project`）
- **AND** 檢查視窗標題是否包含該名稱（不區分大小寫）
- **AND** 若多個視窗匹配，選擇標題包含最長路徑片段的視窗

### Requirement: 聚焦行為依啟動器分流

系統 SHALL 依當前終端啟動器種類決定聚焦方式，避免在多工器環境中套用不適用的視窗比對邏輯。

#### Scenario: herdr 模式不使用視窗標題比對

- **WHEN** 啟動器為 herdr 且使用者觸發「聚焦終端」
- **THEN** 系統不執行依視窗 class 與標題比對的 Win32 聚焦流程
- **AND** 改以 herdr 的 tab 聚焦機制定位目標

#### Scenario: 建立 tab 時即聚焦

- **WHEN** 啟動器為 herdr 且系統為某次啟動建立新的 tab
- **THEN** 系統在建立該 tab 時即要求聚焦

### Requirement: herdr 模式以 tab 識別碼聚焦

在 herdr 啟動器模式下，系統 SHALL 記錄啟動時取得的 tab 識別碼，並於使用者觸發聚焦時以該識別碼聚焦對應 tab。

#### Scenario: 成功聚焦既有 tab

- **WHEN** 使用者對一個由 SessionHub 以 herdr 啟動、且已記錄 tab 識別碼的 session 觸發「聚焦終端」
- **THEN** 系統以該 tab 識別碼要求 herdr 聚焦該 tab
- **AND** 該 tab 成為當前聚焦的 tab

#### Scenario: 無對應 tab 識別碼

- **WHEN** 使用者觸發聚焦，但該 session 沒有已記錄的 tab 識別碼（例如非由本次應用程式執行期啟動）
- **THEN** 系統回傳可辨識的錯誤訊息
- **AND** 前端以 toast 呈現，提示使用者手動切換

#### Scenario: tab 已被關閉

- **WHEN** 系統以已記錄的 tab 識別碼要求聚焦，但該 tab 已不存在
- **THEN** 系統回傳指出該 tab 已不存在的錯誤訊息，不聚焦到其他 tab
