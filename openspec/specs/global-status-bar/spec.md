## Requirements

### Requirement: 全域狀態列常駐顯示

系統 SHALL 在應用程式視窗最底部常駐顯示一條全域狀態列，跨所有 View（Dashboard、Settings、ProjectView）可見，不受 activeView 切換影響。

#### Scenario: 跨 View 常駐

- **WHEN** 使用者切換 activeView 為任意值（dashboard / settings / {projectKey}）
- **THEN** 狀態列仍固定顯示於視窗底部，不消失

#### Scenario: 設定關閉時隱藏

- **WHEN** `settings.showStatusBar` 為 `false`
- **THEN** 狀態列不渲染，主內容區域佔滿全部高度

### Requirement: 狀態列顯示最後一筆 Bridge 事件

系統 SHALL 在狀態列左側顯示最後一筆 Provider Bridge 事件的摘要資訊。

#### Scenario: 有事件時顯示摘要

- **WHEN** 已收到至少一筆 `provider-bridge-event-logged` 事件
- **THEN** 狀態列顯示：接收時間（HH:mm:ss）、provider 名稱、事件類型、狀態色標（targeted=藍、fallback=黃、full_refresh=綠、skipped_*=灰）、以及 cwd 路徑（截斷至 40 字元）

#### Scenario: 無事件時顯示佔位文字

- **WHEN** 尚未收到任何 Bridge 事件（`lastBridgeEvent` 為 null）
- **THEN** 狀態列左側顯示 `t("statusBar.noEvent")` 佔位文字（灰色、低對比）

#### Scenario: 點擊開啟事件監視器

- **WHEN** 使用者點擊狀態列的 Bridge 事件區段
- **THEN** 系統開啟 `BridgeEventMonitorDialog`（等同於設定頁的「開啟事件監視器」按鈕）

### Requirement: 狀態列顯示 Session 活動計數

系統 SHALL 在狀態列右側顯示當前 active（進行中）與 waiting（等待回應）的 session 數量，並可在同區域顯示精簡的 provider quota 摘要。

#### Scenario: 顯示計數徽章

- **WHEN** 狀態列渲染且 sessions 資料已載入
- **THEN** 狀態列右側顯示「▶ N 進行中」與「⏳ N 等待回應」兩個計數徽章
- **AND** 若某類計數為 0，以低對比灰色顯示

#### Scenario: 計數來源

- **WHEN** 計算 session 計數
- **THEN** 系統根據前端 React Query 快取的 `SessionInfo[]` 與 `SessionStats[]`，利用 `is_live` 欄位判斷 active，以 `updated_at` 距今時間判斷 waiting（30 分鐘內且非 archived）
- **AND** 不發出額外的 Tauri command，不進行獨立的後端 polling

#### Scenario: 顯示 provider quota 摘要

- **WHEN** quota monitoring 已啟用且至少一個 provider quota snapshot 可用
- **THEN** 狀態列以精簡 quota chip 顯示各 provider 摘要（provider 縮寫 + 用量百分比）
- **AND** quota chip 不使用水平進度條，避免多 provider 同時顯示時擠壓 session 活動計數資訊

#### Scenario: 資料未載入時顯示

- **WHEN** sessions 資料尚在載入中（isLoading）
- **THEN** 計數區域顯示 `-` 而非數字

### Requirement: 狀態列 quota chip 精簡顯示

狀態列的 provider quota chip SHALL 以「provider 縮寫 + 小型圓環指示 + 用量百分比數字」呈現，不得使用水平進度條；百分比文字與圓環顏色 SHALL 依用量門檻變色（≥90% 危險色、≥70% 警告色、其餘正常色），與 Dashboard QuotaOverview 的門檻與色票一致。

#### Scenario: 顯示精簡 quota chip

- **WHEN** 狀態列渲染某 provider 的 quota snapshot 且首要視窗 utilization 可得
- **THEN** chip 顯示 provider 縮寫（品牌色文字）、小型 SVG 圓環（依百分比繪製弧長）與百分比數字
- **AND** 圓環 stroke 顏色與百分比文字顏色依用量門檻取用 `--quota-bar-ok` / `--quota-bar-warning` / `--quota-bar-danger`

