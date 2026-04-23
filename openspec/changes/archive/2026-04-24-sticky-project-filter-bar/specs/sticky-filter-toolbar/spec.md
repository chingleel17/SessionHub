## ADDED Requirements

### Requirement: 篩選工具列固定於頂部

在 sessions sub-tab 中，包含 toolbar-card 與 tag-filter-bar 的篩選工具列區域 SHALL 固定在捲動容器頂部，不隨 session 卡片捲動。

#### Scenario: 向下捲動 session 列表

- **WHEN** 使用者在 sessions sub-tab 向下捲動 session 列表
- **THEN** 篩選工具列（搜尋框、排序、checkbox、tag chips）SHALL 保持可見固定於頁面上方

#### Scenario: 向上捲回頂部

- **WHEN** 使用者向上捲回列表頂部
- **THEN** 篩選工具列外觀與捲動前一致，不出現重疊或位置錯誤

#### Scenario: 在其他 sub-tab 不影響

- **WHEN** 使用者切換到 Plans & Specs 或 Plan Editor sub-tab
- **THEN** 該 sub-tab 的內容 SHALL 正常捲動，無多餘的 sticky 佔位

### Requirement: 篩選工具列背景遮蓋

sticky 篩選工具列 SHALL 具備不透明背景，確保 session 卡片捲動到工具列後方時不透出。

#### Scenario: session 卡片捲過篩選列下方

- **WHEN** session 卡片向上捲動至篩選工具列下方
- **THEN** 卡片文字 SHALL 被工具列背景完全遮蓋，不透出顯示

### Requirement: 篩選工具列 z-index 層級正確

sticky 篩選工具列的 z-index SHALL 低於 launcher menu overlay，確保 overlay 正常顯示於工具列上方。

#### Scenario: 開啟 launcher menu overlay

- **WHEN** 使用者在 session 卡片上開啟 launcher menu
- **THEN** launcher menu overlay SHALL 顯示於篩選工具列之上，不被工具列遮蓋
