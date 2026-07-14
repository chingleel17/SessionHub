## ADDED Requirements

### Requirement: QuotaOverview 提供「全部」tab 一次顯示所有 provider

QuotaOverview SHALL 在既有 provider tabs 之前提供一個「全部」tab（文案經 i18n）：選取時垂直依序列出所有可見 provider 的完整面板（與單一 provider 面板內容相同）；tab 選取以 localStorage 記憶（沿用既有機制，sentinel 值 `"all"`），localStorage key SHALL 可由 prop 覆寫以供不同宿主（Dashboard、狀態列彈出面板）各自記憶。

#### Scenario: 選取「全部」tab

- **WHEN** 使用者點擊「全部」tab
- **THEN** 內容區垂直列出所有可見 provider 的面板（含各自的視窗用量、重置倒數、錯誤狀態等）
- **AND** 選取值 `"all"` 寫入 localStorage，重新開啟後維持「全部」模式

#### Scenario: 全部模式下的刷新行為

- **WHEN** 「全部」tab 為選取狀態且使用者點擊刷新按鈕
- **THEN** 系統對所有 enabled provider 執行 quota refresh（等同全域刷新）

#### Scenario: 只有單一 provider 可見

- **WHEN** 可見的 quota snapshot 只有一個 provider
- **THEN** 維持既有行為不顯示 tabs 列（含「全部」tab），直接顯示該 provider 面板

#### Scenario: 記憶值失效時回退

- **WHEN** localStorage 記憶的 provider 已不在可見清單中（`"all"` 除外）
- **THEN** 回退至第一個可見 provider 的 tab
