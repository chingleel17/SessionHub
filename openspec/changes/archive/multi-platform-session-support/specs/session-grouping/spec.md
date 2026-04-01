## MODIFIED Requirements

### Requirement: 依專案路徑分組
系統 SHALL 將所有平台的 session 依 `cwd` 欄位分組，相同 `cwd`（路徑正規化後比對）的 session 歸為同一專案群組，無論其來源平台為何。

#### Scenario: 不同平台的 session 屬於相同專案
- **WHEN** Copilot session 的 cwd 為 `D:\project\my-app` 且 OpenCode session 的 cwd 為 `D:\project\my-app`
- **THEN** 系統將兩者歸入同一專案群組，群組內可見兩個平台的 session

#### Scenario: 路徑正規化比對
- **WHEN** 兩個 session 的 cwd 在正規化後相同（如大小寫差異或尾部斜線差異）
- **THEN** 系統將其視為同一專案
