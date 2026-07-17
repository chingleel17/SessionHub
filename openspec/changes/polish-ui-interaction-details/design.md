## Context

`standardize-frontend-ui-foundation` 已建立 `Button`、`IconButton`、`Select` 共用元件與 `ProviderIcon`，但實際體驗仍有五處缺口：

1. `ProviderIcon`（`src/components/ProviderIcon.tsx`）以 `background: currentColor` 的 16px 圓形搭配 8px 縮寫呈現，縮寫文字色為 `--color-surface-panel`，在 provider-tag 的配色下幾乎不可見，看起來像每個標籤前多了一個空白圓點。
2. 設定頁 quota 監控卡片的「立即刷新」仍是 `ghost-button` 文字按鈕（`SettingsView.tsx:667`），與 Dashboard（`QuotaOverview`）、Tray（`TrayQuotaPanel`）已採 IconButton 的刷新操作不一致。
3. `App.css` 中 `.ui-button:hover` 只有單一規則，且大量既有按鈕（`ghost-button` 等）尚未遷移到 `ui-button` 體系，導致多數按鈕、select、checkbox 沒有 hover 回饋、沒有過渡動畫，外觀平面。
4. `.ui-button:hover` 統一改背景為 `--color-action-primary-subtle-bg`（淺藍），primary/danger 的白字在淺藍底上對比不足。
5. 側欄收折：`grid-template-columns` 從 280px 直接跳到 88px 無過渡；收折按鈕在展開時位於品牌區右端、收折時因 `.sidebar-brand` 改為 column 而垂直堆疊，位置跳動；收折後 icon 水平對齊與展開時不一致。

## Goals / Non-Goals

**Goals:**
- 修正 ProviderIcon 的可辨識性，消除「空白圓點」觀感。
- 全站互動元件（button/select/checkbox）具備一致、可辨識的 hover、active、focus-visible 與過渡。
- primary/danger hover 維持文字對比（同色系加深/提亮，不換成淺色底）。
- 側欄收折平滑化：寬度過渡、收折按鈕固定位置、icon 對齊一致。

**Non-Goals:**
- 不重做整體視覺風格或配色系統（遵循既有 sessionhub-minimal-ui token）。
- 不引入 UI framework 或動畫函式庫。
- 不變更任何後端、IPC 或資料行為。
- 不處理行動版（<900px）側欄行為的重新設計，僅維持現狀不退化。

## Decisions

### D1. ProviderIcon 改為高對比縮寫 badge
維持縮寫方案（不引入品牌 SVG，避免授權與維護成本），但修正配色：改用 provider 專屬 accent（沿用 `--color-provider-<name>-bg/text` token）作為底色、以高對比文字呈現 1–2 字縮寫，字級提高至可讀（≥9px），並確保 dark/light 兩主題皆可辨識。若 provider-tag 本身已含文字 label，icon 與文字之間維持 4px 間距。
替代方案：直接移除 icon——但 icon 在收折/緊湊視圖仍有辨識價值，保留並修好較划算。

### D2. 立即刷新改用既有 IconButton + RefreshIcon
`SettingsView` 的 quota 手動刷新改為 `<IconButton label={t("quota.monitoring.manualRefresh")}>` 搭配 `Icons.tsx` 的 refresh 圖示（若尚無則新增線性 refresh icon），tooltip 沿用既有 i18n key，無需新增文案。

### D3. hover/active 以 token 化過渡統一實作
- 在 `ui-button` 基礎上加入 `transition: background var(--motion-fast), border-color var(--motion-fast), color var(--motion-fast)`。
- variant 專屬 hover：
  - primary/danger：`filter: brightness()` 或預先定義的 `--color-action-primary-hover` / error hover token（優先用 token，缺少時新增），文字維持白色。
  - secondary/ghost：維持 subtle 背景 + 邊框強調。
