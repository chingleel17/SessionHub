# SessionHub Design Tokens 完整表

所有 token 定義於 `src/styles/themes/light.css`（`:root`）與 `src/styles/themes/dark.css`（`:root[data-theme="dark"]`）。**寫 CSS 時一律引用變數，不要寫死值**——下表的具體值僅供理解與判斷對比，不應複製進元件。

## 目錄
- [品牌與強調](#品牌與強調)
- [背景與表面](#背景與表面)
- [漸層](#漸層)
- [文字](#文字)
- [邊框](#邊框)
- [圓角](#圓角)
- [陰影](#陰影)
- [Glassmorphism](#glassmorphism)
- [動畫](#動畫)
- [Scrollbar](#scrollbar)
- [Chips / Provider / 狀態](#chips--provider--狀態)
- [禁用項](#禁用項)

## 品牌與強調

| Token | Light | Dark | 用途 |
|---|---|---|---|
| `--color-action-primary` | `#2563eb` | `#3b82f6` | 品牌主色（dark 提亮維持對比） |
| `--color-action-primary-text` | `#ffffff` | `#ffffff` | primary 按鈕文字 |
| `--color-action-primary-subtle-bg` | `rgba(37,99,235,0.08)` | `rgba(59,130,246,0.12)` | hover / active 淡底、subtle 強調 |
| `--color-action-primary-subtle-border` | `rgba(37,99,235,0.18)` | `rgba(59,130,246,0.24)` | subtle 強調邊框 |

**規則**：藍只用於互動/強調（primary 按鈕、active tab 底線、hover、選中）。內容與容器不要用藍。

## 背景與表面

避免純黑/純白高對比。app 與 panel 之間靠**微小色差**分層，不靠邊框切割。

| Token | Light | Dark | 用途 |
|---|---|---|---|
| `--color-surface-app` | `#fbfbfc` | `#0e1014` | 頁面最底畫布 |
| `--color-surface-panel` | `#ffffff` | `#181b22` | 面板 / 需聚焦的卡片 |
| `--color-surface-subtle` | `#f4f5f7` | `#1d212a` | 次要區塊、報告列 |
| `--color-surface-input` | `#ffffff` | `#1c202a` | 輸入框 |
| `--color-surface-accent` | `rgba(37,99,235,0.06)` | `rgba(59,130,246,0.1)` | 極淡強調底 |

dark 的 app(`#0e1014`) 與 panel(`#181b22`) 保留可辨色差，讓需要卡片的浮層在深色下仍看得出邊界。

## 漸層

| Token | 內容 | 用途 |
|---|---|---|
| `--gradient-app` | `--color-surface-app` 之上疊兩層極淡（≤0.06）冷灰 radial 光暈 | app-shell 主背景 |
| `--gradient-panel-header` | 極淡縱向 `linear-gradient(180deg,…)`，兩主題各定義 | 面板/浮層 header（如 preview modal） |

**規則**：漸層是質感點綴，透明度極低。禁止高飽和、高透明度的裝飾漸層當主背景。

## 文字

冷灰中性（去藍味）。層級優先用字級 + 字重 + 色差，而非分隔線。

| Token | Light | Dark |
|---|---|---|
| `--color-text-primary` | `#1a1d23` | `#e6e8ec` |
| `--color-text-secondary` | `#656a72` | `#8b9099` |

## 邊框

極淡，介面像一整塊畫布。

| Token | Light | Dark |
|---|---|---|
| `--color-border-subtle` | `rgba(0,0,0,0.06)` | `rgba(255,255,255,0.09)` |
| `--color-border` | `rgba(0,0,0,0.06)` | `rgba(255,255,255,0.1)` |

**規則**：預設 `border: none`；需要分界才用 `1px solid var(--color-border-subtle)`。禁止粗框、高對比線。

## 圓角

統一四級系統，**不得混用 4/8/24px**。

| Token | 值 | 用途 |
|---|---|---|
| `--radius-button` | `10px` | Button |
| `--radius-input` | `12px` | Input / Select / Textarea / dropdown |
| `--radius-card` | `16px` | Card / Panel |
| `--radius-modal` | `20px` | Modal / Dialog |

舊別名 `--radius-md`(=12px) / `--radius-xl`(=16px) 仍存在但逐步淘汰，新程式碼用上表四個。

## 陰影

| Token | 值 | 用途 |
|---|---|---|
| `--shadow-panel` | `0 2px 12px rgba(0,0,0,0.06)` | 一般 card / panel |
| `--shadow-float` | `0 8px 32px rgba(0,0,0,0.12)` | modal / dropdown / 浮層 |

**規則**：禁止 `0 18px 40px`、`0 28px 60px` 這類重陰影。

## Glassmorphism

僅用於浮層：modal / dropdown / floating panel / context menu。

| Token | 值 |
|---|---|
| `--glass-blur` | `20px` |
| `--glass-bg` | Light `rgba(255,255,255,0.7)` · Dark `rgba(23,26,33,0.7)` |

用法：`background: var(--glass-bg); backdrop-filter: blur(var(--glass-blur)); -webkit-backdrop-filter: blur(var(--glass-blur));`。**Sidebar、card、整頁背景禁止玻璃化。**

## 動畫

| Token | 值 |
|---|---|
| `--motion-fast` | `150ms` |
| `--motion-base` | `200ms` |
| `--motion-slow` | `250ms` |
| `--motion-ease` | `ease-out` |

用於 hover / tab 切換 / 展開 / modal。禁止 bounce、過長、華麗轉場。全域已有 `@media (prefers-reduced-motion: reduce)` 保護，不要破壞。

## Scrollbar

| Token | Light | Dark |
|---|---|---|
| `--color-scrollbar-thumb` | `rgba(100,110,130,0.3)` | `rgba(255,255,255,0.14)` |
| `--color-scrollbar-thumb-hover` | `rgba(100,110,130,0.5)` | `rgba(255,255,255,0.26)` |

全站已有 `* { scrollbar-width: thin; scrollbar-color: … }` + `::-webkit-scrollbar` 樣式（細、圓角、track 透明）。新捲動區自動繼承，不要覆寫回系統預設。

## Chips / Provider / 狀態

既有 token（沿用，勿寫死）：`--color-chip-bg/text`、`--color-muted-chip-bg/text`、`--color-error-chip-bg/text`、`--color-provider-{copilot,opencode,codex}-{bg,text}`、`--color-status-success/error`。Quota 相關見 `--quota-*`。

## 禁用項（出現即為 regression）

- 寫死顏色（`#fff`、`rgba(255,255,255,0.9)`、硬編碼藍如 `rgba(47,109,246,…)`）→ 改對應 token
- 圓角 4px / 8px / 24px / 22px / 14px 等非系統值 → 改四級圓角 token
- 重陰影 `0 18px 40px` / `0 28px 60px` → 改 `--shadow-panel` / `--shadow-float`
- app-shell 的高飽和 radial-gradient → 改 `--gradient-app`
- 內容容器套 glass 或整頁玻璃化 → 只有浮層可 glass
- 帶藍味的文字色 → 改冷灰 `--color-text-*`
