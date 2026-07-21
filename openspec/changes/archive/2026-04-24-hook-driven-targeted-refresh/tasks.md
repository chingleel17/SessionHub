## 1. Rust 後端 - 類型定義

- [x] 1.1 在 `src-tauri/src/types.rs` 新增 `SessionTargetedPayload` struct（含 `session_id: String`, `cwd: String`, `event_type: String`，加上 `#[derive(Serialize, Clone)]` 和 `#[serde(rename_all = "camelCase")]`）
- [x] 1.2 確認 `ProviderBridgeRecord` 的 `cwd` 欄位已存在且為 `Option<String>`（現有，無需新增）

## 2. Rust 後端 - get_session_by_cwd 命令

- [x] 2.1 在 `src-tauri/src/sessions/copilot.rs` 新增 `find_session_by_cwd_internal(copilot_root: &Path, cwd: &str) -> Result<Option<SessionInfo>, String>`，掃描 `session-state/` 目錄，比對 workspace.yaml 中 cwd 欄位（路徑正規化 + 大小寫不敏感）
- [x] 2.2 在 `src-tauri/src/commands/sessions.rs` 新增 `#[tauri::command] get_session_by_cwd(cwd: String, root_dir: Option<String>) -> Result<Option<SessionInfo>, String>`，委派給 `find_session_by_cwd_internal`
- [x] 2.3 在 `lib.rs` 的 `invoke_handler![]` 中登記 `get_session_by_cwd`

## 3. Rust 後端 - 定向事件發送

- [x] 3.1 在 `src-tauri/src/provider/bridge.rs` 新增 `emit_provider_targeted_refresh(app, refresh_state, cwd, event_type) -> Result<bool, String>`：呼叫後端 `find_session_by_cwd_internal` 解析 sessionId，成功時發出 `copilot-session-targeted` 事件（payload: `SessionTargetedPayload`），失敗時 fallback 發出 `copilot-sessions-updated`
- [x] 3.2 修改 `process_provider_bridge_event` in `bridge.rs`：若 record.cwd 有值，嘗試呼叫 `emit_provider_targeted_refresh`；若 cwd 為空，維持原有 `emit_provider_refresh`

## 4. 前端 - 類型定義

- [x] 4.1 在 `src/types/index.ts` 新增 `SessionTargetedPayload` TypeScript interface（`sessionId: string`, `cwd: string`, `eventType: string`）

## 5. 前端 - 事件監聽與定向更新

- [x] 5.1 在 `src/App.tsx` 新增 `copilot-session-targeted` 事件的 `listen` 監聽（在現有 `copilot-sessions-updated` 監聽旁）
- [x] 5.2 在監聽回調中：呼叫 `invoke<SessionInfo | null>("get_session_by_cwd", { cwd: payload.cwd })`
- [x] 5.3 若回傳非 null：使用 `queryClient.setQueriesData(["sessions"], (old) => old 中替換對應 sessionId 的 session)`
- [x] 5.4 若回傳 null：執行 `queryClient.invalidateQueries({ queryKey: ["sessions"] })` fallback 全量刷新

## 6. 測試驗證

- [x] 6.1 在 `src-tauri/src/sessions/copilot.rs` 補充 `find_session_by_cwd_internal` 的單元測試（正常比對、路徑正規化、找不到 session 三個場景）
- [x] 6.2 執行 `cd src-tauri && cargo test` 確認所有 Rust 測試通過
- [x] 6.3 執行 `bun run build` 確認 TypeScript 編譯無錯誤
- [x] 6.4 手動驗證：在有 hook bridge 安裝的環境下，開啟 SessionHub 並在某個 session 中執行 prompt，確認 UI 只更新該 session 而不全量閃爍
