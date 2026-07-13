## ADDED Requirements

### Requirement: Session 卡片顯示 provider 標籤

系統 SHALL 在 session 卡片上顯示小型 provider 標籤，讓使用者一眼辨識 session 來源。

#### Scenario: OpenCode session 標籤

- **WHEN** 顯示 provider 為 `"opencode"` 的 session 卡片
- **THEN** 卡片顯示醒目的 `OpenCode` 標籤（使用特定 accent 顏色）

#### Scenario: Codex session 標籤

- **WHEN** 顯示 provider 為 `"codex"` 的 session 卡片
- **THEN** 卡片顯示醒目的 `Codex` 標籤（使用與 OpenCode 可區分的 provider accent 顏色）

#### Scenario: Copilot session 不顯示標籤

- **WHEN** 顯示 provider 為 `"copilot"` 的 session 卡片
- **THEN** 不顯示 provider 標籤（Copilot 為預設 provider）

### Requirement: Provider 標籤視覺規格

Provider 標籤 SHALL 以固定尺寸、圓角、對比色小 badge 呈現。

#### Scenario: 標籤樣式

- **WHEN** provider 標籤渲染
- **THEN** 標籤字型大小不超過 0.65rem，背景使用 provider 專屬顏色，文字 contrast ratio ≥ 4.5:1

### Requirement: Claude 與 Antigravity provider 標籤顏色

provider 標籤（`.provider-tag`）SHALL 涵蓋 `claude` 與 `antigravity`，各自具備專屬 accent 顏色的背景與文字色票，於 dark 與 light 主題皆有定義，且與既有 copilot/opencode/codex 標籤可明顯區分。

#### Scenario: Claude 標籤顯示品牌色

- **WHEN** 任一處（設定頁 provider integration 卡片、session 卡片、Dashboard）渲染 provider 為 `"claude"` 的標籤
- **THEN** 標籤顯示 Claude 品牌色系（以 `#D97757` 為基準）的背景與文字，非無底色純文字

#### Scenario: Antigravity 標籤顯示品牌色

- **WHEN** 任一處渲染 provider 為 `"antigravity"` 的標籤
- **THEN** 標籤顯示 Antigravity 品牌色系（以 `#4285F4` 為基準）的背景與文字，非無底色純文字

#### Scenario: Antigravity 標籤顯示正式名稱

- **WHEN** 設定頁整合卡片或 toast 訊息需要顯示 antigravity 的 provider 名稱
- **THEN** 顯示「Antigravity」（經 i18n label），而非小寫原始識別字串 `antigravity`

#### Scenario: 雙主題定義與對比

- **WHEN** 使用者切換 dark / light 主題
- **THEN** claude 與 antigravity 標籤在兩個主題下均有對應色票（`--color-provider-claude-bg/text`、`--color-provider-antigravity-bg/text`）
- **AND** 文字 contrast ratio ≥ 4.5:1
