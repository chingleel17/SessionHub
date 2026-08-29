## MODIFIED Requirements

### Requirement: 收折按鈕位置固定

收折／展開按鈕 SHALL 整合於品牌列靠 Sidebar 邊界的尾端位置，並在展開與收折兩種狀態下保持可見、可操作及對齊品牌區；按鈕不得獨占 Dashboard 上方的額外列。

#### Scenario: 展開狀態顯示控制

- **WHEN** Sidebar 處於展開狀態
- **THEN** 收折按鈕顯示於品牌名稱尾端、靠近 Sidebar 邊界
- **AND** Dashboard 導覽項目前不保留僅供此按鈕使用的空白列

#### Scenario: 切換至收折狀態

- **WHEN** 使用者點擊收折按鈕且 Sidebar 完成收折
- **THEN** 展開按鈕仍顯示於品牌 icon 旁或其靠 Sidebar 邊界的尾端區域
- **AND** 按鈕不遮蔽品牌 icon、workspace 內容或 Sidebar 導覽項目

#### Scenario: 切換收折狀態時按鈕不位移

- **WHEN** 使用者連續切換收折與展開
- **THEN** 控制項 SHALL 隨 Sidebar 邊界平滑移動並維持一致的品牌列垂直位置
- **AND** 每個狀態下的點擊目標 SHALL 保持完整可用

## ADDED Requirements

### Requirement: 收折控制圖示反映目標狀態

收折／展開按鈕 SHALL 依 Sidebar 當前狀態顯示方向或語意不同的圖示，使圖示清楚表達下一次啟用將執行收折或展開；按鈕的 accessible name 與 title SHALL 使用對應的在地化文案。

#### Scenario: 展開狀態顯示收折圖示

- **WHEN** Sidebar 處於展開狀態
- **THEN** 按鈕顯示指向收折方向的圖示
- **AND** accessible name 與 title 表達「收折側欄」

#### Scenario: 收折狀態顯示展開圖示

- **WHEN** Sidebar 處於收折狀態
- **THEN** 按鈕顯示指向展開方向的圖示
- **AND** accessible name 與 title 表達「展開側欄」

#### Scenario: 以鍵盤切換側欄

- **WHEN** 鍵盤焦點位於收折／展開按鈕且使用者啟用按鈕
- **THEN** Sidebar SHALL 切換狀態
- **AND** 焦點 SHALL 保留在同一控制項上以便連續操作
