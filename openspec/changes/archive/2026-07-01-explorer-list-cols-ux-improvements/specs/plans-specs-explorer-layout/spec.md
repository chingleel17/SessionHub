## MODIFIED Requirements

### Requirement: 折疊左側面板

系統 SHALL 允許使用者透過折疊切換按鈕收起左側 Explorer 面板。折疊後面板縮小至最小寬度（≤40px），右側面板佔用釋放的空間；折疊狀態下 header 的底部邊框 SHALL 隱藏，header 高度 SHALL 自動縮小至剛好容納折疊按鈕。

#### Scenario: 折疊後面板縮小且 header 底線隱藏

- **WHEN** 使用者點擊折疊切換按鈕
- **THEN** 左側面板 SHALL 縮小至 ≤40px 寬度
- **AND** 右側面板 SHALL 佔用釋放的空間
- **AND** `.explorer-panel-header` 的 `border-bottom` SHALL 不顯示
- **AND** header 高度 SHALL 為 auto

#### Scenario: 展開後 header 底線恢復

- **WHEN** 使用者再次點擊按鈕展開面板
- **THEN** 左側面板 SHALL 恢復至折疊前的寬度
- **AND** `.explorer-panel-header` 的 `border-bottom` SHALL 重新顯示
- **AND** header `min-height` SHALL 恢復為 50px
