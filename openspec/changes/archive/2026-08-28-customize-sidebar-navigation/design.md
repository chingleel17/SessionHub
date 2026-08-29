## Context

`Sidebar.tsx` 已使用原生 HTML drag-and-drop 排列未釘選的 `openProjectKeys`，也允許把未釘選項目拖入釘選區；釘選項目則依 `pinnedProjects` 的陣列順序顯示，但本身不是 drag source 或排序 target。`App.tsx` 擁有釘選狀態並透過既有 `save_settings` IPC 將整個 `AppSettings` 寫入 `%APPDATA%\SessionHub\settings.json`，因此排序不需要新資料模型或 command。

目前收折按鈕是 Sidebar 內的 absolute element，固定在品牌列下方，並由 `.sidebar-menu` 的額外 top margin 騰出一列；展開與收折都渲染同一個 `PanelLeftIcon`。Footer 則由兩個共用 `sidebar-link` 導覽列、版本列與即時狀態列組成，疊加 44px 導覽列高、40px 刷新按鈕、8px grid gap、多處 padding 及 Sidebar 24px 底部 padding，形成過大的垂直占用。專案標籤目前讓專案名稱與分支都可收縮，但未替分支設定最低保留寬度，因此長專案名稱會把分支擠到只剩數個字元。實作必須維持子元件不直接呼叫 `invoke()`、所有文案經 i18n、一般操作 icon 由 `Icons.tsx` 匯出，以及 SessionHub minimal UI 的 token、雙主題與 reduced-motion 規範。

## Goals / Non-Goals

**Goals:**
- 在不增加第三方套件的前提下，延伸既有拖曳模型支援釘選專案排序。
- 讓 `App.tsx` 成為排序持久化與錯誤回復的唯一業務邏輯擁有者，`Sidebar` 僅回報新的可見排序。
- 保留暫時無法對應到 `projectGroups` 的既有釘選 key，避免掃描或 provider 狀態短暫變動造成偏好遺失。
- 將收折控制收進品牌列的尾端空間，移除 Dashboard 前的人為留白，且兩種 Sidebar 寬度下皆可操作。
- 將 footer 整體高度壓縮至約既有占用的一半，同時保留 Agents、設定、完整版本號、即時更新資訊與刷新功能。
- 建立穩定的專案／分支寬度分配，讓分支至少能完整顯示 `master`，並優先截斷過長專案名稱。
- 提供單一、低干擾的專案新增入口，讓使用者從目前已偵測但未出現在 Sidebar 的專案中選擇開啟或釘選。

**Non-Goals:**
- 不讓 Dashboard、全域 Agents、設定或 footer 項目參與自訂排序。
- 不持久化未釘選的已開啟專案順序；其既有 session 內排序行為維持不變。
- 不新增觸控專用拖曳手勢、鍵盤式清單重排模式或第三方 drag-and-drop 套件。
- 不改變 Sidebar 展開／收折狀態是否跨重啟保存。
- 不隱藏、合併或移除 footer 既有資訊與操作，也不改變即時狀態資料更新邏輯。
- 不增加 Sidebar 寬度、不改變專案或分支實際名稱，也不新增第二行標籤。
- 不允許挑選任意檔案系統資料夾，也不建立沒有偵測 session 的虛擬專案。
- 不在釘選與未釘選兩條分隔線重複顯示相同新增入口，不新增候選專案的持久化 catalog。

## Decisions

### 1. 沿用原生 drag-and-drop，但明確標記拖曳來源

將 Sidebar 的拖曳狀態由單一 key 擴充為可區分 `pinned` 與 `open` 來源的狀態，drop handler 依來源執行「釘選內排序」或既有「拖入釘選區」行為。釘選項目本身成為 draggable，並沿用目前以複製陣列、移除再插入的不可變更新方式。

選擇此作法是因為專案已有可工作的原生拖曳流程與樣式，新增套件會放大變更與 bundle 成本。替代方案是導入支援觸控與鍵盤排序的完整 DnD library，但本次需求只涵蓋 Windows 桌面滑鼠操作，屬於不必要擴張。

Tauri 在 Windows 預設啟用的原生檔案拖放會攔截 WebView2 的 HTML drag-and-drop 流程，而 SessionHub 主視窗目前沒有使用原生檔案拖放事件。因此主視窗設定 `dragDropEnabled: false`，讓 `draggable`、`dragstart`、`dragover` 與 `drop` 由 Sidebar 的 WebView 實作處理；未來若要加入外部檔案拖放，必須先設計不會破壞專案排序的替代整合方式，不得直接重新啟用此設定。

