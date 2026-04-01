## ADDED Requirements

### Requirement: Provider 篩選 UI

系統 SHALL 在 session 列表上方提供 provider 篩選切換按鈕，讓使用者快速切換只看特定 provider 的 session。

#### Scenario: 全部 provider 顯示（預設）

- **WHEN** 未套用 provider 篩選
- **THEN** session 列表顯示所有已啟用 provider 的 session

#### Scenario: 篩選至單一 provider

- **WHEN** 使用者點擊 `Copilot` 或 `OpenCode` 篩選按鈕
- **THEN** session 列表只顯示該 provider 的 session
- **AND** 篩選按鈕呈現選中狀態（active style）

#### Scenario: 取消篩選

- **WHEN** 使用者點擊已選中的 provider 篩選按鈕
- **THEN** 恢復顯示所有 provider 的 session

### Requirement: Provider 篩選與其他篩選組合

Provider 篩選 SHALL 與文字搜尋、「隱藏空 session」等篩選條件以 AND 邏輯組合。
