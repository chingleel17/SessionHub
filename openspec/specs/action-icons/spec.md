## ADDED Requirements

### Requirement: Session 操作 icon 按鈕

Session 卡片 SHALL 以 SVG icon 按鈕替代文字按鈕，提供常用操作的快速入口。

#### Scenario: 操作按鈕顯示時機

- **WHEN** 使用者 hover session 卡片
- **THEN** 卡片右上角顯示操作 icon 按鈕組（封存、終端、複製指令、編輯備註）

#### Scenario: Icon 按鈕 tooltip

- **WHEN** 使用者 hover 任一 icon 按鈕
- **THEN** 顯示對應操作說明的 tooltip 文字（使用 i18n key）

#### Scenario: 按鈕可見度

- **WHEN** 使用者離開卡片
- **THEN** icon 按鈕淡出隱藏（或保持顯示於已展開的 panel）

### Requirement: Icon 按鈕視覺規格

所有 icon 按鈕 SHALL 使用統一尺寸與 hover 效果。

#### Scenario: Icon 按鈕規格

- **WHEN** icon 按鈕渲染
- **THEN** 按鈕尺寸為 24×24px，icon 為 16×16px SVG
- **AND** hover 時背景色加深（使用 `--color-surface-hover` 或同等變數）
- **AND** 圓角半徑 4px