#### Scenario: 百分比不可得時的顯示

- **WHEN** provider quota 無可用的 utilization 百分比（無 limit 或無視窗資料）
- **THEN** chip 僅顯示 provider 縮寫（本地彙總 chip 有成本資料時另顯示 `$` 金額）
- **AND** 不顯示圓環與百分比

#### Scenario: 多 provider 同時顯示不擁擠

- **WHEN** 四個以上 provider 的 quota chip 同時顯示且視窗寬度縮小
- **THEN** 各 chip 因移除進度條而寬度精簡，狀態列不產生內容重疊或溢出

### Requirement: 狀態列 quota tooltip 時間視窗標籤本地化

狀態列 quota chip 的 tooltip 中，各 quota 視窗的標籤 SHALL 依 `windowKey` 經 i18n 對映為本地化文字（如 `5h`/`five_hour` →「5 小時」、`7d`/`seven_day`/`weekly` →「1 週」），不得直接顯示後端原始英文縮寫；非時間視窗類的標籤（如 Copilot 的 Premium/Chat 月配額）維持原始標籤。

#### Scenario: Claude tooltip 顯示中文時間視窗

- **WHEN** 使用者 hover Claude 的 quota chip
- **THEN** tooltip 各視窗列顯示「5 小時: N%」「1 週: N%」等本地化標籤，而非「5h」「7d」

#### Scenario: Codex tooltip 顯示中文時間視窗

- **WHEN** 使用者 hover Codex 的 quota chip
- **THEN** tooltip 的 primary/secondary 視窗分別以「5 小時」「1 週」本地化標籤顯示

#### Scenario: 未知 windowKey 回退原始標籤

- **WHEN** 某 provider 的 windowKey 不在對映表中（或 provider 為 copilot 的非時間視窗配額）
- **THEN** tooltip 顯示後端提供的原始 label 字串，不強制對映為任一時間視窗

### Requirement: 狀態列 quota 區域為可點擊控制項

狀態列右側的 quota 區域（含本地彙總 chips 與 remote snapshot chips）SHALL 以可點擊控制項呈現（button 語意），點擊行為為開關狀態列 quota 彈出面板（詳見 `statusbar-quota-popup` capability）；既有 hover tooltip 摘要 SHALL 保留。

#### Scenario: 點擊 quota 區域

- **WHEN** 使用者點擊狀態列任一 quota chip 所在區域
- **THEN** 系統開關 quota 彈出面板
- **AND** chip 的視覺樣式（縮寫、圓環、百分比）維持既有精簡呈現

### Requirement: Codex quota chip tooltip 顯示重置額度摘要

當 Codex 的 quota snapshot 含 `reset_credits` 時，狀態列 Codex chip 的 hover tooltip SHALL 在既有視窗用量行之後追加一行重置額度摘要：可用次數與最近一筆到期時間（本地化格式，與既有 reset 時間格式一致）。

#### Scenario: tooltip 顯示重置額度

- **WHEN** Codex snapshot 的 `reset_credits.available_count` 為 2 且最近一筆額度於 07/21 下午11:59 到期
- **THEN** hover Codex chip 的 tooltip 追加類似「重置額度: 2 次 · 最近到期 07/21 下午11:59」的摘要行

#### Scenario: 無重置額度時 tooltip 不變

- **WHEN** Codex snapshot 的 `reset_credits` 為 null
- **THEN** tooltip 維持既有內容，不出現重置額度行

### Requirement: 狀態列高度與排版

系統 SHALL 確保狀態列不影響主要內容區域的可用空間。

#### Scenario: 高度限制

- **WHEN** 狀態列顯示
- **THEN** 狀態列高度固定為 28px，使用 `position: sticky; bottom: 0`
- **AND** 主內容區域加入對應 `padding-bottom` 避免內容被遮蓋
