## ADDED Requirements

### Requirement: Session 卡片顯示 provider 標籤

系統 SHALL 在 session 卡片上顯示小型 provider 標籤，讓使用者一眼辨識 session 來源。

#### Scenario: OpenCode session 標籤

- **WHEN** 顯示 provider 為 `"opencode"` 的 session 卡片
- **THEN** 卡片顯示醒目的 `OpenCode` 標籤（使用特定 accent 顏色）

#### Scenario: Copilot session 不顯示標籤

- **WHEN** 顯示 provider 為 `"copilot"` 的 session 卡片
- **THEN** 不顯示 provider 標籤（Copilot 為預設 provider）

### Requirement: Provider 標籤視覺規格

Provider 標籤 SHALL 以固定尺寸、圓角、對比色小 badge 呈現。

#### Scenario: 標籤樣式

- **WHEN** provider 標籤渲染
- **THEN** 標籤字型大小不超過 0.65rem，背景使用 provider 專屬顏色，文字 contrast ratio ≥ 4.5:1
