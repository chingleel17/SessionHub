# app-theme

## MODIFIED Requirements

### Requirement: 主題色彩系統

系統 SHALL 以 CSS 自訂屬性（變數）定義品牌主題色彩，支援明暗主題切換，且色彩選擇遵循 Minimal 開發者工具語言：品牌主色為 `#2563EB`（暗色主題提亮為 `#3B82F6`），背景避免純黑與純白高對比。

#### Scenario: 品牌主色

- **WHEN** 應用程式渲染
- **THEN** `--color-action-primary` 於 Light 主題為 `#2563EB`、Dark 主題為 `#3B82F6`
- **AND** primary 按鈕文字 `--color-action-primary-text` 為 `#ffffff`

#### Scenario: 背景與表面色（避免純黑/純白）

- **WHEN** 應用程式渲染
- **THEN** Light 主題 `--color-surface-app` 為 `#FAFAFB`、`--color-surface-panel` 為 `#FFFFFF`
- **AND** Dark 主題 `--color-surface-app` 為 `#0F1115`、`--color-surface-panel` 為 `#171A21`
- **AND** app 與 panel 之間僅以微小色差建立層級，不依賴粗邊框或重陰影切割

#### Scenario: 文字色階

- **WHEN** 應用程式渲染
- **THEN** `--color-text-primary` / `--color-text-secondary` 於 Light 為 `#162033` / `#5A6780`，Dark 為 `#E8EDF5` / `#8A97AC`

## ADDED Requirements

### Requirement: 圓角 token 統一系統

系統 SHALL 以四級圓角 token 定義所有圓角，且不同元件類型不得混用非規範值（禁止 4px / 8px / 24px 混雜）。

#### Scenario: 圓角 token 定義

- **WHEN** 應用程式渲染
- **THEN** 根元素定義 `--radius-button: 10px`、`--radius-input: 12px`、`--radius-card: 16px`、`--radius-modal: 20px`
- **AND** Button 使用 `--radius-button`、Input/Select/Textarea 使用 `--radius-input`、Card/Panel 使用 `--radius-card`、Modal/Dialog 使用 `--radius-modal`

### Requirement: 邊框 token 與極淡邊框規則

系統 SHALL 以極淡邊框呈現分界，預設無邊框，介面整體呈現「一整塊畫布」的連續感。

#### Scenario: 邊框 token 值

- **WHEN** 應用程式渲染
- **THEN** `--color-border-subtle` 於 Light 為 `rgba(0,0,0,0.06)`、Dark 為 `rgba(255,255,255,0.08)`
- **AND** 需要分界的元素使用 `1px solid var(--color-border-subtle)`，其餘預設 `border: none`
- **AND** 系統 SHALL NOT 使用粗邊框或高對比分隔線

### Requirement: 陰影 token 與輕量陰影規則

系統 SHALL 以輕量陰影營造 depth 而不造成壓迫；一般表面與浮層採不同層級。

#### Scenario: 陰影 token 值

- **WHEN** 應用程式渲染
- **THEN** `--shadow-panel` 為 `0 2px 12px rgba(0,0,0,0.06)`、`--shadow-float` 為 `0 8px 32px rgba(0,0,0,0.12)`
- **AND** 一般 card/panel 使用 `--shadow-panel`，Modal/Dropdown 等浮層使用 `--shadow-float`
- **AND** 系統 SHALL NOT 使用 `0 18px 40px` 這類重陰影

### Requirement: Glassmorphism 專用 token 與限用範圍

系統 SHALL 將玻璃效果限定為點綴，僅套用於浮層類元件。

#### Scenario: Glass token 與適用元件

- **WHEN** 渲染 Modal / Dropdown / Floating Panel / Context Menu
- **THEN** 該元件 MAY 使用 `backdrop-filter: blur(var(--glass-blur))`（`--glass-blur: 20px`）搭配 `--glass-bg`（Light `rgba(255,255,255,0.7)`、Dark `rgba(23,26,33,0.7)`）
- **AND** Sidebar、Card、整頁背景 SHALL NOT 使用玻璃效果

### Requirement: 動畫 token 與克制的動效規則

系統 SHALL 以短促、`ease-out` 的過場動畫提升質感，避免華麗轉場。

#### Scenario: 動畫 token 與適用互動

- **WHEN** 發生 Hover、Tab 切換、Sidebar 展開/收合、Modal 開啟
- **THEN** 過場時長介於 `--motion-fast: 150ms` 至 `--motion-slow: 250ms`，緩動為 `--motion-ease: ease-out`
- **AND** 系統 SHALL NOT 使用彈跳（bounce）、過長或華麗轉場
- **AND** 當使用者啟用 `prefers-reduced-motion` 時，過場動畫 SHALL 縮短或停用

### Requirement: app-shell 為連續畫布（含極淡冷灰漸層質感）

app-shell SHALL 呈現為單一連續畫布：可帶極淡的冷灰漸層光暈以增添質感，但 SHALL NOT 以強烈漸層、色塊或重陰影切割視覺區塊。畫布調性為冷灰中性（Linear/Vercel 調），強調藍僅出現於互動/強調元件。

#### Scenario: app-shell 背景漸層

- **WHEN** 應用程式主視窗渲染
- **THEN** app-shell 背景使用 `--gradient-app`：於 `--color-surface-app` 之上疊加極淡（透明度 ≤ 0.06）的冷灰 radial-gradient 光暈
- **AND** SHALL NOT 使用高飽和或高透明度的裝飾性漸層作為主背景

#### Scenario: 冷灰中性文字色

- **WHEN** 應用程式渲染文字
- **THEN** 文字色為冷灰中性（去藍味）：Light `--color-text-primary` `#1a1d23` / `--color-text-secondary` `#656a72`，Dark `#e6e8ec` / `#8b9099`

### Requirement: 面板 header 漸層 token

系統 SHALL 提供 `--gradient-panel-header` 供面板 / 浮層 header（如 preview modal header）使用，於明暗主題各自定義極淡的縱向漸層。

#### Scenario: header 漸層 token 定義

- **WHEN** 應用程式渲染
- **THEN** `--gradient-panel-header` 於 Light 與 Dark 各自定義為極淡的 `linear-gradient(180deg, …)`，與畫布融合、不形成突兀色塊

### Requirement: 自訂 scrollbar 樣式

系統 SHALL 以自訂樣式取代系統預設 scrollbar，呈現細緻、冷灰半透明、圓角的捲軸，明暗主題各自定義。

#### Scenario: scrollbar token 與樣式

- **WHEN** 任一可捲動區域出現捲軸
- **THEN** 捲軸 thumb 使用 `--color-scrollbar-thumb`（hover 用 `--color-scrollbar-thumb-hover`），track 透明，thumb 為圓角
- **AND** Light thumb 為冷灰半透明（如 `rgba(100,110,130,0.3)`）、Dark 為白半透明（如 `rgba(255,255,255,0.14)`）
- **AND** 透過 `scrollbar-width: thin` 與 `::-webkit-scrollbar` 同時支援 Firefox 與 Chromium
