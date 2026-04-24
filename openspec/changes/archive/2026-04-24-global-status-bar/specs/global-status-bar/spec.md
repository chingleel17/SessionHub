## ADDED Requirements

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

系統 SHALL 在狀態列右側顯示當前 active（進行中）與 waiting（等待回應）的 session 數量。

#### Scenario: 顯示計數徽章

- **WHEN** 狀態列渲染且 sessions 資料已載入
- **THEN** 狀態列右側顯示「▶ N 進行中」與「⏳ N 等待回應」兩個計數徽章
- **AND** 若某類計數為 0，以低對比灰色顯示

#### Scenario: 計數來源

- **WHEN** 計算 session 計數
- **THEN** 系統根據前端 React Query 快取的 `SessionInfo[]` 與 `SessionStats[]`，利用 `is_live` 欄位判斷 active，以 `updated_at` 距今時間判斷 waiting（30 分鐘內且非 archived）
- **AND** 不發出額外的 Tauri command，不進行獨立的後端 polling

#### Scenario: 資料未載入時顯示

- **WHEN** sessions 資料尚在載入中（isLoading）
- **THEN** 計數區域顯示 `-` 而非數字

### Requirement: 狀態列高度與排版

系統 SHALL 確保狀態列不影響主要內容區域的可用空間。

#### Scenario: 高度限制

- **WHEN** 狀態列顯示
- **THEN** 狀態列高度固定為 28px，使用 `position: sticky; bottom: 0`
- **AND** 主內容區域加入對應 `padding-bottom` 避免內容被遮蓋
