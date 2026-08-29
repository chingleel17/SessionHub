## 1. 釘選排序狀態與持久化

- [x] 1.1 在 `Sidebar` props 與 `App` 串接 `onReorderPinnedProjects`，以不可變陣列更新重建完整 `pinnedProjects` 順序並保留不可見 key；以可見與不可見 key 混合案例驗證重建結果及其相對順序。
- [x] 1.2 實作釘選排序的樂觀更新、既有 `save_settings` 持久化及失敗回復／toast，並防止快速連續儲存的舊回應覆蓋最新順序；透過模擬成功、失敗及連續排序驗證畫面與儲存 payload 一致。

## 2. Sidebar 拖曳互動

- [x] 2.1 將 Sidebar 拖曳狀態改為可區分 `pinned` 與 `open` 來源，讓釘選項目成為 draggable 並支援釘選區內排序；驗證有效 drop、原位 drop、目標外取消與 active view 不切換。
- [x] 2.2 保留未釘選已開啟專案的既有排序與拖入釘選區行為，並讓新釘選項目加入順序末端；驗證兩種拖曳來源不會互相觸發錯誤 handler。
- [x] 2.3 加入釘選拖曳中與插入位置的 token-based 樣式，且 drag end/drop 後完整清除狀態；在展開、收折、淺色、深色與 reduced-motion 模式檢查回饋可辨識且無版面跳動。
- [x] 2.4 在 Windows Tauri 主視窗停用會攔截 WebView HTML drag-and-drop 的原生檔案拖放，並移除釘選區整體 drop-target 外框與背景；保留項目拖曳狀態及插入位置提示，確認回饋不超出 Sidebar 或遭裁切。

## 3. 收折控制版面與圖示

- [x] 3.1 將收折／展開按鈕整合至品牌列靠 Sidebar 邊界的尾端，移除 `.sidebar-menu` 為獨立控制列保留的上方空間；在 280px 展開與 80px 收折寬度驗證按鈕不遮蔽品牌、導覽或 workspace。
- [x] 3.2 透過 `Icons.tsx` 集中匯出方向明確的收折／展開 icon，依 `isSidebarCollapsed` 渲染代表下一步動作的圖示；驗證 icon、既有 i18n title 與 accessible name 在兩種狀態同步切換。
- [x] 3.3 調整品牌列與控制項的 hover、focus-visible、transition 及窄視窗樣式，全部使用既有 minimal UI token；以鍵盤連續切換並檢查焦點保留、雙主題對比及 reduced-motion 行為。

## 4. Footer 緊湊版面

- [x] 4.1 為 `.sidebar-footer` 範圍內的 Agents、設定導覽列、版本列、即時狀態列與刷新按鈕套用專屬緊湊尺寸，並縮減 footer gap 與 Sidebar 底部 padding；以變更前後量測確認 footer 總垂直占用接近縮減一半且不影響主要 nav。
- [x] 4.2 調整緊湊 footer 在 expanded、collapsed 與窄視窗下的對齊、截斷、focus-visible 及邊界行為；在淺色與深色主題驗證版本、狀態點、時間、刷新 icon 均可辨識且無重疊或裁切。
- [x] 4.3 走查 Agents、設定與刷新操作，確認壓縮後的滑鼠及鍵盤操作、導覽結果、即時狀態與最後同步時間行為皆與變更前一致。

## 5. 專案與分支標籤寬度

- [x] 5.1 調整 `.sidebar-group-label-wrap`、`.sidebar-pinned-item-label` 與 `.sidebar-branch-label` 的 flex、最小寬度及 ellipsis 規則，讓專案名稱優先截斷並為分支保留至少完整 `master` 的寬度；以長專案搭配 `main`、`master` 與長 feature 分支逐一驗證。
- [x] 5.2 驗證無分支專案可使用完整剩餘寬度，且釘選與已開啟未釘選項目套用相同截斷規則；hover 每個截斷案例確認既有 tooltip 顯示完整專案與分支。
- [x] 5.3 在 expanded、collapsed、淺色、深色與窄視窗布局檢查標籤不換行、不溢出、不遮蔽 pin badge 或關閉按鈕。

## 6. Sidebar 專案選擇器

- [x] 6.1 在 Dashboard 下方主要分隔線新增低對比 `+ New` 入口，展開時使用分支文字尺寸、收折時只顯示 `+` icon，並確保多條分隔線時只存在單一入口；驗證 expanded、collapsed 與 pinned/open 區段組合。
- [x] 6.2 在 `App.tsx` 管理 project picker 開關狀態，從 `groupedProjects` 排除 `pinnedProjects` 與 `openProjectKeys` 後傳入 dialog；以全部可用、部分排除、全部排除與零專案案例驗證候選結果。
- [x] 6.3 新增純顯示 `ProjectPickerDialog`，沿用既有 dialog token 與結構顯示專案名稱、分支、路徑、空狀態及可捲動清單，補齊 zh-TW／en-US 文案；在雙主題與大量候選資料下驗證版面。
- [x] 6.4 串接候選專案「開啟」與「釘選」action，分別沿用 `openProjectTab` 與釘選持久化流程，成功後關閉 modal 且不產生重複項目；驗證 active view、排序末端與設定 payload。
- [x] 6.5 完成 dialog 的初始焦點、Tab 操作、Escape／backdrop／關閉按鈕與焦點回復；以純鍵盤走查新增入口、兩種 action 及取消流程。

## 7. 整合驗證

- [x] 7.1 執行 `bun run lint` 與 `bun run build`，修正所有本次變更新增的 lint、TypeScript 或 Vite build 錯誤並確認指令成功。
- [x] 7.2 在 Windows Tauri 開發版以多個釘選、已開啟與候選專案走查排序、拖入釘選區、project picker、取消拖曳、收折／展開、footer 緊湊版面、長專案／分支截斷及專案導覽；確認拖曳可正常啟動、釘選區不顯示整區外框或裁切回饋，重新啟動後釘選順序保留，並模擬設定儲存失敗確認回復與 toast。