- 加入 `:active` 輕微下沉回饋（如 `transform: translateY(1px)` 或加深背景），並在 `prefers-reduced-motion` 下停用 transform 與 transition。
- Select/checkbox：統一 hover 邊框色與 focus-visible outline；checkbox 改用 accent-color 或自訂樣式，加 hover 邊框回饋。
- 盤點 `App.css` 中殘留的 `ghost-button` 等舊按鈕 class，逐一改為 `ui-button ui-button--ghost`（或對應 variant），舊 class 移除或別名到新規則，避免雙軌。

### D4. 側欄收折動畫與固定收折按鈕
- `app-shell` 的 `grid-template-columns` 加上 `transition`（約 200ms ease），內部文字以 `opacity` 淡出避免壓縮變形；`prefers-reduced-motion` 下停用。
- 收折按鈕從 `.sidebar-brand` 內移出，改為側欄內固定位置元素（建議：側欄頂部獨立列或絕對定位於側欄右上角，展開/收折時座標不變）。展開時不再貼在品牌 icon 旁。
- 收折狀態下品牌 icon 與 `.sidebar-link` icon 使用相同的水平置中基準（同寬容器置中），消除切換時的位移。
- 文字隱藏改以 `opacity + width/visibility` 過渡取代瞬間 `display: none`，或至少讓 icon 欄位寬度在兩狀態一致，避免 icon 跳位。

### D4a. 收折/展開共用單一 DOM 與固定 icon 軸（實作後修正）

首版實作在收折時由 React 換為另一套 DOM（40px 置中的 `sidebar-icon-button`、footer quick-actions），並以 `align-items: center` 置中——但 class 翻轉是瞬間的、側欄寬度動畫是漸進的，導致元素先跳到「還很寬的側欄」中央再隨寬度收回（水平漂移），底部即時狀態綠點也因 label 僅 `opacity: 0` 仍佔位、`.realtime-dot` 無 `flex-shrink: 0` 而被壓縮消失。修正為：

- 兩種狀態共用同一套 DOM 與版面：導覽、釘選、已開啟項目、footer 均固定用 `sidebar-link`，收折只由 CSS 淡出文字，不換元件、不置中。
- 收折寬度改為 80px（側欄 padding 14 + 品牌 icon 52 + 14），使既有 icon 軸（中心 x=40px）收折後天然置中，全程零水平位移；收折按鈕 `left: 24px`、即時狀態綠點以 `padding-left: 21px` 對齊同一軸線。
- `sidebar-link` 內距改為 `margin-left: 2px + padding: 0 12px`（合計維持 icon 軸 x=40），讓收折時 52px 最小內容寬對側欄 1px 右邊框留有餘裕，避免右緣被 `overflow: hidden` 裁切。
- 釘選項目兩態統一為「首字母 icon + 右上角 pin 小徽章」；已開啟項目的關閉鈕收折時改為右上角浮動小圓鈕（hover 顯示）；「全部關閉」按鈕收折時淡出僅留分隔線；Dashboard 與釘選區之間的分隔線兩態皆顯示。
- 取捨：收折狀態下底部刷新鈕淡出不可用（Dashboard 仍有刷新入口），釘選項目展開時視覺由 pin icon 改為首字母。

## Risks / Trade-offs

- [`ui-primitives` spec 尚未歸檔] `standardize-frontend-ui-foundation` 已完成但未 archive，`openspec/specs/ui-primitives` 尚不存在 → 本 change 的 delta 以 ADDED 補充新要求而非 MODIFIED 既有要求，避免歸檔順序造成衝突；建議先歸檔前一個 change。
- [grid-template-columns transition 效能] 寬度動畫會觸發 layout → 範圍僅限側欄一欄、200ms 短動畫，實測若掉幀改為 `width` + `transform` 方案。
- [舊 button class 遷移範圍大] 一次全改可能波及未預期畫面 → 逐檔遷移並以 `bun run lint` + 手動走查主要畫面驗證；必要時保留舊 class 為別名一版。
- [收折按鈕移位屬版面變更] 使用者已習慣現位置 → 依需求方（使用者）明確要求「固定在相同位置、展開時不用往上到 icon 旁邊」執行，風險可接受。
