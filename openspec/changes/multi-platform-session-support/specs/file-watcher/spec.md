## MODIFIED Requirements

### Requirement: 目錄變更即時偵測
系統 SHALL 使用 OS filesystem watch 監聽 Copilot `session-state/` 目錄的新增、刪除、修改事件，以及 OpenCode 資料庫檔案（`opencode.db-wal`）的變更事件，並透過 Tauri event 通知前端。

#### Scenario: 新 Copilot session 目錄建立
- **WHEN** Copilot CLI 在 `session-state/` 下建立新的 session 目錄
- **THEN** 前端 UI 在 3 秒內自動更新 session 列表

#### Scenario: OpenCode 資料庫變更
- **WHEN** OpenCode 寫入新 session 資料至 `opencode.db`
- **THEN** 系統偵測 `opencode.db-wal` 檔案變更，前端 UI 自動更新 session 列表

## ADDED Requirements

### Requirement: OpenCode 資料庫 watcher
系統 SHALL 建立獨立的 FS watcher 監聽 OpenCode 資料庫 WAL 檔案變更。

#### Scenario: 啟動 OpenCode watcher
- **WHEN** 應用程式啟動且 OpenCode provider 已啟用
- **THEN** 系統建立 watcher 監聽 `{opencodeRoot}/opencode.db-wal`

#### Scenario: OpenCode provider 停用
- **WHEN** 使用者停用 OpenCode provider
- **THEN** 系統停止並釋放 OpenCode 資料庫的 watcher
