## 1. 後端 InterventionRegistry 與廣播

- [ ] 1.1 在 `src-tauri/src/` 新增 `InterventionRegistry`（`Map<sessionId, { sessionId, projectName, toolLabel, since }>`），置於 AppState 並以 Mutex/RwLock 保護
- [ ] 1.2 定位後端 activity 狀態計算產生 `waiting` / 離開 `waiting` 的路徑（bridge 分支 / `derive_activity_status` 下游），在該處呼叫 Registry 的 upsert / remove
- [ ] 1.3 upsert 時填入最小化欄位：`projectName` 由 session `cwd` 尾段推導（取不到時 fallback sessionId 尾段/provider），`toolLabel` 僅工具類型字串，不寫入指令/路徑/resources
- [ ] 1.4 Registry 變動時 emit app 級 `intervention-list-changed`，payload 為當前清單快照（`#[serde(rename_all = "camelCase")]`）
- [ ] 1.5 提供初次快照機制：新增 command 供視窗查詢當前清單，或在 overlay/主視窗建立時補發一次事件
- [ ] 1.6 為 Registry 的 upsert/remove/快照序列化撰寫 Rust 單元測試

## 2. 前端主視窗調整

- [ ] 2.1 `src/App.tsx` 新增 `intervention-focus-session` listener：收到 sessionId 後執行與 `notification://action-performed` 相同的聚焦 + `setActiveView(getProjectKey(...))` 導航
- [ ] 2.2 確認主視窗聚焦（`show` / `set_focus`）在被其他視窗背景喚起時正常帶到前景
- [ ] 2.3 確認 waiting Toast 觸發（`App.tsx` 既有 `enableInterventionNotification` 判斷）維持不動——不新增 `enable_waiting_toast`，`enable_intervention_notification` 作為 Toast + overlay 的共同總開關

## 3. QuotaOverlay 提醒區

- [ ] 3.1 `src/components/QuotaOverlay.tsx` 訂閱 `intervention-list-changed`，以 local state 保存清單，並在掛載時取得初次快照
- [ ] 3.2 render「需授權」提醒區：標題含總數 (N)，每筆顯示 `專案名 · 工具類型`；清單為 0 筆或 `enableInterventionNotification` 為 `false` 時整區不 render（overlay 提醒與 Toast 共用此總開關）
- [ ] 3.3 compact 與 full 兩種 styleMode 各自版式，視覺遵循 sessionhub-minimal-ui 的 danger 色調 token 與去卡片原則
- [ ] 3.4 提醒區設最大高度 + 內部 `overflow-y` 捲動，避免大量 waiting 撐爆
- [ ] 3.5 卡片點擊 emit `intervention-focus-session`（帶 sessionId）
- [ ] 3.6 新增對應 CSS class 於 `src/App.css` 與 `styles/themes/{dark,light}.css`

## 4. 自動延伸方向

- [ ] 4.1 取得視窗現位置（`outerPosition`）、目前 monitor 與可用工作區（扣工作列，以實作時可用 Tauri API 為準）、提醒區量測高度
- [ ] 4.2 實作方向判斷：`chip 底部 Y + 提醒區高度 > 可用工作區底緣` → 往上，否則往下
- [ ] 4.3 往上延伸：提醒區 DOM 排到 chip 上方（flex order），視窗總高增加時同步將視窗 top 上移等量，使 chip 螢幕 Y 不變
- [ ] 4.4 邊界 fallback：往上空間不足（貼頂緣）時改往下，不使內容超出螢幕頂端
- [ ] 4.5 將方向計算與既有 `syncWindowSize` 合併：同一 effect/rAF 內先算目標 size + position 再一次套用，避免 setSize/setPosition 抖動與閃爍
- [ ] 4.6 手動驗證：overlay 分別貼底、貼頂、置中三種位置，compact 與 full 皆確認延伸方向正確且 chip 不位移

## 5. 驗證與收尾

- [ ] 5.1 觸發某 provider session 進入 waiting，確認 overlay 提醒區即時出現、授權完成後即時消失
- [ ] 5.2 主視窗最小化/關閉狀態下觸發 waiting，確認 overlay 仍顯示提醒（後端 source of truth）
- [ ] 5.3 關閉 `enable_intervention_notification`：確認 Toast 與 overlay 提醒皆不顯示；開啟時兩者恢復
- [ ] 5.4 點擊 overlay 卡片，確認主視窗聚焦並導航至正確 project tab
- [ ] 5.5 執行 `openspec validate add-intervention-overlay-panel --strict` 並修正
- [ ] 5.6 前端 lint/type check 與 `cargo check` 通過
