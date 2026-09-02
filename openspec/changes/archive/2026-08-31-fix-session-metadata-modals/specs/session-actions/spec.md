## ADDED Requirements

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
