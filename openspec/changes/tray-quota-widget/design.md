# Design: Tray Quota Widget

## 架構概觀

```
┌─────────────────────────────────────────────────────┐
│  QuotaCache (already exists)                        │
│  Vec<QuotaSnapshot> in memory                       │
└──────────────────┬──────────────────────────────────┘
                   │ read
┌──────────────────▼──────────────────────────────────┐
│  TrayQuotaManager (Rust, new)                       │
│  • Listens to "quota-snapshots-updated" event       │
│  • Computes tray display state from snapshots       │
│  • Updates tray icon + tooltip                      │
│  • Manages mini-panel window lifecycle              │
└──────────────────┬──────────────────────────────────┘
                   │
         ┌─────────┴──────────┐
         │                    │
┌────────▼──────┐   ┌─────────▼──────────────────────┐
│ Tray Icon     │   │ Mini Panel Window               │
│ (dynamic PNG) │   │ src/windows/TrayQuotaPanel.tsx  │
│ tooltip text  │   │ frameless, always-on-top, small │
└───────────────┘   └────────────────────────────────┘
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

## 2. Mini Panel Window

### 2.1 視窗規格

| 屬性 | 值 |
|------|----|
| 寬度 | 320px |
| 高度 | auto（最高 480px） |
| 位置 | 右下角，系統匣附近（taskbar 上方） |
| 框架 | 無框（frameless），圓角 |
| 層級 | always-on-top |
| 觸發 | tray icon 左鍵點擊 |
| 關閉 | 點擊外部 / 再次點擊 tray icon / Esc |
| 動畫 | 從右下角滑入（CSS transition） |

### 2.2 視窗內容（TrayQuotaPanel.tsx）

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

### 2.3 Tauri Window 建立方式

```rust
// 點擊 tray icon 時
let panel = tauri::WebviewWindowBuilder::new(
    &app,
    "tray-quota-panel",
    tauri::WebviewUrl::App("tray-panel.html".into()),
)
.title("")
.decorations(false)
.always_on_top(true)
.skip_taskbar(true)
.inner_size(320.0, 480.0)
.position(tray_x - 320.0, tray_y - 480.0)
.build()?;
```

需要在 `tauri.conf.json` 的 `windows` 陣列加入 panel 的初始宣告（hidden），或純動態建立。

### 2.4 視窗定位

Windows tray icon 位置可透過 `tray.rect()` 取得，再計算 panel 應出現的位置（右下角，taskbar 上方）。需處理 taskbar 在不同邊（上/下/左/右）的情況。

## 3. 設定項目

在 `AppSettings` 新增：

```rust
pub tray_quota_mode: TrayQuotaMode,  // icon_only / percentage / bar / hidden
pub tray_quota_primary_provider: Option<String>,  // None = 自動選最高用量
pub tray_quota_panel_enabled: bool,  // 是否啟用點擊展開面板
```

前端 `src/types/index.ts`：

```typescript
type TrayQuotaMode = "icon_only" | "percentage" | "bar" | "hidden";
// AppSettings 新增 trayQuotaMode, trayQuotaPrimaryProvider, trayQuotaPanelEnabled
```

## 4. 前端路由

TrayQuotaPanel 是一個獨立的 HTML 入口（`tray-panel.html`）或在現有 app 中用 `?view=tray-panel` query string 路由。

建議用獨立入口以降低 bundle 大小（panel 只需要 quota 相關 UI）。

## 5. Crate 需求

| Crate | 用途 |
|-------|------|
| `image` | 動態繪製 tray icon PNG |
| `imageproc` | 在 icon 上繪製弧形/進度條/文字 |
| `rusttype` 或 `ab_glyph` | 在 icon 上繪製數字文字 |

（這些 crate 在 Windows 上均能靜態編譯，無需外部依賴）

## 7. 參考：costats 的實作模式

[costats](https://github.com/fmdz387/costats) 是一個 Windows 系統匣 app，實作了與我們相似的功能。關鍵設計決策值得參考：

### 7.1 Delegated Token Refresh

costats 不直接呼叫 OAuth token refresh endpoint，而是使用「delegated refresh」：執行 `claude /status` 命令，讓 Claude Code CLI 自己完成 token 刷新，再重新讀取 credentials.json。

```csharp
// costats ClaudeOAuthUsageFetcher.cs - TryDelegatedRefreshAsync()
process.StartInfo.FileName = claudePath;
process.StartInfo.Arguments = "/status";  // 觸發 Claude Code 內部 refresh
// 之後重新讀取 ~/.claude/.credentials.json
```

優點：不需要實作 PKCE 流程，直接借用 Claude Code CLI 已有的 token 管理。

**SessionHub 可採用的實作**：若 access token 已過期，先嘗試執行 `claude /status`（timeout 5 秒），再重新讀取 credentials 並重試 API 呼叫。我們目前已實作直接呼叫 OAuth endpoint 的方式，可作為 fallback。

### 7.2 Credentials 結構（已確認）

costats 確認 `~/.claude/.credentials.json` 的正確格式：

```json
{
  "claudeAiOauth": {
    "accessToken": "sk-ant-oat01-...",
    "refreshToken": "sk-ant-ort01-...",
    "expiresAt": 1234567890000,
    "subscriptionType": "claude_pro",
    "rateLimitTier": "standard"
  }
}
```

### 7.3 API Response 格式（已確認）

`https://api.anthropic.com/api/oauth/usage` 的回應：

```json
{
  "five_hour": {
    "utilization": 72.5,
    "resets_at": "2025-01-01T12:00:00Z"
  },
  "seven_day": {
    "utilization": 38.2,
    "resets_at": "2025-01-08T00:00:00Z"
  },
  "extra_usage": {
    "is_enabled": true,
    "used_credits": 500,
    "monthly_limit": 10000
  }
}
```

注意：
- Windows 在 top level（不在 `rate_limits` 下）
- `utilization` 是 0-100 的值（不是 0-1）
- `extra_usage` 的金額單位是分（÷100 得到美元）

## 6. 已知限制

- **Windows 系統匣圖示最大 32×32 px**，所以文字只能顯示 2-3 位數，解析度有限
- **無框視窗拖移**：mini panel 無標題列，需在 frontend 處理 drag 或固定不可拖移
- **多螢幕**：panel 位置需考慮 DPI scaling 和多螢幕偏移
- **系統匣溢出選單**：若圖示在系統匣溢出區，`tray.rect()` 可能不準確
