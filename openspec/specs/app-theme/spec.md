## ADDED Requirements

### Requirement: 應用程式 Icon 與品牌視覺

系統 SHALL 使用統一的品牌 icon 與視覺語言貫穿整個應用程式介面。

#### Scenario: 應用程式視窗 icon

- **WHEN** 應用程式視窗開啟
- **THEN** 視窗標題列與工作列顯示 SessionHub 品牌 icon

#### Scenario: Sidebar logo 顯示

- **WHEN** sidebar 處於展開狀態
- **THEN** 頂部顯示 SessionHub logo（icon + 文字）
- **AND** 收合時僅顯示 icon（正方形，不截切）

### Requirement: 主題色彩系統

系統 SHALL 以 CSS 自訂屬性（變數）定義品牌主題色彩，支援明暗主題切換。

#### Scenario: 主題色彩變數

- **WHEN** 應用程式渲染
- **THEN** 根元素定義以下 CSS 變數：
  - `--color-accent` — 主要強調色
  - `--color-bg` — 頁面背景
  - `--color-surface` — 卡片 / 面板背景
  - `--color-text` — 主要文字
  - `--color-text-muted` — 次要文字
  - `--color-border` — 邊框顏色
