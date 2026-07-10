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
- **THEN** 狀態列可顯示精簡 quota 摘要（例如 provider 名稱與剩餘百分比）
- **AND** 不擠壓或取代既有 session 活動計數資訊

#### Scenario: 資料未載入時顯示

- **WHEN** sessions 資料尚在載入中（isLoading）
- **THEN** 計數區域顯示 `-` 而非數字
