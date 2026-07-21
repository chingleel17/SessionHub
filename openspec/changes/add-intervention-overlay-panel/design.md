# Design: 介入提醒 Overlay 常駐面板

## Context

`waiting`（等待授權）目前唯一提醒是 Windows Toast（`App.tsx` 於 activity 轉 `waiting` 時 `invoke("send_intervention_notification")`）。Toast 自動消失且落入通知中心，使用者離開座位時會漏接。

已查證的現況：

- **Overlay 視窗**：quota overlay 是獨立 Tauri webview，label 為 `QUOTA_OVERLAY_LABEL = "quota-overlay"`（`lib.rs:28`），以 `index.html?view=quota-overlay` 載入，預設由 `position_window_bottom_right` 定位於螢幕右下（`lib.rs:66`）。已有 `app.get_webview_window(QUOTA_OVERLAY_LABEL).emit("quota-overlay-settings-changed", …)` 的既有跨視窗 emit 模式（`lib.rs:37`）。
- **Overlay 自量測**：`QuotaOverlay.tsx` 已用 `ResizeObserver` + `MutationObserver` 量測 wrapper 的 `scrollWidth/scrollHeight` 並 `getCurrentWindow().setSize(...)` 同步原生視窗（`QuotaOverlay.tsx:146`）。
- **styleMode**：`OverlayStyle = "full" | "compact"`（`types/index.ts:76`）。compact 是一排 chip（使用者實際使用且貼工作列）。
- **設定欄位**：`AppSettings.enable_intervention_notification`（`types.rs:216`）現同時當作 waiting 觸發總開關；前端 `enableInterventionNotification`（`types/index.ts:47`）。
- **通知觸發與導航**：`App.tsx:1743` 於 `status === "waiting" && prev !== "waiting" && enableWaiting` 發 Toast；`notification://action-performed` listener（`App.tsx:1768`）點擊後 `setActiveView(projectKey)` 導航。
- **activity 狀態**：`activityStatusMap` 活在主視窗 React state，overlay 視窗（獨立 webview）無法直接讀取。

跨視窗資料是核心約束：overlay 要在主視窗關閉/最小化時仍能顯示，因此清單須由後端廣播，不能依賴主視窗 state。

## Goals / Non-Goals

**Goals**
- `waiting` 提醒常駐於 overlay，直到該授權被處理（離開 `waiting`）才移除。
- overlay 在主視窗未開啟時仍能顯示提醒（後端為 single source of truth）。
- compact 與 full 兩種 styleMode 都內嵌提醒區；0 筆時整區不佔空間。
- overlay 貼工作列時提醒自動往上長且不推移原 chip 位置；貼頂緣時 fallback 往下。
- 卡片點擊聚焦主視窗並導航到 session。
- waiting Toast 可獨立開關。

**Non-Goals**
- 不改變 `waiting` 訊號的產生方式（沿用 `add-opencode-permission-notification` 的跨 provider 訊號）。
- 不攔截／自動回覆授權。
- 不在提醒區顯示指令、檔案內容或完整路徑（僅工具類型）。
- 不改變 Copilot／Codex／Claude／opencode 既有 bridge 與 activity 行為。
- 不新增獨立提醒視窗（沿用既有 quota-overlay 視窗）。

## Decisions

### 決策 1：後端 `InterventionRegistry` 為 single source of truth，broadcast 給所有視窗

在後端維護一份 `waiting` 清單（key 為 sessionId，value 為 `{ sessionId, projectName, toolLabel, since }`）。activity 狀態進入 `waiting` 時 upsert，離開 `waiting`（active/done/idle 等）時移除；每次變動 emit `intervention-list-changed`（app 級 emit，overlay 與主視窗皆收得到），payload 為當前完整清單快照。

- **替代方案 A**：主視窗把 `activityStatusMap` 的 waiting 子集 `emit_to("quota-overlay", …)`。否決 —— 主視窗關閉/最小化時 state 不更新，overlay 會停在舊清單，違反核心情境。
- **替代方案 B**：overlay 自己 invoke 一個 `get_waiting_list` command 輪詢。否決 —— 輪詢延遲且浪費；既有已是事件驅動（`quota-overlay-settings-changed`），沿用 emit 模式一致性更好。
- **狀態來源**：Registry 的更新點應接在後端 activity 計算之後（`derive_activity_status` / bridge 分支產生 waiting 的同一路徑），而非前端。與 design 慣例「activity 由後端計算」一致。
- **payload 最小化**：僅 sessionId / projectName / toolLabel，不含指令、路徑、resources。projectName 由 session cwd 尾段推導（與 `App.tsx:1740` 同規則）；toolLabel 僅工具類型字串。

### 決策 2：提醒區 render 在 overlay 內，styleMode 各自版式

`QuotaOverlay.tsx` 訂閱 `intervention-list-changed`，以 local state 保存清單，於 quota 內容旁 render `.quota-overlay-intervention` 區塊。0 筆時不 render 該區（既有 `syncWindowSize` 會自動把視窗縮回原尺寸）。

- compact：延續 chip 風格，提醒區為一列 danger 色標題「需授權 (N)」+ 每筆一行 `專案名 · 工具`。
- full：與 quota provider 卡片同排版的區塊。
- 視覺遵循 sessionhub-minimal-ui：danger 色調 token、無新卡片邊框、標題行整合。

### 決策 3：自動延伸方向 —— 量測工作列可用區，優先往下、不足往上、維持 chip 位置

