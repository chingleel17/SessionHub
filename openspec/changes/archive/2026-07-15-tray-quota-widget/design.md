# Design: Tray Quota Widget

## 架構概觀

```
┌─────────────────────────────────────────────────────┐
│  QuotaCache (already exists)                        │
│  Vec<QuotaSnapshot> in memory                       │
└──────────────────┬──────────────────────────────────┘
                   │ read / "quota-snapshots-updated" event
┌──────────────────▼──────────────────────────────────┐
│  TrayQuotaManager (Rust, new)                       │
│  • Computes display state from snapshots            │
│  • Updates tray icon + tooltip                      │
│  • Manages overlay + mini-panel window lifecycle    │
└──────┬───────────────┬──────────────────┬───────────┘
       │               │                  │
┌──────▼──────┐ ┌──────▼───────────┐ ┌────▼─────────────────┐
│ Tray Icon   │ │ Overlay Widget   │ │ Mini Panel Window     │
│ dynamic PNG │ │ 常駐、置頂、穿透  │ │ tray 點擊彈出、失焦隱藏│
│ tooltip     │ │ QuotaOverlay.tsx │ │ TrayQuotaPanel.tsx    │
└─────────────┘ └──────────────────┘ └──────────────────────┘
```

## 1. 系統匣圖示動態更新

### 1.1 Icon 內容模式（可設定）

| 模式 | 說明 | 範例 |
|------|------|------|
| `icon_only` | 只換圖示顏色（綠/黃/紅） | 預設圖示加色環 |
| `percentage` | 顯示主 provider 的最高用量 % | `72%` |
| `bar` | 迷你進度條疊加在圖示上 | 圖示+細長 bar |
| `hidden` | 不顯示任何 quota 資訊於匣圖示 | 純 SessionHub icon |

### 1.2 動態 PNG 生成

在 Rust 端用 `image` crate 動態繪製 32×32 PNG：
- 底層：現有 SessionHub icon
- 疊層：彩色弧形（用量環）或右下角小矩形（bar）
- 文字：若模式為 `percentage`，疊入 8px 白色數字

```rust
// 虛擬碼
fn render_tray_icon(pct: f64, mode: TrayDisplayMode) -> Vec<u8> {
    let mut img = base_icon();
    match mode {
        TrayDisplayMode::Percentage => draw_text(&mut img, &format!("{:.0}%", pct)),
        TrayDisplayMode::Bar => draw_bar(&mut img, pct),
        TrayDisplayMode::IconOnly => colorize(&mut img, quota_color(pct)),
        TrayDisplayMode::Hidden => {} // do nothing
    }
    img.to_png_bytes()
}
```

### 1.3 Tooltip

Tray icon tooltip（Windows 系統匣 hover 顯示）：
```
SessionHub
Claude: 72% (5-hour) · 45% (7-day) — resets in 2h 15m
Copilot: 38%
OpenCode: 125k tok (This month)
```

各 window 的顯示文字取自 `QuotaWindow.label`（如 `"5-hour"`、`"7-day"`），**不硬編碼「5h/7d」**——`QuotaWindow` 結構僅有 `window_key`、`label`、`utilization`、`resets_at`、`group`，無 5h/7d 語意欄位；window 顯示一律以 `label` 為準。`local_scan` provider 的期間文字取自 `LocalTokenUsage.period_label`。

## 2. Overlay Widget（本變更主體）

### 2.1 視窗規格

| 屬性 | 值 |
|------|----|
| 尺寸 | 360px 寬、依內容自動調整高度（最高 720px），避免多 window provider 或 local scan 文字被裁切 |
| 位置 | 使用者可拖曳至任意位置，跨重啟記憶（`tauri-plugin-window-state`） |
| 框架 | `decorations(false)` + `shadow(false)`，圓角由 CSS 實現 |
| 背景 | `transparent(true)`，可選半透明深色或淺色底（透明度可設定 0.3–1.0）；嵌入視圖的 `html`、`body`、`#root` 必須透明以避免白底外框 |
| 層級 | `always_on_top(true)` + `skip_taskbar(true)` |
| 焦點 | `focused(false)`，且**必須在 Rust 端以 `WebviewWindowBuilder` 建立**（`tauri.conf.json` 的 `focus: false` 在 Windows 不生效，tauri#11566 / #7519） |
| 生命週期 | 開啟後常駐，**失焦不隱藏**；只由設定或 tray 選單關閉 |

### 2.2 鎖定 / 編輯雙模式

| | 鎖定模式（預設） | 編輯模式 |
|--|------|------|
| 滑鼠穿透 | `set_ignore_cursor_events(true)`，點擊直接落到底下視窗 | `set_ignore_cursor_events(false)` |
| 拖曳 | 不可 | 可（frontend `data-tauri-drag-region`） |
| 視覺 | 純顯示 | 顯示虛線外框 + 拖曳提示 |
| 切換 | tray 右鍵選單「編輯 Overlay 位置」 | 同選單「鎖定 Overlay」，或點擊 widget 上的鎖定鈕 |

