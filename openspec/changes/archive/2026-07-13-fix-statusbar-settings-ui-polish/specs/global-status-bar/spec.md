## MODIFIED Requirements

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

## ADDED Requirements

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
