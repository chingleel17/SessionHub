## MODIFIED Requirements

### Requirement: 系統提供統一的 provider quota snapshot

系統 SHALL 以統一的 quota snapshot 模型表示各 quota provider 的可用額度資訊，欄位定義如下：

```
provider:      String          — provider key（"claude" / "copilot" / "opencode" / "codex"）
status:        String          — "ok" | "error" | "unsupported" | "no_auth"
source:        String          — "remote_api" | "local_scan"
fetched_at:    String          — ISO 8601 時間戳（最後成功或嘗試的時間）
error_message: Option<String>  — 查詢失敗時的錯誤描述

// 以下欄位依 provider 與查詢結果填入，不可取得時為 null
windows:       Option<Vec<QuotaWindow>>  — rolling window 用量（claude / copilot 適用）
extra_credits: Option<ExtraCredits>      — overage / 超額用量（claude extra_usage 適用）
reset_credits: Option<ResetCredits>      — 手動重置額度（codex 適用）
```

`QuotaWindow` 欄位：

```
window_key:   String  — "five_hour" | "seven_day" | "seven_day_sonnet" | "seven_day_opus" | "seven_day_fable" | "ai_credits" | 一般化的 "seven_day_<model>"（依 API 提供的 scoped model 動態產生）
label:        String  — 顯示用名稱（"5h" / "7d" / "7d Sonnet" / "7d Opus" / "AI Credits" / 動態 scoped model 名稱，例如 "Fable"）
utilization:  f64     — 使用百分比（0.0–100.0）
resets_at:    Option<String>  — ISO 8601 reset 時間
```

`ExtraCredits` 欄位：

```
is_enabled:    bool
monthly_limit: Option<u64>   — 單位：credits
used_credits:  f64
utilization:   Option<f64>   — 百分比；null 表示無法計算
```

`ResetCredits` 欄位：

```
available_count: u32                     — 可用重置次數
credits:         Vec<ResetCreditEntry>   — 各筆額度明細
```

`ResetCreditEntry` 欄位：

```
granted_at: Option<String>  — ISO 8601 獲得時間
expires_at: Option<String>  — ISO 8601 到期時間
status:     String          — API 原始狀態字串（如 "active"）
```

`source` 欄位僅標示資料取得方式（遠端 API 或本機掃描），與 snapshot 是否含 token 用量統計無關。

#### Scenario: 成功取得 quota snapshot

- **WHEN** 後端 quota manager 成功向某個 provider adapter 取得資料
- **THEN** 系統回傳 `status: "ok"` 的標準化 quota snapshot
- **AND** 前端不需要理解 provider-specific 原始格式

#### Scenario: provider 尚未支援

- **WHEN** 使用者啟用了某個平台，但 SessionHub 尚未支援其 quota provider
- **THEN** 系統回傳 `status: "unsupported"` 的 snapshot
- **AND** `error_message` 描述原因

#### Scenario: auth 無法取得

- **WHEN** auth token 讀取失敗（檔案不存在、格式錯誤、token 過期）
- **THEN** 系統回傳 `status: "no_auth"` 的 snapshot
- **AND** `error_message` 說明需要什麼 auth 來源（例如「需要 gh CLI 登入」）

#### Scenario: 舊快照缺少 reset_credits 欄位

- **WHEN** 從 SQLite 載入本欄位新增前序列化的 snapshot JSON
- **THEN** 反序列化成功，`reset_credits` 為 null
- **AND** 不產生錯誤或阻斷載入

#### Scenario: 舊快照含已移除的 local_tokens 欄位

- **WHEN** 從 SQLite 載入 `local_tokens` 欄位移除前序列化的 snapshot JSON
- **THEN** 反序列化成功，多餘的 `localTokens` 鍵被忽略
- **AND** 不產生錯誤或阻斷載入，不需要 DB migration

### Requirement: 各 provider 的 quota 資料來源規格

