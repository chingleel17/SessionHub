## Context

Dashboard 現有 7 張獨立 `stat-card`，每張都有 label + value 的兩行結構，造成統計區域垂直高度過大。Token/互動次數統計直接對所有 session 加總，session 數量多時成本高但對用戶參考價值低。Tab header 在首頁只顯示 title 一行，而專案頁顯示 title + path 兩行，切換時 header 高度跳動。Sidebar 收合時版本號 `div` 直接消失（被 `!isSidebarCollapsed` 包裹邏輯省略）。`SessionStatsBadge` 的時長直接輸出 `durationMinutes` 整數並附上分鐘單位文字，未做小時換算。

## Goals / Non-Goals

**Goals:**
- 統計卡片改為單列 stat bar，加入 icon，大幅縮減垂直高度
- Token/互動統計新增本周 / 本月切換，只計算符合時間範圍的 session
- Tab header 高度在首頁與專案頁之間保持一致，消除切換抖動
- Sidebar 收合時 footer 顯示縮短版本號（tooltip 顯示完整）
- `durationMinutes >= 60` 時顯示為小時（`1.5h`），否則顯示分鐘（`45m`）

**Non-Goals:**
- 不新增後端 API，時間範圍篩選完全在前端計算
- 不改動 session 查詢邏輯或 React Query key 結構
- 不修改 `SessionStats` 後端 struct（`duration_minutes` 欄位保持不變）
- 不改動 tab 的開關邏輯或路由結構

## Decisions

### 1. 時間範圍篩選在前端計算

**決策**：period 切換（week/month）在 `App.tsx` 計算，filter session 的 `updatedAt` 欄位後加總 stats。

**理由**：sessions 已在 React Query cache 中，不需要額外 IPC；`get_session_stats` 每個 session 各自快取，加一層 filter 成本可忽略。

**備選方案**：新增後端 command 接受 `since` 參數 → 需要改動 Rust、新增 IPC，複雜度不合比例。

### 2. stat bar 樣式取代 stat-card grid

**決策**：將 stats 改為水平 `stat-bar` 容器，每個項目為 `stat-bar-item`（icon + value + label 垂直排列），以 `flex` 均分。

**理由**：水平排列可在同一行顯示 6–7 個指標，垂直高度從約 160px 降至約 60px。

**備選方案**：縮小現有卡片字體 → 仍佔兩行，效果有限。

### 3. Tab header 高度統一

**決策**：首頁 `workspace-subtitle` 固定顯示 `t("dashboard.subtitle")`（一行說明文字）；專案頁 `workspace-subtitle` 顯示路徑但改用 `font-size: 0.72rem` 且單行截斷（`text-overflow: ellipsis`）。CSS 確保兩者 `workspace-header` 最小高度一致。

**理由**：最小侵入性修改，不影響 tab 路由邏輯。

### 4. Sidebar 收合版本號

**決策**：`sidebar-version` div 移出 `!isSidebarCollapsed` 條件，收合時顯示 `v{major}.{minor}` 縮短版本（`title` 屬性顯示完整）。

**理由**：版本號是除錯必要資訊，不應在收合時完全消失。

### 5. 時長換算邏輯

**決策**：在 `SessionStatsBadge` 中新增 `formatDuration(minutes: number): string`，`>= 60` 換算為 `Xh` 或 `X.Xh`，否則輸出 `Xm`。翻譯 key `stats.duration` 保留（單位由函式輸出決定，不再從 i18n 取單位）。

**理由**：邏輯簡單，不需後端改動，翻譯影響最小。

## Risks / Trade-offs

- [period 篩選依賴 `updatedAt` 字串比對] → ISO 8601 字串直接比較即可（字母序 = 時間序），低風險
- [stat bar 在視窗極窄時可能擠壓] → 加 `min-width` 與 `overflow-x: auto` fallback
- [首頁副標題文案需翻譯] → 新增 `dashboard.subtitle` 翻譯 key，zh-TW / en-US 各補一條
