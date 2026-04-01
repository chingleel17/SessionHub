## MODIFIED Requirements

### Requirement: 專案分頁新增子分頁機制
系統 SHALL 在現有的專案分頁（tab-item）內部支援子分頁（sub-tab）切換，以區分不同類型的專案資訊。

#### Scenario: 子分頁列渲染
- **WHEN** 使用者進入任何專案分頁
- **THEN** 專案標題下方顯示子分頁列（sub-tab bar），視覺上與頂層分頁區隔（較小字體、無底部邊框突出效果）

#### Scenario: 子分頁列樣式
- **WHEN** 子分頁列渲染
- **THEN** 採用與現有 tab bar 一致的互動模式（點擊切換、active 狀態高亮），但視覺尺寸較小以表示層級關係

### Requirement: sub-tab bar 與 tab bar 共存
系統 SHALL 確保子分頁列不與頂層專案分頁列產生視覺或行為衝突。

#### Scenario: 切換專案分頁
- **WHEN** 使用者從專案 A 切換到專案 B
- **THEN** 專案 B 的子分頁重置為預設（Sessions），不保留專案 A 的子分頁狀態

#### Scenario: Dashboard 分頁無子分頁
- **WHEN** 使用者選中 Dashboard 分頁
- **THEN** 不顯示子分頁列（子分頁僅在專案分頁中出現）

### Requirement: Plans & Specs 分頁 badge
系統 SHALL 在 "Plans & Specs" 子分頁標籤上顯示計數 badge，指示是否有資料。

#### Scenario: 有 plan 或 spec 資料
- **WHEN** 專案目錄下存在 `.sisyphus/plans/` 或 `openspec/changes/` 且內含檔案
- **THEN** "Plans & Specs" 分頁標籤旁顯示數字 badge（plan 數 + active change 數）

#### Scenario: 無 plan 或 spec 資料
- **WHEN** 專案目錄下不存在 `.sisyphus/` 與 `openspec/` 或兩者皆為空
- **THEN** "Plans & Specs" 分頁標籤不顯示 badge（或顯示 0，依設計決定）
