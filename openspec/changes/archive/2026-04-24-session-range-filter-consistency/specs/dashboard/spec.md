## MODIFIED Requirements

### Requirement: 最近活動顯示限制與專案標籤

系統 SHALL 在首頁「最近活動」清單中限制每筆 session 標題的顯示長度，附上所屬專案名稱，且清單內容只顯示目前 Dashboard 時間區間內的 session。

#### Scenario: Summary 超過最大顯示長度

- **WHEN** session 的 summary 長度超過 80 個字元
- **THEN** 系統截斷並附上「…」省略符號
- **AND** 完整內容可透過 `title` 屬性（tooltip）查看

#### Scenario: 顯示所屬專案名稱

- **WHEN** session 的 `cwd` 不為空
- **THEN** 系統在 session 標題後方顯示小型專案名稱標籤（取路徑最後一段）

#### Scenario: 切換時間區間後更新最近活動

- **WHEN** 使用者切換 Dashboard 的本周或本月
- **THEN** 最近活動清單只顯示該時間區間內最新更新的 sessions
- **AND** 不會殘留上一個時間區間的 session

### Requirement: 專案卡片顯示最後一次 Session 標題

系統 SHALL 在首頁「專案分頁預覽」的每個專案卡片中，顯示該專案在目前 Dashboard 時間區間內最近一次 session 的標題。

#### Scenario: 顯示最近 session 標題

- **WHEN** 使用者瀏覽首頁專案清單
- **THEN** 系統在每個專案卡片中以小字顯示最新一筆 session 的 summary（限 60 字元）
- **AND** 若該 session 無 summary 則不顯示此欄位

#### Scenario: 超出時間區間的專案不顯示

- **WHEN** 某專案沒有任何 session 落在目前 Dashboard 時間區間內
- **THEN** 該專案不出現在專案預覽清單中
