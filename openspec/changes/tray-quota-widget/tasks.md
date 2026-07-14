## 1. 設定與型別

- [ ] 1.1 在 `src-tauri/src/types.rs` 的 `AppSettings` struct 新增欄位（皆 `#[serde(default)]`）：
  - `tray_quota_mode: TrayQuotaMode`（預設 `IconOnly`）
  - `tray_quota_primary_provider: Option<String>`（`None` 表示自動選最高用量 provider）
  - `tray_quota_panel_enabled: bool`（`#[serde(default = "default_true")]`）
  - `quota_overlay_enabled: bool`（預設 `false`）
  - `quota_overlay_locked: bool`（`#[serde(default = "default_true")]`）
  - `quota_overlay_opacity: f64`（預設 `0.85`）
  - `quota_overlay_providers: Vec<String>`（預設空 = 全部 enabled provider）
  - 新增 `TrayQuotaMode` enum：`IconOnly / Percentage / Bar / Hidden`，derive `Serialize, Deserialize, Default`
  - 在 `src/types/index.ts` 同步新增 `TrayQuotaMode` 型別與 `AppSettings` 對應欄位

## 2. Tray Icon 動態顯示

- [ ] 2.1 在 `src-tauri/src/` 新增 `tray_icon.rs` 模組，實作 `render_tray_icon_png(pct: f64, mode: TrayQuotaMode) -> Vec<u8>`：
  - 讀取 `icons/icon.png`（32×32）作為底圖
  - `IconOnly`：右下角疊 8×8 彩色圓點（綠 <50% / 黃 50-80% / 紅 >80%）
  - `Percentage`：底部疊白色小文字（`ab_glyph` 繪製）
  - `Bar`：底部疊彩色細 bar（height 3px，長度 = 寬度 × pct）
  - `Hidden`：回傳原底圖位元組
  - 加入 `ab_glyph` 與 `image` 到 `Cargo.toml`

- [ ] 2.2 在 `lib.rs` 的 `run()` 加入 tray icon 初始化與更新邏輯：
  - 啟動時讀取 settings，若 `tray_quota_mode != Hidden` 且 `enable_quota_monitoring = true` 則從 `QuotaCache` 計算初始 pct 並設定 tray icon
  - 監聽 `"quota-snapshots-updated"` 事件，重新計算 pct（取所有 status:ok provider 中最高用量 window 的 utilization）並更新 tray icon
  - 更新 tray tooltip 為多行摘要（格式見 design.md §1.3）

## 3. Overlay Widget（常駐置頂）

- [ ] 3.1 安裝 `tauri-plugin-window-state`（Rust crate + `@tauri-apps/plugin-window-state`，透過 CLI 安裝），在 `lib.rs` 註冊 plugin

- [ ] 3.2 在 `lib.rs` 新增 overlay 視窗建立函式 `create_quota_overlay(app: &AppHandle)`：
  - **必須在 Rust 端**以 `WebviewWindowBuilder` 建立（`tauri.conf.json` 的 `focus: false` 在 Windows 不生效）：
    `transparent(true)`、`decorations(false)`、`shadow(false)`、`always_on_top(true)`、`skip_taskbar(true)`、`focused(false)`、`resizable(false)`、`visible(false)`
  - `url: "index.html?view=quota-overlay"`，label `"quota-overlay"`
  - 建立後由 window-state plugin 還原位置，微調 size 觸發 transparent 重繪（tauri#4881 白底 workaround），再 `show()`
  - 依 `quota_overlay_locked` 呼叫 `set_ignore_cursor_events()`
  - 啟動時若 `quota_overlay_enabled = true` 自動建立

- [ ] 3.3 在 tray 右鍵選單新增兩個項目：
  - 「顯示/隱藏 Quota Overlay」→ toggle `quota_overlay_enabled` 並建立/關閉視窗
  - 「編輯 Overlay 位置 / 鎖定 Overlay」→ toggle `quota_overlay_locked`，即時呼叫 `set_ignore_cursor_events()`，並通知前端切換編輯視覺

- [ ] 3.4 新增 `src/components/QuotaOverlay.tsx` 元件：
  - 每個選定 provider 一列：名稱 + utilization bar + % + reset 倒數（`status: ok`）；`no_auth`/`error` 顯示灰色小圖示
  - 監聽 `"quota-snapshots-updated"` 事件即時更新
  - 編輯模式：整體加 `data-tauri-drag-region`、顯示虛線外框與鎖定鈕；鎖定模式：純顯示
  - 背景半透明深色（透明度取自 `quota_overlay_opacity`），`background: transparent` 於 root、CSS 圓角
  - CSS class 前綴 `.quota-overlay-*`，遵循 sessionhub-minimal-ui 設計語言

