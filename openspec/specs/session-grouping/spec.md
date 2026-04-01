## ADDED Requirements

### Requirement: 依專案路徑分組
系統 SHALL 將 session 依 `cwd` 欄位分組，相同 `cwd` 的 session 歸為同一專案群組。

#### Scenario: 多個 session 屬於相同專案
- **WHEN** 多個 session 的 `cwd` 相同
- **THEN** 系統將這些 session 顯示在同一專案群組下

### Requirement: Windows 路徑大小寫正規化
系統 SHALL 在分組時對 `cwd` 路徑進行大小寫不敏感的正規化，避免因磁碟代號或路徑大小寫差異而產生重複專案群組。

#### Scenario: 路徑大小寫不同但實際為相同目錄
- **WHEN** 兩個 session 的 `cwd` 分別為 `D:\ching\project` 與 `d:\ching\project`
- **THEN** 系統應將其視為相同專案群組，合併顯示
- **AND** 顯示路徑使用先出現的原始大小寫格式
