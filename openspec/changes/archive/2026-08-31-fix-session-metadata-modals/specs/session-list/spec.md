## ADDED Requirements

### Requirement: Project Session 卡片精簡中繼資訊

Project 頁面的 session 卡片 SHALL 顯示更新時間、建立時間與 Summary 數量，且 SHALL NOT 顯示 Git Repo 欄位。

#### Scenario: 顯示 Project Session 卡片

- **WHEN** 使用者在 Project 頁面查看 session 卡片
- **THEN** 卡片中繼資訊區顯示更新時間、建立時間與 Summary 數量
- **AND** 卡片不顯示 Git Repo 標題或 repository 名稱
