## MODIFIED Requirements

### Requirement: 多維度 session 篩選

Session 列表 SHALL 支援多個篩選維度同時作用，以 AND 邏輯組合。篩選工具列 SHALL 以緊湊單行佈局顯示，高度縮減至約 60px（含 padding），較原先高度縮減約 50%。

#### Scenario: 文字搜尋篩選

- **WHEN** 使用者在搜尋框輸入關鍵字
- **THEN** 列表只顯示 summary 或 cwd 包含該關鍵字的 session（不分大小寫）

#### Scenario: 隱藏空 session

- **WHEN** 使用者啟用「隱藏空 session」開關
- **THEN** summary 為空且 summary_count 為 0 的 session 不顯示

#### Scenario: Provider 篩選

- **WHEN** 使用者選擇特定 provider 篩選
- **THEN** 只顯示該 provider 的 session

#### Scenario: 多條件組合

- **WHEN** 多個篩選條件同時啟用
- **THEN** 只顯示同時滿足所有條件的 session

#### Scenario: 緊湊排版

- **WHEN** 使用者開啟 sessions sub-tab
- **THEN** 篩選工具列 SHALL 以 flex 單行排列顯示搜尋框、排序選單、checkbox 與動作按鈕，整體高度 SHALL 不超過 72px

### Requirement: 篩選狀態提示

- **WHEN** 有篩選條件啟用且結果為空
- **THEN** 顯示「符合條件的 session 數量：0」及清除篩選快捷連結
