## MODIFIED Requirements

### Requirement: 釘選專案持久化

系統 SHALL 允許使用者釘選常用專案，釘選狀態持久化儲存於 settings.json。專案身分以工作目錄為準，projectKey 格式為 `normalizePath(repoRoot ?? cwd)`，不包含分支名稱。同一目錄切換 git 分支 SHALL 視為同一專案；位於不同目錄的 git worktree 因 `repoRoot` 不同，SHALL 各自為獨立專案。

#### Scenario: 釘選專案

- **WHEN** 使用者在 Sidebar 或專案 tab 點擊釘選按鈕
- **THEN** 該專案的 projectKey（`normalizePath(repoRoot)`）加入 `pinnedProjects` 陣列並儲存
- **AND** 該專案立即出現在 Sidebar 釘選區

#### Scenario: 取消釘選

- **WHEN** 使用者點擊已釘選專案的釘選按鈕
- **THEN** 從 `pinnedProjects` 移除，Sidebar 釘選區不再顯示該專案

#### Scenario: 同目錄切換分支保持釘選

- **WHEN** 使用者在已釘選專案的目錄中以 `git checkout` 切換至另一個分支
- **THEN** projectKey 不變，該專案 SHALL 維持釘選狀態
- **AND** 已開啟的專案 tab 與其 sub-tab 狀態 SHALL 保持不變

#### Scenario: 不同目錄的 worktree 各自獨立

- **WHEN** 同一 repo 於不同目錄建立 git worktree，且各自有 session
- **THEN** 各 worktree 的 `repoRoot` 不同，SHALL 產生不同的 projectKey
- **AND** 各 worktree SHALL 可獨立釘選，互不干擾

#### Scenario: 同一 repo 多分支同時釘選

- **NOTE** 此情境標題沿用自舊版 spec，其原有行為（同目錄不同分支各自釘選）已不再適用；分支不再構成專案身分，舊資料依下述方式合併
- **WHEN** 既有設定中同一目錄存在多個分支的釘選 key（如 `path:branchA` 與 `path:branchB`）
- **THEN** 兩者 SHALL 合併為單一 `path` key，Sidebar 釘選區顯示一個項目
- **AND** 該專案 SHALL 涵蓋該目錄下所有分支的 session

#### Scenario: 釘選專案 tab 排序

- **WHEN** 同時有釘選與非釘選的專案 tab 開啟
- **THEN** 釘選專案 tab 排在 Dashboard 之後、一般專案之前
- **AND** 釘選專案 tab 顯示固定標記

#### Scenario: settings 載入時的 key 正規化

- **WHEN** 應用程式啟動並從 settings.json 載入 `pinnedProjects`
- **THEN** 舊格式 `path:branch` 的 key SHALL 截去 `:branch` 後綴，僅保留路徑部分並套用 `normalizePath`
- **AND** 不含 `:` 分隔的舊格式 key（僅路徑）SHALL 整體做 `normalizePath`，保持向後相容
- **AND** 磁碟機代號的冒號（如 `d:\repo`）SHALL NOT 被誤判為 branch 分隔符
- **AND** 正規化後重複的 key SHALL 去重，避免 Sidebar 釘選區出現重複項目

### Requirement: Sidebar 釘選區快速導覽

Sidebar SHALL 在主導覽區顯示釘選專案的快捷連結。

#### Scenario: Sidebar 釘選區

- **WHEN** sidebar 展開且有釘選專案
- **THEN** Sidebar 中段顯示釘選專案列表（以路徑最後一段為名稱）
- **AND** 點擊即切換至對應專案 tab（未開啟則先開啟）

## ADDED Requirements

### Requirement: 專案分支標籤

專案分組可能包含同一目錄下多個分支的 session，介面 SHALL 顯示最近更新 session 所屬的分支作為該專案的分支標籤。

#### Scenario: 顯示目前工作分支

- **WHEN** 某專案的 session 分屬多個分支
- **THEN** 專案標題列與 Dashboard 卡片 SHALL 顯示 `updatedAt` 最新之 session 的分支名稱

#### Scenario: 無分支資訊

- **WHEN** 該專案所有 session 皆無分支資訊
- **THEN** SHALL NOT 顯示分支標籤
