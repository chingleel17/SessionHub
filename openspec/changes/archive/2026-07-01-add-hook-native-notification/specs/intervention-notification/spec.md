## MODIFIED Requirements

### Requirement: Hook 腳本通知（獨立路徑）

系統 SHALL 透過隨附的 `snoretoast.exe` 與共用 `notify.cjs` 模組，在 provider hook 腳本中提供獨立於 SessionHub 運行狀態的系統通知；通知由 hook 事件直接觸發，不依賴 SessionHub 應用內的 activity 狀態判定。

#### Scenario: Hook 腳本發送 Toast

- **WHEN** hook 腳本在「完成」「等待回應/需決策」或「需授權」事件點執行通知邏輯
- **THEN** 腳本透過 `notify.cjs` 呼叫 `snoretoast.exe` 發送 Windows Toast
- **AND** Toast 的 `tag` 設為 `sessionhub-{session_id}`、`group` 設為 `intervention`，避免同一 session 多次通知疊加

#### Scenario: Hook 腳本通知不依賴 SessionHub 運行狀態

- **WHEN** hook 腳本執行時 SessionHub 行程未開啟
- **THEN** 通知仍正常發送，不因 SessionHub 未運行而失敗

#### Scenario: 三個 provider 皆能觸發

- **WHEN** Copilot、Codex 或 Claude 任一 provider 的 hook 在對應事件點執行
- **THEN** 系統依該 provider 的觸發點對映發送通知，不限於 Copilot
