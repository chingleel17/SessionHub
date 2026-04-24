## MODIFIED Requirements

### Requirement: 筐選多維度過濾

Session 清單 SHALL 支援多維度等過濾條件的組合，包含 provider、是否隱藏空 session、文字搜尋與更新時間區間。

#### Scenario: 過濾空 session

- **WHEN** 使用者啟用「隱藏空 session」
- **THEN** summary 為空且 summary_count 為 0 的 session SHALL 被隱藏

#### Scenario: 過濾特定 provider

- **WHEN** 使用者選擇 provider 等過濾
- **THEN** 僅顯示符合 provider 的 session

#### Scenario: 過濾更新時間區間

- **WHEN** 使用者將更新時間篩選切換為「本周」或「本月」
- **THEN** 系統只顯示 `updatedAt` 落在所選區間內的 session
- **AND** 其他搜尋、tag 與 provider 條件仍以 AND 邏輯一併套用

#### Scenario: 多條件同時過濾

- **WHEN** 多個過濾條件同時啟用
- **THEN** 系統以 AND 邏輯對它們進行組合過濾

## ADDED Requirements

### Requirement: Project session 清單分頁

ProjectView 的 session 卡片列表 SHALL 使用分頁渲染，避免一次顯示所有符合條件的 session。

#### Scenario: 預設顯示第一頁
- **WHEN** 使用者進入 Sessions 子分頁
- **THEN** 系統只渲染第一頁的 session 卡片
- **AND** 顯示目前結果總數、目前頁碼與可切換的上一頁/下一頁控制

#### Scenario: 篩選條件變更時重置頁碼
- **WHEN** 使用者修改搜尋、排序、tag、provider、隱藏空 session 或更新時間篩選
- **THEN** 系統自動切回第 1 頁
- **AND** 以新的篩選結果重新計算總頁數

#### Scenario: 篩選後無結果
- **WHEN** 所有篩選條件套用後沒有任何 session
- **THEN** 系統顯示空狀態與 `0` 筆符合條件的結果摘要
- **AND** 不顯示空白分頁按鈕列
