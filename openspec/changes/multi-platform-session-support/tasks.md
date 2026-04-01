## 1. Rust 後端：型別與 Provider 基礎架構

- [x] 1.1 在 `SessionInfo` struct 新增 `provider: String` 欄位（`#[serde(default = "default_provider")]`，預設值 `"copilot"`），同步更新所有建構 `SessionInfo` 的位置
- [x] 1.2 在 `AppSettings` struct 新增 `opencode_root: String`（`#[serde(default)]`，預設 `~/.local/share/opencode/`）和 `enabled_providers: Vec<String>`（預設 `["copilot", "opencode"]`）
- [x] 1.3 更新 `AppSettings::default()` 方法，填入 `opencode_root` 與 `enabled_providers` 的預設值
- [x] 1.4 在 `types/index.ts` 前端型別同步新增 `provider` 到 `SessionInfo` 與 `opencodeRoot` / `enabledProviders` 到 `AppSettings`

## 2. Rust 後端：OpenCode Provider 實作

- [x] 2.1 新增 `default_opencode_root()` 函式，回傳 `%USERPROFILE%\.local\share\opencode\` 路徑
- [x] 2.2 新增 `open_opencode_db_readonly(opencode_root: &Path)` 函式，以 `SQLITE_OPEN_READ_ONLY` 開啟 `opencode.db`
- [x] 2.3 新增 `scan_opencode_sessions_internal(opencode_root: &Path, show_archived: bool, metadata_conn: &Connection)` 函式，查詢 OpenCode 的 session + project 表並映射為 `Vec<SessionInfo>`
- [x] 2.4 實作 OpenCode unix timestamp (ms) → ISO 8601 字串轉換邏輯
- [x] 2.5 在 `scan_opencode_sessions_internal` 中整合 `read_session_meta`，讀取 metadata.db 的備註/標籤

## 3. Rust 後端：多來源合併

- [x] 3.1 修改 `get_sessions` command 簽名，新增 `enabled_providers: Vec<String>` 參數
- [x] 3.2 修改 `scan_sessions` 函式，依 `enabled_providers` 條件呼叫 Copilot 掃描 與/或 OpenCode 掃描
- [x] 3.3 合併多來源結果後依 `updated_at` 降序排序
- [x] 3.4 加入容錯處理：OpenCode DB 不存在或 schema 不符時靜默忽略，記錄 eprintln 警告

## 4. Rust 後端：FS Watcher 擴展

- [x] 4.1 在 `WatcherState` 新增 `opencode: Mutex<Option<RecommendedWatcher>>` 欄位
- [x] 4.2 新增 `create_opencode_watcher(app, opencode_root)` 函式，監聽 `opencode.db-wal` 變更時發出 `sessions-updated` 事件
- [x] 4.3 修改 `restart_session_watcher_internal`，依設定同時啟動/停止 OpenCode watcher
- [x] 4.4 確保 OpenCode provider 停用時釋放 watcher

## 5. 前端：設定頁面擴展

- [x] 5.1 在 `SettingsView` 新增 OpenCode 路徑設定欄位（含路徑驗證提示）
- [x] 5.2 在 `SettingsView` 新增 "啟用的平台" 區塊，含 Copilot 與 OpenCode checkbox
- [x] 5.3 新增翻譯 key：opencodeRoot label、enabledProviders 相關文字
- [x] 5.4 修改 `App.tsx` 中 `save_settings` mutation，傳送新增的設定欄位

## 6. 前端：Platform Tag 元件

- [x] 6.1 新增 CSS class `.provider-tag`、`.provider-tag--copilot`、`.provider-tag--opencode`，定義各平台的配色
- [x] 6.2 在 session card 元件中新增 provider tag 顯示邏輯，位於使用者標籤之前
- [x] 6.3 確保 provider tag 與 user tag 視覺區隔（實心 vs 外框線）

## 7. 前端：Platform Filter UI

- [x] 7.1 在 session 列表區域（toolbar 或 sidebar）新增平台篩選 checkbox 群組
- [x] 7.2 修改 `App.tsx` 中 `get_sessions` 的 useQuery，將 `enabledProviders` 加入 query key 與 invoke 參數
- [x] 7.3 篩選變更時 invalidate sessions query 觸發重新載入

## 8. 前端：Dashboard 多平台統計

- [x] 8.1 修改 Dashboard 統計邏輯，依 `provider` 欄位計算各平台 session 數量
- [x] 8.2 在 Dashboard UI 新增平台分佈顯示區塊

## 9. Rust 後端：Sisyphus Reader

- [x] 9.1 新增 `SisyphusData`、`SisyphusBoulder`、`SisyphusPlan`、`SisyphusNotepad` struct（含 `#[serde(rename_all = "camelCase")]`）
- [x] 9.2 新增 `scan_sisyphus_internal(project_dir: &Path)` 函式：偵測 `.sisyphus/` 目錄存在，回傳 `SisyphusData`
- [x] 9.3 實作 `boulder.json` 解析：讀取 `active_plan`、`plan_name`、`agent`、`session_ids`、`started_at`，解析失敗時回傳 `active_plan: None`
- [x] 9.4 實作 `plans/*.md` 掃描：列舉所有 `.md` 檔案，從 Markdown heading 取得 `title`，從 `## TL;DR` section 取得摘要，比對 `boulder.json` 判斷 `is_active`
- [x] 9.5 實作 `notepads/*/` 掃描：列舉子目錄，偵測 `issues.md` 與 `learnings.md` 存在
- [x] 9.6 實作 `evidence/*.txt` 與 `drafts/*.md` 掃描：回傳檔名清單

