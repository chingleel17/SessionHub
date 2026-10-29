# Spec Delta

## ADDED Requirements

### Requirement: Explorer 在分層載入時保留操作狀態

系統 SHALL 在同一專案的背景區段加入或重整時保留已選文件及使用者手動設定的來源與群組收折狀態；切換專案則使用新專案自身的狀態。

#### Scenario: 背景區段加入既有 Explorer

- **WHEN** 使用者選取進行中變更的文件並收合來源，主規格或封存結果隨後加入
- **THEN** 已選文件與內容維持可見，來源不自行重新展開
- **AND** 新區段依既有群組預設狀態呈現

#### Scenario: 分層重整期間文件暫時不見

- **WHEN** 文件因 change 從進行中移至封存而在中間區段暫時不存在
- **THEN** Explorer 等候背景區段完成後再決定是否清除選取

#### Scenario: 切換專案

- **WHEN** 使用者從 A 專案切換至 B 專案
- **THEN** B 專案不沿用 A 專案的文件選取與收折狀態
