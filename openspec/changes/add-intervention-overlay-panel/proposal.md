## Why

目前 session 卡在等待授權（`waiting`）時，唯一的提醒是 Windows Toast（`send_intervention_notification`）。Toast 會在數秒後自動消失並落入通知中心，使用者不在電腦前或未即時注意時就會錯過，導致背景 session 長時間卡著等授權卻無人知曉。需要一個常駐、不會自動消失的視覺提醒，直到該授權被處理為止。

## What Changes

- 後端新增 `InterventionRegistry`：以 `waiting` session 清單作為 single source of truth，於 activity 狀態進入／離開 `waiting` 時更新，並 emit `intervention-list-changed`（帶最小化的 `sessionId` / `projectName` / `toolLabel`）供主視窗與 quota overlay 視窗訂閱。
- 既有 `QuotaOverlay`（`quota-overlay` 視窗）內嵌「需授權」提醒區：訂閱 `intervention-list-changed` 自行 render，`full` 與 `compact` 兩種 styleMode 皆支援，清單為 0 筆時整區不顯示。
- 提醒區具備自動延伸方向：overlay 貼近工作列（螢幕可用區底緣）時，若往下延伸會被工作列遮擋，則改為往上延伸並維持 quota chip 的螢幕位置不動；overlay 貼近螢幕頂緣而上方無空間時 fallback 往下。
- 提醒區每筆卡片可點擊：聚焦主視窗並導航到對應 session，複用既有 `notification://action-performed` 的導航邏輯。
- overlay 提醒區受既有 `enable_intervention_notification` 設定控制（作為 waiting 介入提醒的總開關，同時管 Toast 與 overlay）：關閉時 Toast 與 overlay 提醒皆不顯示。不新增設定項。
- 提醒區僅顯示工具類型（如 `Bash` / `Read` / `Edit` / `Write`），沿用既有隱私原則，不顯示指令、檔案內容或完整路徑。

## Capabilities

### New Capabilities

- `intervention-registry`: 後端維護 `waiting` session 清單（single source of truth），於進入／離開 `waiting` 時更新並 emit `intervention-list-changed` 廣播事件，payload 僅含最小化欄位（sessionId、projectName、toolLabel）。
- `intervention-overlay-panel`: quota overlay 視窗內嵌「需授權」提醒區的顯示、空清單隱藏、自動延伸方向（避開工作列/螢幕邊界並維持 chip 位置）、卡片點擊導航等前端行為契約。

### Modified Capabilities

（無 — overlay 提醒沿用既有 `enable_intervention_notification` 開關，不改動 `intervention-notification` 的 Toast 觸發條件、文案或設定語意。）

## Impact

- Rust：新增 `InterventionRegistry`（狀態集合與 `intervention-list-changed` 廣播）；`lib.rs` 既有 overlay emit 模式（`quota-overlay-*` 事件）延伸廣播新事件。不新增 `AppSettings` 欄位。
- 前端：`src/components/QuotaOverlay.tsx` 新增「需授權」提醒區與延伸方向邏輯（複用既有 `syncWindowSize` 量測 + Tauri window position / monitor work area）；overlay 提醒是否顯示受既有 `enableInterventionNotification` 控制；`locales/*` 新增提醒區文案。不新增設定開關。
- 依賴：`add-opencode-permission-notification` 提供跨 provider 一致的 `waiting` 訊號（Claude hook、opencode plugin bridge、Copilot/Codex 既有），本 change 為其下游消費者。
- 不改變 waiting 訊號的產生方式，也不改變 Copilot／Codex／Claude／opencode 既有 bridge 與 activity 行為。
