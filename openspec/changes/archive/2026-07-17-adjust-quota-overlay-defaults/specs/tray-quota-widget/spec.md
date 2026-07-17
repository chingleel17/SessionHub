## MODIFIED Requirements

### Requirement: 常駐桌面 Quota Overlay Widget

系統 SHALL 提供一個常駐桌面、永遠置頂、不搶焦點的無框透明 overlay widget，顯示選定 provider 的 quota 用量，其可見性不受焦點變化影響。

#### Scenario: Overlay 建立與置頂行為

- **WHEN** `quota_overlay_enabled: true`（啟動時或設定變更時）
- **THEN** 系統在 Rust 端以 `WebviewWindowBuilder` 建立 overlay 視窗：transparent、無框、無陰影、always-on-top、skip-taskbar、`focused(false)`
- **AND** 每次 `show()` 之後必須重新呼叫 `set_skip_taskbar(true)`——Windows 上 show 會重置 taskbar 樣式（tauri#10422），僅在 builder 設定 skip-taskbar 不足以避免 overlay 出現在工作列與 Alt+Tab
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

- **WHEN** `quota_overlay_style: compact`（預設）
- **THEN** overlay 以單列水平 chips 呈現：每個 provider 顯示縮寫 + 迷你圓環 + 最高 window 用量百分比（同狀態列 QuotaRing 視覺）
- **AND** hover chip 顯示該 provider 各 window 明細 tooltip
- **AND** 視窗寬度縮至內容寬（max-content），外觀為藥丸形圓角
- **AND** `quota_overlay_style: full` 維持進度條列表版型（使用者可於設定切換）

#### Scenario: 鎖定模式滑鼠穿透

- **WHEN** `quota_overlay_locked: true`（預設）
- **THEN** overlay 呼叫 `set_ignore_cursor_events(true)`，滑鼠點擊穿透至底下視窗
- **AND** overlay 不可拖曳、不可互動，僅顯示
- **AND** 鎖定時不顯示工具列與鎖定圖示（不佔版面）；工具列僅於編輯模式出現

#### Scenario: 首次啟用預設定位於螢幕右下角

- **WHEN** overlay 首次建立（`tauri-plugin-window-state` 尚無該視窗 label 的已存位置紀錄）
- **THEN** 系統將 overlay 定位於主螢幕右下角，保留 16px 邊距，不落在系統預設位置
- **AND** 若視窗尺寸大於可用螢幕範圍，定位座標不小於螢幕左上角原點

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

### Requirement: Tray 與 Overlay 顯示設定

系統 SHALL 在 Settings 的 quota monitoring 區塊提供 tray 圖示模式、panel 開關、overlay 開關/透明度/provider 選擇等設定，變更後即時生效。

#### Scenario: 設定即時生效

- **WHEN** 使用者在 Settings 變更任一 tray/overlay 設定並儲存
- **THEN** tray 圖示立即依新模式重繪
- **AND** overlay 依 `quota_overlay_enabled` 立即建立或關閉，依 `quota_overlay_opacity`、`quota_overlay_locked`、`quota_overlay_providers`、`quota_overlay_theme`、`quota_overlay_style` 立即更新，不需重啟 app
- **AND** Windows transparent WebView 必要時透過尺寸微調重繪，不重新載入 WebView，避免快照資料或鎖定狀態短暫重置
- **AND** 所有新增設定欄位缺席時以 serde default 回填（向後相容既有 settings 檔）；`quota_overlay_opacity` 預設 `0.3`、`quota_overlay_style` 預設 `compact`
