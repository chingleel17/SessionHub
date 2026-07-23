---
name: sessionhub-minimal-ui
description: SessionHub 專屬的 Minimal UI 設計系統。當你要在 SessionHub 專案中新增或重構任何前端頁面、view、元件、對話框、面板，或調整 CSS / 樣式 / 配色 / 版面時，務必先套用這套設計語言——即使使用者沒有明說「設計」或「minimal」。涵蓋 design token（圓角/邊框/陰影/色彩/漸層/動畫/scrollbar）、冷灰中性配色、去卡片版面原則、收折分區標題行整合、tab/button/list/modal/glass 元件範式。凡碰到 src/App.css、src/styles/themes/*.css、或 src/components/*.tsx 的視覺呈現，都應參照本 skill 以維持全站一致。
---

# SessionHub Minimal UI 設計系統

這套設計語言是 SessionHub 從「傳統管理後台」重構為「專業開發者工具 / AI Agent Workspace」的成果。目標視覺參考 **Linear / Raycast / Arc / Vercel Dashboard / Notion**，而非 Discord / Jira / Azure Portal。

核心優先序（衝突時據此取捨）：

> **高級感 > 華麗感 · 舒適度 > 視覺衝擊 · 長時間使用體驗 > 設計展示效果**

當你為 SessionHub 寫任何 UI，先問自己：這看起來像「一整塊安靜的畫布」，還是「一堆卡片和框線」？答案應該是前者。

## 設計哲學（先理解，再套用）

1. **靠留白、字級、背景色差建立層級——不是靠邊框和卡片。** 傳統後台每個區塊都框起來，資訊被切碎、視覺疲勞。這裡反過來：預設無框，只有真正需要聚焦或浮起來的東西才有邊界。
2. **冷灰中性是基調，藍是點綴。** 整個畫布是極淡的冷灰漸層光暈（Linear/Vercel 調），品牌藍 `#2563EB` 只出現在互動與強調元件（primary 按鈕、active tab 底線、hover）。文字也去藍味，用冷灰。
3. **玻璃感是點綴，不是主體。** 只有浮層（modal / dropdown / floating panel / context menu）用 glassmorphism。Sidebar、card、整頁背景一律不玻璃化。
4. **動效克制。** 150–250ms `ease-out`，用於 hover / tab 切換 / 展開 / modal。不彈跳、不華麗轉場，且尊重 `prefers-reduced-motion`。

## 硬性規範（這些是已定案的，照做以維持一致）

這些值都已定義為 CSS 變數，**永遠用變數，不要寫死顏色或數值**。完整 token 表見 `references/tokens.md`。

- **圓角統一系統**：Button `--radius-button`(10px) · Input `--radius-input`(12px) · Card `--radius-card`(16px) · Modal `--radius-modal`(20px)。不要出現 4/8/24px 混用。
- **邊框**：預設 `border: none`；需要分界時 `1px solid var(--color-border-subtle)`（極淡）。禁止粗框、高對比分隔線。
- **陰影**：一般表面 `var(--shadow-panel)`（`0 2px 12px rgba(0,0,0,0.06)`），浮層 `var(--shadow-float)`。禁止 `0 18px 40px` 這類重陰影。
- **背景**：app-shell 用 `var(--gradient-app)`（冷灰漸層畫布）。面板/浮層 header 可用 `var(--gradient-panel-header)`。避免高飽和裝飾漸層。
- **文字**：冷灰中性 token（`--color-text-primary` / `--color-text-secondary`），不要用帶藍味的舊色。
- **scrollbar**：全站已有自訂樣式（細、冷灰半透明、圓角）。新捲動區自動繼承，不要覆寫成系統預設。

## 去卡片原則（本設計系統最容易做錯的地方）

重構既有頁面時，最常見的錯誤是「保留一堆卡片、只改圓角數值」——這不是 minimal，使用者會立刻看出來。真正的去卡片：

- **容器**（清單、表格、分組、內容區、側欄）：移除 `border` + `border-radius` + `box-shadow` + 背景色塊，改**透明 + 留白 + 極淡 hairline**。
- **列/項目**（session、skill、command、server）：不要每個框起來。改 **row + hover 柔和背景切換**（`var(--color-action-primary-subtle-bg)` 或極淡白/黑），列間用 `border-bottom` hairline 或純靠 hover 區分。對標 GitHub / Linear Issues。
- **分組**（如「專案」「全域」）：不用卡片包，改**標題 + hairline 分界 + 內容**。
- **例外——保留卡片的只有**：modal / dropdown / floating panel / context menu（浮層本該有邊界）。

**深色主題陷阱**：去卡片時最常見的 bug 是硬編碼淺色（`rgba(255,255,255,0.x)`、淺色漸層）在深色下露出白底。**一律用主題 token**，並在深色下實測。若某元件靠 `[data-theme="dark"]` 覆寫才不露白，優先改成 light 基底就用 token（讓深色自動吃 dark token），而非兩邊各維護一份。

## 收折分區的標題行整合（SessionHub 特有範式）

SessionHub 的分區（Agents / MCP 的「專案」「全域」）採一個特定範式，套用其他分區時照此：

- **操作按鈕 + 路徑 meta 放在收折標題行**，不要另設獨立的「標題 + 說明 + 操作」工具列行（那會多一層視覺噪音）。
- 用 `CollapsibleSection` 的 `titleMeta`（標題旁小字，如設定檔路徑，可截斷）與 `actions`（標題列最右操作按鈕）插槽。`actions` 不參與收折點擊。
- **單一 scope（無分組）情境**：改成內容頂部一列內嵌操作列（路徑靠左小字、按鈕靠右），不要退回卡片式工具列。
- 收折時點「新增」類主操作應**自動展開**再執行。

## 套用流程（新頁面或重構既有頁面）

1. **讀 token**：需要具體值時看 `references/tokens.md`，但寫 CSS 時一律引用變數。
2. **判斷卡片政策**：這個容器是「內容」還是「浮層」？內容→去卡片；浮層→保留邊界。參考「去卡片原則」。
3. **套元件範式**：tab、button、list、modal、glass 的標準寫法見 `references/patterns.md`。
4. **雙主題實測**：light 與 dark 都看過，特別檢查深色是否露白、對比是否足夠、hover/active 是否正確。
5. **動效與可及性**：互動加 `--motion-*` 過場；確認 `prefers-reduced-motion` 不被破壞。

## 元件範式速查

詳細寫法（含 code）見 `references/patterns.md`。速記：

- **Tab**：Linear 式。active = 底部 `2px var(--color-action-primary)` 底線；inactive = 無背景、`--color-text-secondary`、hover 轉深。不要 Bootstrap 式帶框 tab。
- **Button**：Primary（filled 藍、`--radius-button`、無框）· Secondary（ghost，透明底 hover 淡背景）· Tertiary（純文字）。減少 outlined button。
- **List**：row + hover，不要卡片堆疊。
- **Modal / Dropdown / Popover**：`var(--glass-bg)` + `backdrop-filter: blur(var(--glass-blur))` + `var(--shadow-float)` + 對應圓角。內容較長時 modal 用 `flex column` + `overflow:hidden`，讓內容區內部滾動（scrollbar 才不會溢出圓角）。header 要有關閉鈕。

## 參考檔案

- `references/tokens.md` — 完整 design token 表（light + dark 所有值，含用途與禁用項）。碰配色/尺寸時查這裡。
- `references/patterns.md` — 元件與版面範式的實際 CSS/JSX 寫法（tab、button、list、modal、glass、去卡片、收折標題行、scrollbar）。寫元件時查這裡。
