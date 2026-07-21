## ADDED Requirements

### Requirement: 讀取 session todos 清單

系統 SHALL 提供 `read_session_todos` Tauri command，讀取指定 Copilot session 目錄下 `session.db` 的 `todos` 表，回傳 `SessionTodo[]`，每筆包含 `id`、`title`、`status`、`description`（可為 null）、`updatedAt`。

#### Scenario: 成功讀取有 todos 的 session

- **WHEN** `read_session_todos` 被呼叫且 `<session_dir>/session.db` 存在且含有 `todos` 表
- **THEN** 回傳按 `id` ASC 排序的 todos 陣列，status 值保留原始字串（`pending`、`in_progress`、`done`、`blocked`）

#### Scenario: session.db 不存在

- **WHEN** `<session_dir>/session.db` 不存在
- **THEN** 回傳空陣列（`Ok(vec![])`），不回傳 error

#### Scenario: todos 表不存在

- **WHEN** `session.db` 存在但無 `todos` 表（舊版 Copilot CLI session）
- **THEN** 回傳空陣列，不回傳 error

#### Scenario: schema 欄位不符

- **WHEN** `todos` 表存在但缺少預期欄位（如 `description`）
- **THEN** 缺少欄位補 null 或預設值，其餘欄位正常回傳，不拋出 error

### Requirement: UI 顯示 session todos

SessionView 或 ProjectView 的 session 詳細面板 SHALL 在 Copilot session 中顯示 todos 清單區塊，列出每個 todo 的狀態標記、標題。

#### Scenario: session 有 todos

- **WHEN** `read_session_todos` 回傳非空陣列
- **THEN** UI SHALL 顯示 todos 清單，每筆呈現狀態 badge（pending / in_progress / done / blocked）與 title

#### Scenario: session 無 todos

- **WHEN** `read_session_todos` 回傳空陣列
- **THEN** 不顯示 todos 區塊（或顯示「無任務資料」）

#### Scenario: live session 自動更新 todos

- **WHEN** `SessionStats.isLive = true` 且 UI 自動 refetch stats
- **THEN** todos 清單隨 refetch 同步更新

### Requirement: Session 卡片摘要 badge 區整合 Plan 與任務入口

系統 SHALL 將 session 的 Plan 與任務入口整合到 session 卡片底部的摘要 badge 區，與互動數、token、時長、LIVE 指示同區顯示，避免入口分散在不同區域。

#### Scenario: session 有 plan

- **WHEN** session `hasPlan = true`
- **THEN** 摘要 badge 區顯示可點擊的 Plan badge，且點擊後開啟該 session 的 Plan 編輯分頁

#### Scenario: session 有任務資料

- **WHEN** `read_session_todos` 回傳至少一筆 todo
- **THEN** 摘要 badge 區顯示可點擊的任務總數 badge，格式至少包含總任務數

#### Scenario: plan 與任務入口共用同一區塊

- **WHEN** session 同時有 plan 與任務資料
- **THEN** Plan badge 與任務相關 badge SHALL 出現在同一摘要 badge 區，不分散於 header chip 與 action button 的不同區域

### Requirement: Session 卡片摘要 badge 區顯示任務狀態統計

系統 SHALL 在 session 有任務資料時，於摘要 badge 區顯示任務總數與各狀態數量，僅顯示 count 大於 0 的狀態 badge。

#### Scenario: 顯示內建狀態統計

- **WHEN** 任務包含 `done`、`pending`、`in_progress`、`blocked` 任一狀態，且其數量大於 0
- **THEN** UI 顯示對應狀態 badge 與數量

#### Scenario: 顯示未知狀態統計

- **WHEN** 任務包含非預期但非空字串的其他狀態，且其數量大於 0
- **THEN** UI 仍顯示該狀態的 badge 與數量，不忽略該資料

#### Scenario: 狀態數量為 0 時不顯示

- **WHEN** 某個任務狀態的數量為 0
- **THEN** UI 不顯示該狀態 badge

### Requirement: 任務摘要 badge 可開啟任務子分頁

系統 SHALL 允許使用者從 session 卡片的任務總數 badge 或任務狀態 badge 開啟該 session 的任務子分頁，行為與既有 Plan 子分頁一致。

#### Scenario: 點擊任務總數 badge

- **WHEN** 使用者點擊任務總數 badge
- **THEN** ProjectView 開啟該 session 的 todos 子分頁，顯示完整任務清單

#### Scenario: 點擊任務狀態 badge

- **WHEN** 使用者點擊任務狀態 badge
- **THEN** ProjectView 仍開啟同一個該 session 的 todos 子分頁，而不是建立不同狀態各自一個分頁

#### Scenario: 已開啟任務分頁時再次點擊 badge

- **WHEN** 該 session 的 todos 子分頁已開啟
- **THEN** 系統切換到既有分頁，不重複新增相同 session 的 todos 分頁

### Requirement: todos 僅對 Copilot provider 啟用

系統 SHALL 只對 `provider === "copilot"` 的 session 讀取及顯示 todos，OpenCode sessions 不顯示此區塊。

#### Scenario: OpenCode session 不顯示 todos

- **WHEN** 選取的 session 的 `provider === "opencode"`
- **THEN** UI 不呈現 todos 區塊、任務摘要 badge 或 todos 子分頁入口，也不呼叫 `read_session_todos`