拖曳視覺回饋只套用於被拖曳項目及目標項目的插入位置，不替 `.sidebar-pinned-section` 增加 outline 或整區背景。整區外框除了增加不必要的視覺噪音，也會因 `.sidebar-menu` 的水平裁切而缺邊；若未來調整回饋樣式，仍須維持項目層級且不得超出 Sidebar 邊界。

### 2. Sidebar 回報可見釘選順序，App 重建完整設定順序

`Sidebar` 根據 `visiblePinnedGroups` 產生新的可見 key 順序並透過 `onReorderPinnedProjects` 回報。`App.tsx` 以目前完整的 `pinnedProjects` 為基底，只替換其中屬於可見專案的槽位；無法對應到當前 `projectGroups` 的 key 保留在原槽位，因此不會因一次暫時不完整的 session 掃描被刪除。

替代方案是直接以可見順序覆蓋 `pinnedProjects`，實作較短但會默默刪除暫時不可用的釘選偏好，與既有載入時保留未知 key 的行為不一致。

### 3. 排序採樂觀更新，儲存失敗時回復

`App.tsx` 在有效順序變更時先更新 `pinnedProjects`，再使用既有 settings payload 與 `save_settings` 儲存。成功後同步 settings query；失敗則回復排序前陣列並呼叫既有 toast 錯誤管道。無有效位移時 callback 不執行，避免多餘磁碟寫入。

替代方案是等待儲存完成後才更新畫面，但拖曳完成會有可感延遲；只做樂觀更新而不回復則會讓畫面順序與重啟後設定不一致。

### 4. 收折控制與品牌列共用垂直空間

控制項以品牌列尾端、靠 Sidebar 邊界的緊湊 icon button 呈現，定位基準跟隨 Sidebar 邊界，兩種寬度下皆保留完整 hit target；`.sidebar-menu` 移除目前僅為控制項保留的 top margin。控制項使用既有 surface、border、radius 與 motion token，不引入獨立卡片或高對比裝飾。

選擇品牌列而非 Dashboard active 邊框，是因為收折是整體版面控制，不應成為 Dashboard 專屬操作；使用者切換到專案、Agents 或設定頁後仍應在一致位置找到它。替代方案是放入 workspace header，但會把 Sidebar 的控制責任移入每個內容頁共同 header，並增加版面耦合。

### 5. 依狀態切換方向明確的集中式 icon

`Icons.tsx` 匯出所需的 panel-open/panel-close 或左右方向 icon，`Sidebar` 依 `isSidebarCollapsed` 選擇代表下一步動作的圖示；同一條件也選擇現有 `sidebar.expand`／`sidebar.collapse` 文案。按鈕 DOM 保持同一個 element，只替換 icon，以維持鍵盤焦點與連續操作。

替代方案是旋轉同一個 `PanelLeftIcon`，但該圖示的差異不夠明確且容易與「目前狀態」而非「下一步動作」混淆。

### 6. 以 footer 專屬緊湊尺寸取代全域縮小 Sidebar link

為 `.sidebar-footer` 範圍內的導覽列、版本列、即時狀態列與刷新按鈕設定專屬緊湊尺寸，搭配縮小 grid gap 與 Sidebar 底部 padding，讓 footer 的總垂直占用接近原本的一半。導覽 icon 與文字維持現有語意層級，狀態點仍對齊 Sidebar icon 軸；壓縮以移除多餘留白為主，不使用 transform scale，避免模糊文字、錯位 hit area 或 focus outline。

選擇 footer scope 覆寫而非直接縮小 `.sidebar-link` 與 `.sidebar-icon-button` 全域規則，是為了不影響 Dashboard、釘選專案、已開啟專案及其他共用 icon button。替代方案是改寫 JSX 為全新 compact 元件，但目前 footer 結構足以透過語意 class 精準調整，新增元件會造成不必要分支。

### 7. 分支取得最小保留寬度，專案名稱吸收主要壓縮

讓 `.sidebar-pinned-item-label` 成為可伸展且可收縮的主要欄位，並以 `min-width: 0`、單行與 ellipsis 吸收空間不足；`.sidebar-branch-label` 設定至少等同 `master` 的字元寬度並保留自身 ellipsis，使短分支完整顯示、長分支在保留區內截斷。外層 label wrap 繼續單列 overflow hidden，既有按鈕 title 提供完整專案與分支內容。

選擇 CSS flex 寬度分配而非依字串長度在 React 計算，是因為實際寬度受字型、縮放、語系與 Sidebar 狀態影響，瀏覽器布局能更準確處理。替代方案是替分支配置固定較大寬度，但會讓沒有分支或極短分支的專案名稱浪費空間。

### 8. 單一分隔線入口開啟 App 層級專案選擇 dialog

