## 1. 階段一：排序與搜尋範圍調整

- [x] 1.1 `src/types/index.ts` 的 `SortKey` union 移除 `"summaryCount"`
- [x] 1.2 `ProjectView.tsx` 的 `filterAndSortSessions` 排序 switch 移除 `case "summaryCount"`
- [x] 1.3 排序選單移除「依 summary 數」`<option>`
- [x] 1.4 兩份 locale 移除 `session.sortSummaryCount` 鍵值
- [x] 1.5 `filterAndSortSessions` 的搜尋 haystacks 移除 `session.cwd`
- [x] 1.6 兩份 locale 更新 `session.searchPlaceholder`，移除「路徑」字樣

## 2. 階段一：自訂日期區間

- [x] 2.1 `SessionUpdatedRange` 型別擴充 `"custom"`
- [x] 2.2 `getUpdatedRangeStart` 改為回傳 `{ start, end }` 上下界，並更新 `isSessionInUpdatedRange` 的判斷
- [x] 2.3 新增 `customRangeStart` / `customRangeEnd` state 與更新時間選單的 `<option value="custom">`
- [x] 2.4 選為 `custom` 時渲染兩個 `<input type="date">`，切離時收合並清除
- [x] 2.5 實作區間邊界：起始日 00:00 至結束日 23:59:59；單一端點時只套用該側界限
- [x] 2.6 起訖顛倒時顯示提示並維持前一次有效結果，不套用無效區間
- [x] 2.7 兩份 locale 新增自訂區間相關文案（選項名、起訖標籤、顛倒提示）

## 3. 階段一：chip 文案與空對話預設值

- [x] 3.1 `App.tsx` 的 `hideEmptySessions` state 更名為 `showEmptySessions`，預設值改為 `false`（即預設隱藏空對話）
- [x] 3.2 `ProjectView` 的 prop 與 callback 同步更名為 `showEmptySessions` / `onShowEmptySessionsChange`
- [x] 3.3 `filterAndSortSessions` 的空 session 判斷反轉為 `if (!showEmpty && !session.hasEvents) return false`
- [x] 3.4 chip 文案改為「空對話」，啟用狀態代表「顯示空對話」
- [x] 3.5 `hiddenCount` 提示改為在 chip **未**啟用時顯示被隱藏筆數
- [x] 3.6 已封存 chip 文案由「顯示已封存 session」改為「已封存」（僅文案，不動 `showArchived` 持久化行為）
- [x] 3.7 兩份 locale 更新 `project.showArchivedToggle`、`session.filter.hideEmpty`（改為 showEmpty）與 `session.filter.hiddenCount`

## 4. 階段一：工具列排版

- [x] 4.1 搜尋框加寬（提高 flex 比例與 minWidth）
- [x] 4.2 `.filter-bar` 加上 `flex-wrap: wrap` 與適當 row gap，允許窄視窗換行
- [x] 4.3 確認日期輸入展開後工具列不壓縮其他控制項至不可用寬度

## 5. 階段一：驗證

- [x] 5.1 執行 `bun run lint` 與 `tsc` 確認無型別與 lint 錯誤
- [x] 5.2 手動驗證：排序選單只剩三項、搜尋路徑不再命中、自訂區間篩選正確、空對話預設隱藏且點擊 chip 後顯示
- [x] 5.3 確認篩選條件變更時分頁重置回第 1 頁仍正常

## 6. 階段二：Rust 內容擷取

