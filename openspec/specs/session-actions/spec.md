## ADDED Requirements

### Requirement: 封存 session
系統 SHALL 將指定 session 的目錄從 `session-state/<id>/` 移動至 `session-state-archive/<id>/`。

#### Scenario: 封存成功
- **WHEN** 使用者點擊 session 的「封存」按鈕並確認
- **THEN** 系統將 session 目錄移動至 archive 位置
