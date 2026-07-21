## Requirements

### Requirement: 釘選專案持久化

系統 SHALL 允許使用者釘選常用專案，釘選狀態持久化儲存於 settings.json。同一 repo 的不同分支（worktree）各自擁有獨立的 projectKey（格式：`normalizePath(repoRoot):branch`），SHALL 可以同時被釘選，互不干擾。

#### Scenario: 釘選專案

- **WHEN** 使用者在 Sidebar 或專案 tab 點擊釘選按鈕
- **THEN** 該專案的 projectKey（`repoRoot:branch`）加入 `pinnedProjects` 陣列並儲存
- **AND** 該專案立即出現在 Sidebar 釘選區

#### Scenario: 取消釘選

- **WHEN** 使用者點擊已釘選專案的釘選按鈕
- **THEN** 從 `pinnedProjects` 移除，Sidebar 釘選區不再顯示該分支
- **AND** 同一 repo 其他已釘選的分支不受影響

#### Scenario: 同一 repo 多分支同時釘選

- **WHEN** 使用者依序釘選同一 repo 的分支 A 與分支 B
- **THEN** `pinnedProjects` 陣列中同時包含 `repoRoot:branchA` 與 `repoRoot:branchB`
- **AND** Sidebar 釘選區顯示兩個分支的獨立項目

#### Scenario: 釘選專案 tab 排序

- **WHEN** 同時有釘選與非釘選的專案 tab 開啟
- **THEN** 釘選專案 tab 排在 Dashboard 之後、一般專案之前
- **AND** 釘選專案 tab 顯示固定標記

#### Scenario: settings 載入時的 key 正規化

- **WHEN** 應用程式啟動並從 settings.json 載入 `pinnedProjects`
- **THEN** 系統 SHALL 對每個 key 的路徑部分（`:` 之前）套用 `normalizePath`
- **AND** branch 部分（`:` 之後）保持原樣，不做路徑正規化
- **AND** 不含 `:` 的舊格式 key（僅路徑）SHALL 整體做 `normalizePath`，保持向後相容

### Requirement: Sidebar 釘選區快速導覽

Sidebar SHALL 在主導覽區顯示釘選專案的快捷連結。

#### Scenario: Sidebar 釘選區

- **WHEN** sidebar 展開且有釘選專案
- **THEN** Sidebar 中段顯示釘選專案列表（以路徑最後一段為名稱）
- **AND** 點擊即切換至對應專案 tab（未開啟則先開啟）