在 Dashboard 下方建立固定存在的 project divider，線段尾端放置低對比 `+ New` 按鈕；展開時文字使用與 branch label 相同的 12px 層級，收折時只保留 `+` icon 與 tooltip。入口只出現一次，已開啟區段原有分隔線與全部關閉操作維持原功能，避免兩條線各自重複新增按鈕。

`Sidebar` 僅透過 `onRequestProjectPicker` 通知 `App.tsx`。App 以 `groupedProjects` 過濾掉 `pinnedProjects` 與 `openProjectKeys`，將候選清單傳給新的純顯示 `ProjectPickerDialog`；dialog 使用既有 backdrop/card、focus 與關閉慣例，掛載於 App 的 dialog 區而非 Sidebar 內，避免 Sidebar overflow 裁切。

選擇 App 層級 dialog 是因為候選篩選、開啟 view 與釘選持久化均屬業務邏輯，符合 IPC 與狀態集中於 App 的架構。替代方案是在 Sidebar 內以 portal 顯示 modal，雖可避開 clipping，但會讓 Sidebar 同時負責候選資料與跨區段 action。

### 9. 候選來源限於目前已偵測的 project groups

候選專案直接來自現有 `groupedProjects`，因此每筆都有可導覽的 project key、title、branch、path 與 sessions。點擊「開啟」沿用 `openProjectTab` 並切換 active view；點擊「釘選」沿用既有釘選持久化流程，加入釘選順序末端。任一成功 action 後關閉 dialog；候選為空時顯示在地化空狀態。

不提供任意資料夾 picker，因為未被 sessions 偵測的路徑沒有 `ProjectGroup`，若要支援會需要虛擬專案型別、獨立持久化與空專案畫面，超出這次 Sidebar 導覽改善範圍。

## Risks / Trade-offs

- [原生 HTML drag-and-drop 對觸控與純鍵盤排序支援有限] → 本次明確以 Windows 桌面滑鼠為範圍；按鈕導覽與收折控制仍保持完整鍵盤可用性。
- [重新啟用 Tauri 原生檔案拖放會使 Sidebar HTML 拖曳失效] → 主視窗維持 `dragDropEnabled: false`；新增外部檔案拖放前須另行設計並回歸驗證專案排序。
- [釘選區整體 drop target 樣式可能被捲動容器裁切] → 僅使用項目層級的拖曳與插入位置提示，不對整個釘選區繪製外框或背景。
- [釘選與未釘選共用 drop zone 可能觸發錯誤行為] → 使用含來源類型的拖曳狀態，所有 drag end、drop 與取消路徑集中清除狀態。
- [快速連續排序可能造成非同步儲存回應順序競爭] → 排序儲存採序列化或以最新請求識別，只允許最新操作的失敗回復影響目前畫面。
- [品牌列尾端控制在 80px 收折寬度下空間有限] → 實作與驗證時以完整 hit target、不遮蔽品牌 icon 與不越界為優先；必要時讓控制貼齊內側邊界，而非縮小到不可操作尺寸。
- [移除 menu top margin 會改變 Sidebar 垂直密度] → 同時檢查 expanded、collapsed、窄視窗及長專案清單，確認 Dashboard 與 footer 沒有碰撞或裁切。
- [接近 50% 的 footer 壓縮可能降低點擊舒適度] → 優先移除 gap、padding 與空白，再縮小列高；以鍵盤 focus、滑鼠點擊及文字截斷實測確認仍可用，不以視覺縮放達成目標。
- [footer 專屬覆寫可能被後段 collapsed 規則蓋過] → 將 compact 與 collapsed 樣式一併檢查，確保版本號、狀態點與隱藏文字的既有行為不退化。
- [替分支保留寬度會使更多專案名稱提早截斷] → 僅保留約 `master` 所需的最小寬度，分支不存在時不配置該欄位，並以既有 tooltip 提供完整標籤。
- [候選專案很多時 modal 清單可能超出視窗] → dialog 內容區設定最大高度與內部捲動，header、空狀態及操作維持在容器內。
- [快速點擊專案 action 可能重複開啟或釘選] → action 開始後關閉或暫停 dialog 操作，並由既有陣列去重與釘選儲存流程保護最終狀態。
- [Sidebar 收折時 `+ New` 文字沒有空間] → 收折狀態只顯示 `+` icon，保留在地化 accessible name 與 tooltip，不新增額外列。

## Migration Plan

不需要資料遷移。既有 `pinnedProjects` 本來就是有序陣列，升級後直接將目前陣列順序視為初始自訂順序；舊版也能讀取排序後的同一欄位。若需回退應用程式版本，設定檔仍維持相容，只會失去 UI 重新排序能力。
