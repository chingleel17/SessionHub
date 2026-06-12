## ADDED Requirements

### Requirement: Codex 作為第三 session provider

系統 SHALL 支援 Codex 作為第三個 AI coding session provider，並可與 Copilot、OpenCode 同時啟用。

#### Scenario: 三個 provider 同時啟用

- **WHEN** `enabledProviders` 同時包含 `"copilot"`、`"opencode"`、`"codex"`
- **THEN** 系統分別掃描三個 provider 的 session 來源
- **AND** 回傳合併清單，依 `updatedAt` 降冪排序

#### Scenario: 單獨啟用 Codex

- **WHEN** `enabledProviders` 僅包含 `"codex"`
- **THEN** 系統僅掃描 Codex session
- **AND** 不掃描 Copilot 與 OpenCode

### Requirement: Codex session 來源採日期分層 JSONL 檔案

Codex session SHALL 從 `<codexRoot>/sessions/<YYYY>/<MM>/<DD>/*.jsonl` 遞迴掃描。

#### Scenario: 掃描當日 session 檔案

- **WHEN** 系統掃描 Codex root 底下的 `sessions` 目錄
- **THEN** 系統遞迴搜尋所有符合日期分層路徑的 `.jsonl` 檔案
- **AND** 每個檔案視為單一 Codex session 來源

#### Scenario: 日期資料夾不存在

- **WHEN** `codexRoot/sessions` 不存在或尚未建立任何日期資料夾
- **THEN** 系統回傳空的 Codex session 結果
- **AND** 不應因此使其他 provider 掃描失敗

### Requirement: Codex session metadata 由 JSONL 事件推導

系統 SHALL 從 Codex session JSONL 的 `session_meta` 事件解析 session 基本資訊，並映射為既有 `SessionInfo` 欄位。

#### Scenario: 解析 session_meta 事件

- **WHEN** JSONL 檔案包含 `type = "session_meta"` 的事件
- **THEN** 系統使用 `payload.id` 作為 session ID
- **AND** 使用 `payload.cwd` 作為 `cwd`
- **AND** 使用 `payload.timestamp` 作為 `createdAt` 的主要來源

#### Scenario: 推導更新時間

- **WHEN** JSONL 檔案包含後續事件且事件帶有 `timestamp`
- **THEN** 系統以最後一筆可解析事件的 `timestamp` 作為 `updatedAt`
- **AND** 若無可用事件時間，系統退回使用檔案最後修改時間

#### Scenario: 缺少 session_meta

- **WHEN** JSONL 檔案缺少 `session_meta` 事件或其必要欄位
- **THEN** 系統不得讓整體掃描失敗
- **AND** 系統可略過該檔案或將其標記為解析失敗的 session

### Requirement: Codex session 依 cwd 參與專案分組

Codex session SHALL 與其他 provider 一樣，使用 `cwd` 作為專案分組依據。

#### Scenario: 與既有 provider 共用同一專案

- **WHEN** Codex session 的 `cwd` 與某個 Copilot 或 OpenCode session 相同
- **THEN** 系統將它們合併顯示於同一個 ProjectView

#### Scenario: 缺少 cwd

- **WHEN** Codex session 無法解析 `cwd`
- **THEN** 系統將該 session 歸入 uncategorized 類別
- **AND** 仍應保留其 provider 為 `"codex"`