- [x] 6.1 `sessions/mod.rs` 新增 `extract_session_texts(provider, session_dir) -> Vec<String>` 泛用分派函式
- [x] 6.2 `sessions/claude.rs` 實作擷取：逐行解析 `.jsonl`，取 `entry_type` 為 `user`/`assistant` 且 `is_meta != Some(true)` 的訊息文字
- [x] 6.3 `sessions/codex.rs` 實作擷取：逐行解析 `.jsonl` 的對話訊息文字
- [x] 6.4 `sessions/copilot.rs` 實作擷取：讀取 `events.jsonl` 的對話訊息文字
- [x] 6.5 `sessions/antigravity.rs` 實作擷取：讀取 `transcript.jsonl` 的對話訊息文字
- [x] 6.6 確認 opencode SQLite 訊息內容的實際表與欄位（design.md 的 Open Question），實作 `sessions/opencode.rs` 的擷取查詢
- [x] 6.7 各實作皆排除工具呼叫參數與檔案內容快照，只取對話文字
- [x] 6.8 各實作皆採逐行串流解析，不整檔讀入記憶體
- [x] 6.9 各實作皆不將逐字稿內容寫入 `metadata.db`、寫入檔案或存放於跨呼叫快取（決策七硬性約束）

## 7. 階段二：搜尋指令

- [x] 7.1 定義 `SessionSearchTarget` struct（`id`、`provider`、`session_dir`），採 `#[serde(rename_all = "camelCase")]`
- [x] 7.2 `commands/sessions.rs` 新增 `search_session_content(query, sessions) -> Result<Vec<String>, String>`
- [x] 7.3 實作不分大小寫子字串比對，命中即提前結束該 session 掃描
- [x] 7.4 空白或全空白關鍵字直接回傳空清單，不讀取任何檔案
- [x] 7.5 單一 session 讀取或解析失敗時略過並繼續，不使整個指令失敗
- [x] 7.6 `lib.rs` 註冊新 command
- [x] 7.7 指令回傳值只含 session ID，不夾帶任何訊息文字
- [x] 7.8 新增 Rust 單元測試：關鍵字命中/未命中、空關鍵字、檔案不存在時略過、大小寫不敏感
- [x] 7.9 新增 Rust 測試：執行搜尋後 `metadata.db` 的資料表列數與檔案大小不因搜尋而改變

## 8. 階段二：前端整合

- [x] 8.1 `App.tsx` 新增 `search_session_content` 的 invoke 包裝並以 prop 傳入 `ProjectView`（子元件不直接 invoke）
- [x] 8.2 `ProjectView` 新增 `searchInContent` 開關 state 與工具列 UI
- [x] 8.3 新增 `contentMatchIds: Set<string> | null` state；`null` 代表不參與篩選
- [x] 8.4 實作 300ms debounce 呼叫，並以 request id 或 AbortController 捨棄在途的過期回應
- [x] 8.5 `filterAndSortSessions` 新增 `contentMatchIds` 參數，關鍵字判斷改為 `matchesTextFields || contentMatchIds?.has(session.id)`（OR 而非 AND，啟用開關只擴大命中範圍），維持同步純函式
- [x] 8.6 送入指令的範圍：已通過 provider / tag / 空對話 / 日期區間篩選的 session，包含那些**未**通過文字欄位比對者（正是內容搜尋要撈回的目標）
- [x] 8.7 搜尋進行中顯示 loading 指示並保留前一次結果，避免列表閃爍為空
- [x] 8.8 關閉開關時將 `contentMatchIds` 設回 `null` 並捨棄在途結果
- [x] 8.9 指令錯誤時以 showToast 顯示訊息，列表退回僅套用同步篩選的結果
- [x] 8.10 內容搜尋開關變更時一併重置分頁至第 1 頁
- [x] 8.11 兩份 locale 新增內容搜尋開關、loading 與錯誤文案

## 9. 階段二：驗證

- [x] 9.1 執行 `cargo test` 確認 Rust 測試通過
- [x] 9.2 執行 `bun run lint` 與 `tsc` 確認無型別與 lint 錯誤
- [x] 9.3 手動驗證：對各 provider 的 session 以已知對話關鍵字搜尋，確認命中正確
- [x] 9.4 手動驗證：連續輸入時不逐鍵觸發掃描、關閉開關立即回到同步搜尋
