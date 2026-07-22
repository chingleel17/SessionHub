## MODIFIED Requirements

### Requirement: Hook 離線系統通知

系統 SHALL 在 provider hook 腳本中提供獨立於 SessionHub 行程的系統通知能力，使用隨附的 `snoretoast.exe` 發送 Windows Toast；當 SessionHub 未開啟時通知仍正常發送。所有 hook 通知 SHALL 使用 `com.ching.sessionhub` 作為 Windows Application User Model ID，使 Windows 將其解析為 SessionHub 應用程式與圖示，而非執行 hook 的終端機。

#### Scenario: SessionHub 未運行時 hook 仍發通知

- **WHEN** provider hook 腳本在對應事件點執行
- **AND** SessionHub 行程未開啟
- **THEN** hook 透過隨附的 `snoretoast.exe` 發送 Windows Toast 通知
- **AND** 通知發送不因 SessionHub 未運行而失敗
- **AND** 通知以 `com.ching.sessionhub` 歸類為 SessionHub

#### Scenario: notify 模組失敗不阻斷 hook

- **WHEN** `notify.cjs` 呼叫 `snoretoast.exe` 發生錯誤
- **THEN** hook 不拋出例外、不阻斷其餘 hook 流程
- **AND** 錯誤寫入 `%APPDATA%\SessionHub\logs\hook-errors.log`