各 provider 的 quota adapter SHALL 依下列場景所定義的資料來源與解析規則取得 snapshot。所有 adapter SHALL NOT 掃描本機 session 檔案統計 token 用量作為額度資料。

#### Scenario: Claude adapter - Anthropic OAuth usage API

- **WHEN** `claude` 在 enabledProviders 中且 `~/.claude/.credentials.json` 存在並含有效 OAuth token
- **THEN** 後端呼叫 `GET https://api.anthropic.com/api/oauth/usage`，帶 header `anthropic-beta: oauth-2025-04-20`
- **AND** 回傳 `five_hour`、`seven_day`、`seven_day_sonnet`、`seven_day_opus`（null 的窗口略過）
- **AND** 回傳 `extra_usage`（若 `is_enabled: true`）
- **AND** source 標示為 `remote_api`

- **WHEN** `.credentials.json` 不存在或 token 讀取失敗
- **THEN** 回傳 `status: "no_auth"`，`error_message: "Claude OAuth token 不可讀，請確認 Claude Code 已登入"`

#### Scenario: Claude 視窗百分比正規化（含 <= 1% 邊界）

- **WHEN** usage API 回傳的視窗百分比欄位（頂層 `utilization` / `used_percentage`、`limits[].percent`）為 0–100 範圍
- **THEN** 系統一律將其除以 100 得到 `utilization`（0.0–1.0），不得以「值是否大於 1」啟發式判斷數值範圍
- **AND** 當實際用量 <= 1%（如 `utilization: 1` 代表 1%）時，解析結果為 `0.01`，不得誤判為比例值而顯示 100%

#### Scenario: Copilot adapter - GitHub billing API

- **WHEN** `copilot` 在 enabledProviders 中且 `gh` CLI 可用（spawn `gh auth token` 成功）
- **THEN** 後端以取得的 token 呼叫 `GET https://api.github.com/users/{username}/settings/billing/ai_credit/usage`
- **AND** 回傳 `ai_credits` window（已用量、剩餘量、reset 時間）
- **AND** source 標示為 `remote_api`

- **WHEN** `gh` CLI 不存在或 `gh auth token` 失敗
- **THEN** 回傳 `status: "no_auth"`，`error_message: "需要安裝並登入 gh CLI"`

#### Scenario: OpenCode adapter - 無遠端額度來源

- **WHEN** `opencode` 在 enabledProviders 中
- **THEN** 後端回傳 `status: "ok"`、`source: "local_scan"` 的 snapshot
- **AND** `windows` 欄位為 null（OpenCode 無帳號層級的額度 API）
- **AND** 系統不掃描本機 session 檔案統計 token 用量

#### Scenario: Codex adapter - 遠端 usage API 取得 rate limit 窗口

- **WHEN** `codex` 在 enabledProviders 中且 `{codexRoot}/auth.json`（或 `$CODEX_HOME/auth.json`、`~/.codex/auth.json`）存在並含有效 access token
- **THEN** 後端呼叫 `GET https://chatgpt.com/backend-api/wham/usage`，帶 header `Authorization: Bearer <token>`（若有 account_id 則附 `ChatGPT-Account-Id`）
- **AND** 由 `rate_limit.primary_window` 與 `rate_limit.secondary_window` 解析出 rolling window 用量
- **AND** source 標示為 `remote_api`
- **AND** 系統不掃描 `{codexRoot}/` 下的 JSONL 統計 token 用量

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

### Requirement: Codex adapter 查詢手動重置額度

Codex quota adapter SHALL 在成功取得 usage 資料後，以相同憑證（access_token 與選填的 account_id）呼叫 `GET https://chatgpt.com/backend-api/wham/rate-limit-reset-credits`，將回應解析為 `reset_credits` 欄位（`available_count` 與 `credits[]` 各筆的 granted_at / expires_at / status），時間戳統一轉為 ISO 8601 字串。此查詢 SHALL 為 best-effort：任何失敗不得改變 snapshot 的 `status` 與既有欄位。

