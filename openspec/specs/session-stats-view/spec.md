## ADDED Requirements

### Requirement: Session 卡片統計 badge 行

每個 SessionCard SHALL 在卡片底部顯示緊湊的統計 badge 行。

#### Scenario: Stats 已載入

- **WHEN** SessionCard 渲染且 stats 已取得
- **THEN** 卡片底部顯示：互動次數（N turns）、output tokens（48K tokens）、時長（35m）

#### Scenario: Stats 載入中

- **WHEN** stats 尚未取得
- **THEN** badge 區域顯示骨架佔位，不產生版面跳動

#### Scenario: 無統計資料

- **WHEN** session 無 events.jsonl（所有計數為 0）
- **THEN** badge 行隱藏

### Requirement: Session stats 詳情 panel

系統 SHALL 在每個 session 提供可展開的統計詳情 panel。

#### Scenario: 使用者開啟 panel

- **WHEN** 使用者點擊 session 卡片的統計詳情 icon 按鈕
- **THEN** panel 於卡片下方展開，顯示完整統計（tool breakdown、models、reasoning 等）

#### Scenario: Tool breakdown 表格

- **WHEN** panel 開啟且 toolBreakdown 非空
- **THEN** 顯示工具名稱與呼叫次數列表，依次數降冪排列

#### Scenario: Live session 說明

- **WHEN** stats.isLive 為 true
- **THEN** panel 顯示「Session 進行中」提示，統計標示為即時快照
