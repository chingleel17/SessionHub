## ADDED Requirements

### Requirement: Provider integration 顯示 quota 診斷資訊

系統 SHALL 在 provider integration 管理區塊中顯示與 quota monitoring 相關的診斷資訊，例如 quota source、auth 狀態、最後刷新結果與手動刷新入口。

#### Scenario: 顯示 quota source 與狀態
- **WHEN** 使用者開啟設定頁的 provider integration 區塊
- **THEN** 系統可顯示該 provider quota 的資料來源與目前狀態
- **AND** 不要求使用者離開 SessionHub 才能查看 quota 診斷

#### Scenario: 手動刷新 provider quota
- **WHEN** 使用者在 provider integration 卡片點擊 quota refresh
- **THEN** 系統重新查詢該 provider 或相關 quota provider 的 snapshot
- **AND** 更新畫面中的最後刷新時間與錯誤訊息
