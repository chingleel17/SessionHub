## MODIFIED Requirements

### Requirement: Dashboard Analytics Panel

Dashboard SHALL 在原分析區顯示精簡消耗摘要、同單位迷你趨勢與前往完整分析的入口，使用共同分析口徑；帳戶配額區保持獨立可用，不再顯示完整專案分布圓餅圖。

#### Scenario: 進入 Dashboard 自動載入圖表
- **WHEN** 使用者切換至 Dashboard
- **THEN** 自動查詢目前本週／本月、所有專案、已啟用來源及未封存 session 的事件用量
- **AND** 顯示摘要、迷你趨勢及資料完整性

#### Scenario: 定時自動重整
- **WHEN** Dashboard 可見且達到既有設定的刷新間隔
- **THEN** 背景刷新而不清空既有摘要，並標示更新狀態

#### Scenario: 切換統計週期同步更新圖表
- **WHEN** 切換本週／本月
- **THEN** 摘要與迷你趨勢共同查詢新區間，不將舊結果誤標為新區間

#### Scenario: 首次載入失敗
- **WHEN** 首次分析查詢失敗
- **THEN** 顯示錯誤與重試，不影響其他 Dashboard 功能及 quota overview

#### Scenario: 前往完整分析
- **WHEN** 點擊查看完整分析
- **THEN** 開啟獨立分析頁並攜帶目前期間與其他篩選條件

### Requirement: Analytics Panel 可折疊

Dashboard Analytics Panel SHALL 保留既有可折疊行為與設定，收起精簡消耗內容，不隱藏獨立帳戶配額資訊。

#### Scenario: 折疊 Analytics Panel
- **WHEN** 點擊折疊
- **THEN** 消耗內容收起，只留下標題與入口，狀態寫入 settings.json

#### Scenario: 展開時若資料過期則重新查詢
- **WHEN** 展開且資料超過既有刷新間隔
- **THEN** 以當前條件背景重新查詢並標示更新狀態
