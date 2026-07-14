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
Claude: 72% (5h) · 45% (7d) — resets in 2h 15m
Copilot: 38%
OpenCode: 125k tok (本月)
```

## 2. Overlay Widget（本變更主體）

### 2.1 視窗規格

| 屬性 | 值 |
|------|----|
| 尺寸 | 精簡版型：約 220px 寬、每 provider 一列（約 28px/列），auto 高 |
| 位置 | 使用者可拖曳至任意位置，跨重啟記憶（`tauri-plugin-window-state`） |
| 框架 | `decorations(false)` + `shadow(false)`，圓角由 CSS 實現 |
| 背景 | `transparent(true)`，半透明深色底（透明度可設定 0.3–1.0） |
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

- 只顯示設定中勾選的 provider（`status: ok` 顯示 bar；`no_auth`/`error` 顯示灰色小圖示）
- bar 顏色三段：綠 <50% / 黃 50-80% / 紅 >80%
- 資料來源：監聽 `"quota-snapshots-updated"` 事件即時更新
- 編輯模式時整個 widget 為 drag region，並顯示外框

### 2.4 Windows transparent 已知 bug 與 workaround

- 初始化白底：建立後需觸發一次 resize 才會真正透明（tauri#4881）→ 建立時 `visible: false`，還原位置 + 微調 size 後再 `show()`
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
| 框架 | 無框（frameless），圓角 |
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
│  ████████████░░░░ 5h: 72%  │
│  ██████░░░░░░░░░░ 7d: 38%  │
│  resets in 2h 15m           │
├─────────────────────────────┤
│  Copilot                 API │
│  ████████░░░░░░░░    48%   │
├─────────────────────────────┤
│  OpenCode         本地估算  │
│  125k input / 89k output    │
│  本月（各供應商合計）        │
├─────────────────────────────┤
│  [立即刷新]  上次更新: 2分前 │
└─────────────────────────────┘
```

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
```

前端 `src/types/index.ts`：

```typescript
type TrayQuotaMode = "icon_only" | "percentage" | "bar" | "hidden";
// AppSettings 新增 trayQuotaMode, trayQuotaPrimaryProvider, trayQuotaPanelEnabled,
// quotaOverlayEnabled, quotaOverlayLocked, quotaOverlayOpacity, quotaOverlayProviders
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
- **click-through 為整窗開關**，無區域級穿透，故採鎖定/編輯雙模式
- **`focus: false` 設定檔寫法在 Windows 失效**（tauri#11566 / #7519），overlay 必須在 Rust 端 `WebviewWindowBuilder::focused(false)` 建立
- **transparent 視窗初始白底 / 拖曳殘影**（tauri#4881 / #14764），需按 §2.4 workaround 處理
- **多螢幕**：panel 定位需考慮 DPI scaling；overlay 由 window-state plugin 處理
- **系統匣溢出選單**：若圖示在溢出區，`tray.rect()` 可能不準確
