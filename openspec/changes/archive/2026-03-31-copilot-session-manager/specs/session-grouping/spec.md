## ADDED Requirements

### Requirement: 依專案路徑分組
系統 SHALL 將 session 依 `cwd` 欄位分組，相同 `cwd` 的 session 歸為同一專案群組。

#### Scenario: 多個 session 屬於相同專案
- **WHEN** 多個 session 的 `cwd` 相同
- **THEN** 系統將這些 session 顯示在同一專案群組下
