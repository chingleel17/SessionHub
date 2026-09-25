## MODIFIED Requirements

### Requirement: ProjectView 子分頁架構

ProjectView SHALL 以子分頁組織專案內視圖；Sidebar 提供 Dashboard、獨立消耗分析與既有其他全域及專案入口，消耗分析不取代專案 Analytics 子頁籤。

#### Scenario: 子分頁清單（預設）
- **WHEN** 使用者進入專案
- **THEN** 保留 Sessions、條件顯示的 Plans & Specs、Agents 及 Analytics 的既有相對順序，Analytics 在靜態子頁籤最後

#### Scenario: Plan sub-tab 動態新增
- **WHEN** 使用者從 session 開啟 plan
- **THEN** 新增 Plan 子頁籤並以 session_id 識別，沿用現有命名與行為

#### Scenario: Plan sub-tab 關閉
- **WHEN** 使用者關閉 Plan 子頁籤
- **THEN** 移除該頁籤並返回 Sessions
