## MODIFIED Requirements

### Requirement: 各 provider 的 quota 資料來源規格

系統 SHALL 為每個 provider 定義其 quota 資料來源與解析方式，將 provider-specific 的原始回應正規化為統一的 quota snapshot。

#### Scenario: Claude adapter - Anthropic OAuth usage API

- **WHEN** `claude` 在 enabledProviders 中且 `~/.claude/.credentials.json` 存在並含有效 OAuth token
- **THEN** 後端呼叫 `GET https://api.anthropic.com/api/oauth/usage`，帶 header `anthropic-beta: oauth-2025-04-20`
- **AND** 回傳 `five_hour`、`seven_day`、`seven_day_sonnet`、`seven_day_opus`（null 的窗口略過）
- **AND** 回傳 `extra_usage`（若 `is_enabled: true`）
- **AND** source 標示為 `remote_api`

- **WHEN** `.credentials.json` 不存在或 token 讀取失敗
- **THEN** 回傳 `status: "no_auth"`，`error_message: "Claude OAuth token 不可讀，請確認 Claude Code 已登入"`

#### Scenario: Copilot adapter - GitHub billing API

- **WHEN** `copilot` 在 enabledProviders 中且 `gh` CLI 可用（spawn `gh auth token` 成功）
- **THEN** 後端以取得的 token 呼叫 `GET https://api.github.com/users/{username}/settings/billing/ai_credit/usage`
- **AND** 回傳 `ai_credits` window（已用量、剩餘量、reset 時間）
- **AND** source 標示為 `remote_api`

- **WHEN** `gh` CLI 不存在或 `gh auth token` 失敗
- **THEN** 回傳 `status: "no_auth"`，`error_message: "需要安裝並登入 gh CLI"`

#### Scenario: OpenCode adapter - 本地掃描

- **WHEN** `opencode` 在 enabledProviders 中
- **THEN** 後端掃描 `{opencodeRoot}/sessions/` 計算本月 token 用量
- **AND** 回傳 `local_tokens` 欄位（input / output tokens）
- **AND** source 標示為 `local_scan`
- **AND** `windows` 欄位為 null（無遠端 quota 資料）

#### Scenario: Codex adapter - 遠端 usage API 取得 rate limit 窗口

- **WHEN** `codex` 在 enabledProviders 中且 `{codexRoot}/auth.json`（或 `$CODEX_HOME/auth.json`、`~/.codex/auth.json`）存在並含有效 access token
- **THEN** 後端呼叫 `GET https://chatgpt.com/backend-api/wham/usage`，帶 header `Authorization: Bearer <token>`（若有 account_id 則附 `ChatGPT-Account-Id`）
- **AND** 由 `rate_limit.primary_window` 與 `rate_limit.secondary_window` 解析出 rolling window 用量
- **AND** 額外掃描 `{codexRoot}/` 下的 JSONL 計算本月 token 用量，回傳於 `local_tokens`
- **AND** source 標示為 `remote_api`

- **WHEN** auth.json 不存在或 token 讀取失敗
- **THEN** 回傳 `status: "no_auth"`，`error_message` 說明需重新登入 Codex CLI

#### Scenario: Codex adapter - 依窗口時長分類 rate limit 窗口

- **WHEN** 解析某個非 null 的 `rate_limit` window 物件（`primary_window` 或 `secondary_window`）
- **THEN** 系統依該物件的 `limit_window_seconds` 欄位決定窗口類型，而非依欄位名（primary/secondary）
- **AND** 時長約 5 小時（18000 秒附近）→ `window_key: "five_hour"`、`label: "5h"`
- **AND** 時長約 7 天（604800 秒附近）→ `window_key: "seven_day"`、`label: "7d"`
- **AND** 其他時長 → 以實際時長動態產生標籤（例如 30 天 → `label: "30d"`），不硬套 5h/7d
- **AND** 若該物件缺 `limit_window_seconds`，退回以 `reset_after_seconds`（或 `reset_at - now`）推估時長後套用相同分類

#### Scenario: Codex adapter - 官方移除 5h 限制後只剩單一窗口

- **WHEN** usage API 回傳 `primary_window` 為長期限制窗口且 `secondary_window` 為 `null`
- **THEN** 系統依 `primary_window.limit_window_seconds` 的真實時長標示該窗口（例如 30 天 → `30d`）
- **AND** 不再把該窗口錯標為「5h」
- **AND** 不為缺席的 `secondary_window` 產生任何窗口

#### Scenario: Codex adapter - 無任何 rate limit 窗口

- **WHEN** `rate_limit` 缺席，或 `primary_window` 與 `secondary_window` 皆為 `null`
- **THEN** `windows` 欄位為 null（前端顯示無 rate limit 資料）
- **AND** 仍可回傳本月 `local_tokens`（若掃描到用量）
