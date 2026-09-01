## ADDED Requirements

### Requirement: 封存 session

系統 SHALL 將指定 session 的目錄從 `session-state/<id>/` 移動至 `session-state-archive/<id>/`。

#### Scenario: 封存成功

- **WHEN** 使用者點擊 session 的「封存」按鈕並確認
- **THEN** 系統將 session 目錄移動至 archive 位置

### Requirement: 刪除空 session（批次）

系統 SHALL 提供批次刪除無對話內容的空 session 功能。

#### Scenario: 批次刪除空 session

- **WHEN** 使用者點擊「刪除空 session」並確認
- **THEN** 系統刪除所有 summary 為空且 summary_count 為 0 的 session 目錄（同時刪除 Copilot 與 OpenCode）
- **AND** 顯示已刪除筆數的 toast 通知

#### Scenario: 無空 session 時

- **WHEN** 無符合條件的空 session
- **THEN** 顯示「無需清理」提示，不執行任何刪除

### Requirement: 操作按鈕改為 icon 化

Session 卡片 SHALL 使用 SVG icon 按鈕取代文字按鈕，並搭配 tooltip 說明。

#### Scenario: Icon 按鈕顯示

- **WHEN** 使用者 hover session 卡片
- **THEN** 顯示操作 icon 按鈕（封存、開啟終端、複製指令、編輯備註等）
- **AND** 每個按鈕 hover 時顯示 tooltip 文字

### Requirement: Session 卡片快速編輯入口

Session 卡片 SHALL 提供直接從內容區快速進入編輯的入口，避免只能透過操作列 icon。

#### Scenario: 點擊標籤 chip 快速編輯

- **WHEN** 使用者點擊 session 卡片右側任一標籤 chip
- **THEN** 系統開啟單一標籤編輯對話框

#### Scenario: 快速編輯目標必須與點擊項目一致

- **WHEN** 使用者在排序、分頁或篩選後點擊任一標籤或備註進行編輯
- **THEN** 系統 SHALL 始終編輯被點擊的 session 與被點擊的標籤項目，不得因清單索引變動而錯位

#### Scenario: 點擊備註文字快速編輯

- **WHEN** 使用者點擊 session 卡片中的備註文字區
- **THEN** 系統開啟編輯備註對話框

### Requirement: Session 中繼資料儲存後一致回顯

系統 SHALL 將使用者儲存的標籤與備註套用至正確的 session，並在儲存成功後立即於目前畫面及後續資料重新載入中回傳最新值。

#### Scenario: 儲存標籤後立即顯示

- **WHEN** 使用者為 session 新增或修改標籤並成功儲存
- **THEN** 該 session 卡片立即顯示最新標籤
- **AND** 再次開啟標籤編輯器時輸入值包含剛儲存的標籤

#### Scenario: 儲存備註後立即顯示

- **WHEN** 使用者新增或修改 session 備註並成功儲存
- **THEN** 該 session 卡片立即顯示最新備註
- **AND** 再次開啟備註編輯器時輸入值包含剛儲存的備註

#### Scenario: 清除中繼資料後立即隱藏

- **WHEN** 使用者清除 session 的備註或移除標籤並成功儲存
- **THEN** 該 session 卡片立即移除對應的備註或標籤
- **AND** 再次開啟編輯器時不再顯示已清除的值

#### Scenario: 重新查詢後保留最新中繼資料

- **WHEN** 中繼資料儲存成功後系統重新查詢、增量掃描或重新啟動應用程式
- **THEN** 系統從持久儲存回傳該 session 的最新標籤與備註
- **AND** 舊的 session 掃描快取不得覆蓋最新中繼資料

#### Scenario: 中繼資料儲存失敗

- **WHEN** 標籤或備註無法成功寫入持久儲存
- **THEN** 系統顯示錯誤提示
- **AND** session 卡片不得將未成功儲存的值呈現為已完成狀態
