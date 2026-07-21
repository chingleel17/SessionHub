## ADDED Requirements

### Requirement: Sidebar 顯示已開啟專案區塊

Sidebar SHALL 在釘選專案區塊下方顯示「已開啟的專案」區塊，列出所有目前已開啟（`openProjectKeys`）的專案。

#### Scenario: 已開啟清單顯示

- **WHEN** 使用者開啟一個或多個專案
- **THEN** Sidebar 在釘選區塊下方顯示「已開啟」標題與對應專案列表
- **AND** 當前作用中的專案項目 SHALL 以 `active` 樣式標示

#### Scenario: 無已開啟專案時隱藏區塊

- **WHEN** `openProjectKeys` 為空陣列
- **THEN** 已開啟區塊 SHALL 不顯示（不佔空間）

### Requirement: 已開啟專案項目提供關閉按鈕

每個已開啟專案項目 SHALL 在項目右側顯示 × 關閉按鈕。

#### Scenario: 點擊 × 關閉專案

- **WHEN** 使用者點擊已開啟專案項目的 × 按鈕
- **THEN** 該專案從 `openProjectKeys` 中移除
- **AND** 若被關閉的專案正是 `activeView`，系統 SHALL 自動切換至 `dashboard`

#### Scenario: × 按鈕不影響項目導覽

- **WHEN** 使用者點擊專案項目的文字區域（非 × 按鈕）
- **THEN** 系統切換至該專案視圖，不關閉

### Requirement: 折疊狀態下已開啟專案的呈現

Sidebar 折疊時，已開啟專案 SHALL 以 icon button（首字母）形式呈現，並在 hover 時顯示 × 關閉按鈕。

#### Scenario: 折疊模式 hover 顯示關閉

- **WHEN** Sidebar 為折疊狀態且使用者 hover 已開啟項目
- **THEN** icon button 右上角顯示微型 × 按鈕
- **AND** 點擊 × 執行關閉，點擊 icon 本身執行導覽
