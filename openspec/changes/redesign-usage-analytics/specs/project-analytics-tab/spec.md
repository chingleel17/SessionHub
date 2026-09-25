## MODIFIED Requirements

### Requirement: ProjectView Analytics 子頁籤

ProjectView SHALL 保留 Analytics 子頁籤，位於 Agents 之後，使用與獨立分析頁相同的分析工作區，固定目前專案 cwd。

#### Scenario: Analytics 頁籤顯示
- **WHEN** 使用者進入專案
- **THEN** Analytics 仍位於既有靜態子頁籤末端、Agents 之後，不移除 Sessions、Plans & Specs 或動態 Plan 子頁籤

#### Scenario: 初始狀態
- **WHEN** 首次切換至 Analytics
- **THEN** 自動載入近 30 個本地日、按日、該 cwd 的分析，顯示載入／補算狀態，不要求點擊產生圖表

### Requirement: 分組粒度切換

Analytics SHALL 提供日／週／月粒度，變更後自動以共同查詢口徑更新。

#### Scenario: 切換 groupBy
- **WHEN** 使用者切換粒度
- **THEN** 自動重新查詢，X 軸依日、ISO 週或月更新，tooltip 保留完整日期範圍

### Requirement: 查詢結果在頁籤存活期間快取

Analytics SHALL 在應用存活期間保存各專案的篩選與查詢快取，快取識別包含完整條件及來源／索引版本，不依賴呈現元件是否掛載。

#### Scenario: 切換子頁籤後切回
- **WHEN** 使用者離開後回到同專案 Analytics
- **THEN** 相同條件且未過期的結果立即重用，過期則背景刷新並保留條件

#### Scenario: 切換不同專案
- **WHEN** 使用者檢視另一專案 Analytics
- **THEN** 不沿用前一專案結果或 cwd，返回時恢復各自條件

## ADDED Requirements

### Requirement: Analytics 自動查詢與手動重新整理

Analytics SHALL 以首次進入與有效條件變更自動查詢取代僅手動產生圖表的行為，同時提供手動重新整理。

#### Scenario: 自動查詢及手動重新整理
- **WHEN** 首次進入、條件改變或點擊重新整理
- **THEN** 由前端集中 IPC 以固定 cwd 和目前條件查詢，顯示正確載入狀態

#### Scenario: 查詢完成顯示圖表
- **WHEN** 分析查詢成功
- **THEN** 顯示共用摘要、同單位趨勢、模型排行及 session 明細，標示期間與完整性
- **AND** 專案排行在固定 cwd 下不重複占用完整區塊

#### Scenario: 查詢失敗
- **WHEN** 查詢失敗
- **THEN** 呼叫 showToast 並提供重試，保留成功結果時標示原條件與過期狀態，首次失敗顯示錯誤狀態

## REMOVED Requirements

### Requirement: 手動觸發圖表查詢

**Reason**: 專案 Analytics 改為首次進入及有效條件變更時自動載入，已不再提供「產生圖表」按鈕。

**Migration**: 改用共用分析工作區的日期、分組、來源篩選與手動重新整理；既有固定 cwd 與載入錯誤回饋保留。
