## ADDED Requirements

### Requirement: OpenCode JSON storage 根目錄結構

本規格記錄 OpenCode JSON storage 的完整根目錄結構，提供解析實作的參考。

#### Scenario: 根目錄子結構

- `session/<projectID>/ses_*.json` — session metadata
- `message/<sessionID>/msg_*.json` — 訊息記錄
- `part/<messageID>/prt_*.json` — 訊息內容塊
- `project/*.json` — 專案 metadata（以 SHA256 hex 為檔名）
- `session_diff/` — session 差異記錄
- `migration` — schema 版本號文件（純文字）

### Requirement: 實體欄位定義

#### Scenario: Session 實體（ses\_\*.json）

| 欄位           | 型別   | 說明                         |
| -------------- | ------ | ---------------------------- |
| `id`           | string | `ses_` 前綴                  |
| `projectID`    | string | SHA256 hex（64 字元）        |
| `directory`    | string | 工作目錄絕對路徑             |
| `title`        | string | session 標題（對應 summary） |
| `time.created` | number | Unix timestamp 毫秒          |
| `time.updated` | number | Unix timestamp 毫秒          |

#### Scenario: Message 實體（msg\_\*.json）

| 欄位               | 型別                    | 說明                |
| ------------------ | ----------------------- | ------------------- |
| `id`               | string                  | `msg_` 前綴         |
| `sessionID`        | string                  | 關聯 session        |
| `role`             | `"user" \| "assistant"` | 訊息角色            |
| `modelID`          | string                  | 使用的模型 ID       |
| `tokens.input`     | number                  | 輸入 token 數       |
| `tokens.output`    | number                  | 輸出 token 數       |
| `tokens.reasoning` | number                  | 推理 token 數       |
| `time.created`     | number                  | Unix timestamp 毫秒 |

#### Scenario: Part 實體（prt\_\*.json）

| type          | 關鍵欄位                                                            |
| ------------- | ------------------------------------------------------------------- |
| `text`        | `text: string`                                                      |
| `tool`        | `state.tool: string`、`state.status`、`state.input`、`state.output` |
| `step-start`  | `snapshot: string`（40 char hex）                                   |
| `step-finish` | `reason`、`tokens.*`、`cost`                                        |

### Requirement: 時間戳格式

所有 `time.*` 欄位 SHALL 為 Unix timestamp 毫秒（讀取時需除以 1000 轉為秒）。
