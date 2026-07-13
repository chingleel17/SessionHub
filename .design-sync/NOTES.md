# design-sync 筆記（SessionHub）

- SessionHub 是 Tauri 應用程式，不是元件庫：沒有 dist build，轉換器以 synth-entry 模式從 `src/` 合成 entry。
- 所有 `src/components/*.tsx` 都是 prop 驅動、無 Tauri IPC 依賴；context 只需 `I18nProvider`（`src/i18n/I18nProvider.tsx`）與 `ThemeProvider`（`src/theme/ThemeProvider.tsx`），皆為純瀏覽器程式碼，經 `extraEntries` 併入 bundle、`cfg.provider` 串接（I18n 外層、Theme 內層）。
- 同步範圍（使用者於 2026-07-12 確認）：18 個展示型元件 + 23 個 icon（皆出自 `Icons.tsx`）；大型 view（DashboardView、ProjectView、PlansSpecsView、AgentsConfigView、McpConfigView、SettingsView、Sidebar、SessionTodosTab、ProjectAnalyticsTab、PlanEditor）以 `componentSrcMap: null` 排除。
- CSS 入口：`src/App.css`（131KB，含全部元件樣式），開頭 `@import` `src/styles/themes/light.css` 與 `dark.css`（design token；dark 以 `[data-theme="dark"]` 切換）。
- 套件管理器：bun（root 有 `bun.lock`）。
- i18n 預設 locale 由 `navigator.language` 決定，headless chromium 下為 en-US。
- `node_modules/session-hub` 是指向 repo root 的 junction（Windows：`New-Item -ItemType Junction`）——synth-entry 模式需要它，且讓預覽的 `import from 'session-hub'` 可解析。fresh clone / bun install 後若消失需重建。
- CSS 走 `.design-sync/ds-styles.ts`（extraEntries）讓 esbuild inline App.css 的 theme @import；不要用 `cssEntry`（會原樣 append，留下無法解析的相對 @import → [CSS_IMPORT_MISSING]）。
- render check 用系統 Chrome：`DS_CHROMIUM_PATH="/c/Program Files/Google/Chrome/Application/chrome.exe"`（playwright 以 PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD=1 安裝在 .ds-sync）。

## 預覽撰寫要點（wave learnings 收攏）

- DropdownMenu 的 `trigger` 是 render-prop `({ ref, onClick, open }) => ReactNode`（非 ReactNode）；開啟狀態可在預覽中用 useRef 存 onClick + useEffect 模擬點擊來靜態截圖。選單項 class：`dropdown-menu-item`（`--default` 粗體、`:disabled`）。
- App.css 有 `@media (max-width: 900px)` 使 `.dialog-actions` 直向堆疊——dialog 類 760px viewport 下按鈕直排是真實響應式行為，非樣式缺失。
- TrendChart 系列實際渲染為「填滿多邊形」而非描邊線：`.trend-chart-series--*` 的 `fill` 在 `.trend-chart-path { fill: none }` 之後、同 specificity 而勝出——production 同樣如此，可能是上游 app bug（已知，非同步缺陷）。共用 y 軸使 interaction/cost 系列被 token 量壓平，亦為真實行為。
- 圖表色票：trend colorClass 用 `trend-chart-series--primary/secondary/accent`；pie 用 app 的 PIE_COLORS（#6366f1 #14b8a6 #f59e0b #ef4444 #8b5cf6 #06b6d4）。
- SessionStatsPanel 的模型成本區僅在 `provider === "copilot"` 且 modelMetrics 非空時渲染。
- StatusBar chips 渲染條件：provider 在 `quotaEnabledProviders` 且（providerQuotas 有非零用量，或 snapshot `status:"ok"` 且 source 為 remote_api/antigravity）；antigravity chip 只顯示 group 含 "gemini" 的 window。
- QuotaOverview 是 tab 式——每個 cell 只看得到 active provider；error/no_auth 狀態要獨立 export。
- ProjectStatsBanner 對空 sessions 回傳 null——別寫空列表 story；loading 軸用 `sessionStatsLoading`。
- ExplorerTree：群組節點 `defaultOpen: true` 才會展開入鏡；葉節點要有 `filePath` 才可選取。
- 預覽中相對時間用 `Date.now() ± offset` 計算，倒數/幾分鐘前標籤在任何擷取時間都合理。

## Known render warns

- `[TOKENS_MISSING]` 12 個 CSS 變數（--color-accent、--color-border-default、--color-surface-card、--color-surface-hover、--radius-lg、--color-error-text、--color-text-muted、--radius-sm 等）：App.css 有引用但 repo 從未定義，實際 app 也一樣靠 var() 後備值運作——上游現實，非同步缺陷。
- `[FONT_MISSING]` Inter / Fira Code / Cascadia Code：SessionHub 的字型棧是系統字型設計（`"Segoe UI", Inter, Arial, sans-serif`、mono 有 `monospace` 後備），repo 不含任何字型檔或 @font-face；接受系統字型替代。

## Re-sync risks（下次同步注意）

- `node_modules/session-hub` junction 不在版控內：fresh clone 或 `node_modules` 重建後會消失，重建指令見上方筆記；缺了會 `ENOENT .../node_modules/session-hub/package.json`。
- 預覽中的假資料（SessionInfo/QuotaSnapshot/BridgeEventLogEntry 等）是手寫快照：`src/types/index.ts` 型別加欄位不會壞（皆 optional 居多），但若必填欄位變更，對應 preview 會編譯失敗、退回 floor card——build log 找 `! preview build failed`。
- `sessionhub-minimal-ui` skill 是設計語言的另一份真相；若該 skill 更新 token/範式，conventions.md 需同步校驗。
- TrendChart「填滿多邊形」行為可能是上游 bug（`.trend-chart-series--*` 的 fill 蓋掉 `fill: none`）；若上游修掉，TrendChart 預覽外觀會改變並觸發 re-verify——屬預期。
- 大型 view（DashboardView 等 10 個）以 `componentSrcMap: null` 排除；若日後要納入，需 mock Tauri 資料流，成本高。
- 驗證用瀏覽器是系統 Chrome（`DS_CHROMIUM_PATH`），Chrome 大版本更新可能使截圖像素微變，出現 [SPOT_CHECK] 時確認 sheet 即可。
