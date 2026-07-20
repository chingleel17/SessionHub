## Requirements

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
local_tokens:  Option<LocalTokenUsage>   — 本地掃描的 token 累計（opencode / codex 適用）
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

`LocalTokenUsage` 欄位：

```
input_tokens:  u64
output_tokens: u64
period_label:  String  — 例如 "本月" / "本週"
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

### Requirement: quota 顯示範圍跟隨 enabledProviders 設定

系統 SHALL 只為 `AppSettings.enabledProviders` 中包含的 provider 查詢並顯示 quota snapshot。未勾選的 provider 不查詢、不在 UI 中出現。

#### Scenario: 使用者調整 enabledProviders

- **WHEN** 使用者在設定頁新增或移除某個 provider
- **THEN** 下次 quota refresh 時只查詢目前 enabledProviders 中的 provider
- **AND** 已移除 provider 的 quota snapshot 從記憶體快取與 SQLite 中清除

### Requirement: 各 provider 的 quota 資料來源規格

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

### Requirement: Claude adapter 解析 limits 陣列中的 scoped-model 每週視窗

Claude quota adapter SHALL 在解析頂層時間視窗後，額外掃描 usage API 回應的 `limits` 陣列，為每個 `group == "weekly"` 且 `scope.model.display_name` 非空的項目產生一個 `QuotaWindow`：`window_key` 為 `seven_day_<display_name 小寫>`、`label` 為該 `display_name`、`utilization` 為 `percent / 100`、`resets_at` 為該項目自身的 `resets_at`。此解析 SHALL 為資料驅動，不硬編特定模型名稱；若解析出的 `window_key` 與已產生的頂層視窗重複，SHALL 略過以避免重複計入。

`QuotaWindow.window_key` 的合法值 SHALL 擴充為包含 `seven_day_fable` 及一般化的 `seven_day_<model>`（依 API 提供的 scoped model 動態產生）。

#### Scenario: 解析 Fable scoped 週視窗

- **WHEN** usage API 的 `limits` 含一項 `group: "weekly"`、`percent: 100`、`scope.model.display_name: "Fable"`、`resets_at: "2026-07-14T16:00:00Z"`
- **THEN** snapshot 的 `windows` 含一個 `window_key: "seven_day_fable"`、`label: "Fable"`、`utilization: 1.0`、`resets_at` 為該項目自身值的視窗

#### Scenario: 不同 scoped 模型自動產生視窗

- **WHEN** `limits` 含某個 `scope.model.display_name` 為 SessionHub 未預先對映的模型
- **THEN** 系統仍為其產生 `seven_day_<model 小寫>` 視窗
- **AND** 前端在無本地化對映時退回顯示該模型的 `display_name`

#### Scenario: scoped reset 時間獨立於週視窗

- **WHEN** 某 scoped 週視窗的 `resets_at` 與 `weekly_all`（seven_day）不同
- **THEN** 該 scoped 視窗顯示自己的 `resets_at`，不套用週視窗的 reset 時間

#### Scenario: 與頂層視窗去重

- **WHEN** 某 scoped model 解析出的 `window_key` 已由頂層視窗解析產生
- **THEN** 系統略過該 `limits` 項目，`windows` 中不出現重複 `window_key`

#### Scenario: 無 scoped 週視窗

- **WHEN** `limits` 缺席，或其中沒有任何 `scope.model.display_name` 非空的每週項目
- **THEN** `windows` 僅含既有頂層視窗，不新增 scoped 視窗

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

### Requirement: Codex adapter 查詢手動重置額度

Codex quota adapter SHALL 在成功取得 usage 資料後，以相同憑證（access_token 與選填的 account_id）呼叫 `GET https://chatgpt.com/backend-api/wham/rate-limit-reset-credits`，將回應解析為 `reset_credits` 欄位（`available_count` 與 `credits[]` 各筆的 granted_at / expires_at / status），時間戳統一轉為 ISO 8601 字串。此查詢 SHALL 為 best-effort：任何失敗不得改變 snapshot 的 `status` 與既有欄位。

#### Scenario: 成功取得重置額度

- **WHEN** reset-credits API 回傳 200 且內容可解析
- **THEN** snapshot 的 `reset_credits.available_count` 為 API 的可用次數
- **AND** `reset_credits.credits` 逐筆包含 granted_at / expires_at（ISO 8601）與 status

#### Scenario: reset-credits 查詢失敗不影響主 snapshot

- **WHEN** usage API 成功但 reset-credits API 回傳錯誤（4xx / 5xx / 網路失敗 / 解析失敗）
- **THEN** snapshot 維持 `status: "ok"`，windows 與 local_tokens 照常填入
- **AND** `reset_credits` 為 null，不寫入 `error_message`

