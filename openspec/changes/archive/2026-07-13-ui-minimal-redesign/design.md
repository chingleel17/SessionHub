# Design: ui-minimal-redesign

## 設計定位

SessionHub = 專業開發者工具 + AI Agent Workspace。視覺語言以 **Minimal UI** 為核心，融合少量 Apple HIG / Fluent Design（depth、layer、surface 的空間感）與克制的 Glassmorphism。

參考對象：Linear、Raycast、Arc、Apple Developer、Notion、Vercel Dashboard。
反面對象：Discord、Jira、Azure Portal、傳統企業後台。

視覺密度定位：`Notion ← SessionHub → GitHub`（比 Notion 密一點、比 Jira/Slack/VSCode 設定頁鬆很多）。

核心優先序：**高級感 > 華麗感；舒適度 > 視覺衝擊；長時間使用體驗 > 設計展示效果。**

## 為何一次交付「設計系統 + 一個頁面」

- 純寫設計系統無法驗證落地性，容易變成 Dribbble 概念稿。
- 一次全站套用風險過大、diff 難審。
- 折衷：本 change 定義完整 token 與規則，並以 Project 頁作為第一個實作範例；後續每個頁面各開一個 change，依循此設計系統套用。

## Design Tokens

沿用現有 `--color-*` 命名以維持向後相容（避免大範圍改 class name），僅調整值並新增新 token。

### 色彩 — 品牌

| Token | Light | Dark | 說明 |
|---|---|---|---|
| `--color-action-primary` | `#2563EB` | `#3B82F6` | 品牌強調色（暗色略提亮以維持對比） |
| `--color-action-primary-text` | `#ffffff` | `#ffffff` | primary 按鈕文字 |

### 色彩 — 背景與表面（避免純黑/純白高對比）

| Token | Light | Dark | 說明 |
|---|---|---|---|
| `--color-surface-app` | `#FAFAFB` | `#0F1115` | 頁面底色（最底層畫布） |
| `--color-surface-panel` | `#FFFFFF` | `#171A21` | 卡片/面板 |
| `--color-surface-input` | `#FFFFFF` | `#1C202A` | 輸入框 |

背景層級策略：以 app → panel 之間的**微小色差**建立層級，不靠邊框或陰影切割。app-shell **移除 radial-gradient**，維持「一整塊畫布」。

### 色彩 — 文字

| Token | Light | Dark |
|---|---|---|
| `--color-text-primary` | `#162033` | `#E8EDF5` |
| `--color-text-secondary` | `#5A6780` | `#8A97AC` |

文字層級優先用**字級 + 字重 + 色差**建立，而非分隔線。

### 圓角（統一系統，不得混用 4/8/24px）

| Token | 值 | 用途 |
|---|---|---|
| `--radius-button` | `10px` | Button |
| `--radius-input` | `12px` | Input / Select / Textarea |
| `--radius-card` | `16px` | Card / Panel |
| `--radius-modal` | `20px` | Modal / Dialog |

（現有 `--radius-md: 12px` / `--radius-xl: 24px` 保留為別名對映到 input/card，逐步淘汰 24px。）

### 邊框（極淡，介面像一整塊畫布）

| Token | Light | Dark |
|---|---|---|
| `--color-border-subtle` | `rgba(0,0,0,0.06)` | `rgba(255,255,255,0.08)` |
| `--color-border` | `rgba(0,0,0,0.06)` | `rgba(255,255,255,0.08)` |

規則：預設 `border: none`；需要分界時才用 `1px solid var(--color-border-subtle)`。**禁止**粗框、高對比分隔線。

### 陰影（輕，具 depth 但不壓迫）

| Token | 值 | 用途 |
|---|---|---|
| `--shadow-panel` | `0 2px 12px rgba(0,0,0,0.06)` | 一般 card/panel |
| `--shadow-float` | `0 8px 32px rgba(0,0,0,0.12)` | Modal/Dropdown 等浮層 |

規則：移除 `0 18px 40px` 這類重陰影。一般表面只用 `--shadow-panel`；浮層才用 `--shadow-float`。

### Glassmorphism（點綴，非主體）

| Token | 值 |
|---|---|
| `--glass-blur` | `20px` |
| `--glass-bg` | Light `rgba(255,255,255,0.7)` / Dark `rgba(23,26,33,0.7)` |

**僅限** Modal / Dropdown / Floating Panel / Context Menu 使用 `backdrop-filter: blur(var(--glass-blur))` + `--glass-bg`。Sidebar、Card、整頁**不**玻璃化。

### 動畫

| Token | 值 |
|---|---|
| `--motion-fast` | `150ms` |
| `--motion-base` | `200ms` |
| `--motion-slow` | `250ms` |
| `--motion-ease` | `ease-out` |

套用於 Hover、Tab 切換、Sidebar 展開、Modal 開啟。禁止彈跳（bounce）、過長、華麗轉場。尊重 `prefers-reduced-motion`。

## 元件風格規則

### Layout

- 避免 Card-in-Card-in-Card；避免每區塊都有粗邊框；避免過多分隔線。
- 層級靠留白 / 字級 / 背景色差；僅在「需要聚焦」的區域用 Card。
- 頁面結構：Header → Navigation Tabs → Workspace(Sidebar + Content)；Content 保持開放感。

### Tabs（Apple / Linear 式）

- Active：底部 accent line（`2px` `--color-action-primary`）**或** pill（淡藍底 `--color-action-primary-subtle-bg` + primary 文字）。二擇一，全站一致。
- Inactive：無背景、`--color-text-secondary` 文字；hover 時文字轉深。
- 禁止 Bootstrap 式（每個 tab 都有邊框/背景塊）。

### Button

- Primary：filled，`--color-action-primary` 底、白字、`--radius-button`、無邊框、`--shadow-panel` 或無陰影。
- Secondary：ghost（透明底、hover 淡背景）。
- Tertiary：text only。
- 盡量減少 outlined button。

### List / Table

- 不要每列都是卡片。改 row + hover 柔和背景切換（`rgba(255,255,255,0.04)` dark / `rgba(0,0,0,0.03)` light）+ 極細分隔線（或無線，靠 hover 區分）。
- 風格對標 GitHub Issues / Linear Issues。

### Sidebar

- 輕量、不厚重、無大型深色區塊（不玻璃化）。
- 樹狀結構走 IDE 式清楚層級（Folder → AGENTS.md / Skills / Commands）。

## Open Questions（實作時決策）

1. **Tabs active 樣式**：底線 vs pill —— 建議 sub-tab 用底線（省垂直空間、貼近 Linear）；主導覽若有需要再議。實作時以底線為預設。
2. **Session 列表**：完全 row 化 vs 保留輕量卡片 —— Project 頁試做時兩者都可接受，優先「去粗框/重陰影 + 新圓角 + 極淡邊框 + 增留白」的輕量卡片，降低對 SessionCard 內部結構的改動。
3. **暗色主色 `#3B82F6`**：暗背景下 `#2563EB` 對比稍弱，故暗色提亮到 `#3B82F6`；若偏好單一主色可統一用 `#2563EB`。
