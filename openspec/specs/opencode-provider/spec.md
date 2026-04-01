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

OpenCode session SHALL 從 `<opencodeRoot>/session/<projectID>/ses_*.json` 讀取 metadata。

#### Scenario: OpenCode session 解析

- **WHEN** 系統掃描 OpenCode session 目錄
- **THEN** 每個 ses\_\*.json 解析為 SessionInfo，provider 設為 `"opencode"`
- **AND** session.directory 對應 cwd, session.title 對應 summary
- **AND** session.time.created / time.updated 對應 created_at / updated_at

### Requirement: OpenCode session 分組依 projectID

OpenCode session SHALL 以 `session.projectID` 作為分組依據，對應 project.directory（cwd）。

#### Scenario: OpenCode session 分組

- **WHEN** 系統依 cwd 將 OpenCode session 分組
- **THEN** 以 project.directory（SHA256 雜湊對應的工作目錄）作為 cwd
- **AND** 與 Copilot session 共用同一 cwd 的專案合併顯示在同一 ProjectView
