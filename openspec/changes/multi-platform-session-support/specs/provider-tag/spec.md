## ADDED Requirements

### Requirement: Session card 平台標籤
系統 SHALL 在每個 session card 上顯示來源平台的標籤（tag），讓使用者一眼辨識此 session 來自哪個工具。

#### Scenario: Copilot session 顯示標籤
- **WHEN** 顯示一個 provider 為 `"copilot"` 的 session card
- **THEN** card 上顯示 "Copilot" 標籤，使用 Copilot 品牌色系

#### Scenario: OpenCode session 顯示標籤
- **WHEN** 顯示一個 provider 為 `"opencode"` 的 session card
- **THEN** card 上顯示 "OpenCode" 標籤，使用 OpenCode 品牌色系

### Requirement: 標籤樣式一致性
系統 SHALL 以統一的 tag 元件樣式呈現平台標籤，與既有的使用者自訂標籤（tags）視覺上有所區別。

#### Scenario: 平台標籤與使用者標籤並存
- **WHEN** 一個 session 同時有 provider tag 與使用者自訂 tag
- **THEN** 平台標籤顯示在使用者標籤之前，兩者樣式有明確區隔（如平台標籤為實心填色，使用者標籤為外框線型）
