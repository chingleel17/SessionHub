# Design: 狀態列 Quota 彈出面板、Dashboard 全部平台模式與 Codex 重置額度

## Context

- Codex quota adapter（`src-tauri/src/quota/codex.rs`）已會讀取 `auth.json` 憑證並呼叫 `https://chatgpt.com/backend-api/wham/usage` 取得 rate limit windows。codex-reset-checker 證實同一組憑證可呼叫 `GET https://chatgpt.com/backend-api/wham/rate-limit-reset-credits` 取得手動重置額度：`available_count`（可用次數）與 `credits[]`（每筆含 `granted_at` / `expires_at` / `status`）。
- `QuotaSnapshot`（`types.rs`）已有 `windows` / `local_tokens` / `extra_credits` 等 provider 選填欄位，前後端以 serde camelCase 對應，並經記憶體 + SQLite 雙層快取（以 JSON 序列化整個 snapshot），新增選填欄位不需 DB migration。
- 狀態列（`StatusBar.tsx`）的 quota chip 目前是 `<span title=...>` 純顯示；Dashboard 的 `QuotaOverview.tsx` 已是純 props 驅動元件（snapshots + refresh 回呼），tab 選取記憶於 localStorage（key `quota-overview-active-provider`）。
- 進行中的 `tray-quota-widget` 變更做的是系統匣 / 桌面 overlay，與本變更（應用視窗內的狀態列彈窗）互不重疊。

## Goals / Non-Goals

**Goals:**

1. Codex snapshot 附帶重置額度資訊（次數、每筆期限、狀態），並顯示於狀態列 tooltip 與 Dashboard Codex 面板。
2. 點擊狀態列 quota 區域彈出浮動面板，重用 `QuotaOverview` 呈現完整額度資訊。
3. `QuotaOverview` 新增「全部」tab，一次垂直列出所有可見 provider 面板。

**Non-Goals:**

- 不實作「執行重置」動作（只做查詢顯示，與 codex-reset-checker 相同的唯讀原則）。
- 不做系統匣 / overlay 顯示（屬 `tray-quota-widget`）。
- 不新增獨立 Tauri command 或改動 quota refresh 排程；重置額度隨既有 Codex snapshot fetch 一併取得。
- 其他 provider 不套用 reset_credits（欄位維持 `None`）。

## Decisions

### D1. 重置額度資料放在 `QuotaSnapshot.reset_credits` 選填欄位

新增 struct：

```rust
#[serde(rename_all = "camelCase")]
struct ResetCreditEntry {
    granted_at: Option<String>,  // ISO 8601
    expires_at: Option<String>,  // ISO 8601
    status: String,              // API 原始狀態字串（如 "active"）
}

#[serde(rename_all = "camelCase")]
struct ResetCredits {
    available_count: u32,
    credits: Vec<ResetCreditEntry>,
}
```

`QuotaSnapshot` 加 `#[serde(default)] reset_credits: Option<ResetCredits>`。

- 為何不是塞進 `extra_credits`：`ExtraCredits` 語意是金額型 overage（Claude extra_usage），欄位形狀完全不同，硬套會污染兩邊語意。
- 為何不另開 command：重置額度與 usage 同源（同憑證、同 host），跟著 snapshot 走可直接繼承既有的雙層快取、背景輪詢、手動刷新與 SQLite 持久化，前端零新增 IPC。
- `#[serde(default)]` 確保舊 SQLite 快取 JSON 反序列化不會失敗。API 回傳的時間戳為 epoch 秒或 ISO 字串皆轉為 ISO 8601 字串儲存（與 `resets_at` 慣例一致）。

### D2. reset-credits API 失敗採 best-effort，不影響主 snapshot

Codex adapter 在 usage 查詢成功後追加呼叫 reset-credits 端點；該呼叫失敗（4xx/5xx/網路/解析錯誤）時僅記 `eprintln!` 並讓 `reset_credits = None`，snapshot 維持 `status: "ok"`。理由：重置額度是輔助資訊，且該端點為非公開 API，穩定性不可假設；不能讓它拖垮既有的 rate limit 顯示。

