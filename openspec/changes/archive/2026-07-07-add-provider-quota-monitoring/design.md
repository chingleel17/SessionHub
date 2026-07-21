## Context

SessionHub 目前已經有三種與「即時狀態」相關的能力：session 掃描、provider bridge、analytics 聚合。但這些資料都還停留在 session 與 token 使用結果層，沒有一個統一的 quota subsystem 來回答「某個 provider 還剩多少額度、何時 reset、資料來源是否可信」。

目標是把各平台訂閱方案的用量與剩餘額度直接整合進 SessionHub，讓使用者不需要切回 Claude Code、GitHub、Copilot CLI 等各自的後台，就能在同一個畫面一目了然。這個需求的參考方向來自 `ccusage`（開源 Python 工具，逆向工程了 Anthropic OAuth usage API）以及 Claude Code 的 `statusline-command.sh`（`~/.claude/statusline-command.sh`，直接揭示了 `rate_limits.five_hour.used_percentage` 的 JSON 格式）。

這個需求還有一個額外維度：設計上不能把 quota 讀取邏輯直接塞進單一 command，而要先定義 adapter 介面，再由 SessionHub 內建實作各 provider，未來保留擴充空間。

## Goals / Non-Goals

**Goals:**

- 在 SessionHub 內提供統一的 provider quota snapshot，至少能表達 provider、狀態、來源、使用量、剩餘量與 reset 時間。
- 以內建 quota adapters 為主路徑，讓常見 provider 可直接被 SessionHub 查詢。
- 保留插件式 connector 擴充點，允許未來新增 provider-specific quota 來源，而不必重寫核心 UI。
- 提供背景 refresh、手動 refresh，以及 bridge 事件觸發的節流更新能力。
- 在 Dashboard、Settings 與 status bar 顯示 quota 摘要與診斷資訊。

**Non-Goals:**

- 本次不要求一次支援所有平台；首版僅支援 `enabledProviders` 中勾選的 provider（copilot / opencode / codex / claude）。
- 本次不直接嵌入或依賴 `opencode-quota` 的 OpenCode plugin runtime。
- 本次不要求 quota 資料一定由 hook 直接提供；hook/bridge 只作為 refresh trigger，不作為唯一資料來源。
- 本次不實作複雜的多租戶雲端同步或伺服器端配額管理。
- 本次不在 UI 中暴露或讓使用者輸入任何 auth token；所有 token 從本地檔案讀取。

## Decisions

### 1. 採用「quota manager + provider adapter」的內建架構

SessionHub 後端新增 quota manager，統一調度多個 provider adapter。每個 adapter 回傳標準化 quota snapshot，而不是讓前端直接理解不同平台的原始 payload。

原因：這與現有 session/provider 的模組化方向一致，也能把 provider-specific 的 auth、API、fallback 估算收斂在後端。

替代方案：

- 前端直接各自呼叫不同 quota API。缺點是 auth 與錯誤處理會分散在 `App.tsx`。
- 直接依賴 `opencode-quota` 套件做全部資料來源。缺點是會被 OpenCode plugin lifecycle 與其內部模組邊界綁住。

### 2. 首版顯示範圍：跟隨 `enabledProviders` 設定

quota 顯示的 provider 集合直接以 `AppSettings.enabledProviders` 為準（`copilot` / `opencode` / `codex` / `claude`），動態來源，不在程式碼中硬寫死 provider 清單。provider 未勾選時不查詢也不顯示其 quota。

原因：這讓功能與現有設定體系一致，使用者已在設定頁管理哪些平台啟用，quota 監控跟隨即可。

### 3. 各 provider 的 quota 來源（已確認可行性）

| Provider | quota 來源 | 資料型態 | auth 取得方式 |
|----------|-----------|---------|--------------|
| **claude** | Anthropic OAuth usage API：`GET https://api.anthropic.com/api/oauth/usage`，需 header `anthropic-beta: oauth-2025-04-20` | rolling window utilization（5h / 7day / 7day-sonnet / 7day-opus）+ reset 時間；extra_usage credit 餘額 | OAuth access token 從 `~/.claude/.credentials.json` 讀取（與 `ccusage` 開源工具相同做法） |
| **copilot** | GitHub billing API：`GET /users/{username}/settings/billing/ai_credit/usage` | AI credits 已用 / 剩餘 / reset 時間 | `gh auth token` spawn 取得 token；username 從 `gh api user` 取得 |
| **opencode** | 本地 transcript 掃描（`{opencodeRoot}/sessions/`），無遠端 quota API | token 已用量；無剩餘額度資料 | 無需 auth；標示 `source: local` |
| **codex** | 本地 JSONL 掃描（`{codexRoot}/`），無對應訂閱 quota API | token 已用量；無剩餘額度資料 | 無需 auth；標示 `source: local` |

