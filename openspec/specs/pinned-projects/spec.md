## ADDED Requirements

### Requirement: 釘選專案持久化

系統 SHALL 允許使用者釘選常用專案，釘選狀態持久化儲存於 settings.json。

#### Scenario: 釘選專案

- **WHEN** 使用者在 Sidebar 或專案 tab 點擊釘選按鈕
- **THEN** 該專案的 cwd 加入 `pinnedProjects` 陣列並儲存
- **AND** 該專案立即出現在 Sidebar 釘選區

#### Scenario: 取消釘選

- **WHEN** 使用者點擊已釘選專案的釘選按鈕
- **THEN** 從 `pinnedProjects` 移除，Sidebar 釘選區不再顯示

#### Scenario: 釘選專案 tab 排序

- **WHEN** 同時有釘選與非釘選的專案 tab 開啟
- **THEN** 釘選專案 tab 排在 Dashboard 之後、一般專案之前
- **AND** 釘選專案 tab 顯示📌圖示或固定標記

### Requirement: Sidebar 釘選區快速導覽

Sidebar SHALL 在主導覽區顯示釘選專案的快捷連結。

#### Scenario: Sidebar 釘選區

- **WHEN** sidebar 展開且有釘選專案
- **THEN** Sidebar 中段顯示釘選專案列表（以路徑最後一段為名稱）
- **AND** 點擊即切換至對應專案 tab（未開啟則先開啟）