註：Tauri 的 `set_ignore_cursor_events` 是整窗開關，無法區域級穿透（tauri#2090），因此採顯式模式切換而非滑鼠座標輪詢（省效能、不誤觸，為 Rainmeter 類工具標準 UX）。

### 2.3 Widget 內容（QuotaOverlay.tsx）

```
┌────────────────────────────┐
│ Claude  ████████░░ 72% 2h15m│  ← 每 provider 一列：名稱 + bar + % + reset 倒數
│ Copilot ████░░░░░░ 38%      │
│ Codex   ██████░░░░ 55% 30d  │
└────────────────────────────┘
```

- 只顯示設定中勾選的 provider，各 window 一列，window 名稱取自 `QuotaWindow.label`
- 狀態顯示規則（依 `QuotaSnapshot.status`）：
  - `ok`：顯示 bar + 百分比 + reset 倒數
  - `no_auth` / `error`：顯示灰色小圖示，不顯示 bar
  - `rate_limited`：沿用該 provider 上次快取的 bar 數值，加註「略舊」視覺標記（避免限流時清空顯示）
  - `unsupported`：該 provider 該列**完全不顯示**
- bar 顏色三段：綠 <50% / 黃 50-80% / 紅 >80%
- 資料來源：監聽 `"quota-snapshots-updated"` 事件即時更新
- 編輯模式時整個 widget 為 drag region，並顯示外框
- 百分比僅顯示於 bar 右側一處；meta 行只放 reset 倒數與略舊標記，避免重複
- 鎖定模式不渲染工具列（無鎖頭列）；工具列僅編輯模式出現
- 視窗尺寸貼合內容：前端以 ResizeObserver 量測 wrapper，`setSize(LogicalSize)` 同步原生視窗，不出現滾動條；Rust 端不強制固定尺寸

### 2.6 精簡版型（`quota_overlay_style: compact`）

```
╭  CC ◔ 54%   GH ◔ 53%   CX ● 100%  ╮   ← 一列水平 chips，藥丸形
```

- 每 provider 一個 chip：縮寫（PROVIDER_ABBR）+ 迷你圓環（同狀態列 QuotaRing，14px）+ 最高 window 用量 %
- hover 顯示各 window 明細 tooltip
- 寬度 max-content，明顯窄於完整版；設定頁「Overlay 版型」切換，即存即生效
- 編輯模式使用 Windows `app-region: drag`；鎖定鈕使用 `app-region: no-drag`，避免按鈕被拖曳區攔截
- 鎖定鈕使用鎖頭 icon，切換時更新 `set_ignore_cursor_events()` 與前端狀態；不得透過重新載入 WebView 套用設定，避免快取快照短暫消失

### 2.4 Windows transparent 已知 bug 與 workaround

- 初始化白底：建立後需觸發一次 resize 才會真正透明（tauri#4881）→ 建立時 `visible: false`，還原位置 + 微調 size 後再 `show()`
- 設定變更時同樣可微調尺寸觸發重繪；僅還原 window-state 的位置，不還原歷史尺寸，避免舊版小尺寸裁切新內容
- 拖曳殘影（ghost titlebar，tauri#14764）→ 確保 `shadow(false)` 並在 CSS 端 `background: transparent`
- `decorations(false)` + `shadow(false)` 並用時標題列可能殘留（tauri#14859）→ 實作後需在 Windows 11 實測

### 2.5 位置記憶

採用官方 `tauri-plugin-window-state`：
- 自動儲存/還原 overlay 視窗位置
- 內建多螢幕邊界驗證（螢幕拔除或解析度改變後不會還原到畫面外）
- 座標以 physical pixel 儲存，還原時依當下螢幕 scale factor 換算，避免跨 DPI 螢幕跑位

## 3. Mini Panel Window（tray 點擊彈出）

### 3.1 視窗規格

| 屬性 | 值 |
|------|----|
| 寬度 | 320px |
| 高度 | auto（最高 480px） |
| 位置 | 右下角，系統匣附近（taskbar 上方） |
| 框架 | 無框（frameless）、圓角、不透明主題面板，避免背景透出降低閱讀性 |
| 層級 | always-on-top |
| 觸發 | tray icon 左鍵點擊 |
| 關閉 | 點擊外部（blur 自動隱藏）/ 再次點擊 tray icon / Esc |
| 動畫 | 從右下角滑入（CSS transition） |

### 3.2 視窗內容（TrayQuotaPanel.tsx）

```
┌─────────────────────────────┐
│  SessionHub Quota           │ ← 標題 + 設定按鈕（小齒輪）
├─────────────────────────────┤
│  Claude                  API │
│  ████████████░░░░ {label}:72%│ ← window 名稱取自 QuotaWindow.label
│  ██████░░░░░░░░░░ {label}:38%│
│  resets in 2h 15m           │
├─────────────────────────────┤
│  Copilot                 API │
│  ████████░░░░░░░░    48%   │
├─────────────────────────────┤
│  OpenCode         本地估算  │
│  125k input / 89k output    │
│  {period_label}（各供應商合計）│ ← 期間文字取自 LocalTokenUsage.period_label
├─────────────────────────────┤
│  [↻]       上次更新: 2分前 │
└─────────────────────────────┘
```

