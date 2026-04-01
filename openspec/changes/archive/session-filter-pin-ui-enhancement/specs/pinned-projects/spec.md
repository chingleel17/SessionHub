## ADDED Requirements

### Requirement: 釘選專案
系統 SHALL 允許使用者將專案（project group）釘選，釘選後該專案出現在 Sidebar 的「釘選專案」快速入口區段，並在 tab 列中排序至最前方。

#### Scenario: 釘選專案操作
- **WHEN** 使用者點擊 project group header 的釘選 icon
- **THEN** 系統將該 project key 加入 `pinnedProjects` 並儲存至 settings.json，釘選 icon 切換為已釘選狀態

#### Scenario: 取消釘選操作
- **WHEN** 使用者點擊已釘選 project 的釘選 icon
- **THEN** 系統將該 project key 從 `pinnedProjects` 移除並儲存，釘選 icon 切換回未釘選狀態

### Requirement: Sidebar 釘選專案快速入口
系統 SHALL 在 Sidebar 導覽列中顯示「釘選專案」區段，列出所有已釘選的專案名稱，點擊可直接切換至該專案 view。

#### Scenario: Sidebar 顯示釘選區段
- **WHEN** 至少一個專案已被釘選
- **THEN** Sidebar 顯示「釘選專案」區段，列出所有釘選專案名稱（使用 project title 顯示）

#### Scenario: Sidebar 無釘選時隱藏區段
- **WHEN** 沒有任何釘選專案
- **THEN** Sidebar 不顯示「釘選專案」區段

#### Scenario: 點擊 Sidebar 釘選專案
- **WHEN** 使用者點擊 Sidebar 中的釘選專案項目
- **THEN** 系統切換 activeView 至該專案，效果等同點擊 tab

### Requirement: 釘選專案持久化
系統 SHALL 將釘選狀態儲存於 `AppSettings.pinnedProjects`（string[] 型別，預設空陣列），透過現有 `save_settings` command 持久化。

#### Scenario: 釘選狀態跨重啟保留
- **WHEN** 使用者釘選專案後重新啟動應用程式
- **THEN** 先前的釘選狀態從 settings.json 讀取並正確還原

#### Scenario: 釘選專案已不存在時的處理
- **WHEN** settings.json 中的 pinnedProjects 包含一個已無對應 session 的 project key
- **THEN** 系統靜默忽略該項目（不顯示、不報錯）