### D3. 彈出面板由 StatusBar 自行管理開合、內容重用 QuotaOverview

- `StatusBar` 內加 `isQuotaPopupOpen` state；把右側兩組 quota 區塊外包一層可點擊容器（`<button>`），點擊 toggle。彈出面板以 `position: fixed`（或 absolute 於狀態列容器）錨定右下、狀態列上方，`z-index` 高於主內容。
- 面板內容直接渲染 `<QuotaOverview snapshots={...} onRefreshProvider={...} />`；App.tsx 已把 `quotaSnapshots` 傳給 StatusBar，僅需再把既有的 refresh 回呼下傳。符合「子元件不 invoke()、由 props 驅動」慣例。
- 點外關閉：面板掛載時註冊 document `mousedown` listener（ref 判斷 click target），Escape 鍵亦關閉。不引入 portal 套件，狀態列本身已在 App 根層、無 overflow 裁切問題。
- 替代方案（做成 App 層 dialog、重用 ConfirmDialog 模式）被否決：popup 是狀態列的局部 UI，錨定位置與開合狀態都屬於 StatusBar，放 App 層徒增 props 往返。
- hover tooltip 行為保留不變（原生 `title`），點擊為新增互動。

### D4. 「全部」tab 以 sentinel 值 `"all"` 實作於 QuotaOverview

- tabs 陣列前置一個「全部」tab（i18n key，如 `quota.tab.all`），`activeProvider === "all"` 時渲染 `visible.map(snap => <ProviderPanel .../>)` 垂直堆疊；footer 的刷新按鈕在該模式下改為刷新全部（呼叫 `onRefresh`）。
- localStorage 沿用既有 key，值 `"all"` 合法；provider key 不可能是 `"all"`，無碰撞。
- 彈出面板與 Dashboard 共用元件即自動獲得「全部」模式；為避免兩處互搶記憶，`QuotaOverview` 增加選填 prop `storageKey`（預設維持現值，popup 傳入 `quota-popup-active-provider`）。

### D5. 重置額度顯示位置

- **Dashboard / 彈出面板**：`ProviderPanel` 在 windows 區塊之後、local tokens 之前新增 `qo-reset-credits` 區塊（僅 `snap.resetCredits` 存在時渲染）：一行「重置額度：可用 N 次」，之下每筆 credit 顯示狀態徽章與到期倒數（重用 `formatResetCountdown` / `formatResetDateTime`）。已過期（`expires_at` 早於現在）的條目以低對比顯示。
- **狀態列 tooltip**：`QuotaSnapshotChip` tooltip 追加一行（如「重置額度: 2 次 · 最近到期 07/21 下午11:59」）。

## Risks / Trade-offs

- [非公開 API 變動] `rate-limit-reset-credits` 是 ChatGPT 內部端點，欄位或路徑可能無預警改變 → D2 的 best-effort 策略確保只損失重置額度顯示；解析採寬鬆模式（缺欄位以 default 帶過）。
- [帳號無此功能] 免費或未開通帳號可能回 404/空資料 → 視同無資料（`None`），UI 不渲染該區塊，不顯示錯誤。
- [每次 refresh 多一次 HTTP 呼叫] 增加些微延遲與 API 負載 → 沿用既有 refresh 節奏（背景輪詢 + debounce），不額外輪詢；可接受。
- [popup 與 tooltip 重複] chip 同時有 title tooltip 與點擊 popup，hover 後點擊會先後出現兩層資訊 → 保留 tooltip（低成本快速瞄一眼），popup 供詳細互動；如實測干擾，再於實作時將 chip 的 title 移除（tooltip 資訊已包含於 popup）。
- [「全部」模式面板過長] 5 個 provider 垂直堆疊在 popup 中可能超出視窗高度 → popup 容器設 `max-height` + `overflow-y: auto`（沿用 minimal-ui scrollbar token）。

## Open Questions

- reset-credits 回應中 `expires_at` 的實際格式（epoch 秒 vs ISO 字串）需在實作時以真實回應確認；解析函式同時支援兩者。
