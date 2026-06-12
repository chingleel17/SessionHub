## ADDED Requirements

### Requirement: OpenCode 作為第二 session provider

系統 SHALL 支援 OpenCode 作為第二 AI coding session provider，與 Copilot 並存運作。

#### Scenario: 雙 provider 同時啟用

- **WHEN** copilotRoot 與 opencodeRoot 均已設定且 enabledProviders 包含兩者
- **THEN** 系統分別掃描兩個 provider 的 session 目錄
- **AND** 回傳合併清單，依 updated_at 降冪排序

#### Scenario: 單獨啟用 OpenCode

- **WHEN** enabledProviders 僅包含 `"opencode"`
- **THEN** 系統僅掃描 OpenCode session，不掃描 Copilot

#### Scenario: Provider 停用

- **WHEN** enabledProviders 不包含某 provider
- **THEN** 系統跳過該 provider 的掃描，不回傳其 session

### Requirement: OpenCode session 資料結構

OpenCode session SHALL 優先從 `opencode.db` 讀取 metadata，並在必要時退回 `<opencodeRoot>/storage/session/<projectID>/ses_*.json` 舊格式。

#### Scenario: OpenCode session 解析

- **WHEN** 系統掃描 OpenCode session 資料來源
- **THEN** 每個 session 解析為 SessionInfo，provider 設為 `"opencode"`
- **AND** session.directory 或對應 project/worktree 對應 cwd，session.title 對應 summary
- **AND** session time 欄位對應 created_at / updated_at

#### Scenario: 最新 session 僅存在 DB

- **WHEN** bridge 事件指向的 session 已存在於 `opencode.db`
- **AND** 舊 JSON storage 沒有對應 session 檔案
- **THEN** SessionHub refresh 後仍應顯示該 session

### Requirement: OpenCode session 分組依 project / worktree

OpenCode session SHALL 以 project 或 worktree 對應的 cwd 作為分組依據。

#### Scenario: OpenCode session 分組

- **WHEN** 系統依 cwd 將 OpenCode session 分組
- **THEN** 以 project/worktree 對應的工作目錄作為 cwd
- **AND** 與 Copilot session 共用同一 cwd 的專案合併顯示在同一 ProjectView
