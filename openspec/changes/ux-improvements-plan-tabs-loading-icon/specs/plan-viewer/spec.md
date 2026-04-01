## MODIFIED Requirements

### Requirement: Plan 編輯器雙欄等高佈局
Plan 編輯器 SHALL 以 CSS Grid 雙欄佈局顯示原始 Markdown 欄與預覽欄，兩欄高度 SHALL 相同（`align-items: stretch`），原始 Markdown 的 textarea 高度 SHALL 撐滿欄位高度。

#### Scenario: 開啟 plan 編輯器
- **WHEN** 使用者開啟 plan sub-tab
- **THEN** 原始欄與預覽欄高度相同，textarea 撐滿左欄，預覽欄可垂直捲動

#### Scenario: 內容較長時
- **WHEN** plan 內容較多，預覽欄需要捲動
- **THEN** 預覽欄出現垂直捲軸，左欄 textarea 高度不受影響，兩欄高度仍一致

### Requirement: App Icon 更新
系統 SHALL 使用 SessionHub 品牌 icon（深藍色 `#1a2744` 背景、白色「S」字母，圓角矩形）替換舊版 icon，覆蓋 `src-tauri/icons/` 下所有格式與尺寸。

#### Scenario: 應用程式圖示顯示
- **WHEN** 使用者在工作列或開始選單看到 SessionHub
- **THEN** 顯示 SessionHub 品牌 icon，非舊版 CS icon
