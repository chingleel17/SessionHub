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
