# tray-quota-widget

## ADDED Requirements

### Requirement: 系統匣圖示動態反映 quota 用量

系統 SHALL 在 quota monitoring 啟用時，依設定模式在系統匣圖示上動態反映主要 provider 的當前用量。

#### Scenario: 依模式渲染 tray 圖示

- **WHEN** `enable_quota_monitoring: true` 且 `tray_quota_mode != hidden`，且 `"quota-snapshots-updated"` 事件觸發
- **THEN** 系統取 `tray_quota_primary_provider` 指定 provider（`None` 時自動取所有 status:ok provider 中最高用量 window）的 utilization
- **AND** 依 `tray_quota_mode` 重繪 tray 圖示：`icon_only` 疊彩色圓點 / `percentage` 疊百分比文字 / `bar` 疊進度條
- **AND** 顏色三段：綠 <50%、黃 50-80%、紅 >80%
- **AND** 更新 tooltip 為各 enabled provider 的多行用量摘要

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
- **AND** bar 顏色三段：綠 <50%、黃 50-80%、紅 >80%
- **AND** `status: no_auth / error` 的 provider 顯示灰色錯誤指示，不顯示 bar

#### Scenario: 鎖定模式滑鼠穿透

- **WHEN** `quota_overlay_locked: true`（預設）
- **THEN** overlay 呼叫 `set_ignore_cursor_events(true)`，滑鼠點擊穿透至底下視窗
- **AND** overlay 不可拖曳、不可互動，僅顯示

#### Scenario: 編輯模式拖曳與位置記憶

- **WHEN** 使用者由 tray 右鍵選單切換至編輯模式（`quota_overlay_locked: false`）
- **THEN** overlay 關閉滑鼠穿透、顯示虛線外框，可整體拖曳
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
- **THEN** 於系統匣上方彈出 320px 寬無框面板，顯示所有 enabled provider 的 quota 詳情（bar、百分比、reset 倒數、錯誤狀態、local tokens）與「立即刷新」按鈕
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
- **AND** overlay 依 `quota_overlay_enabled` 立即建立或關閉，依 `quota_overlay_opacity`、`quota_overlay_locked`、`quota_overlay_providers` 立即更新
- **AND** 所有新增設定欄位缺席時以 serde default 回填（向後相容既有 settings 檔）
