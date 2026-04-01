## ADDED Requirements

### Requirement: OpenCode JSON 儲存根目錄結構定義
系統文件 SHALL 記錄 OpenCode JSON storage 的完整根目錄結構，包含各子目錄的用途、檔案命名規則與檔案數量級。

#### Scenario: 文件描述根目錄
- **WHEN** 開發者查閱 opencode-storage-schema 規格
- **THEN** 規格 SHALL 包含以下子目錄定義：
  - `session/`：session metadata，按 `{projectID}/ses_*.json` 組織
  - `message/`：訊息記錄，按 `{sessionID}/msg_*.json` 組織
  - `part/`：訊息內容塊，按 `{messageID}/prt_*.json` 組織
  - `project/`：專案 metadata，扁平化 JSON 檔案
  - `session_diff/`：session 差異記錄
  - `directory-readme/`：目錄說明注入記錄
  - `directory-agents/`：agent 目錄配置記錄
  - `agent-usage-reminder/`：agent 使用追蹤記錄
  - `rules-injector/`：規則注入配置
  - `todo/`：任務清單
  - `migration`：schema 版本號文件（純文字）

### Requirement: 各實體 JSON Schema 定義
系統文件 SHALL 記錄 session、message、part（含 4 種子類型）、project 的完整欄位定義，包含欄位名稱、型別與說明。

#### Scenario: Session 實體欄位
- **WHEN** 開發者查閱 session 實體定義
- **THEN** 規格 SHALL 定義以下欄位：
  - `id: string` — 格式 `ses_{alphanumeric}`
  - `slug: string` — 人類可讀 URL slug
  - `version: string` — 系統版本號
  - `projectID: string` — 關聯 project（SHA256 hex）
  - `parentID: string | null` — 父 session ID
  - `directory: string` — 工作目錄絕對路徑
  - `title: string` — session 標題
  - `time.created: number` — Unix timestamp（毫秒）
  - `time.updated: number` — Unix timestamp（毫秒）
  - `permission: array` — 細粒度權限規則
  - `summary.additions: number` — 程式碼新增行數
  - `summary.deletions: number` — 程式碼刪除行數
  - `summary.files: number` — 異動檔案數

#### Scenario: Message 實體欄位
- **WHEN** 開發者查閱 message 實體定義
- **THEN** 規格 SHALL 定義以下欄位：
  - `id: string` — 格式 `msg_{alphanumeric}`
  - `sessionID: string` — 關聯 session
  - `role: "user" | "assistant"` — 訊息角色
  - `time.created: number` — Unix timestamp（毫秒）
  - `time.completed: number | null` — 完成時間（assistant only）
  - `parentID: string | null` — 前一個 message（對話鏈）
  - `modelID: string` — 使用的模型 ID
  - `providerID: string` — AI provider ID
  - `mode: string` — agent 名稱
  - `tokens.total: number` — 總 token 數
  - `tokens.input: number` — 輸入 token 數
  - `tokens.output: number` — 輸出 token 數
  - `tokens.reasoning: number` — 推理 token 數
  - `tokens.cache.read: number` — 快取讀取 token 數
  - `tokens.cache.write: number` — 快取寫入 token 數
  - `cost: number` — 費用（通常為 0）
  - `finish: "stop" | "tool-calls" | "max_tokens"` — 完成原因

#### Scenario: Part 實體 - text 類型
- **WHEN** 開發者查閱 part text 子類型定義
- **THEN** 規格 SHALL 定義：
  - `id: string` — 格式 `prt_{alphanumeric}`
  - `messageID: string` — 關聯 message
  - `sessionID: string` — 關聯 session
  - `type: "text"` — 固定值
  - `text: string` — 文字內容
  - `synthetic: boolean` — 是否為 AI 生成內容
  - `time.start: number` / `time.end: number` — Unix timestamp（毫秒）

#### Scenario: Part 實體 - tool 類型
- **WHEN** 開發者查閱 part tool 子類型定義
- **THEN** 規格 SHALL 定義：
  - `type: "tool"` — 固定值
  - `callID: string` — 工具呼叫 ID
  - `tool: string` — 工具名稱（如 `bash`、`glob`、`task`）
  - `state.status: "completed" | "running" | "failed"` — 執行狀態
  - `state.input: object` — 工具輸入參數
  - `state.output: string` — 工具輸出結果
  - `state.title: string` — 工具呼叫標題

#### Scenario: Part 實體 - step-start / step-finish 類型
- **WHEN** 開發者查閱 part step 子類型定義
- **THEN** 規格 SHALL 定義 step-start：
  - `type: "step-start"` — 固定值
  - `snapshot: string` — 40 char hex 狀態快照 hash
- **AND** step-finish：
  - `type: "step-finish"` — 固定值
  - `reason: string` — 步驟結束原因
  - `snapshot: string` — 狀態快照 hash
  - `tokens.input/output/reasoning/cache` — per-step token 統計
  - `cost: number` — 步驟費用

### Requirement: 實體關聯圖定義
系統文件 SHALL 記錄各實體之間的關聯方式與方向。

#### Scenario: 主要關聯鏈
- **WHEN** 開發者查閱實體關聯
- **THEN** 規格 SHALL 描述以下關聯：
  - `project.id` ← `session.projectID`（一對多）
  - `session.id` ← `message.sessionID`（一對多）
  - `message.id` ← `part.messageID`（一對多）
  - `message.parentID` → `message.id`（對話鏈，自我關聯）
  - `session.id` ← `directory-agents.sessionID`（一對一）
  - `session.id` ← `agent-usage-reminder.sessionID`（一對一）
  - `session.id` ← `directory-readme.sessionID`（一對一）

### Requirement: ID 格式規範定義
系統文件 SHALL 記錄各類實體的 ID 生成格式與前綴規則。

#### Scenario: ID 格式一覽
- **WHEN** 開發者查閱 ID 格式
- **THEN** 規格 SHALL 定義：
  - `session.id`：前綴 `ses_`，後接 alphanumeric（約 26 字元總長）
  - `message.id`：前綴 `msg_`，後接 alphanumeric（約 26 字元總長）
  - `part.id`：前綴 `prt_`，後接 alphanumeric（約 22 字元總長）
  - `project.id`：SHA256 hex（64 字元，無前綴）
  - `snapshot`（step）：40 char hex（git-like hash）

### Requirement: 時間戳格式規範定義
系統文件 SHALL 記錄所有時間欄位的格式。

#### Scenario: 時間戳為毫秒 Unix timestamp
- **WHEN** 開發者查閱時間欄位
- **THEN** 規格 SHALL 明確說明所有 `time.*` 欄位為 Unix timestamp，單位為**毫秒**（非秒）
- **AND** 規格 SHALL 說明讀取時需除以 1000 才能轉換為秒級 Unix time
