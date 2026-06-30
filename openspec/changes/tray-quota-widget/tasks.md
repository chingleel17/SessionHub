## 1. Tray Icon 動態顯示

- [ ] 1.1 在 `src-tauri/src/types.rs` 的 `AppSettings` struct 新增三個欄位：
  - `tray_quota_mode: TrayQuotaMode`（`#[serde(default)]`，預設 `IconOnly`）
  - `tray_quota_primary_provider: Option<String>`（`#[serde(default)]`，`None` 表示自動選最高用量 provider）
  - `tray_quota_panel_enabled: bool`（`#[serde(default = "default_true")]`）
  - 新增 `TrayQuotaMode` enum：`IconOnly / Percentage / Bar / Hidden`，derive `Serialize, Deserialize, Default`
  - 在 `src/types/index.ts` 新增 `TrayQuotaMode = "icon_only" | "percentage" | "bar" | "hidden"` 及對應 `AppSettings` 欄位

- [ ] 1.2 在 `src-tauri/src/` 新增 `tray_icon.rs` 模組，實作 `render_tray_icon_png(pct: f64, mode: TrayQuotaMode) -> Vec<u8>`：
  - 讀取 `icons/icon.png`（32×32）作為底圖
  - `IconOnly` 模式：在圖示右下角疊一個 8×8 彩色圓點（綠 &lt;50% / 黃 50-80% / 紅 &gt;80%）
  - `Percentage` 模式：在圖示底部疊 `72%` 白色小文字（用 `ab_glyph` crate 繪製）
  - `Bar` 模式：在圖示底部疊一條彩色細 bar（height 3px），長度 = 寬度 × pct
  - `Hidden` 模式：直接回傳原底圖位元組
  - 加入 `ab_glyph`（字型繪製）與 `image`（PNG 解碼/編碼）到 `Cargo.toml`

- [ ] 1.3 在 `lib.rs` 的 `run()` 加入 tray icon 初始化與更新邏輯：
  - 啟動時讀取 settings，若 `tray_quota_mode != Hidden` 且 `enable_quota_monitoring = true` 則從 `QuotaCache` 計算初始 pct 並呼叫 `render_tray_icon_png()`，設定 tray icon
  - 監聽 `"quota-snapshots-updated"` 事件，每次觸發時重新計算 pct（取所有 status:ok provider 中最高用量 window 的 utilization）並更新 tray icon
  - 更新 tray tooltip 為多行摘要（格式見 design.md §1.3）

## 2. Mini Panel 視窗

- [ ] 2.1 在 `src-tauri/tauri.conf.json` 的 `windows` 陣列加入 tray panel 視窗宣告：
  - `label: "tray-quota-panel"`，`url: "index.html?view=tray-panel"`
  - `decorations: false`，`visible: false`，`skipTaskbar: true`
  - `width: 320`，`height: 480`，`alwaysOnTop: true`，`resizable: false`

- [ ] 2.2 在 `lib.rs` 的 tray icon `on_left_click` handler 實作面板開關邏輯：
  - 若 `tray_quota_panel_enabled = false` 則點擊 tray icon 改為開啟主視窗（與現有行為一致）
  - 若 panel 視窗不存在，動態建立（`WebviewWindowBuilder`）並設定位置至系統匣上方（用 `tray.rect()` 計算）
  - 若 panel 視窗已可見，則隱藏；若隱藏中，則顯示並移至最前
  - 點擊主視窗外部（`focus-changed` 事件 or `blur`）時自動隱藏 panel

- [ ] 2.3 在 `src/App.tsx` 的 routing 加入 `?view=tray-panel` 路徑：
  - 讀取 `window.location.search` 中的 `view` 參數
  - 若 `view === "tray-panel"` 則渲染 `<TrayQuotaPanel />` 而非正常 app layout

- [ ] 2.4 新增 `src/components/TrayQuotaPanel.tsx` 元件：
  - 尺寸固定 320px 寬，auto 高（最多 480px）
  - 標題列：`SessionHub Quota` + 小齒輪按鈕（點擊開啟主 app Settings 頁）
  - 對每個 `status: "ok"` 的 provider 顯示：provider 名稱、各 window 的 utilization bar + 百分比 + resets 倒數
  - 對 `status: "no_auth" / "error"` 的 provider 顯示：簡短錯誤訊息
  - 對 `source: "local_scan"` 的 provider 顯示：本月 input/output tokens
  - 底部：「立即刷新」按鈕 + `上次更新: N 分鐘前`
  - 新增相關 CSS：無框、圓角、背景模糊（`backdrop-filter: blur(8px)`）、深色主題適配

- [ ] 2.5 在 `src/App.css` 新增 `.tray-panel-*` 系列 CSS class：
  - `.tray-panel-root`：固定尺寸、overflow hidden、圓角 12px
  - `.tray-panel-header`：padding 12px 16px、flex row
  - `.tray-panel-provider`：border-top 分隔、padding 10px 16px
  - `.tray-panel-bar`：height 6px、圓角 3px、顏色依用量三段（green/yellow/red）
  - `.tray-panel-footer`：padding 8px 16px、flex row、justify-content space-between

## 3. Settings 整合

- [ ] 3.1 在 `src/components/SettingsView.tsx` 的 quota monitoring 卡片新增 tray 顯示設定區塊：
  - `tray_quota_mode` 下拉選單（四個選項：圖示顏色指示 / 顯示百分比 / 進度條 / 隱藏）
  - `tray_quota_panel_enabled` checkbox（「點擊 tray 圖示展開 quota 面板」）
  - `tray_quota_primary_provider` 下拉選單（「自動」+ 各 enabledProvider）
  - 新增翻譯 key 至 `src/locales/zh-TW.ts` 和 `src/locales/en-US.ts`

- [ ] 3.2 在 `src-tauri/src/commands/settings.rs` 的 `save_settings` command 中，儲存設定後觸發 tray icon 重新渲染（呼叫 tray 更新邏輯），確保設定變更立即反映在匣圖示上

## 4. 驗證

- [ ] 4.1 執行 `cd src-tauri && cargo check` 確認無編譯錯誤；執行 `bun run tsc --noEmit` 確認前端型別正確
- [ ] 4.2 手動驗證：
  - 系統匣圖示在 `enable_quota_monitoring: true` 時隨 quota 快照更新而變色/顯示數字
  - 點擊 tray icon 彈出 mini panel，再次點擊或點擊 panel 外側則關閉
  - Settings 中切換 `tray_quota_mode`，圖示即時更新
  - `tray_quota_panel_enabled: false` 時點擊 tray icon 改為開啟主視窗
