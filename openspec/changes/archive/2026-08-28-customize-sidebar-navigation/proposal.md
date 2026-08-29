## Why

目前 Sidebar 的釘選專案無法依使用者工作習慣調整順序，收折控制獨占 Dashboard 上方一整列且未反映目前展開狀態，底部導覽與狀態區也使用過多垂直空間，造成導覽效率與空間利用率不佳。既有已開啟專案已有拖曳基礎，現在可將一致的排序體驗延伸到主要導覽，並一併改善 Sidebar 上下兩端的資訊密度。

## What Changes

- 允許使用者按住並拖曳 Sidebar 的釘選專案，直接調整主要 nav 項目的顯示順序。
- 將釘選順序寫回既有 `pinnedProjects` 設定，重新啟動後仍保留使用者排序。
- 為拖曳中的項目與插入位置提供清楚、符合既有 minimal UI token 的項目層級視覺回饋，不替整個釘選區加框，並避免拖曳誤觸導覽。
- 將收折／展開控制移至品牌列靠 Sidebar 邊界的位置，不再為控制項保留獨立列，讓 Dashboard 向上補回空間。
- 依 Sidebar 當前狀態顯示方向不同的收折或展開圖示，並維持正確的在地化 accessible name、title 與鍵盤操作。
- 壓縮 Sidebar footer 中 Agents、設定、版本號、即時更新時間與刷新操作的列高、內距及區塊間距，使整體垂直占用約縮減一半。
- 同步縮減 Sidebar 底部留白，但維持 footer 資訊可讀、操作可點擊、鍵盤 focus 可辨識，且不改變既有功能或顯示內容。
- 限制 Sidebar 專案名稱可占寬度，空間不足時優先將過長專案名稱顯示為省略號，為分支名稱保留至少可完整顯示 `master` 的寬度。
- 較長分支名稱超過保留寬度時才截斷為省略號，並維持完整「專案 · 分支」內容可由既有 tooltip 取得。
- 在 Dashboard 下方的主要專案分隔線整合低對比 `+ New` 入口，文字尺寸與分支標籤一致，避免新增獨立工具列或重複按鈕。
- 點擊入口後開啟專案選擇 modal，列出所有目前已偵測、尚未釘選且尚未開啟的專案，並讓使用者選擇直接開啟或加入釘選區。
- 候選專案清單提供名稱、分支與路徑資訊，以及沒有可加入專案時的空狀態；本次不支援選取任意未偵測資料夾。
- 維持展開與收折共用同一套 Sidebar DOM、既有 icon 軸對齊、雙主題與 reduced-motion 行為。

## Capabilities

### New Capabilities
- `sidebar-navigation-order`: 定義釘選專案 nav 的拖曳排序、排序持久化、視覺回饋及可用專案變動時的順序維護。
- `sidebar-footer-density`: 定義 Sidebar footer 導覽、版本與即時狀態資訊的緊湊版面、可讀性及操作性。
- `sidebar-project-label-layout`: 定義 Sidebar 專案名稱與分支名稱的寬度分配、截斷優先序及完整資訊存取方式。
- `sidebar-project-picker`: 定義 Sidebar 分隔線新增入口、可加入專案的篩選規則、modal 清單及開啟／釘選行為。

### Modified Capabilities
- `sidebar-collapse`: 將收折控制整合至品牌列靠 Sidebar 邊界，依展開狀態切換圖示，並移除原本要求控制項獨占固定列的行為。

## Impact

- 前端：`src/components/Sidebar.tsx` 的釘選項目拖曳互動、專案／分支標籤結構、新增專案入口、收折控制結構與 icon；`src/App.tsx` 的候選專案篩選、modal 狀態、開啟／釘選 action、釘選順序更新及設定持久化 callback；新增純顯示的專案選擇 dialog。
- 樣式：`src/App.css` 的品牌列、收折控制定位、Sidebar menu 與 footer 間距、專案／分支寬度分配、釘選拖曳狀態、底部留白，以及展開／收折與響應式版面。
- 圖示與翻譯：`src/components/Icons.tsx` 可能需匯出方向明確的 panel 與新增 icon；補齊 `src/locales/` 的收折／展開、專案選擇與空狀態文案。
- 設定：沿用 `AppSettings.pinnedProjects` 的既有有序陣列與 `save_settings` IPC，不新增資料庫 schema、Tauri command 或第三方相依套件。
- Tauri 視窗：主視窗停用未使用且會攔截 WebView HTML 拖曳事件的原生檔案拖放，以確保 Windows 桌面版的 Sidebar 排序可啟動。
- 驗證：前端 lint/build，並以元件或互動測試覆蓋拖曳排序、持久化 payload、候選專案篩選、modal 操作、狀態圖示、footer 緊湊版面與操作可及性。
