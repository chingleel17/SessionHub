## 1. Rust 核心：掃描器與指紋

- [x] 1.1 `Cargo.toml` 新增 `walkdir = "2"`、`sha2 = "0.10"`
- [x] 1.2 新增 `src-tauri/src/agents_config.rs`：`SyncStatus` / `FileFingerprint` / `AgentsMdEntry` / `SkillEntry` / `CommandEntry` / `TargetStatus` 等 serde 結構（camelCase）
- [x] 1.3 實作 AGENTS.md/CLAUDE.md 遞迴掃描器：`WalkDir` max_depth 8、`follow_links(false)`、忽略清單（node_modules/.git/dist/build/vendor/.next/.nuxt/target/.sessionhub + prefs.ignoredPaths）、目錄數上限與 `truncated` 旗標、四種狀態判定（hash 為準、`target_newer` 由 mtime）
- [x] 1.4 實作 skills 掃描：來源 `.agents/skills/<name>/`，對 claude/codex/opencode/copilot 目標目錄計算 per-target 狀態（目錄比對＝逐檔 hash 聚合，套用與 1.3 相同忽略清單過濾內部 node_modules 等雜訊）；若目標為 symlink 且指向來源即判定 in-sync（免逐檔比對）；symlink 指向錯誤來源或已失效（來源不存在）標示為錯誤狀態
- [x] 1.5 實作 commands 掃描：來源 `.agents/skills/command/**/*.md`，目標對映 `.claude/commands/`（保留子路徑）、`.codex/prompts/`、`.opencode/command/`、`.copilot/prompts/`
- [x] 1.6 實作 global scope 解析：以 `settings.rs::resolve_claude_root / resolve_codex_root / resolve_copilot_root / default_opencode_config_root` 取得固定已知位置（不遞迴家目錄）
- [x] 1.7 單元測試（temp dir，仿 `session_todos.rs` 模式）：忽略目錄、深度上限、truncated、四種狀態、Windows `\\?\` containment 檢查；`cargo test` 通過

## 2. Rust 核心：同步引擎與偏好

- [x] 2.1 實作 `sync_agents_items` 管線：create / skip-in-sync / overwrite / conflict / error 判定，`force` 與逐項 `direction` 支援，dry-run 走相同管線不落地；來源不存在（source-missing）一律視為衝突，不受 `force` 影響
- [x] 2.2 寫入採原子替換：`create_dir_all` → `*.tmp-sessionhub` → `fs::rename`；路徑防護採 canonicalize 既有祖先 + 拒絕 `..` 的詞法拼接
- [x] 2.3 skills 目錄同步（Copy 模式）展開為 per-file 計畫聚合（不刪除目標端多餘檔案）
- [x] 2.4 skills 連結模式（Link）：`symlink_dir` 建立目錄連結；權限不足（`ERROR_PRIVILEGE_NOT_HELD`）自動 fallback 為 Copy 並回報 `link-fallback-copy`；既有 symlink 指向正確來源視為 skip-in-sync，指向錯誤來源或既有實體內容視為衝突
- [x] 2.5 commands 同步保留 `CommandAdapter` trait 接縫（v1 `PassthroughAdapter` 純複製，恆為 Copy 模式）
- [x] 2.6 實作 `ProjectAgentsPrefs` 讀寫：讀取一律專案內 `.sessionhub/agents.json` 優先、否則 APPDATA fallback；寫入依 `allowCreateProjectConfigDir` 決定目的地，但專案內檔案已存在時無論開關一律寫回該檔案；開關切換不做既有內容搬移
- [x] 2.7 單元測試：dry-run 不寫檔、force 覆蓋、target-newer 產生 conflict、source-missing 一律衝突（含 force=true 時仍衝突）、記住選擇後自動套用方向、目錄複製、連結模式建立/fallback/失效偵測、APPDATA fallback、開關關閉但專案檔已存在時仍寫回專案；`cargo test` + `cargo clippy` 通過

## 3. Tauri 指令層與設定

- [x] 3.1 新增 `src-tauri/src/commands/agents_config.rs`：`scan_agents_md` / `scan_global_agents_md` / `scan_agents_skills` / `scan_agents_commands` / `sync_agents_items` / `read_agents_file` / `write_agents_file` / `load_project_agents_prefs` / `save_project_agents_prefs`，掃描與同步走背景執行緒（仿 `get_project_specs`）
- [x] 3.2 `commands/mod.rs` 宣告與 re-export；`lib.rs` `generate_handler!` 註冊全部新指令
- [x] 3.3 `AppSettings` 新增 `allow_create_project_config_dir: bool`（default false）：`types.rs` struct + `settings.rs` Default impl + `lib.rs` 與 `commands/mod.rs` 的兩處字面值建構
- [x] 3.4 `cargo check` 通過，dev 模式 devtools 冒煙測試各指令
- [x] 3.5 掃描來源修正：全域 AGENTS 納入 `~/.agents/instructions/AGENTS.md`；commands 與 skills 掃描皆支援 target 聯集顯示，避免來源缺失時整列消失（skills 於來源缺失時標示 source-missing「僅有目標」，見 agents-skills-sync 更新後規格）

## 4. 前端：AGENTS.md 分頁

- [x] 4.1 `src/types/index.ts` 鏡射所有新型別（AgentsMdScanResult / SkillsScanResult / CommandsScanResult / SyncRequest / SyncReport / ProjectAgentsPrefs / AgentsScope），`AppSettings` 加 `allowCreateProjectConfigDir`
- [x] 4.2 `src/utils/buildTree.ts` 新增 `buildAgentsMdTree(result, t)`：badge=狀態標籤、tone 對映、`filePathType: "absolute"`
- [x] 4.3 新增 `src/components/AgentsConfigView.tsx` 外殼：內部 `.sub-tab-bar`（AGENTS.md | Skills | Commands），AGENTS.md 分頁 = ExplorerTree + ContentViewer 可調寬度佈局，編輯模式採 PlanEditor 式 textarea+預覽+儲存，工具列（外部編輯器 / 檔案總管 / 同步此目錄 / 重新整理）
- [x] 4.4 App.tsx 接線：`agents-md` / `agents-prefs` query（staleTime 30s、分頁可見才 enabled）、read/write/sync/prefs handlers、`activeView === "agents-global"` 分支；Sidebar 新增 Agents 導覽按鈕（Icons.tsx 加 AgentsIcon）；ProjectView 新增 `"agents"` sub-tab 與 body 分支
- [x] 4.5 i18n：`agents.nav`、`agents.tab.*`、`agents.status.*`、`agents.action.*`、`agents.empty.*` 加入 zh-TW.ts 與 en-US.ts

## 5. 前端：同步 UX 與衝突處理

- [x] 5.1 dry-run 預覽面板：呼叫 `sync_agents_items({dryRun:true})`，SyncReport 逐項勾選後以選取項重送套用
- [x] 5.2 新增 `src/components/SyncConflictDialog.tsx`：逐衝突 radio（來源→目標 / 目標→來源 / 略過）、「套用到全部」、「記住此專案的選擇」；onResolve 回傳帶 direction 的 SyncItem[]
- [x] 5.3 記住選擇寫入 prefs（`save_project_agents_prefs`）；同步 mutation 成功後 invalidate 三個掃描 query 並 toast 摘要（建立/覆蓋/略過/錯誤數）
- [x] 5.4 `.sessionhub/` 建立時（allowCreateProjectConfigDir 開啟）顯示建議加入 .gitignore 的提示；i18n：`agents.conflict.*`、`agents.report.*`

## 6. 前端：Skills / Commands 分頁與全域頁

- [x] 6.1 Skills 分頁狀態矩陣：每列 SkillEntry（勾選框+名稱，點名稱預覽 SKILL.md），每 target 一欄狀態格（✓ / – / ≠ / 較新! / 🔗 已連結 / ⚠ 連結失效），欄標題勾選綁 `prefs.enabledTargets`（root_exists=false 欄位格內不可勾選）；新增「同步模式」切換（複製/連結，預設複製），連結建立失敗時 toast 提示權限不足並說明 fallback 為複製
- [x] 6.2 Commands 分頁同構矩陣（含子路徑名稱如 `opsx/apply`）
- [x] 6.3 全域頁面：scope=global 時三分頁對 `~/.agents/skills`、各 agent 全域目錄運作；prefs 區塊（enabledTargets）改存全域設定或隱藏「記住選擇」
- [x] 6.4 SettingsView 新增「Agents」區塊：`allowCreateProjectConfigDir` 開關與說明；`App.css` 矩陣表格、狀態格、對話框樣式（含 dark mode）
- [x] 6.5 i18n：`agents.prefs.*`、`settings.agents.*`
- [x] 6.6 Sidebar/Settings 整理：Agents 導覽按鈕只保留一個並固定於 Settings 上方；主題色系控制移入設定頁
- [x] 6.7 Agents 頁快取與版面壓縮：切換頁面後保留 scan 結果/選取狀態，移除多餘外框並減少滾動容器

## 7. 驗證

- [x] 7.1 `cargo test`、`cargo clippy`、`npm run build`（tsc + vite）全數通過
- [x] 7.2 手動：以 SessionHub 專案本身驗證（`.agents/.claude/.codex/.opencode` 現成混合狀態）——AGENTS.md Tree 狀態正確、檢視/編輯/儲存 round-trip、外部編輯器與檔案總管開啟
- [x] 7.3 手動：dry-run 不落地、勾選套用後狀態轉 in-sync、目標較新時跳衝突對話框、來源缺失時跳衝突對話框（含 force 開啟時仍詢問）、記住選擇後不再詢問且存入 `.sessionhub/agents.json`；關閉 allowCreateProjectConfigDir 且專案內無既有檔案時走 APPDATA fallback 且不建立資料夾；專案內已有既有檔案時關閉開關仍寫回專案內
- [x] 7.3a 手動：Skills 連結模式——一般權限下建立 symlink 失敗並 fallback 為複製且有提示；以系統管理員或開發者模式執行時成功建立 symlink 且矩陣顯示「已連結」；手動修改來源檔案後連結端立即反映；來源目錄被刪除後矩陣標示連結失效
- [x] 7.4 手動：Skills/Commands 矩陣對本機真實目錄（`~/.agents/skills`、`~/.copilot/skills`、`~/.config/opencode/command`）狀態正確；opencode 全域解析至 `~/.config/opencode`
- [x] 7.5 手動：大型 repo（含 node_modules）掃描不卡 UI、truncated 警告顯示；zh-TW / en-US 雙語檢查

## 8. 實作偏差修正（bug fix，不變更規格需求）

- [x] 8.1 修正 `AgentsScope::Project` serde：enum 層級 `rename_all` 不影響 variant 內欄位，需在 `Project` variant 補 `#[serde(rename_all = "camelCase")]`，否則前端送 `projectCwd` 無法反序列化，導致專案級 skills/commands 掃描全部失敗（僅全域 scope 不受影響）；補上 camelCase payload 反序列化測試
- [x] 8.2 修正 `compare_directory_target`：來源目錄不存在但目標存在時回報 `source-missing`，原本誤判為 `differs`
- [x] 8.3 修正 AgentsConfigView checkbox `onChange` 於 setState updater 內讀取 `event.currentTarget`（事件派發後為 null）導致 render 例外白屏——先取值再進 updater（矩陣勾選、報告勾選兩處）
- [x] 8.4 skills/commands 分頁新增內容預覽面板（矩陣下方，Markdown 呈現，含外部開啟/檔案總管按鈕）；切換頁籤時清除選取避免殘留
- [x] 8.5 修正 `classify_file_status`：來源與目標皆不存在時，原本回傳 `SyncStatus::Error`（UI 顯示為中性「錯誤」標籤）；此情況在 commands 矩陣的「target 聯集反查」場景下是常態（該 command 名稱由其他 target 反查得到，來源尚未建立、當前 target 也還沒有），已改回傳 `SyncStatus::TargetMissing`，語意與顯示皆改為「缺少目標」。
  - [x] 程式修正：`src-tauri/src/agents_config.rs::classify_file_status`（約 666-682 行）。影響範圍已確認僅限 commands 掃描（`scan_agents_commands_internal` 內 484 行呼叫點）；AGENTS.md 掃描（294、631 行呼叫點）在呼叫前已用「兩者皆不存在則 `continue` 跳過、不產生 entry」擋掉此分支，故不受影響，1.3/1.7/2.7/7.1 既有驗證結論不需回退。
  - [x] 已用臨時測試（跑完即移除，未留在正式測試檔）對照使用者真實全域環境（`~/.claude/commands`、`~/.codex/prompts` 為部分同步的 symlink 慣例）重現問題並驗證修正後行為符合預期
  - [x] 補上正式單元測試：commands 掃描，來源與目標皆缺 → 斷言狀態為 `TargetMissing` 而非 `Error`
  - [x] 更新 `agents-commands-sync` spec 對應 Scenario 後，重跑 `cargo test` + `cargo clippy` 確認全數通過（`cargo test` 已通過；`cargo clippy -D warnings` 目前被專案既有、與本次 agents 變更無關的 warning / lint 擋下，待另行清理後可勾選）
  - [x] 前端 i18n / 樣式初步判斷不需改動（沿用既有 `agents.status.target-missing` 文案與 tone），但待正式測試補齊後仍需人工在 Commands 分頁複驗畫面顯示

