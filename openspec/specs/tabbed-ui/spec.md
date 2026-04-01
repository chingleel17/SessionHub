## ADDED Requirements

### Requirement: 固定 Dashboard 分頁

系統 SHALL 在分頁列最左側顯示固定的 Dashboard 分頁，不可關閉。

#### Scenario: 應用程式啟動

- **WHEN** 應用程式啟動
- **THEN** Dashboard 分頁自動開啟且為當前作用分頁

### Requirement: Tab header 高度一致

系統 SHALL 確保 Dashboard 分頁與專案分頁的 workspace-header 高度保持一致，切換時不產生高度跳動。

#### Scenario: 從 Dashboard 切換至專案分頁

- **WHEN** 使用者點擊任一專案 tab
- **THEN** workspace-header 高度不改變
- **AND** 專案路徑以縮小字體（不超過 0.75rem）單行截斷顯示於 title 下方

### Requirement: ProjectView 內子分頁機制

ProjectView 頁面 SHALL 包含子分頁機制，預設顯示 Sessions 子分頁。

#### Scenario: 預設子分頁

- **WHEN** 使用者點擊專案 tab
- **THEN** ProjectView 內顯示 Sessions 子分頁（含現有 session 列表）
- **AND** 如果該專案有 openspec 目錄，額外顯示 Plans & Specs 子分頁

#### Scenario: Plan 子分頁動態新增

- **WHEN** 使用者從 session 卡片開啟 plan
- **THEN** ProjectView 子分頁列新增以 session ID 為 key 的 Plan 分頁
- **AND** Plan 子分頁顯示關閉按鈕（×）

#### Scenario: 跨專案狀態保留

- **WHEN** 使用者切換至其他專案再切回
- **THEN** 已開啟的 plan 子分頁 SHALL 仍然存在（不因切換而關閉）

### Requirement: 釘選專案 tab 排序優先

釘選專案（pinned projects）的 tab SHALL 排列在 Dashboard 之後、其他專案之前。

#### Scenario: Sidebar 快速導覽釘選進入專案

- **WHEN** 使用者點擊 sidebar 中的釘選專案快捷連結
- **THEN** 系統切換至該專案 tab，若未開啟則先開啟
