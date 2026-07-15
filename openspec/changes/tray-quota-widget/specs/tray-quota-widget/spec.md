# tray-quota-widget

## ADDED Requirements

### Requirement: 系統匣圖示動態反映 quota 用量

系統 SHALL 在 quota monitoring 啟用時，依設定模式在系統匣圖示上動態反映主要 provider 的當前用量。

#### Scenario: 依模式渲染 tray 圖示

- **WHEN** `enable_quota_monitoring: true` 且 `tray_quota_mode != hidden`，且 `"quota-snapshots-updated"` 事件觸發
- **THEN** 系統取 `tray_quota_primary_provider` 指定 provider（`None` 時自動取所有 status:ok provider 中最高用量 window）的 utilization
- **AND** 依 `tray_quota_mode` 重繪 tray 圖示：`icon_only` 疊彩色圓點 / `percentage` 疊百分比文字 / `bar` 疊進度條
- **AND** 顏色三段：綠 <50%、黃 50-80%、紅 >80%
- **AND** 更新 tooltip 為各 enabled provider 的多行用量摘要，window 名稱取自 `QuotaWindow.label`，`local_scan` 期間取自 `LocalTokenUsage.period_label`

#### Scenario: 隱藏模式

- **WHEN** `tray_quota_mode = hidden` 或 `enable_quota_monitoring = false`
- **THEN** tray 圖示維持原始 SessionHub icon，不疊加任何 quota 資訊

### Requirement: 常駐桌面 Quota Overlay Widget

系統 SHALL 提供一個常駐桌面、永遠置頂、不搶焦點的無框透明 overlay widget，顯示選定 provider 的 quota 用量，其可見性不受焦點變化影響。

#### Scenario: Overlay 建立與置頂行為

- **WHEN** `quota_overlay_enabled: true`（啟動時或設定變更時）
- **THEN** 系統在 Rust 端以 `WebviewWindowBuilder` 建立 overlay 視窗：transparent、無框、無陰影、always-on-top、skip-taskbar、`focused(false)`
- **AND** overlay 疊於所有一般視窗（含最大化與 borderless fullscreen 視窗）之上
- **AND** overlay 出現與更新不奪取其他應用的鍵盤焦點
- **AND** 使用者切換、最大化其他視窗時 overlay 保持可見（失焦不隱藏）

#### Scenario: Overlay 內容顯示

- **WHEN** overlay 顯示中且 `"quota-snapshots-updated"` 事件觸發
- **THEN** 對 `quota_overlay_providers`（空 = 全部 enabled）中每個 provider 顯示一列：名稱 + utilization bar + 百分比 + reset 倒數
- **AND** window 名稱一律取自 `QuotaWindow.label`（不硬編碼 5h/7d）；`source: local_scan` 的期間文字取自 `LocalTokenUsage.period_label`
- **AND** bar 顏色三段：綠 <50%、黃 50-80%、紅 >80%
- **AND** `status: no_auth / error` 的 provider 顯示灰色錯誤指示，不顯示 bar
- **AND** `status: rate_limited` 的 provider 沿用上次快取的 bar 數值並加註略舊標記，不清空顯示
- **AND** `status: unsupported` 的 provider 不列出（該列完全不顯示）
- **AND** overlay 視窗尺寸貼合內容（前端量測後同步原生視窗大小），不顯示滾動條，長的 local scan 期間文字不可以撐破或裁切視窗
- **AND** 用量百分比僅顯示一次（bar 右側）；reset 倒數行不重複顯示百分比
- **AND** overlay 可依 `quota_overlay_theme` 使用深色或淺色配色，且 webview 根背景保持透明，不顯示白色外框

#### Scenario: 精簡版型（圓環一列）

- **WHEN** `quota_overlay_style: compact`
- **THEN** overlay 以單列水平 chips 呈現：每個 provider 顯示縮寫 + 迷你圓環 + 最高 window 用量百分比（同狀態列 QuotaRing 視覺）
- **AND** hover chip 顯示該 provider 各 window 明細 tooltip
- **AND** 視窗寬度縮至內容寬（max-content），外觀為藥丸形圓角
- **AND** `quota_overlay_style: full`（預設）維持進度條列表版型

#### Scenario: 鎖定模式滑鼠穿透

