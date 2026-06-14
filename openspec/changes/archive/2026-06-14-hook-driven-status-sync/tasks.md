## 1. 後端：擴充 claude-activity-hint 事件 payload

- [x] 1.1 在 `src-tauri/src/types.rs` 的 `ActivityHintPayload`（或對應結構）中新增 `status`、`detail`、`last_activity_at` 欄位
- [x] 1.2 在 `src-tauri/src/watcher.rs`（或 bridge 處理邏輯）中，根據 hook eventType 計算並填入 `status` / `detail`（對應表：SessionStart/UserPromptSubmit→active+thinking；PreToolUse→active+tool_call；PostToolUse→active+working；Stop+normal→idle；Stop+error/interrupt→waiting）
- [x] 1.3 新增 `check_jq_available` Tauri command（`src-tauri/src/commands/tools.rs`），執行 `jq --version`，回傳 `bool`
- [x] 1.4 在 `src-tauri/src/lib.rs` 的 `invoke_handler` 中註冊 `check_jq_available`

## 2. 後端：sh hook scripts 新增

- [x] 2.1 新增 `.claude/hooks/modules/db-ops.sh`，實作 `invoke_with_retry`（bash retry loop）
- [x] 2.2 新增 `.claude/hooks/modules/task-queue.sh`（輕量佔位模組）
- [x] 2.3 新增 `.claude/hooks/modules/record-event.sh`，實作 `read_hook_payload`（stdin → 變數）、`get_hook_string_value`（jq 查詢）、`write_bridge_event_record`（jq 組 JSON 後 append 到 bridge 檔案），開頭加 jq 可用性偵測
- [x] 2.4 新增 `.claude/hooks/on-session-start.sh`，解析參數、呼叫 `write_bridge_event_record`，eventType="session.started"
- [x] 2.5 新增 `.claude/hooks/on-pre-tool-use.sh`，eventType="tool.pre"
- [x] 2.6 新增 `.claude/hooks/on-post-tool-use.sh`，eventType="tool.post"
- [x] 2.7 新增 `.claude/hooks/on-user-prompt-submit.sh`，eventType="prompt.submitted"，截斷 prompt 至 80 字元
- [x] 2.8 新增 `.claude/hooks/on-stop.sh`，eventType="session.stop"

## 3. 後端：claude.rs 整合 sh scripts

- [x] 3.1 在 `src-tauri/src/provider/claude.rs` 中以 `include_str!` 嵌入 8 個新 .sh 檔案（3 個模組 + 5 個入口腳本）
- [x] 3.2 更新 `hook_script_entries()` 陣列，加入 8 個 sh 檔案的 (路徑, 內容) 項目
- [x] 3.3 新增 `render_claude_hook_command_sh` 函式，產生 `sh '<path>/on-xxx.sh' --bridge-path '<path>' --provider claude` 格式的指令
- [x] 3.4 修改 `managed_hook_group`：`command` 欄位改用 sh 指令，`commandWindows` 欄位保留現有 ps1 指令
- [x] 3.5 將 `HOOK_SCRIPT_VERSION` 從 `"1"` 升為 `"2"`，確保重新安裝時會覆寫舊版腳本

## 4. 前端：activityStatusMap 事件驅動更新

- [x] 4.1 在 `src/App.tsx` 中找到 `claude-activity-hint` 事件監聽，擴充處理邏輯：若 payload 含 `status` 欄位，呼叫 `setActivityStatusMap`（或 queryClient patch）直接更新對應 sessionId 的 entry
- [x] 4.2 建立 `activityStatusMap` 的可寫狀態（改為 `useState<Map<string, SessionActivityStatus>>`），或透過 `queryClient.setQueryData` patch `activityStatusQuery` 的 cache

## 5. 前端：移除輪詢、改事件驅動

- [x] 5.1 在 `src/App.tsx` 的 `activityStatusQuery` 定義中，依 Claude integration 安裝狀態動態設定 `refetchInterval`：`installed` 時為 `false`，否則為 `30_000`
- [x] 5.2 確保 `activityStatusQuery` 保留 `refetchOnMount: true`，以取得初始狀態

## 6. 前端：統一狀態計數來源

- [x] 6.1 重構 `src/App.tsx` 中 `activeSessions/waitingSessions/idleSessions/doneSessions` 的 `useMemo`：移除 `sessionStatsMap.isLive` 判斷與 `updatedAt` 時間差計算，改為 `activityStatusMap.get(s.id)?.status` 映射
- [x] 6.2 `doneSessions` 計數維持以 `s.isArchived` 判斷（不依賴 activityStatusMap）
- [x] 6.3 驗證首頁看板欄位數量與底部狀態列數量一致

## 7. 前端：jq 偵測 UI

- [x] 7.1 在 `src/App.tsx` 新增 `jqAvailableQuery`（呼叫 `check_jq_available`，`staleTime: Infinity`，`gcTime: Infinity`），在設定頁面可見時才 enable
- [x] 7.2 將 `jqAvailable` 透過 props 傳入 `SettingsView`
- [x] 7.3 在 `src/components/SettingsView.tsx` 的 Claude integration 區塊下方，當 `jqAvailable === false` 時顯示 info banner
- [x] 7.4 在 `src/locales/zh-TW.ts` 與 `src/locales/en-US.ts` 新增 `settings.jqNotFound.title`、`settings.jqNotFound.body`、`settings.jqNotFound.winget` 等 i18n key
