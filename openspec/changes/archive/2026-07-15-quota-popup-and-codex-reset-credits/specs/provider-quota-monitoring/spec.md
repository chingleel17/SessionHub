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
local_tokens:  Option<LocalTokenUsage>   — 本地掃描的 token 累計（opencode / codex 適用）
extra_credits: Option<ExtraCredits>      — overage / 超額用量（claude extra_usage 適用）
reset_credits: Option<ResetCredits>      — 手動重置額度（codex 適用）
```

`QuotaWindow` 欄位：

```
window_key:   String  — "five_hour" | "seven_day" | "seven_day_sonnet" | "seven_day_opus" | "ai_credits"
label:        String  — 顯示用名稱（"5h" / "7d" / "7d Sonnet" / "7d Opus" / "AI Credits"）
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

## ADDED Requirements

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