- **WHEN** `quota_overlay_locked: true`（預設）
- **THEN** overlay 呼叫 `set_ignore_cursor_events(true)`，滑鼠點擊穿透至底下視窗
- **AND** overlay 不可拖曳、不可互動，僅顯示
- **AND** 鎖定時不顯示工具列與鎖定圖示（不佔版面）；工具列僅於編輯模式出現

#### Scenario: 編輯模式拖曳與位置記憶

- **WHEN** 使用者由 tray 右鍵選單切換至編輯模式（`quota_overlay_locked: false`）
- **THEN** overlay 關閉滑鼠穿透、顯示虛線外框，可整體拖曳
- **AND** 編輯模式使用原生 Windows drag region；鎖頭按鈕不屬於 drag region 且可正常操作
- **AND** 位置由 window-state plugin 持久化，重啟後還原
- **AND** 還原位置若超出目前可用螢幕範圍（如副螢幕已拔除），overlay 回到主螢幕可見位置

#### Scenario: 獨佔全螢幕限制

- **WHEN** 前景應用以 exclusive fullscreen 模式執行
- **THEN** overlay 允許被遮蓋（OS 層級限制），系統不嘗試以 DirectX hook 等注入方式繞過
- **AND** Settings 中的 overlay 啟用選項附說明文字告知此限制

### Requirement: Tray 點擊彈出 Mini Panel

系統 SHALL 在使用者左鍵點擊 tray 圖示時，於系統匣附近彈出精簡 quota 面板，並於失焦時自動隱藏。

#### Scenario: 開關 panel

- **WHEN** `tray_quota_panel_enabled: true` 且使用者左鍵點擊 tray 圖示
- **THEN** 於系統匣上方彈出 320px 寬無框、不透明面板，顯示所有 enabled provider 的 quota 詳情（bar、百分比、reset 倒數、錯誤狀態、local tokens）與刷新 icon 按鈕
- **AND** panel 以 tray 所在螢幕的座標與 DPI 定位，不可在多螢幕環境超出可視範圍
- **AND** panel 失焦（blur）、再次點擊 tray 圖示或按 Esc 時隱藏
- **AND** panel 的自動隱藏邏輯不影響 overlay widget

#### Scenario: panel 停用時回復預設行為

- **WHEN** `tray_quota_panel_enabled: false` 且使用者左鍵點擊 tray 圖示
- **THEN** 開啟主視窗（與現有行為一致）

### Requirement: Tray 與 Overlay 顯示設定

系統 SHALL 在 Settings 的 quota monitoring 區塊提供 tray 圖示模式、panel 開關、overlay 開關/透明度/provider 選擇等設定，變更後即時生效。

#### Scenario: 設定即時生效

- **WHEN** 使用者在 Settings 變更任一 tray/overlay 設定並儲存
- **THEN** tray 圖示立即依新模式重繪
- **AND** overlay 依 `quota_overlay_enabled` 立即建立或關閉，依 `quota_overlay_opacity`、`quota_overlay_locked`、`quota_overlay_providers`、`quota_overlay_theme`、`quota_overlay_style` 立即更新，不需重啟 app
- **AND** Windows transparent WebView 必要時透過尺寸微調重繪，不重新載入 WebView，避免快照資料或鎖定狀態短暫重置
- **AND** 所有新增設定欄位缺席時以 serde default 回填（向後相容既有 settings 檔）

### Requirement: OpenCode Gateway 觸發上游 quota 更新

系統 SHALL 將 OpenCode 本地 token 統計與上游帳號 quota 分開處理，並在 OpenCode gateway 活動時更新已啟用的上游 quota。

#### Scenario: OpenCode local scan

- **WHEN** OpenCode quota adapter 刷新
- **THEN** 系統僅從本機 `opencode.db` 或 session JSON 統計當月 input/output tokens
- **AND** 系統不將該數值表示為 OpenCode Go 或 Zen 訂閱額度

#### Scenario: OpenCode bridge event

- **WHEN** 收到 OpenCode provider bridge event 且 quota monitoring 啟用
- **THEN** 系統刷新所有 `quota_enabled_providers` 的 quota adapter
- **AND** Codex、Copilot adapter 各自查詢其帳號 quota，不依賴 OpenCode local scan 數值