quota snapshot 中 `source` 欄位明確標示 `remote_api` 或 `local_scan`，讓 UI 能向使用者說明資料性質的差異。

### 4. quota source 與 session platform 分開建模

SessionHub 的 session platform（copilot / opencode / codex / claude）與其背後的 quota provider 一一對應（本版）。雖然理論上 OpenCode 可能對接多個模型，但首版以 platform key 直接作為 quota snapshot 的 provider key，避免過度設計。未來若需多對多對應可在 adapter 層擴充。

### 5. 更新策略採「背景輪詢為主，bridge trigger 為輔」

quota refresh 主要由 app startup、固定輪詢（`quota_refresh_interval` 分鐘）、手動 refresh 驅動；收到 provider bridge 事件時，作為節流信號觸發單一 provider refresh（debounce 30 秒，避免同一個 provider 連發）。

原因：quota 通常需要查本地 auth 或遠端 API，bridge event 無法可靠攜帶完整額度資訊；但它很適合提示「剛剛有 activity，值得 refresh」。

### 6. quota snapshot 快取策略：記憶體 + SQLite 雙層

- **記憶體層**（`Mutex<HashMap<String, QuotaSnapshot>>`）：app 執行期間直接讀寫，響應速度快，不需序列化。
- **SQLite 落地**（新增 `quota_snapshots` table）：每次 refresh 成功後寫入；app 啟動時從 DB 載入作為初始快照，避免首次顯示為空白。

兩層快取並存：記憶體層是主要讀取來源，DB 層是跨重啟持久化。

注意：現有 `provider_quota` table 是「本地計算的累計 token 用量」，新的 `quota_snapshots` table 是「從訂閱服務查回的剩餘額度快照」，語意不同，兩者並存不衝突。

### 7. 外部 connector 擴充點：Rust trait 邊界，首版全部內建

定義 `QuotaAdapter` trait（`fetch_snapshot() -> Result<QuotaSnapshot, String>`），每個 provider 一個 struct 實作。首版全部內建（`ClaudeAdapter`、`CopilotAdapter`、`OpenCodeAdapter`、`CodexAdapter`），不提供外部 plugin 載入機制。trait 邊界明確後，未來若需外部 connector 可在不改動 UI 與 manager 的情況下新增。

## Risks / Trade-offs

- [Claude OAuth token 位置不穩定] -> 讀取 `~/.claude/.credentials.json`，解析失敗時回傳 `error` 狀態 snapshot，不 crash；參考 `ccusage` 的 token 讀取實作。
- [GitHub `gh` CLI 不一定安裝] -> `gh auth token` 失敗時回傳 `unsupported` 狀態，在 UI 標示「需安裝 gh CLI」。
- [遠端 API rate limit] -> 有記憶體 + SQLite 雙層快取，加上 refresh interval 控制，不會過度查詢。
- [opencode / codex 無遠端 quota] -> 顯示本地掃描的已用量，並在 UI 標示 `本地估算` 而非真實剩餘額度。
- [Dashboard 與 status bar 容易資訊過載] -> Dashboard 顯示詳細 overview，status bar 僅顯示精簡摘要（provider 名稱 + utilization %）。

## Migration Plan

1. 新增 `QuotaSnapshot` 型別、`quota_snapshots` SQLite table、記憶體快取結構。
2. 實作四個內建 adapter（claude / copilot / opencode / codex）。
3. 新增 `get_quota_snapshots` / `refresh_quota` Tauri commands；接入 app startup 與背景輪詢。
4. 擴充 `AppSettings` 加入 `enable_quota_monitoring` / `quota_refresh_interval`；更新 Settings UI。
5. 在 Dashboard 加入 quota overview 區塊；在 global status bar 加入精簡 quota 摘要。
6. 補 bridge-triggered refresh（debounce 30 秒）。

## Resolved Decisions

**OQ-1（首版 provider 範圍）**：顯示 `enabledProviders` 中勾選的所有 provider，不硬寫死。

**OQ-2（快取策略）**：記憶體 + SQLite 雙層快取，記憶體為主要讀取，SQLite 提供跨重啟持久化。

**OQ-3（connector 機制）**：純 Rust trait 邊界，首版全部內建 adapter；各 provider 依其可取得的資料型態採用最適合的機制（Claude → OAuth API；Copilot → GitHub billing API via `gh` CLI；OpenCode / Codex → 本地掃描）。
