## ADDED Requirements

### Requirement: Overlay 內嵌需授權提醒區

quota overlay 視窗 SHALL 訂閱 `intervention-list-changed` 並內嵌「需授權」提醒區；清單為 0 筆時整區不 render，清單非空時顯示總數與每筆項目。

#### Scenario: 有 waiting 時顯示提醒區

- **WHEN** overlay 收到 `intervention-list-changed` 且清單含至少一筆
- **THEN** overlay 顯示「需授權」提醒區，標題含當前總數 (N)
- **AND** 每筆顯示 `專案名 · 工具類型`，僅顯示工具類型不顯示指令或路徑

#### Scenario: 清單為空時整區隱藏

- **WHEN** overlay 收到的清單為 0 筆
- **THEN** overlay 不 render 提醒區，視窗尺寸自動縮回僅含 quota 內容的大小

#### Scenario: compact 與 full 兩種模式皆支援

- **WHEN** overlay 的 styleMode 為 `compact` 或 `full`
- **THEN** 兩種模式皆 render 需授權提醒區，各自採用對應版式（compact 為精簡列、full 為區塊）

### Requirement: 提醒區自動延伸方向避開螢幕邊界

overlay SHALL 依視窗位置與螢幕可用工作區（扣除工作列）自動決定提醒區延伸方向：往下延伸不會被工作列或螢幕底緣遮擋時往下，否則往上延伸；往上延伸時 quota chip 的螢幕位置維持不變。

#### Scenario: 貼工作列時往上延伸且 chip 不位移

- **WHEN** overlay 貼近螢幕可用工作區底緣
- **AND** 提醒區往下延伸會超出可用工作區底緣（被工作列遮擋）
- **THEN** 提醒區改為往上延伸
- **AND** quota chip 那一列的螢幕垂直位置維持不變（視窗上緣同步上移，chip 不被推走）

#### Scenario: 下方空間充足時往下延伸

- **WHEN** overlay 下方於可用工作區內有足夠空間容納提醒區
- **THEN** 提醒區往下延伸，chip 維持在上方

#### Scenario: 貼頂緣無上方空間時 fallback 往下

- **WHEN** overlay 貼近螢幕頂緣，往上延伸所需空間不足
- **THEN** 提醒區 fallback 往下延伸，不使內容超出螢幕頂端而不可見

#### Scenario: 大量 waiting 不撐出螢幕

- **WHEN** 同時有大量 session 處於 waiting 使提醒區內容過高
- **THEN** 提醒區高度受上限約束並在內部捲動，標題仍顯示總數 (N)
- **AND** 提醒區不延伸超出螢幕可視範圍

### Requirement: 提醒卡片點擊聚焦並導航

overlay 提醒區的每筆卡片 SHALL 於點擊時使主視窗取得焦點並導航至該 session 所屬 project tab，複用既有通知點擊的導航語意。

#### Scenario: 點擊卡片聚焦主視窗並導航

- **WHEN** 使用者點擊提醒區某筆卡片
- **THEN** overlay 發出帶該 `sessionId` 的聚焦意圖事件
- **AND** 主視窗取得焦點並帶到前景
- **AND** 前端路由切換至該 session 所屬 project tab（與 `notification://action-performed` 相同的 `projectKey` 定位邏輯）
