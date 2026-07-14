# Tasks: 狀態列 Quota 彈出面板、Dashboard 全部平台模式與 Codex 重置額度

## 1. 後端型別與 Codex 重置額度查詢

- [x] 1.1 在 `src-tauri/src/types.rs` 新增 `ResetCredits` / `ResetCreditEntry` struct（serde camelCase），並為 `QuotaSnapshot` 加上 `#[serde(default)] reset_credits: Option<ResetCredits>`；修正所有既有 `QuotaSnapshot { ... }` 建構處（各 quota adapter 填 `None`）與相關測試 fixture
- [x] 1.2 在 `src-tauri/src/quota/codex.rs` 實作 `fetch_reset_credits()`：以既有憑證呼叫 `GET https://chatgpt.com/backend-api/wham/rate-limit-reset-credits`（帶 Authorization 與選填 ChatGPT-Account-Id header），解析 `available_count` 與 `credits[]`（granted_at / expires_at / status，時間戳支援 epoch 秒與 ISO 字串，統一輸出 ISO 8601）
- [x] 1.3 在 `fetch_snapshot()` usage 成功路徑追加 best-effort 呼叫：失敗僅 `eprintln!` 並保持 `reset_credits: None`，不改變 status 與既有欄位；空 credits 或 404 視為無資料
- [x] 1.4 新增 Rust 單元測試：reset-credits 回應解析（正常、時間戳兩種格式、缺欄位寬鬆解析、空清單），以及舊快取 JSON（無 resetCredits 欄位）反序列化成功
- [x] 1.5 `cargo test` 全綠

## 2. 前端型別與 QuotaOverview 擴充

- [x] 2.1 在 `src/types/index.ts` 新增 `ResetCredits` / `ResetCreditEntry` 型別，`QuotaSnapshot` 加 `resetCredits?: ResetCredits | null`
- [x] 2.2 `QuotaOverview.tsx`：`ProviderPanel` 在 windows 區塊後新增 `qo-reset-credits` 區塊（可用次數、每筆狀態徽章 + 到期倒數與絕對時間，重用 `formatResetCountdown` / `formatResetDateTime`；已過期條目低對比；`resetCredits` 為 null 時不渲染）
- [x] 2.3 `QuotaOverview.tsx`：新增「全部」tab（sentinel `"all"`，i18n key `quota.tab.all`，置於 provider tabs 之前）；選取時垂直列出所有 visible provider 的 `ProviderPanel`，footer 刷新改為呼叫 `onRefresh`（全域刷新）；記憶值失效回退第一個 provider；單一 provider 時維持不顯示 tabs
- [x] 2.4 `QuotaOverview.tsx`：新增選填 prop `storageKey`（預設 `quota-overview-active-provider`），tab 記憶讀寫改用該 key
- [x] 2.5 在 `src/locales/` 各語系檔新增文案 key（全部 tab、重置額度標題、可用次數、到期、已過期、tooltip 摘要等）
- [x] 2.6 `App.css` 新增 `qo-reset-credits` 相關樣式與「全部」模式堆疊間距（遵循 sessionhub-minimal-ui token）

## 3. 狀態列彈出面板與 tooltip

- [x] 3.1 `StatusBar.tsx`：quota 區域改為可點擊 button 容器，新增 `isQuotaPopupOpen` state 與 toggle；保留既有 chip 視覺與 hover tooltip
- [x] 3.2 `StatusBar.tsx`：實作彈出面板容器（fixed 錨定右下、狀態列上方、`max-height` + 內部捲動、z-index 高於主內容），內部渲染 `<QuotaOverview storageKey="quota-popup-active-provider" />`，snapshots 與 refresh 回呼由 App 經 props 下傳（不新增 IPC）
- [x] 3.3 `StatusBar.tsx`：點外關閉（document mousedown + ref 判斷）與 Escape 關閉，unmount 時清除 listener
- [x] 3.4 `StatusBar.tsx`：`QuotaSnapshotChip` tooltip 在 Codex snapshot 含 `resetCredits` 時追加重置額度摘要行（可用次數 + 最近到期時間）
- [x] 3.5 `App.tsx`：將既有 quota refresh handler 下傳給 StatusBar；`App.css` 新增面板樣式（glass/panel 範式、動畫遵循 minimal-ui）

## 4. 驗證與收尾

- [x] 4.1 `npm run build`（tsc + vite）與 `cargo test` 全數通過
- [x] 4.2 實機驗證：狀態列點擊開合面板、點外/Escape 關閉、面板與 Dashboard tab 記憶互不干擾、「全部」tab 顯示所有平台、Codex 面板與 tooltip 顯示重置額度（含無資料帳號不顯示）
- [x] 4.3 `openspec validate quota-popup-and-codex-reset-credits` 通過
