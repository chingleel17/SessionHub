## ADDED Requirements

### Requirement: 讀取所有 session
系統 SHALL 掃描設定的 Copilot 根目錄下 `session-state/` 子目錄，讀取每個子目錄內的 `workspace.yaml`，並回傳結構化 session 清單。

#### Scenario: 正常讀取 session 列表
- **WHEN** 使用者開啟應用程式且 `session-state/` 目錄存在
- **THEN** 系統顯示所有解析成功的 session，每筆包含：id、summary（若存在）、cwd、created_at、updated_at、summary_count
