## 1. Rust 後端：型別與 Provider 基礎架構

- [ ] 1.1 在 `SessionInfo` struct 新增 `provider: String` 欄位（`#[serde(default = "default_provider")]`，預設值 `"copilot"`），同步更新所有建構 `SessionInfo` 的位置
- [ ] 1.2 在 `AppSettings` struct 新增 `opencode_root: String`（`#[serde(default)]`，預設 `~/.local/share/opencode/`）和 `enabled_providers: Vec<String>`（預設 `["copilot", "opencode"]`）
- [ ] 1.3 更新 `AppSettings::default()` 方法，填入 `opencode_root` 與 `enabled_providers` 的預設值
- [ ] 1.4 在 `types/index.ts` 前端型別同步新增 `provider` 到 `SessionInfo` 與 `opencodeRoot` / `enabledProviders` 到 `AppSettings`

## 2. Rust 後端：OpenCode Provider 實作

- [ ] 2.1 新增 `default_opencode_root()` 函式，回傳 `%USERPROFILE%\.local\share\opencode\` 路徑
- [ ] 2.2 新增 `open_opencode_db_readonly(opencode_root: &Path)` 函式，以 `SQLITE_OPEN_READ_ONLY` 開啟 `opencode.db`
- [ ] 2.3 新增 `scan_opencode_sessions_internal(opencode_root: &Path, show_archived: bool, metadata_conn: &Connection)` 函式，查詢 OpenCode 的 session + project 表並映射為 `Vec<SessionInfo>`
- [ ] 2.4 實作 OpenCode unix timestamp (ms) → ISO 8601 字串轉換邏輯
- [ ] 2.5 在 `scan_opencode_sessions_internal` 中整合 `read_session_meta`，讀取 metadata.db 的備註/標籤

## 3. Rust 後端：多來源合併

- [ ] 3.1 修改 `get_sessions` command 簽名，新增 `enabled_providers: Vec<String>` 參數
- [ ] 3.2 修改 `scan_sessions` 函式，依 `enabled_providers` 條件呼叫 Copilot 掃描 與/或 OpenCode 掃描
- [ ] 3.3 合併多來源結果後依 `updated_at` 降序排序
- [ ] 3.4 加入容錯處理：OpenCode DB 不存在或 schema 不符時靜默忽略，記錄 eprintln 警告

## 4. Rust 後端：FS Watcher 擴展

- [ ] 4.1 在 `WatcherState` 新增 `opencode: Mutex<Option<RecommendedWatcher>>` 欄位
- [ ] 4.2 新增 `create_opencode_watcher(app, opencode_root)` 函式，監聽 `opencode.db-wal` 變更時發出 `sessions-updated` 事件
- [ ] 4.3 修改 `restart_session_watcher_internal`，依設定同時啟動/停止 OpenCode watcher
- [ ] 4.4 確保 OpenCode provider 停用時釋放 watcher

## 5. 前端：設定頁面擴展

- [ ] 5.1 在 `SettingsView` 新增 OpenCode 路徑設定欄位（含路徑驗證提示）
- [ ] 5.2 在 `SettingsView` 新增 "啟用的平台" 區塊，含 Copilot 與 OpenCode checkbox
- [ ] 5.3 新增翻譯 key：opencodeRoot label、enabledProviders 相關文字
- [ ] 5.4 修改 `App.tsx` 中 `save_settings` mutation，傳送新增的設定欄位

## 6. 前端：Platform Tag 元件

- [ ] 6.1 新增 CSS class `.provider-tag`、`.provider-tag--copilot`、`.provider-tag--opencode`，定義各平台的配色
- [ ] 6.2 在 session card 元件中新增 provider tag 顯示邏輯，位於使用者標籤之前
- [ ] 6.3 確保 provider tag 與 user tag 視覺區隔（實心 vs 外框線）

## 7. 前端：Platform Filter UI

- [ ] 7.1 在 session 列表區域（toolbar 或 sidebar）新增平台篩選 checkbox 群組
- [ ] 7.2 修改 `App.tsx` 中 `get_sessions` 的 useQuery，將 `enabledProviders` 加入 query key 與 invoke 參數
- [ ] 7.3 篩選變更時 invalidate sessions query 觸發重新載入

## 8. 前端：Dashboard 多平台統計

- [ ] 8.1 修改 Dashboard 統計邏輯，依 `provider` 欄位計算各平台 session 數量
- [ ] 8.2 在 Dashboard UI 新增平台分佈顯示區塊

## 9. 測試與驗證

- [ ] 9.1 Rust 單元測試：`scan_opencode_sessions_internal` 對測試用 SQLite 資料庫的讀取與映射
- [ ] 9.2 Rust 單元測試：OpenCode DB 不存在時的容錯行為
- [ ] 9.3 Rust 單元測試：`enabled_providers` 篩選邏輯（僅 copilot / 僅 opencode / 全部）
- [ ] 9.4 前端型別檢查：`bun run build` 通過
- [ ] 9.5 整合驗證：`bun run tauri dev` 啟動後可同時看到 Copilot 與 OpenCode session