延伸方向在 overlay 前端計算，需三項資訊（Tauri 皆可取）：視窗現位置 `getCurrentWindow().outerPosition()`、目前 monitor 與其**可用工作區**（扣掉工作列）、提醒區量測高度。

判斷：`chip 區底部 Y + 提醒區高度 > 可用工作區底緣` → 往上延伸；否則往下。

「往上延伸」的實作不是移動整個視窗上緣把 chip 頂走，而是：提醒區在 DOM 中排到 chip **上方**（`flex-direction` 反轉或 order），視窗總高增加時**同步將視窗 top 上移等量**，使 chip 那排的螢幕 Y 座標維持不變（貼工作列位置不動）。往下延伸則維持現行「chip 在上、往下長」。

- **可用工作區來源**：優先用 Tauri monitor 的 work area（已扣工作列）；若該版本 API 僅提供 monitor 全尺寸，退而用 monitor size 減去偵測到的 overlay 到底緣距離做保守判斷。實作時以實際可用 API 為準。
- **邊界 fallback**：往上延伸需要的上方空間也不足（overlay 貼頂緣）→ fallback 往下（寧可被工作列擋一部分，也不要超出螢幕頂端不可見）。
- **與既有 syncWindowSize 整合**：延伸方向與視窗 top 調整需與現有 size 同步邏輯合併，避免 `setSize` 與 `setPosition` 互相抖動（同一 effect 內先算好目標 size + position 再一次套用）。
- **替代方案**：固定永遠往上長。否決 —— overlay 放螢幕上半部時會往上超出可視或擋更多畫面；自動判斷才通用。

### 決策 4：卡片點擊複用既有導航路徑

overlay 在獨立 webview，點擊需跨視窗觸發主視窗導航。發一個 app 級事件（如 `intervention-focus-session`，帶 sessionId），主視窗 listener 執行與 `notification://action-performed` 相同的 `setActiveView(getProjectKey(...))` 並聚焦主視窗（`show`/`set_focus`）。

- **替代方案**：overlay 直接 invoke 導航 command。否決 —— 導航是主視窗 React state（activeView），須由主視窗處理；overlay 只負責發意圖事件。

### 決策 5：沿用既有 `enable_intervention_notification` 當總開關，不新增設定項

overlay 提醒與 waiting Toast 講的是同一件事的兩種呈現（Toast=瞬間、overlay=常駐）。因此 overlay 提醒是否顯示 SHALL 沿用既有 `enable_intervention_notification`：關閉時 Toast 與 overlay 皆不出，作為「waiting 介入提醒」的統一總開關。`App.tsx:1743` 的 Toast 觸發維持不動，overlay render 加上同一開關判斷。

- **替代方案 A**：新增獨立 `enable_waiting_toast`，讓使用者只關 Toast 保留 overlay。否決 —— 多一個與 `enable_intervention_notification` 語意重疊的設定項，違反既有 `add-opencode-permission-notification` 的「不新增設定項」精神，且使用者實際需求是「要不要 waiting 提醒」而非「用哪種呈現」。
- **替代方案 B**：overlay 完全獨立於通知開關（只看 quota overlay 開否 + 有無 waiting）。否決 —— 使用者關掉介入通知卻仍看到 overlay 提醒，語意矛盾。
- main 合併後現況已有 `enable_intervention_notification`（waiting，預設 true）與 `enable_session_end_notification`（done，預設 false）；本 change 不動兩者，僅讓 overlay 一併受前者控制。

## Risks / Trade-offs

- **[setSize 與 setPosition 抖動]** 往上延伸同時改 size 與 position，若分兩次非同步套用，透明 webview 可能閃爍或位置跳動。→ 同一 rAF/effect 內先算目標值，一次套用；沿用既有 `refreshTransparentWebview` 的透明刷新時機。
- **[work area API 可用性]** 不同 Tauri 版本對「可用工作區（扣工作列）」的支援不一。→ 以實作時實際可用 API 為準；不可得時用保守估計，並在 spec 以行為（不被工作列擋、不超出螢幕）而非特定 API 描述需求。
- **[多筆 waiting 撐爆 overlay 高度]** 大量 session 同時 waiting 會讓提醒區過高。→ 提醒區設最大高度 + 內部捲動（`overflow-y`），標題仍顯示 (N) 總數；避免頂到螢幕外。
- **[主視窗與 overlay 清單不同步]** 兩者都訂閱同一 `intervention-list-changed`，理論上一致；但主視窗若在 emit 後才啟動會錯過。→ Registry 在主視窗/overlay 建立時補發一次當前快照（或提供 command 供初次拉取）。
- **[projectName 反查失敗]** waiting 事件可能早於 session 掃描到。→ Registry 存 sessionId 即可，projectName 缺失時顯示 sessionId 尾段或 provider 名，不阻擋提醒。

## Migration Plan

1. 後端新增 `InterventionRegistry` 與 `intervention-list-changed` 廣播，接上既有 activity waiting 計算點。
2. `AppSettings` 新增 `enable_waiting_toast`（預設 true）與解析預設；前端型別、SettingsView 開關、locales 對應。
3. `QuotaOverlay.tsx` 訂閱事件、render 提醒區、實作延伸方向與點擊導航；`App.tsx` Toast 觸發改讀新開關並補主視窗 focus 事件 listener。
4. **Rollback**：移除提醒區 render、Registry 廣播與新設定欄位即可回到現況；無資料遷移。

## Open Questions

- 提醒區最大高度與可視筆數上限的具體值（預設先取「不超過可用工作區的一定比例 + 捲動」，實作時定 token）。