- window 標籤（如「5-hour」「7-day」）一律取自 `QuotaWindow.label`，不硬編碼
- `local_scan` 期間文字取自 `LocalTokenUsage.period_label`（如「本月」），不硬編碼
- 狀態顯示規則同 §2.3：`ok` 顯示完整；`no_auth`/`error` 顯示錯誤訊息；`rate_limited` 沿用上次快取數值並註記略舊；`unsupported` 該 provider 不列出

### 3.3 視窗定位

Windows tray icon 位置可透過 `tray.rect()` 取得，再計算 panel 應出現的位置（右下角，taskbar 上方）。需處理 taskbar 在不同邊（上/下/左/右）的情況。

## 4. 設定項目

在 `AppSettings` 新增：

```rust
// Tray 圖示
pub tray_quota_mode: TrayQuotaMode,           // icon_only / percentage / bar / hidden
pub tray_quota_primary_provider: Option<String>, // None = 自動選最高用量
pub tray_quota_panel_enabled: bool,           // 點擊 tray 是否展開面板

// Overlay widget
pub quota_overlay_enabled: bool,              // 是否顯示常駐 overlay（預設 false）
pub quota_overlay_locked: bool,               // 鎖定（穿透）/ 編輯，預設 true
pub quota_overlay_opacity: f64,               // 0.3–1.0，預設 0.85
pub quota_overlay_providers: Vec<String>,     // 空 = 全部 enabled provider
pub quota_overlay_theme: OverlayTheme,         // dark / light，預設 dark
pub quota_overlay_style: OverlayStyle,         // full / compact，預設 full
```

前端 `src/types/index.ts`：

```typescript
type TrayQuotaMode = "icon_only" | "percentage" | "bar" | "hidden";
type OverlayStyle = "full" | "compact";
// AppSettings 新增 trayQuotaMode, trayQuotaPrimaryProvider, trayQuotaPanelEnabled,
// quotaOverlayEnabled, quotaOverlayLocked, quotaOverlayOpacity, quotaOverlayProviders,
// quotaOverlayTheme, quotaOverlayStyle
```

## 5. 前端路由

Overlay 與 Mini Panel 共用現有 app 入口，以 query string 區分：
- `?view=quota-overlay` → 渲染 `<QuotaOverlay />`
- `?view=tray-panel` → 渲染 `<TrayQuotaPanel />`

（若之後 bundle 大小成為問題，再拆獨立 HTML 入口——YAGNI）

## 6. Crate / Plugin 需求

| 依賴 | 用途 |
|-------|------|
| `image` | 動態繪製 tray icon PNG |
| `ab_glyph` | 在 icon 上繪製數字文字 |
| `tauri-plugin-window-state` | overlay 位置跨重啟記憶（含多螢幕驗證） |

（均可在 Windows 靜態編譯，無外部依賴）

## 7. 已知限制

- **蓋不過 exclusive fullscreen**：Windows z-order band 機制限制，任何一般視窗都無法蓋過獨佔全螢幕應用；borderless windowed 正常。Afterburner 式 in-game OSD 需 DX hook，不採用。需在設定頁對使用者說明
- **Windows 系統匣圖示最大 32×32 px**，文字只能顯示 2-3 位數
- **系統匣圖示位置無法程式化調整**：Windows 系統匣圖示的排列位置由 OS 管理，app 無法移動；「可調整位置」（如 FPS Monitor OSD）僅 overlay widget 支援（見 §2.2 編輯模式），tray 圖示不適用
- **click-through 為整窗開關**，無區域級穿透，故採鎖定/編輯雙模式
- **`focus: false` 設定檔寫法在 Windows 失效**（tauri#11566 / #7519），overlay 必須在 Rust 端 `WebviewWindowBuilder::focused(false)` 建立
- **transparent 視窗初始白底 / 拖曳殘影**（tauri#4881 / #14764），需按 §2.4 workaround 處理
- **多螢幕**：panel 定位需考慮 DPI scaling；overlay 由 window-state plugin 處理
- **系統匣溢出選單**：若圖示在溢出區，`tray.rect()` 可能不準確

## 8. OpenCode Gateway 刷新

OpenCode quota adapter 僅讀取本機 `opencode.db` 或 session JSON，統計本月透過 OpenCode 使用的所有上游模型 input/output tokens；它不是 OpenCode Go / Zen 的訂閱額度查詢。Codex、Copilot 仍由各自 adapter 查詢帳號額度。

OpenCode bridge event 無法可靠識別實際上游帳號，因此收到 OpenCode 事件時，系統刷新所有已啟用 quota adapter；其他 provider event 則只刷新該 provider。此策略讓透過 OpenCode gateway 使用 Codex/Copilot 時，兩者額度能跟隨活動更新。
