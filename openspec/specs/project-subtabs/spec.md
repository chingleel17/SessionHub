## ADDED Requirements

### Requirement: ProjectView 子分頁架構

ProjectView SHALL 以子分頁（sub-tab）機制組織專案內的不同視圖，頂層分頁列只包含 Dashboard 與各專案。

#### Scenario: 子分頁清單（預設）

- **WHEN** 使用者進入一個專案 tab
- **THEN** ProjectView 子分頁包含：
  1. Sessions（永遠存在，顯示 session 列表）
  2. Plans & Specs（當專案目錄下有 openspec/ 資料夾時顯示）

#### Scenario: Plan sub-tab 動態新增

- **WHEN** 使用者從 session 卡片點擊開啟 plan
- **THEN** 在 ProjectView 子分頁列新增 `Plan: <session summary>` 子分頁
- **AND** sub-tab 以 session_id 為唯一 key

#### Scenario: Plan sub-tab 關閉

- **WHEN** 使用者點擊 plan sub-tab 上的 × 按鈕
- **THEN** 該 sub-tab 從列表中移除，視圖返回 Sessions

### Requirement: 跨專案切換 plan sub-tab 保留

已開啟的 plan sub-tab SHALL 在使用者切換專案後仍保留，當切回時仍可存取。

#### Scenario: 切換專案後切回

- **WHEN** 使用者切換至其他專案後再切回到有已開啟 plan 的專案
- **THEN** plan sub-tab 仍存在，且顯示相同內容