#### Scenario: 帳號無重置額度功能

- **WHEN** reset-credits API 回傳空的 credits 清單或表示無此功能（如 404）
- **THEN** `reset_credits` 為 null 或 `available_count: 0` 且 `credits` 為空
- **AND** 前端據此不渲染重置額度區塊

### Requirement: Dashboard Codex 面板顯示重置額度

Dashboard 的 QuotaOverview Codex 面板 SHALL 在 snapshot 含 `reset_credits` 時顯示重置額度區塊：可用次數、各筆額度的狀態與到期倒數（含絕對時間）；已過期條目以低對比樣式呈現。無 `reset_credits` 時不渲染該區塊。

#### Scenario: 顯示可用重置額度

- **WHEN** Codex snapshot 的 `reset_credits.available_count` 為 2 且含兩筆 active 額度
- **THEN** Codex 面板顯示「可用 2 次」與兩筆額度各自的狀態與到期倒數（例如「6 天 23 小時後到期 · 07/21 下午11:59」）

#### Scenario: 無重置額度資料

- **WHEN** Codex snapshot 的 `reset_credits` 為 null
- **THEN** Codex 面板不顯示重置額度區塊，其餘內容照常

### Requirement: quota monitoring 以內建 adapter 為主，保留 Rust trait 擴充點

系統 SHALL 透過 `QuotaAdapter` Rust trait 統一各 provider 的查詢介面，首版全部為內建實作。

```rust
trait QuotaAdapter {
    fn provider_key(&self) -> &str;
    fn fetch_snapshot(&self, settings: &AppSettings) -> QuotaSnapshot;
}
```

#### Scenario: 使用內建 adapter

- **WHEN** quota manager 調度某個 provider
- **THEN** 直接呼叫對應的內建 adapter struct 實作 `fetch_snapshot()`

### Requirement: quota 資料支援背景 refresh 與手動 refresh

系統 SHALL 支援應用啟動時刷新、背景輪詢刷新（依 `quota_refresh_interval`）與使用者手動刷新。

#### Scenario: 應用啟動時刷新

- **WHEN** SessionHub 啟動且 `enable_quota_monitoring: true`
- **THEN** 系統先從 SQLite 載入上次快照（立即顯示），再非同步執行一次完整 quota refresh

#### Scenario: 背景輪詢刷新

- **WHEN** 距上次 refresh 超過 `quota_refresh_interval` 分鐘
- **THEN** 系統自動對所有 enabledProviders 執行 refresh
- **AND** 成功結果同時寫入記憶體快取與 SQLite

#### Scenario: 使用者手動刷新

- **WHEN** 使用者在 Dashboard 或 Settings 點擊 quota refresh
- **THEN** 系統對所有 enabledProviders（或指定單一 provider）執行 refresh
- **AND** 更新最新 snapshot 與最後刷新時間

### Requirement: bridge 事件可觸發節流 quota refresh

系統 SHALL 能在收到 provider bridge 事件後，以節流方式觸發對應 provider 的 quota refresh。

#### Scenario: provider bridge 事件後刷新

- **WHEN** 系統收到新的 provider bridge event（任何 event type）
- **THEN** 系統為對應 provider 排程一次 quota refresh
- **AND** 同一個 provider 在 30 秒內的重複事件不重複觸發 refresh（debounce）

### Requirement: quota snapshot 雙層快取

系統 SHALL 維護記憶體快取（`Mutex<HashMap<String, QuotaSnapshot>>`）與 SQLite 持久化快取（`quota_snapshots` table）。

#### Scenario: 快取讀取優先順序

- **WHEN** 前端請求 quota snapshots
- **THEN** 後端直接回傳記憶體快取（不重新查詢）

#### Scenario: 跨重啟持久化

- **WHEN** app 啟動時記憶體快取為空
- **THEN** 系統從 `quota_snapshots` SQLite table 載入上次快照
- **AND** 前端可立即看到上次的 quota 資料（帶 `fetched_at` 時間戳）

### Requirement: quota 查詢失敗時保留診斷資訊

系統 SHALL 在 quota 查詢失敗時保留錯誤狀態與失敗訊息，供 UI 顯示診斷。

#### Scenario: quota source 查詢失敗

- **WHEN** 某個 provider quota source 因 auth、網路或格式錯誤而失敗
- **THEN** 系統回傳包含 `status: "error"` 與 `error_message` 的 snapshot
- **AND** 不得阻斷其他 provider 的 quota 結果
- **AND** SQLite 中保留上次成功的快照（失敗時不覆蓋成功記錄）