#### Scenario: 成功取得重置額度

- **WHEN** reset-credits API 回傳 200 且內容可解析
- **THEN** snapshot 的 `reset_credits.available_count` 為 API 的可用次數
- **AND** `reset_credits.credits` 逐筆包含 granted_at / expires_at（ISO 8601）與 status

#### Scenario: reset-credits 查詢失敗不影響主 snapshot

- **WHEN** usage API 成功但 reset-credits API 回傳錯誤（4xx / 5xx / 網路失敗 / 解析失敗）
- **THEN** snapshot 維持 `status: "ok"`，windows 照常填入
- **AND** `reset_credits` 為 null，不寫入 `error_message`

#### Scenario: 帳號無重置額度功能

- **WHEN** reset-credits API 回傳空的 credits 清單或表示無此功能（如 404）
- **THEN** `reset_credits` 為 null 或 `available_count: 0` 且 `credits` 為空
- **AND** 前端據此不渲染重置額度區塊

## ADDED Requirements

### Requirement: provider 無可顯示額度資料時呈現說明文字

當某個 `status: "ok"` 的 provider snapshot 不含任何可渲染的額度內容（`windows` 為 null 或空陣列，且無 `extra_credits`、`reset_credits`）時，quota UI SHALL 保留該 provider 的區塊並顯示一行說明文字，告知此 provider 無可取得的額度資料。UI SHALL NOT 呈現空白區塊，亦 SHALL NOT 隱藏該 provider 使其看似未啟用。

說明文字 SHALL 透過翻譯鍵取得（zh-TW 與 en-US 皆需提供），不得硬編於 JSX。

#### Scenario: OpenCode 卡片顯示無額度資料

- **WHEN** Dashboard 的 QuotaOverview 渲染 OpenCode snapshot（`status: "ok"`、`windows: null`）
- **THEN** 卡片顯示 provider 名稱、來源徽章與一行「無額度資料」說明文字
- **AND** 不出現空白內容區

#### Scenario: Tray mini panel 顯示無額度資料

- **WHEN** tray mini panel 渲染不含任何額度內容的 provider snapshot
- **THEN** 該 provider 列顯示名稱與「無額度資料」說明文字

#### Scenario: Overlay 顯示無額度資料

- **WHEN** overlay widget 渲染不含任何額度內容的 provider snapshot
- **THEN** 該列顯示「無額度資料」說明文字，不渲染 utilization bar

#### Scenario: 有額度資料時不顯示說明文字

- **WHEN** provider snapshot 含至少一個 `QuotaWindow`
- **THEN** 三處 UI 皆照常渲染額度內容，不顯示「無額度資料」說明文字

## REMOVED Requirements

### Requirement: LocalTokenUsage 本機 token 用量統計

**Reason**: 該欄位呈現的是 SessionHub 自行掃描本機 session 檔推估的 token 數，與同一 UI 中來自 provider 官方 API 的實際額度百分比口徑不一致，易被誤讀為同源資料。功能僅 codex 與 opencode 兩家具備，無法跨供應商比較；且 Codex 的解析邏輯已因 rollout 檔案格式變更（token 移至 `payload.info.last_token_usage`）而長期失效，顯示恆為 `0k / 0k`。

**Migration**: 跨供應商的 token 用量統計改由 Analytics 功能提供（`get_analytics_data_internal` 已支援 day / week / month 分組與跨 provider 彙總）。`QuotaSnapshot.local_tokens` 欄位與 `LocalTokenUsage` 型別自 Rust 與 TypeScript 兩端移除；SQLite 既有快照 JSON 中的 `localTokens` 鍵由 serde 忽略，不需 DB migration。`source: "local_scan"` 標記保留不變，仍用於 opencode 與 antigravity 的資料來源徽章。