## 10. Rust 後端：OpenSpec Reader

- [x] 10.1 新增 `OpenSpecData`、`OpenSpecChange`、`OpenSpecSpec` struct（含 `#[serde(rename_all = "camelCase")]`）
- [x] 10.2 新增 `scan_openspec_internal(project_dir: &Path)` 函式：偵測 `openspec/` 目錄存在，回傳 `OpenSpecData`
- [x] 10.3 實作 `config.yaml` 解析：讀取 `schema` 欄位，解析失敗時回傳 `schema: None`
- [x] 10.4 實作 `changes/` 掃描：列舉子目錄（排除 `archive/`），偵測各 change 下 `proposal.md`、`design.md`、`tasks.md` 是否存在，計算 `specs/` 子目錄數量
- [x] 10.5 實作 `changes/archive/` 掃描：與 active changes 相同邏輯
- [x] 10.6 實作 `specs/` 掃描：列舉子目錄，記錄 `spec.md` 路徑

## 11. Rust 後端：新增 Tauri Commands

- [x] 11.1 新增 `#[tauri::command] fn get_project_plans(project_dir: String)` → 呼叫 `scan_sisyphus_internal`，回傳 `SisyphusData`
- [x] 11.2 新增 `#[tauri::command] fn get_project_specs(project_dir: String)` → 呼叫 `scan_openspec_internal`，回傳 `OpenSpecData`
- [x] 11.3 新增 `#[tauri::command] fn read_plan_content(file_path: String)` → 讀取指定 `.md` 檔案完整內容回傳
- [x] 11.4 在 `invoke_handler![]` 中登記 `get_project_plans`、`get_project_specs`、`read_plan_content`

## 12. 前端：型別擴展（Sisyphus / OpenSpec）

- [x] 12.1 在 `types/index.ts` 新增 `SisyphusData`、`SisyphusBoulder`、`SisyphusPlan`、`SisyphusNotepad` 型別
- [x] 12.2 在 `types/index.ts` 新增 `OpenSpecData`、`OpenSpecChange`、`OpenSpecSpec` 型別

## 13. 前端：ProjectView 子分頁重構

- [x] 13.1 在 `ProjectView.tsx` 新增子分頁狀態 `activeSubTab: "sessions" | "plans-specs"`，預設 `"sessions"`
- [x] 13.2 新增子分頁列 UI（sub-tab bar），包含 "Sessions" 與 "Plans & Specs" 按鈕
- [x] 13.3 將現有 session 列表邏輯（搜尋、排序、篩選、card 清單）包入 Sessions 子分頁條件渲染
- [x] 13.4 確保切換專案分頁時子分頁重置為預設
- [x] 13.5 新增子分頁相關 CSS class：`.sub-tab-bar`、`.sub-tab-item`、`.sub-tab-item--active`
- [x] 13.6 新增翻譯 key：`sessions`、`plansAndSpecs` 子分頁標題

## 14. 前端：PlansSpecsView 元件

- [x] 14.1 新增 `PlansSpecsView.tsx` 元件，接收 `projectDir: string` prop
- [x] 14.2 在 `App.tsx` 新增 `get_project_plans` 與 `get_project_specs` 的 React Query useQuery（以 `projectDir` + `activeSubTab` 為 query key，啟用 `enabled: activeSubTab === "plans-specs"`）
- [x] 14.3 將 query 結果透過 props 傳入 `PlansSpecsView`
- [x] 14.4 實作 `.sisyphus` 區塊：Active Plan banner、Plans 清單、Notepads 清單、Evidence/Drafts 可收合區塊
- [x] 14.5 實作 `openspec` 區塊：Active Changes 清單、Archived Changes 可收合區塊、Specs 清單
- [x] 14.6 實作空狀態提示：兩者皆不存在時顯示「此專案尚無 plan 或 spec 資料」
- [x] 14.7 實作 Markdown 內容預覽：點擊 plan/spec 條目時呼叫 `read_plan_content`，在 detail panel 中顯示（複用 PlanEditor 唯讀模式或新建 MarkdownPreview 元件）
- [x] 14.8 新增 PlansSpecsView 相關 CSS class
- [x] 14.9 新增翻譯 key：sisyphus/openspec 區塊標題、空狀態文字、Active Plan 等

## 15. 測試與驗證

- [ ] 15.1 Rust 單元測試：`scan_opencode_sessions_internal` 對測試用 SQLite 資料庫的讀取與映射
- [ ] 15.2 Rust 單元測試：OpenCode DB 不存在時的容錯行為
- [ ] 15.3 Rust 單元測試：`enabled_providers` 篩選邏輯（僅 copilot / 僅 opencode / 全部）
- [ ] 15.4 Rust 單元測試：`scan_sisyphus_internal` 對測試用 `.sisyphus/` 目錄的讀取
- [ ] 15.5 Rust 單元測試：`scan_openspec_internal` 對測試用 `openspec/` 目錄的讀取
- [ ] 15.6 Rust 單元測試：`.sisyphus/` 或 `openspec/` 不存在時的空結構回傳
- [x] 15.7 前端型別檢查：`bun run build` 通過
- [ ] 15.8 整合驗證：`bun run tauri dev` 啟動後可同時看到 Copilot 與 OpenCode session
- [ ] 15.9 整合驗證：進入專案頁面，切換到 Plans & Specs 分頁，可看到 `.sisyphus` 與 `openspec` 資料