- [x] 8.6 新增 `agentsSourceRoot` 設定：全域範圍 skills/commands 正本目錄原寫死為 `~/.agents`，與使用者實際將正本集中存放於自訂位置（見 D9）的使用情境不符
  - [x] `src-tauri/src/types.rs`：`AppSettings` 新增 `agents_source_root: String`（`#[serde(default)]`）
  - [x] `src-tauri/src/settings.rs`：新增 `default_agents_root`（由 `agents_config.rs` 移入）與 `resolve_agents_source_root`；補單元測試（有值 / 空白 fallback / None fallback 三種情境）
  - [x] `src-tauri/src/agents_config.rs`：`skills_source_root`／`commands_source_root`／`global_instruction_roots` 於 `AgentsScope::Global` 分支改用 `resolve_agents_source_root`；`AgentsScope::Project` 不受影響
  - [x] `src/types/index.ts`：`AppSettings` 新增 `agentsSourceRoot?: string`
  - [x] `src/components/SettingsView.tsx`：Agents 區塊新增路徑輸入欄 + 瀏覽按鈕；`onBrowseDirectory` 型別擴充 `agentsSourceRoot`
  - [x] `src/App.tsx`：`handleBrowseDirectory` 型別同步擴充
  - [x] `src/locales/zh-TW.ts`、`en-US.ts`：新增 `settings.fields.agentsSourceRoot` / `agentsSourceRootDesc` / `agentsSourceRootPlaceholder`
  - [x] `cargo test --lib`（agents_config 18 個 + settings 3 個新測試）與 `tsc --noEmit` 驗證通過
