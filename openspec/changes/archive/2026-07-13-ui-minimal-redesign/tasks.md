# Tasks: ui-minimal-redesign

## 1. Design tokens（全域）

- [x] 1.1 `light.css`：更新 `--color-action-primary` → `#2563EB`；`--color-surface-app` → `#FAFAFB`；`--color-surface-panel` → `#FFFFFF`；`--color-surface-input` → `#FFFFFF`；文字色 `#162033` / `#5A6780`。
- [x] 1.2 `light.css`：`--color-border-subtle` / `--color-border` → `rgba(0,0,0,0.06)`；`--shadow-panel` → `0 2px 12px rgba(0,0,0,0.06)`。
- [x] 1.3 `light.css`：新增圓角 token `--radius-button:10px` `--radius-input:12px` `--radius-card:16px` `--radius-modal:20px`（保留 `--radius-md`/`--radius-xl` 別名對映，逐步淘汰 24px）。
- [x] 1.4 `light.css`：新增 `--shadow-float`、`--glass-blur`、`--glass-bg`、`--motion-fast/base/slow`、`--motion-ease`。
- [x] 1.5 `dark.css`：對應更新 `--color-action-primary` → `#3B82F6`；`--color-surface-app` → `#0F1115`；`--color-surface-panel` / `--color-surface` → `#171A21`；`--color-surface-input` → `#1C202A`；`--color-border(-subtle)` → `rgba(255,255,255,0.08)`；`--shadow-panel` → `0 2px 12px rgba(0,0,0,0.06)`；文字色 `#E8EDF5` / `#8A97AC`；新增同組 float/glass/motion token。

## 2. app-shell 與共用元件（App.css）

- [x] 2.1 移除 `.app-shell` 的 radial-gradient，背景改單色 `--color-surface-app`。
- [x] 2.2 共用 `input/select/textarea`：圓角改用 `--radius-input`，邊框改 `--color-border-subtle`。
- [x] 2.3 共用 button 樣式盤點：Primary（filled `--radius-button`）、ghost/text 階層；減少 outlined。
- [x] 2.4 全域 hover / tab / modal 過場套用 `--motion-*`；加上 `prefers-reduced-motion` 保護。
- [x] 2.5 Modal/Dropdown/Context Menu 套用 glass token（`backdrop-filter` + `--glass-bg` + `--shadow-float`）；確認 sidebar/card 未被玻璃化。

## 3. Project 頁試做

- [x] 3.1 `.sub-tab-item` / `.sub-tab-item--active`：改 Linear 式（active 底線 `2px --color-action-primary`、inactive 無背景 + secondary 文字、hover 轉深）。
- [x] 3.2 `.sticky-project-shell` / header：去粗框重陰影，改 `--radius-card` + `--shadow-panel` 或無陰影 + 極淡邊框。
- [x] 3.3 `.toolbar-card` / `.filter-bar` / `.tag-filter-bar`：去卡片感，改背景色差 + 留白建立層級（sticky-shell 內已覆寫為透明無框；基底 toolbar-card 圓角經 `--radius-xl` 別名連動至 16px）。
- [x] 3.4 Session 列表（`.session-list` / `SessionCard`）：套用新圓角、極淡邊框、`--shadow-panel` 以下陰影、row hover 柔和背景；增加內容區留白。
- [x] 3.5 dark.css 中 Project 頁相關覆寫（sub-tab / sticky-shell / list hover）同步調整為新規範。

## 4. 驗證

- [x] 4.1 Light 與 Dark 兩主題各自檢視 Project 頁：對比、留白、hover、tab active 樣式符合規範。
- [x] 4.2 確認 sidebar 展開/收合、tab 切換、modal 開啟動畫為 150–250ms ease-out，無彈跳。
- [x] 4.3 確認未破壞其他頁面（Dashboard / Settings / Agents）配色（僅 token 連帶變動，版面不改）。
- [x] 4.4 `openspec validate ui-minimal-redesign --strict` 通過。

## 5. 冷灰中性配色與畫布質感（實作迭代）

- [x] 5.1 品牌主色維持 `#2563EB`（dark 提亮 `#3B82F6`）；文字改冷灰中性（去藍味）：light `#1a1d23`/`#656a72`、dark `#e6e8ec`/`#8b9099`。
- [x] 5.2 恢復並改良背景漸層為極淡冷灰光暈：新增 `--gradient-app`（app-shell 冷灰 radial 光暈）、`--gradient-panel-header`（面板 header 用）；app-shell 背景改 `--gradient-app`。
- [x] 5.3 dark 表面色差調整：`--color-surface-app` `#0e1014`、`--color-surface-panel` `#181b22`（保留可辨色差）；新增 `--color-surface-subtle`。
- [x] 5.4 新增自訂 scrollbar（細、冷灰半透明、圓角）：`--color-scrollbar-thumb(-hover)` 兩主題 + 全域 `::-webkit-scrollbar` 與 `scrollbar-width/color`。
- [x] 5.5 sidebar 漸層去藍味改冷灰。

## 6. Agents 頁去卡片化與版面整合（實作迭代）

- [x] 6.1 去卡片：`.agents-matrix-card` / `.agents-sync-report` / `.agents-detail-view` / `.agents-content-pane` / `.agents-content-actions` / `.agents-vscode-list` / `.collapsible-section` 移除邊框/圓角/陰影/背景色塊，改透明 + hairline；清除對應 dark 覆寫。保留 sync modal 內 conflict-dialog/item 與 preview modal 的卡片（浮層需邊界）。
- [x] 6.2 matrix table、compat-note、root-link-banner、各 hover 的硬編碼淺色/藍全 token 化（跨主題自適配）。
- [x] 6.3 `CollapsibleSection` 新增 `titleMeta`（標題旁小字，如設定檔路徑）與 `actions`（標題列操作按鈕）slot；header 由單一 button 改為 flex 容器（toggle button + meta + actions）。
- [x] 6.4 MCP 頁：操作按鈕（外開/資料夾/重整/新增）+ 路徑 meta 移至收折標題列；移除獨立 provider header 行；單一 scope 情境改內嵌操作列（`.mcp-inline-header`）。
- [x] 6.5 AGENTS.md 頁：操作按鈕（重整/外開/資料夾/同步）移至收折標題列；移除多餘 title 行（`.agents-toolbar`）；單一 scope 情境改內嵌操作列（`.agents-inline-header`）。
- [x] 6.6 儲存按鈕改用新增的 `SaveIcon`（取代誤用的 SyncIcon），且僅在編輯狀態顯示。
- [x] 6.7 指示檔樹側欄 `.explorer-panel` / `.explorer-panel-header` 去淺色漸層與 blur，改透明融入畫布（light + dark）。
- [x] 6.8 收折分區移除左縮排：`.collapsible-section-body` 左右 padding 歸零、header 貼齊左緣，標題與展開內容左緣對齊。
- [x] 6.9 Skills/Commands 搜尋框高度對齊 tab（36px），覆寫全域 input `min-height`。
- [x] 6.10 ContentViewer preview modal 修正：改 `flex column` + `overflow:hidden`（滾動條落在圓角內，不溢出）；header 加關閉鈕（`×`）；header 底色去硬編碼淺藍改 `--gradient-panel-header`。
