## MODIFIED Requirements

### Requirement: 多維度 session 篩選

Session 列表 SHALL 支援多個篩選維度同時作用，以 AND 邏輯組合。篩選工具列 SHALL 以彈性佈局排列，允許於視窗寬度不足時自動換行；搜尋框 SHALL 佔據明顯大於其他控制項的寬度比例。

#### Scenario: 文字搜尋篩選

- **WHEN** 使用者在搜尋框輸入關鍵字
- **THEN** 列表只顯示 session ID、summary、notes 或 tags 包含該關鍵字的 session（不分大小寫）
- **AND** 系統 SHALL NOT 將 session 的 cwd 路徑納入比對

#### Scenario: 空對話預設隱藏

- **WHEN** 使用者進入 sessions sub-tab 且未點擊「空對話」chip
- **THEN** `hasEvents` 為 false 的 session SHALL 不顯示

#### Scenario: 顯示空對話

- **WHEN** 使用者點擊「空對話」chip 使其進入啟用狀態
- **THEN** `hasEvents` 為 false 的 session SHALL 一併顯示於列表中

#### Scenario: Provider 篩選

- **WHEN** 使用者選擇特定 provider 篩選
- **THEN** 只顯示該 provider 的 session

#### Scenario: 多條件組合

- **WHEN** 多個篩選條件同時啟用
- **THEN** 只顯示同時滿足所有條件的 session

#### Scenario: 彈性排版

- **WHEN** 使用者開啟 sessions sub-tab
- **THEN** 篩選工具列 SHALL 依序顯示搜尋框、排序選單、更新時間選單與篩選 chip
- **AND** 當可用寬度不足以容納全部控制項時，工具列 SHALL 自動換行而非壓縮或裁切控制項

## ADDED Requirements

### Requirement: 自訂日期區間篩選

「更新時間」篩選 SHALL 於既有的「全部時間 / 近一週 / 近一月」之外提供「自訂區間」選項，讓使用者以起訖日期查詢特定期間的 session。

#### Scenario: 選擇自訂區間

- **WHEN** 使用者將更新時間選單切換為「自訂區間」
- **THEN** 系統 SHALL 展開起始日期與結束日期兩個日期輸入欄位

#### Scenario: 套用完整起訖日期

- **WHEN** 使用者同時填入起始日期與結束日期
- **THEN** 列表只顯示 `updatedAt` 落在該區間內的 session
- **AND** 區間 SHALL 包含起始日當日 00:00 至結束日當日 23:59:59

#### Scenario: 只填單一端點

- **WHEN** 使用者只填入起始日期
- **THEN** 列表只顯示 `updatedAt` 不早於該日期的 session
- **WHEN** 使用者只填入結束日期
- **THEN** 列表只顯示 `updatedAt` 不晚於該日期的 session

#### Scenario: 起訖日期顛倒

- **WHEN** 使用者填入的起始日期晚於結束日期
- **THEN** 系統 SHALL 顯示提示訊息並維持上一次有效的篩選結果，不套用無效區間

#### Scenario: 切離自訂區間

- **WHEN** 使用者將更新時間選單由「自訂區間」切換回其他選項
- **THEN** 系統 SHALL 收合日期輸入欄位並改以所選的預設區間篩選

### Requirement: 篩選 chip 文案

篩選工具列的切換 chip SHALL 使用簡短標籤以節省橫向空間。

#### Scenario: 已封存 chip

- **WHEN** 使用者檢視篩選工具列
- **THEN** 控制封存 session 顯示與否的 chip 標籤 SHALL 為「已封存」

#### Scenario: 空對話 chip

- **WHEN** 使用者檢視篩選工具列
- **THEN** 控制空 session 顯示與否的 chip 標籤 SHALL 為「空對話」
- **AND** chip 的啟用狀態 SHALL 代表「顯示空對話」而非「隱藏空對話」
