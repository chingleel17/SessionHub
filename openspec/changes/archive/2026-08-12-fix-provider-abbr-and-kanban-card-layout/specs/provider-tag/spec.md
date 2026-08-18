## ADDED Requirements

### Requirement: Provider 縮寫代碼單一來源

Provider 縮寫代碼（用於 provider 標籤、icon、quota 顯示等所有位置）SHALL 由單一共用函式定義，不得在不同元件中各自維護獨立的對照表。

#### Scenario: 各處縮寫代碼一致

- **WHEN** 任一處（session 卡片、Dashboard Kanban ProjectCard、Quota Overlay、狀態列）需要顯示 provider 縮寫代碼
- **THEN** 皆呼叫同一個共用函式取得縮寫，不同位置對同一 provider 顯示相同的縮寫代碼

#### Scenario: 已知 provider 的縮寫對照

- **WHEN** provider 為 `claude`、`copilot`、`opencode`、`codex`、`antigravity` 之一
- **THEN** 分別顯示 `CC`、`CP`、`OC`、`CX`、`AG`

#### Scenario: 未知 provider 的後備顯示

- **WHEN** provider 不在已知對照表中
- **THEN** 以該 provider 識別碼前兩碼大寫作為縮寫顯示，不得顯示空白或拋出錯誤
