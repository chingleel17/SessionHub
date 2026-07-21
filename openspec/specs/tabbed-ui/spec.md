## ADDED Requirements

### Requirement: ProjectView 內子分頁機制

ProjectView 頁面 SHALL 包含子分頁機制，預設顯示 Sessions 子分頁。專案的開啟與關閉透過 Sidebar 操作，不再透過頂部 tab 列。

#### Scenario: 預設子分頁

- **WHEN** 使用者從 Sidebar 點擊已開啟或釘選的專案項目
- **THEN** ProjectView 內顯示 Sessions 子分頁（含現有 session 列表）
- **AND** 如果該專案有 openspec 目錄，額外顯示 Plans & Specs 子分頁

#### Scenario: Plan 子分頁動態新增

- **WHEN** 使用者從 session 卡片開啟 plan
- **THEN** ProjectView 子分頁列新增以 session ID 為 key 的 Plan 分頁
- **AND** Plan 子分頁顯示關閉按鈕（×）

#### Scenario: 跨專案狀態保留

- **WHEN** 使用者切換至其他專案再切回
- **THEN** 已開啟的 plan 子分頁 SHALL 仍然存在（不因切換而關閉）
