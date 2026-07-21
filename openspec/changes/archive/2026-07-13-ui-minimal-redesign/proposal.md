# Proposal: ui-minimal-redesign

## Why

SessionHub 目前的視覺語言偏向「傳統管理後台」：圓角混雜（12px / 24px）、陰影偏重（`0 18px 40px`）、app-shell 帶 radial-gradient 背景、內容區大量 card 堆疊與粗邊框。這與產品定位（專業開發者工具 / AI Agent Workspace）不符，長時間使用時視覺壓迫、疲勞。

目標是把整體 UI 重新定位為 Minimal、沉穩、高質感的開發者工具，視覺參考 Linear / Raycast / Vercel Dashboard / Notion，而非 Discord / Jira / Azure Portal。核心原則：**高級感 > 華麗感、舒適度 > 視覺衝擊、長時間使用體驗 > 設計展示效果**。

本 change 一次交付兩件事：(1) 全域**設計系統**的重新定義（design tokens、圓角/邊框/陰影/色彩/元件風格規範）；(2) 以 **Project 專案頁** 作為首個套用範例，驗證設計系統落地。其餘頁面（Dashboard、Settings、Agents 等）留待後續 change 逐頁套用，以控制單次改動範圍與風險。

## What Changes

### 設計系統（全域）

- **色彩**：品牌主色改為 `#2563EB`。背景避免純黑/純白高對比，Light 用 `#FAFAFB`（app）/`#FFFFFF`（surface），Dark 用 `#0F1115`（app）/`#171A21`（surface）。
- **圓角統一系統**：Button `10px`、Input `12px`、Card `16px`、Modal `20px`。移除現有 4/8/24px 混用。
- **邊框規則**：預設 `border: none` 或 `1px solid rgba(0,0,0,0.06)`（暗色對應 `rgba(255,255,255,0.08)`）。移除粗框與高對比分隔線，介面看起來像「一整塊畫布」。
- **陰影規則**：標準陰影降為 `0 2px 12px rgba(0,0,0,0.06)`。移除 `0 18px 40px` 這類重陰影。
- **背景層級**：移除 app-shell 的 radial-gradient；改以留白、字級、背景色差建立層級，而非邊框與 card 堆疊。
- **Glassmorphism 限用**：僅 Modal / Dropdown / Floating Panel / Context Menu 使用 `backdrop-blur(20px)` + 半透明底；Sidebar、Card、整頁不玻璃化。
- **Tabs 風格**：Apple / Linear 式。Active 用底部 accent line 或 pill；Inactive 無背景、淡色文字。移除 Bootstrap 式 tab。
- **Button 階層**：Primary（filled 柔和藍）、Secondary（ghost）、Tertiary（text only）；減少 outlined button。
- **動畫**：Hover / Tab 切換 / Sidebar 展開 / Modal 開啟採 150–250ms `ease-out`；不做彈跳、過長或華麗轉場。
- **List / Table**：避免每列都是卡片，改 row hover + 柔和背景切換 + 極細分隔線（GitHub / Linear Issues 風格）。

### Project 頁試做

- Project 頁 sub-tab bar 改 Linear/Apple 式 tab（active 底線或 pill，inactive 無背景）。
- Sessions 分頁的 session 列表：由 card 堆疊改為 row-hover 清單風格（或保留卡片但去除粗邊框/重陰影、套用新圓角與極淡邊框），內容區保持開放感、增加留白。
- Toolbar / filter 區去除粗框與重陰影，改用背景色差與留白建立層級。

## Capabilities

### Modified Capabilities

- `app-theme`: 重新定義品牌主色與明暗背景、擴充 design token（圓角層級、邊框層級、陰影層級、glass 專用 token、動畫時長/緩動、冷灰中性文字、`--gradient-app` / `--gradient-panel-header` 漸層、scrollbar token）、新增設計系統落地規則（邊框/陰影/圓角/glass/tabs/button/list/scrollbar 的使用約束，app-shell 冷灰漸層畫布）。
- `project-subtabs`: Project 頁 sub-tab bar 與 sessions 內容區改採 Minimal 設計語言（Linear/Apple 式 tab、row-hover 清單、開放留白、去粗框重陰影）。
- `agents-config-view-ux`: Agents 頁（AGENTS.md / Skills / Commands / MCP）去卡片化（移除多層卡片框，改留白 + hairline）；收折分區標題列整合操作按鈕與設定檔路徑小字、移除獨立 title 行；儲存按鈕圖示修正且僅編輯時顯示；ContentViewer preview modal 修正滾動與關閉鈕。

## Impact

- 前端樣式：`src/styles/themes/light.css`、`src/styles/themes/dark.css`（token 重定義與新增）；`src/App.css`（app-shell 背景、共用 input/button/textarea、Project 頁相關 class）。
- 前端元件：`src/components/ProjectView.tsx`（sub-tab / toolbar / list class 調整，不改資料邏輯）；視需要 `src/components/SessionCard.tsx`（列樣式）。
- 前端元件（Agents）：`src/components/AgentsConfigView.tsx`、`src/components/McpConfigView.tsx`、`src/components/CollapsibleSection.tsx`（新增 titleMeta/actions slot）、`src/components/ContentViewer.tsx` 相關 modal、`src/components/Icons.tsx`（新增 SaveIcon）。
- 非目標（Non-goals）：本 change 不改任何資料流、IPC、業務邏輯；**不重構 Dashboard / Settings / Sidebar 版面**（僅 tokens 變動會連帶影響其配色）；不新增功能。（註：Agents 頁版面原列為 non-goal，實作迭代中依使用者需求納入去卡片化重整；Session 卡片去卡片化延後至後續 change。）
- 風險：token 全域變動會即時影響所有頁面配色。為控制風險，token 需維持向後相容的變數命名（沿用現有 `--color-*` 名稱，僅調整值並新增新 token），避免大範圍改 class。
