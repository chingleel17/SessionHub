## ADDED Requirements

### Requirement: Sidebar 項目顯示 repo 名稱與分支

Sidebar 中的 ProjectGroup 項目 SHALL 顯示 `repoName · branch` 格式，當 `branchLabel` 存在時顯示分支副標籤，確保同名 repo 的不同分支可視覺區分。

#### Scenario: 有分支資訊時顯示分支副標

- **WHEN** ProjectGroup 的 `branchLabel` 為 `"feature/foo"`
- **THEN** sidebar 項目 SHALL 顯示 `repoName · feature/foo` 或類似格式，分支以視覺上較次要的樣式（較小字體或次要色）呈現

#### Scenario: 無分支資訊時只顯示 repo 名稱

- **WHEN** ProjectGroup 的 `branchLabel` 為 `null`
- **THEN** sidebar 項目 SHALL 只顯示 `repoName`，不顯示分隔符號或空白分支

#### Scenario: 折疊 Sidebar 時仍可辨識

- **WHEN** Sidebar 處於折疊狀態（`isSidebarCollapsed = true`）
- **THEN** 圖示按鈕的 `title` tooltip SHALL 包含完整的 `repoName · branch` 資訊

### Requirement: ProjectView 標題區顯示分支副標題

ProjectView 的專案標題區 SHALL 在 repo 名稱旁顯示分支標籤（badge 或副標）。

#### Scenario: 標題區顯示 repo 名稱加分支

- **WHEN** 進入一個 `branchLabel` 為 `"main"` 的 ProjectGroup
- **THEN** ProjectView 標題 SHALL 顯示 repo 名稱為主標，分支名稱以 badge 或副標題形式顯示在旁邊

#### Scenario: 無分支時不顯示分支 badge

- **WHEN** ProjectGroup 的 `branchLabel` 為 `null`
- **THEN** ProjectView 標題區 SHALL 只顯示 repo 名稱，不顯示分支 badge

### Requirement: SessionCard 不顯示分支欄位

SessionCard SHALL 不顯示 `gitBranch` 欄位，因為分支資訊已在 ProjectGroup 標題層呈現。

#### Scenario: 移除 session card 上的分支列

- **WHEN** 渲染一個有 `gitBranch` 值的 SessionCard
- **THEN** card 內容 SHALL 不顯示分支欄位或分支相關標籤
