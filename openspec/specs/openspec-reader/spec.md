## ADDED Requirements

### Requirement: 掃描 openspec 目錄

系統 SHALL 掃描專案工作目錄下的 `openspec/` 子目錄，讀取 specs 與 changes 結構，並在 ProjectView 的 Plans & Specs 子分頁中顯示。

#### Scenario: 偵測 openspec 存在

- **WHEN** 系統掃描 session 的 cwd
- **THEN** 若 `<cwd>/openspec/` 存在，SessionInfo 的相關標記設為 true
- **AND** ProjectView 顯示 Plans & Specs 子分頁入口

#### Scenario: 讀取 specs 清單

- **WHEN** 使用者開啟 Plans & Specs 子分頁
- **THEN** 系統列出 `<cwd>/openspec/specs/` 下所有子目錄名稱
- **AND** 每個子目錄代表一個 spec 項目，點擊可查看 spec.md 內容

#### Scenario: 讀取 active changes 清單

- **WHEN** 使用者開啟 Plans & Specs 子分頁
- **THEN** 系統列出 `<cwd>/openspec/changes/`（不含 archive）下的 change 資料夾
- **AND** 顯示每個 change 的 tasks.md 完成進度（已勾 / 總數）
