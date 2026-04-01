## Context

SessionHub 是 Tauri 2 + React 桌面應用，前端狀態集中在 `App.tsx`，子元件為純 props 驅動。本次修正五個 UX 問題：

1. `ProjectView` 目前以 `key={activeProject.key}` 強制重新 mount，導致其內部 `openPlanKeys`/`activeSubTab` state 每次切換專案都被重置。
2. 切換專案時 `sessionsQuery` 與 `sessionStatsQueries` 尚未回傳時缺乏視覺回饋。
3. Plan sub-tab 的可關閉 div 容器缺少對應 CSS，外觀破版。
4. App icon 仍為舊版 CS 圖示，尚未更換為 SessionHub 品牌。
5. `plan-editor-layout` 雙欄高度不一致，textarea 高度未撐滿。

## Goals / Non-Goals

**Goals:**
- Plan tab 開啟狀態跨專案切換後不遺失（應用程式未關閉期間）
- 切換專案時顯示 skeleton loading，告知使用者資料載入中
- Plan sub-tab 可關閉標籤外觀正確（與普通 tab 視覺一致，右側有 × 符號）
- 更新 `src-tauri/icons/` 所有尺寸 icon 為 SessionHub 品牌設計（深底白 S）
- Plan 編輯器原始欄與預覽欄等高

**Non-Goals:**
- 不新增 IPC command
- 不改動 Rust 後端
- 不引入 CSS 框架
- 不處理 app icon 以外的品牌資源（splashscreen、tray icon 除外，icon 本體已足夠）

## Decisions

### D1：plan tab state 提升方式

**決策**：在 `App.tsx` 新增 `projectSubTabStates: Map<string, { openPlanKeys: string[]; activeSubTab: string }>` state。`ProjectView` 改為「受控」模式，接收 `openPlanKeys`、`activeSubTab`、`onSubTabStateChange` props。移除 `ProjectView` 上的 `key={activeProject.key}`（或改為只以 session 列表資料為 key 的 reconcile 方式）。

**捨棄的替代方案**：
- 保留 `key` 強制 mount + 用 `useRef` cache — 複雜且違反 React 慣例
- 使用全域 context — proposal 明確禁止引入新全域狀態管理庫

**注意**：移除 `key` 後 ProjectView 不再重新 mount，因此切換專案時內部的 `searchTerm`、`sortKey` 等 state 也會保留。這是預期行為（使用者重回專案時搜尋條件仍在）。

### D2：Loading 樣式策略

**決策**：在 `ProjectView` 新增 `sessionsLoading` prop（bool）。當為 true 時，sessions 列表區域顯示 3 個骨架卡片（`<div className="skeleton-card" />`），透過 CSS 動畫模擬載入感。不依賴任何第三方 skeleton 套件。

**捨棄的替代方案**：
- spinner 置中 — 骨架比 spinner 更能預告版面，減少版面跳動感（CLS）

### D3：Plan sub-tab 樣式

**決策**：`.sub-tab-item--closeable` 改為 `display: inline-flex; align-items: center; gap: 4px`，子元素 `.sub-tab-label` 繼承文字樣式，`.sub-tab-close` 為小型透明按鈕（`16×16`、`opacity: 0.5` hover 上升至 `1`）。此結構沿用現有 top-level `tab-item` + `tab-close` 的視覺模式。

### D4：App Icon

**決策**：使用 SVG 產生 PNG（32×32, 128×128, 256×256, 512×512）並用 `cargo-tauri icon` 指令批量轉換。設計：`#1a2744` 深藍背景，圓角矩形，白色粗體「S」置中。`.ico` 由 Tauri 工具自動從 PNG 生成。

### D5：Plan 編輯器等高

**決策**：`.plan-editor-layout` 設 `display: grid; grid-template-columns: 1fr 1fr; align-items: stretch`，並加上 `min-height: 400px`。`.plan-textarea` 設 `height: 100%; resize: none; box-sizing: border-box`。`.plan-preview` 設 `overflow-y: auto`。

## Risks / Trade-offs

- **[D1 風險] 移除 ProjectView key 後 searchTerm 跨專案保留** → 此為可接受的正向副作用，使用者搜尋條件被保留；若未來需重置可在 `activeProject` 變更時呼叫 reset 函數。
- **[D1 風險] onSubTabStateChange 頻繁觸發** → 只在 openPlanKeys 或 activeSubTab 實際變更時呼叫，不在每次 render 觸發。
- **[D4 風險] Icon 更新後需重新 build 才生效** → 文件說明需要 `bun run tauri build`，開發模式下 icon 不會熱重載。
- **[D5 trade-off] textarea 固定高度** → 移除 `resize: vertical` 改為等高 grid，使用者無法自由調整高度；但雙欄等高的一致性比自由拖拉更重要。
