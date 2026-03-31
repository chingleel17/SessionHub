## ADDED Requirements

### Requirement: 終端執行檔驗證
系統 SHALL 在儲存設定前驗證使用者指定的終端執行檔路徑是否存在且可執行。

#### Scenario: 有效的終端路徑
- **WHEN** 使用者輸入終端路徑並儲存
- **THEN** 系統驗證路徑對應的可執行檔存在
