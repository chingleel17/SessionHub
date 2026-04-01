## ADDED Requirements

### Requirement: Session action icon buttons
系統 SHALL 將 session 卡片上所有動作按鈕（開啟終端、複製指令、封存、取消封存、刪除）改為 SVG icon button，並在 hover 時顯示 tooltip 文字說明。

#### Scenario: Icon button 顯示
- **WHEN** 使用者查看 session 卡片
- **THEN** 動作區域顯示對應 SVG icon（TerminalIcon、CopyIcon、ArchiveIcon / UnarchiveIcon、DeleteIcon），不顯示文字標籤

#### Scenario: Hover tooltip
- **WHEN** 使用者將滑鼠移至 icon button 上
- **THEN** 顯示對應操作的文字說明 tooltip（「開啟終端」、「複製指令」、「封存」、「取消封存」、「刪除」）

#### Scenario: Icon button 可存取性
- **WHEN** icon button 渲染於 DOM
- **THEN** 每個 button 具有 `aria-label` 屬性，內容與 tooltip 相同

### Requirement: Project header icon buttons
系統 SHALL 將 project group header 上的動作按鈕（開啟終端、釘選）改為 SVG icon button，並在 hover 時顯示 tooltip。

#### Scenario: Project header icon 顯示
- **WHEN** 使用者查看 project group header
- **THEN** 顯示 PinIcon（已釘選則顯示 UnpinIcon）及其他可用動作 icon

#### Scenario: Project icon button tooltip
- **WHEN** 使用者將滑鼠移至 project header 的 icon button
- **THEN** 顯示對應操作說明 tooltip