- [ ] 3.5 在 `src/App.tsx` routing 加入 `?view=quota-overlay` 與 `?view=tray-panel` 分支：
  - 讀取 `window.location.search` 的 `view` 參數，分別渲染 `<QuotaOverlay />` / `<TrayQuotaPanel />` 而非正常 app layout

## 4. Mini Panel 視窗

- [ ] 4.1 在 `lib.rs` 的 tray icon `on_left_click` handler 實作面板開關邏輯：
  - 若 `tray_quota_panel_enabled = false` 則點擊 tray icon 改為開啟主視窗（與現有行為一致）
  - 若 panel 視窗不存在，動態建立（`WebviewWindowBuilder`，`url: "index.html?view=tray-panel"`，`decorations(false)`、`skip_taskbar(true)`、`always_on_top(true)`、320×480）並用 `tray.rect()` 計算位置至系統匣上方
  - 若 panel 已可見則隱藏；隱藏中則顯示並移至最前
  - panel `blur` 時自動隱藏（僅 panel，overlay 不受此邏輯影響）

- [ ] 4.2 新增 `src/components/TrayQuotaPanel.tsx` 元件：
  - 尺寸固定 320px 寬，auto 高（最多 480px）
  - 標題列：`SessionHub Quota` + 小齒輪按鈕（點擊開啟主 app Settings 頁）
  - 對每個 `status: "ok"` 的 provider 顯示：provider 名稱、各 window 的 utilization bar + 百分比 + resets 倒數
  - 對 `status: "no_auth" / "error"` 的 provider 顯示簡短錯誤訊息
  - 對 `source: "local_scan"` 的 provider 顯示本月 input/output tokens
  - 底部：「立即刷新」按鈕 + `上次更新: N 分鐘前`

- [ ] 4.3 在 `src/App.css` 新增 `.tray-panel-*` 與 `.quota-overlay-*` 系列 CSS class：
  - `.tray-panel-root`：固定尺寸、overflow hidden、圓角 12px、背景模糊（`backdrop-filter: blur(8px)`）
  - `.tray-panel-header` / `.tray-panel-provider` / `.tray-panel-footer`：見 design.md §3.2 版型
  - `.tray-panel-bar` / `.quota-overlay-bar`：height 6px、圓角、顏色依用量三段（green/yellow/red）
  - `.quota-overlay-root`：透明背景、圓角、緊湊列版型；`.quota-overlay-editing`：虛線外框

## 5. Settings 整合

- [ ] 5.1 在 `src/components/SettingsView.tsx` 的 quota monitoring 卡片新增 tray/overlay 設定區塊：
  - `tray_quota_mode` 下拉選單（圖示顏色指示 / 顯示百分比 / 進度條 / 隱藏）
  - `tray_quota_panel_enabled` checkbox（「點擊 tray 圖示展開 quota 面板」）
  - `tray_quota_primary_provider` 下拉選單（「自動」+ 各 enabledProvider）
  - `quota_overlay_enabled` checkbox（「顯示桌面常駐 Quota Overlay」）+ 說明文字註明無法覆蓋獨佔全螢幕應用
  - `quota_overlay_opacity` slider（0.3–1.0）
  - `quota_overlay_providers` 多選（各 enabledProvider）
  - 新增翻譯 key 至 `src/locales/zh-TW.ts` 和 `src/locales/en-US.ts`

- [ ] 5.2 在 `src-tauri/src/commands/settings.rs` 的 `save_settings` command 中，儲存設定後：
  - 觸發 tray icon 重新渲染
  - 依 `quota_overlay_enabled` 建立/關閉 overlay 視窗；依 `quota_overlay_locked`、`quota_overlay_opacity` 即時更新 overlay 狀態

## 6. 驗證

- [ ] 6.1 執行 `cd src-tauri && cargo check` 確認無編譯錯誤；執行 `bun run tsc --noEmit` 確認前端型別正確
- [ ] 6.2 手動驗證 tray：
  - 系統匣圖示在 `enable_quota_monitoring: true` 時隨 quota 快照更新而變色/顯示數字
  - 點擊 tray icon 彈出 mini panel，再次點擊或點擊 panel 外側則關閉
  - Settings 切換 `tray_quota_mode`，圖示即時更新
  - `tray_quota_panel_enabled: false` 時點擊 tray icon 改為開啟主視窗
- [ ] 6.3 手動驗證 overlay：
  - 啟用後 overlay 常駐置頂，切換其他視窗（含最大化）不被遮蓋、不搶焦點（在其他 app 打字時 overlay 出現不中斷輸入）
  - 鎖定模式下點擊 overlay 區域，滑鼠事件穿透到底下視窗
  - 編輯模式可拖曳，重啟 app 後位置保留；拔除副螢幕後 overlay 不會消失在畫面外
  - 背景透明無白底、拖曳無殘影（Windows 11 實測）
  - quota 快照更新時 overlay bar 即時變化
