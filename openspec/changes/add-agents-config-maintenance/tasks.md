## 1. Rust 核心：掃描器與指紋

- [ ] 1.1 `Cargo.toml` 新增 `walkdir = "2"`、`sha2 = "0.10"`
- [ ] 1.2 新增 `src-tauri/src/agents_config.rs`：`SyncStatus` / `FileFingerprint` / `AgentsMdEntry` / `SkillEntry` / `CommandEntry` / `TargetStatus` 等 serde 結構（camelCase）
- [ ] 1.3 實作 AGENTS.md/CLAUDE.md 遞迴掃描器：`WalkDir` max_depth 8、`follow_links(false)`、忽略清單（node_modules/.git/dist/build/vendor/.next/.nuxt/target/.sessionhub + prefs.ignoredPaths）、目錄數上限與 `truncated` 旗標、四種狀態判定（hash 為準、`target_newer` 由 mtime）
- [ ] 1.4 實作 skills 掃描：來源 `.agents/skills/<name>/`，對 claude/codex/opencode/copilot 目標目錄計算 per-target 狀態（目錄比對＝逐檔 hash 聚合，套用與 1.3 相同忽略清單過濾內部 node_modules 等雜訊）；若目標為 symlink 且指向來源即判定 in-sync（免逐檔比對）；symlink 指向錯誤來源或已失效（來源不存在）標示為錯誤狀態
- [ ] 1.5 實作 commands 掃描：來源 `.agents/skills/command/**/*.md`，目標對映 `.claude/commands/`（保留子路徑）、`.codex/prompts/`、`.opencode/command/`、`.copilot/prompts/`
- [ ] 1.6 實作 global scope 解析：以 `settings.rs::resolve_claude_root / resolve_codex_root / resolve_copilot_root / default_opencode_config_root` 取得固定已知位置（不遞迴家目錄）
- [ ] 1.7 單元測試（temp dir，仿 `session_todos.rs` 模式）：忽略目錄、深度上限、truncated、四種狀態、Windows `\\?\` containment 檢查；`cargo test` 通過

## 2. Rust 核心：同步引擎與偏好

- [ ] 2.1 實作 `sync_agents_items` 管線：create / skip-in-sync / overwrite / conflict / error 判定，`force` 與逐項 `direction` 支援，dry-run 走相同管線不落地；來源不存在（source-missing）一律視為衝突，不受 `force` 影響
- [ ] 2.2 寫入採原子替換：`create_dir_all` → `*.tmp-sessionhub` → `fs::rename`；路徑防護採 canonicalize 既有祖先 + 拒絕 `..` 的詞法拼接
- [ ] 2.3 skills 目錄同步（Copy 模式）展開為 per-file 計畫聚合（不刪除目標端多餘檔案）
- [ ] 2.4 skills 連結模式（Link）：`symlink_dir` 建立目錄連結；權限不足（`ERROR_PRIVILEGE_NOT_HELD`）自動 fallback 為 Copy 並回報 `link-fallback-copy`；既有 symlink 指向正確來源視為 skip-in-sync，指向錯誤來源或既有實體內容視為衝突
- [ ] 2.5 commands 同步保留 `CommandAdapter` trait 接縫（v1 `PassthroughAdapter` 純複製，恆為 Copy 模式）
- [ ] 2.6 實作 `ProjectAgentsPrefs` 讀寫：讀取一律專案內 `.sessionhub/agents.json` 優先、否則 APPDATA fallback；寫入依 `allowCreateProjectConfigDir` 決定目的地，但專案內檔案已存在時無論開關一律寫回該檔案；開關切換不做既有內容搬移
- [ ] 2.7 單元測試：dry-run 不寫檔、force 覆蓋、target-newer 產生 conflict、source-missing 一律衝突（含 force=true 時仍衝突）、記住選擇後自動套用方向、目錄複製、連結模式建立/fallback/失效偵測、APPDATA fallback、開關關閉但專案檔已存在時仍寫回專案；`cargo test` + `cargo clippy` 通過

## 3. Tauri 指令層與設定

- [ ] 3.1 新增 `src-tauri/src/commands/agents_config.rs`：`scan_agents_md` / `scan_global_agents_md` / `scan_agents_skills` / `scan_agents_commands` / `sync_agents_items` / `read_agents_file` / `write_agents_file` / `load_project_agents_prefs` / `save_project_agents_prefs`，掃描與同步走背景執行緒（仿 `get_project_specs`）
- [ ] 3.2 `commands/mod.rs` 宣告與 re-export；`lib.rs` `generate_handler!` 註冊全部新指令
- [ ] 3.3 `AppSettings` 新增 `allow_create_project_config_dir: bool`（default false）：`types.rs` struct + `settings.rs` Default impl + `lib.rs` 與 `commands/mod.rs` 的兩處字面值建構
- [ ] 3.4 `cargo check` 通過，dev 模式 devtools 冒煙測試各指令

## 4. 前端：AGENTS.md 分頁

- [ ] 4.1 `src/types/index.ts` 鏡射所有新型別（AgentsMdScanResult / SkillsScanResult / CommandsScanResult / SyncRequest / SyncReport / ProjectAgentsPrefs / AgentsScope），`AppSettings` 加 `allowCreateProjectConfigDir`
- [ ] 4.2 `src/utils/buildTree.ts` 新增 `buildAgentsMdTree(result, t)`：badge=狀態標籤、tone 對映、`filePathType: "absolute"`
- [ ] 4.3 新增 `src/components/AgentsConfigView.tsx` 外殼：內部 `.sub-tab-bar`（AGENTS.md | Skills | Commands），AGENTS.md 分頁 = ExplorerTree + ContentViewer 可調寬度佈局，編輯模式採 PlanEditor 式 textarea+預覽+儲存，工具列（外部編輯器 / 檔案總管 / 同步此目錄 / 重新整理）
- [ ] 4.4 App.tsx 接線：`agents-md` / `agents-prefs` query（staleTime 30s、分頁可見才 enabled）、read/write/sync/prefs handlers、`activeView === "agents-global"` 分支；Sidebar 新增 Agents 導覽按鈕（Icons.tsx 加 AgentsIcon）；ProjectView 新增 `"agents"` sub-tab 與 body 分支
- [ ] 4.5 i18n：`agents.nav`、`agents.tab.*`、`agents.status.*`、`agents.action.*`、`agents.empty.*` 加入 zh-TW.ts 與 en-US.ts

## 5. 前端：同步 UX 與衝突處理

- [ ] 5.1 dry-run 預覽面板：呼叫 `sync_agents_items({dryRun:true})`，SyncReport 逐項勾選後以選取項重送套用
- [ ] 5.2 新增 `src/components/SyncConflictDialog.tsx`：逐衝突 radio（來源→目標 / 目標→來源 / 略過）、「套用到全部」、「記住此專案的選擇」；onResolve 回傳帶 direction 的 SyncItem[]
- [ ] 5.3 記住選擇寫入 prefs（`save_project_agents_prefs`）；同步 mutation 成功後 invalidate 三個掃描 query 並 toast 摘要（建立/覆蓋/略過/錯誤數）
- [ ] 5.4 `.sessionhub/` 建立時（allowCreateProjectConfigDir 開啟）顯示建議加入 .gitignore 的提示；i18n：`agents.conflict.*`、`agents.report.*`

## 6. 前端：Skills / Commands 分頁與全域頁

- [ ] 6.1 Skills 分頁狀態矩陣：每列 SkillEntry（勾選框+名稱，點名稱右側預覽 SKILL.md），每 target 一欄狀態格（✓ / – / ≠ / 較新! / 🔗 已連結 / ⚠ 連結失效），欄標題勾選綁 `prefs.enabledTargets`（root_exists=false 欄位格內不可勾選）；新增「同步模式」切換（複製/連結，預設複製），連結建立失敗時 toast 提示權限不足並說明 fallback 為複製
- [ ] 6.2 Commands 分頁同構矩陣（含子路徑名稱如 `opsx/apply`）
- [ ] 6.3 全域頁面：scope=global 時三分頁對 `~/.agents/skills`、各 agent 全域目錄運作；prefs 區塊（enabledTargets）改存全域設定或隱藏「記住選擇」
- [ ] 6.4 SettingsView 新增「Agents」區塊：`allowCreateProjectConfigDir` 開關與說明；`App.css` 矩陣表格、狀態格、對話框樣式（含 dark mode）
- [ ] 6.5 i18n：`agents.prefs.*`、`settings.agents.*`

## 7. 驗證

- [ ] 7.1 `cargo test`、`cargo clippy`、`npm run build`（tsc + vite）全數通過
- [ ] 7.2 手動：以 SessionHub 專案本身驗證（`.agents/.claude/.codex/.opencode` 現成混合狀態）——AGENTS.md Tree 狀態正確、檢視/編輯/儲存 round-trip、外部編輯器與檔案總管開啟
- [ ] 7.3 手動：dry-run 不落地、勾選套用後狀態轉 in-sync、目標較新時跳衝突對話框、來源缺失時跳衝突對話框（含 force 開啟時仍詢問）、記住選擇後不再詢問且存入 `.sessionhub/agents.json`；關閉 allowCreateProjectConfigDir 且專案內無既有檔案時走 APPDATA fallback 且不建立資料夾；專案內已有既有檔案時關閉開關仍寫回專案內
- [ ] 7.3a 手動：Skills 連結模式——一般權限下建立 symlink 失敗並 fallback 為複製且有提示；以系統管理員或開發者模式執行時成功建立 symlink 且矩陣顯示「已連結」；手動修改來源檔案後連結端立即反映；來源目錄被刪除後矩陣標示連結失效
- [ ] 7.4 手動：Skills/Commands 矩陣對本機真實目錄（`~/.agents/skills`、`~/.copilot/skills`、`~/.config/opencode/command`）狀態正確；opencode 全域解析至 `~/.config/opencode`
- [ ] 7.5 手動：大型 repo（含 node_modules）掃描不卡 UI、truncated 警告顯示；zh-TW / en-US 雙語檢查
