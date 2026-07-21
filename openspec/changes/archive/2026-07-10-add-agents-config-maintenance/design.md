# Design: add-agents-config-maintenance

## Context

現況與可重用的既有基礎：

- **Explorer 模式**：`PlansSpecsView.tsx` 已實作「左側 ExplorerTree + 右側 ContentViewer、可調寬度、重新整理」的外殼；`TreeNode`（`src/types/index.ts`）以 `filePath` + `filePathType: "absolute" | "openspec"` 決定葉節點讀取方式；Tree 由 `src/utils/buildTree.ts` 的純函式建構。`PlanEditor.tsx` 是「textarea + markdown 預覽 + 儲存 + 外部編輯器開啟」的編輯模式範本。
- **IPC 集中於 App.tsx**：子元件為純 props 驅動；掃描走 TanStack Query（`sisyphusQuery`/`openspecQuery`，staleTime 30s）；`openPath` / `revealItemInDir`（plugin-opener）已接好。
- **Rust 掃描慣例**：`openspec_scan.rs` / `sisyphus.rs` 為單層 `fs::read_dir` 掃描；`resolve_openspec_file_internal` 示範 canonicalize + `starts_with` 的路徑穿越防護；`get_project_specs` 示範背景執行緒非同步掃描。專案目前**沒有**遞迴 walker 與 ignore 機制。
- **Agent 根目錄解析**：`settings.rs` 已有 `resolve_claude_root` / `resolve_codex_root` / `resolve_copilot_root` / `default_opencode_config_root`（`~/.config/opencode`）。
- **本機慣例（已實地確認）**：專案級 `.agents/skills/<name>/SKILL.md` 為來源 → `.claude/skills`、`.codex/skills`、`.opencode/skills`、`.copilot/skills`；commands 來源為 `.agents/skills/command/*.md` → `.claude/commands/`（可含子目錄如 `opsx/apply.md`）、`.codex/prompts/`、`.opencode/command/`、`.copilot/prompts/`。全域來源預設為 `~/.agents/skills/`（含 `command/` 子目錄），可於設定頁以 `agentsSourceRoot` 覆寫（見 D8）。
- **額外實際目錄**：使用者環境另有 `~/.agents/instructions/AGENTS.md`，需納入全域 AGENTS 掃描；部分專案或全域環境可能已存在 target 端 commands，但來源端 `.agents/skills/command/` 尚未建立，此情況在 UI 中仍需可見，避免使用者誤判為「沒有 command」。

## Goals / Non-Goals

**Goals:**
- 一眼看清 AGENTS.md/CLAUDE.md/skills/commands 在專案與全域的分布與同步狀態。
- Rust 內建 agents-sync 語意的同步引擎：dry-run 預覽、逐項勾選、原子寫入、衝突詢問與記住選擇。
- 全域頁與專案子分頁共用同一元件，僅以 scope 區分資料來源。

**Non-Goals:**
- 不呼叫外部 agents-sync CLI（純 Rust 實作，不依賴 Node）。
- v1 不做「刪除目標端多餘檔案」的鏡像同步（僅 create / overwrite）。
- v1 commands 為位元組複製，不做各 agent frontmatter 格式轉換（保留 `CommandAdapter` trait 接縫）。commands 不支援連結模式（v1 僅複製；命令檔案格式可能因 target 而異，連結會過度綁定單一格式，留待日後評估）。
- 不自動修改 `.gitignore`（僅 UI 提示建議忽略 `.sessionhub/`）。
- 不接 file watcher 自動刷新（v1 手動 Refresh + staleTime）。
- Skills 目錄比對不遞迴計算 skill 內部子目錄的巢狀 node_modules/建置產物大小（見 D5a）。
- `allowCreateProjectConfigDir` 開關切換時不做既有偏好檔案搬移／合併（見 D7）。

## Decisions

### D1. 同步引擎以 Rust 內建，新增 `walkdir` + `sha2` 依賴

手刻遞迴在 Windows 上易踩 junction/symlink 迴圈與中途權限錯誤；`walkdir` 提供 `max_depth`、`filter_entry`、`follow_links(false)` 與逐項容錯。一致性判定以 sha256 內容雜湊為準（跨次執行穩定），mtime 僅用於「衝突 vs 自動覆蓋」的判斷。不引入 `ignore` crate（固定忽略清單即足夠，無需 gitignore 語意）。

### D2. 模組切分

- `src-tauri/src/agents_config.rs`：核心純邏輯——遞迴掃描器、`FileFingerprint`、同步管線、`ProjectAgentsPrefs` 讀寫。可單元測試（temp dir，仿 `session_todos.rs` 測試模式）。
- `src-tauri/src/commands/agents_config.rs`：`#[tauri::command]` 薄包裝；掃描與同步以背景執行緒執行（仿 `get_project_specs`）。

### D3. 資料結構（serde camelCase，前端 `src/types/index.ts` 鏡射）

```rust
enum SyncStatus { InSync, TargetMissing, Differs, SourceMissing }  // kebab-case 序列化

struct FileFingerprint { path, exists, hash: Option<String>, mtime_ms: Option<u64>, size: Option<u64> }

// AGENTS.md／CLAUDE.md
struct AgentsMdEntry { dir, rel_dir, source: FileFingerprint /*AGENTS.md*/, target: FileFingerprint /*CLAUDE.md*/, status, target_newer: bool }
struct AgentsMdScanResult { root, entries: Vec<AgentsMdEntry>, truncated: bool, scanned_dirs: u64 }

// Skills / Commands
struct TargetStatus { target_id /*claude|codex|opencode|copilot*/, target_root, status, target_newer }
struct SkillEntry { name, source_dir, skill_md_path, file_count, targets: Vec<TargetStatus> }
struct CommandEntry { name /*含子路徑如 "opsx/apply"*/, source_path, targets: Vec<TargetStatus> }
struct SkillsScanResult { source_root, skills: Vec<SkillEntry>, targets: Vec<TargetInfo> }   // Commands 同構
struct TargetInfo { target_id, root, root_exists }

// 同步
struct SyncRequest { items: Vec<SyncItem>, dry_run: bool, force: bool, mode: SyncMode /*copy|link，僅 skills 適用；agents-md/commands 固定 copy*/ }
struct SyncItem { source, target, direction /*source-to-target|target-to-source*/ }
struct SyncActionResult { source, target, action /*create|overwrite|skip-in-sync|conflict|error|link-fallback-copy*/, reason: Option<String>, bytes: Option<u64> }
struct SyncReport { dry_run, actions: Vec<SyncActionResult>, conflicts: u32, errors: u32 }
enum SyncMode { Copy, Link }

// 專案偏好
struct ProjectAgentsPrefs { conflict_choice: Option<String> /*source-wins|target-wins|null=ask*/, ignored_paths: Vec<String>, enabled_targets: Vec<String> }
```

### D4. Tauri 指令

```
scan_agents_md(project_cwd) -> AgentsMdScanResult
scan_global_agents_md() -> AgentsMdScanResult              // 僅固定已知 agent root 的指示檔
scan_agents_skills(scope) -> SkillsScanResult              // scope: { kind: "project", projectCwd } | { kind: "global" }
scan_agents_commands(scope) -> CommandsScanResult
sync_agents_items(request: SyncRequest) -> SyncReport      // 單一通用同步指令（檔案或目錄皆可）
read_agents_file(file_path) -> String                      // 或重用既有 read_plan_content
write_agents_file(scope_root, file_path, content)          // canonicalize + starts_with 防護
load_project_agents_prefs(project_cwd) -> ProjectAgentsPrefs
save_project_agents_prefs(project_cwd, prefs)
```

Global scope 的根目錄透過既有 `settings.rs::resolve_*_root` 與 `default_opencode_config_root` 解析（尊重使用者在設定頁的覆蓋值）。

### D5. 掃描器

- `WalkDir::new(root).max_depth(8).follow_links(false)` + `filter_entry` 忽略：`node_modules, .git, dist, build, vendor, .next, .nuxt, target, .sessionhub` + `prefs.ignored_paths`。
- 上限約 20,000 個目錄，超過即停止並設 `truncated: true`（UI 顯示警告）。
- 目錄含 AGENTS.md **或** CLAUDE.md 即產生 entry：兩者 hash 相等 → `in-sync`；僅 AGENTS.md → `target-missing`；皆存在但 hash 不同 → `differs`（`target_newer` 由 mtime 判定）；僅 CLAUDE.md → `source-missing`。
- 全域掃描**絕不**遞迴整個家目錄，僅檢查固定已知位置（各 agent root 的指示檔與 skills/command 目錄），並額外納入 `~/.agents/instructions/AGENTS.md` 與同目錄的 `CLAUDE.md` 對位。

### D5a. Skills 目錄比對範圍

計算 skill 目錄的內容雜湊聚合時，比對範圍套用與 D5 相同的忽略清單（`node_modules, .git, dist, build, vendor, .next, .nuxt, target`），避免使用者慣例中偶爾混入 skill 目錄的建置產物或相依套件（如觀察到的 `.opencode/node_modules`）拖慢比對或造成假性「內容不同」。忽略清單套用於 skill 來源目錄與各 target 目錄雙側。

### D5b. Skills 矩陣僅列出來源存在的項目

Skills 矩陣的列完全由**來源**掃描結果決定；若某 target 目錄下存在一個來源已不存在的同名 skill（例如來源被刪除但目標仍保留），系統不會為其產生獨立列，該項目對使用者不可見、也不參與同步。此為已知限制（v1 不做鏡像刪除／回收），非缺陷。

### D5c. Commands 矩陣列來源改為來源與 target 聯集

Commands 與 skills 不同，使用者常先在 `.claude/commands` 或 `.opencode/command` 維護既有命令，再逐步回補 `.agents/skills/command/`。因此 commands 矩陣的列 SHALL 由「來源目錄 + 各 target 目錄」的聯集決定：

- 若來源存在，`source_path` 與同步來源皆指向來源端。
- 若來源不存在但某 target 端存在，該列仍須顯示於矩陣，供使用者檢視現況並透過衝突流程決定是否回補來源。
- 實際同步時仍以預期來源路徑（`.agents/skills/command/...`）作為 canonical source path；若來源缺失，沿用 `source-missing` 衝突語意。

### D5d. SyncRequest 產生規則（Skills/Commands 矩陣 → 同步）

前端提交同步時，`SyncRequest.items` 為「使用者勾選的列（skill/command）」×「目前啟用的 target（`prefs.enabledTargets` 交集使用者本次額外勾選）」之笛卡爾積，並排除以下情況：
- 該 target 根目錄未偵測到（`TargetInfo.root_exists == false`）的組合會顯示於矩陣但不可勾選。
已 `in-sync` 的組合**仍會**產生 `SyncItem` 送入 dry-run（顯示為 `skip-in-sync`，方便使用者確認範圍完整），但套用階段對 `skip-in-sync` 項目不執行任何寫入。

### D6. 同步演算法（移植 agents-sync 語意）

逐 `SyncItem`：

1. 目標不存在 → `create`；hash 相等 → `skip-in-sync`。
2. **來源不存在但目標存在**（`source-missing`，僅 agents-md 會發生）：一律視為衝突（不比較 mtime），回報 `conflict`、不寫入；使用者於對話框選擇「目標→來源」可補回來源，或「略過」保持現狀。此情況不受 `force` 影響——`force` 僅加速「來源存在時」的覆蓋，不會單方面決定用目標覆蓋來源。
3. 來源、目標皆存在且 hash 不同：
   - `force == true` 或 item 帶明確 `direction` → 依方向 `overwrite`；
   - 目標 mtime 較新且專案無記住的 `conflict_choice` → 回報 `conflict`、不寫入（前端跳 SyncConflictDialog，使用者選定方向後帶 `direction` 重送）；
   - 其餘 → 預設來源→目標 `overwrite`。
4. `dry_run: true` 跑完全相同的管線但不落地，`SyncReport` 即為預覽（UI 逐項勾選後以 `dry_run: false` 重送選取項）。
5. 目錄同步（skills，`mode: Copy`）= 對來源目錄內逐檔展開為 per-file 計畫後聚合；不刪除目標端多餘檔案。
6. 寫入：`fs::create_dir_all` 父目錄 → 寫入 `*.tmp-sessionhub` 暫存檔 → `fs::rename` 原子替換。

### D6a. Skills 連結同步模式（`mode: Link`）

僅 skills 適用（agents-md 與 commands 恆為 `Copy`）。行為：

1. 目標不存在或目標非 symlink → 嘗試以 `std::os::windows::fs::symlink_dir` 於目標路徑建立指向來源 skill 目錄的目錄 symlink（先移除/略過既有內容，視 `force` 而定，非強制時遇既有實體內容視為衝突詢問）。
2. 建立失敗（`ERROR_PRIVILEGE_NOT_HELD`，即無開發者模式/管理員權限）→ 自動 fallback 為 `Copy` 模式執行該項，`SyncActionResult.action = "link-fallback-copy"`，`reason` 說明權限不足，UI 顯示提示（可開啟 Windows 開發者模式或以系統管理員身分執行以取得連結能力）。
3. 目標已是指向相同來源的 symlink → `skip-in-sync`。
4. 目標是指向**不同**來源的 symlink，或目標存在實體內容 → 視為衝突，依 D6 衝突流程詢問（方向為「以連結取代目標」或「略過」）。
5. 連結模式下 `SyncStatus` 判定：只要目標為指向正確來源的 symlink 即視為 `in-sync`，不需逐檔 hash 比對（symlink 保證內容永遠相同）。

### D7. 專案偏好持久化

`allowCreateProjectConfigDir` 僅控制**寫入/新建**行為，不影響讀取：

- **讀取**：一律優先檢查 `<project>/.sessionhub/agents.json`，存在即讀取使用（不論開關狀態）；不存在則讀取 APPDATA fallback；兩者皆無則用預設值。
- **寫入**：
  - 開關開啟：寫入 `<project>/.sessionhub/agents.json`（必要時建立資料夾；首次建立顯示 gitignore 提示）。
  - 開關關閉：
    - 若專案內該檔案已存在（他人建立或先前已建立）→ 仍寫入該既有檔案（尊重既有選擇，不因開關關閉而卡住既有工作流程）。
    - 若專案內該檔案不存在 → 寫入 APPDATA fallback（`project-agents/<hash-of-lowercased-project-path>.json`），不在專案內建立新檔案。
- **開關切換不做遷移**：從關閉切到開啟（或反之）不會自動搬移／合併 APPDATA 與專案內兩處的既有內容；兩處各自獨立累積，讀取時依上述優先序取用其一（不合併欄位）。

### D8. 前端結構

- `AgentsConfigView.tsx`（共用外殼）：props 含 `scope`、三組掃描資料、prefs 與 `onSync/onReadFile/onWriteFile/onRefresh/onOpenPath/onRevealPath/onOpenInEditor`。內部分頁列沿用 `.sub-tab-bar` 樣式：**AGENTS.md | Skills | Commands**。
  - AGENTS.md 分頁：左 ExplorerTree（`buildAgentsMdTree`；badge=狀態、tone 對映 in-sync→done / target-missing→not_started / differs→in_progress）+ 右 ContentViewer；「編輯」切換為 PlanEditor 式編輯區；工具列含外部編輯器開啟、檔案總管顯示、同步此目錄。
  - Skills / Commands 分頁：狀態矩陣表格——每列一個 entry（勾選框 + 名稱，點名稱右側預覽 md），每個 target 一欄狀態格（✓ 一致 / – 缺少 / ≠ 差異 / 較新! / 🔗 已連結）；欄標題勾選框綁 `prefs.enabledTargets`（root_exists=false 的欄位仍顯示但格內不可勾選）；Skills 分頁另有「同步模式」切換（複製 / 連結，預設複製）；底部「預覽同步（dry-run）」→ SyncReport 逐項勾選 →「套用」。
- `SyncConflictDialog.tsx`：逐衝突 radio（來源→目標 / 目標→來源 / 略過）+「套用到全部」+「記住此專案的選擇」；onResolve 回傳帶 direction 的 `SyncItem[]`；記住的選擇由 App.tsx 寫入 prefs。
- App.tsx 接線：`activeView === "agents-global"` 新分支 + Sidebar 單一 Agents 按鈕（位置固定於 Settings 上方，不因收折狀態重複渲染）；ProjectView 新增 `"agents"` sub-tab；Query keys `["agents-md"|"agents-skills"|"agents-commands", scopeKey]` 與 `["agents-prefs", projectCwd]`，以較長 `staleTime` 保留掃描結果，並搭配 component-level state 保留當前分頁/選取狀態，避免在切換其他頁面後回來時重新載入與跳回初始畫面；同步 mutation 成功後 invalidate 三個掃描 query。
- 視覺與密度：Agents 頁 SHALL 以工具面板為取向，移除不必要的外框、優先使用 icon button、壓縮 header 與 tab 區塊高度、減少巢狀滾動容器，讓主要矩陣與內容檢視區在常見桌面視窗中盡量完整顯示。
- 主題切換：全域主題控制 SHALL 移至 Settings，不再在 Sidebar footer 顯示獨立切換器，避免導覽與設定控制重複。

### D9. 全域 agents 來源根目錄可設定（`agentsSourceRoot`）

全域範圍的 skills/commands 正本目錄原寫死為 `~/.agents`（`default_agents_root`），但實際上不同使用者可能將正本集中存放於自訂位置（例如以自己的同步腳本推送到各工具目錄），`.agents` 目錄本身可能不存在或只是巧合產物。因此新增 `AppSettings.agentsSourceRoot`（`#[serde(default)]`，預設空字串）：

- **僅套用於全域範圍**：`skills_source_root`／`commands_source_root`／`global_instruction_roots`（AGENTS.md 全域掃描）在 `AgentsScope::Global` 時，改以 `resolve_agents_source_root(settings.agents_source_root)` 解析根目錄；未設定（空字串）時 fallback 至原本的 `~/.agents`。
- **不套用於專案範圍**：`AgentsScope::Project` 固定使用 `<project>/.agents`，不受此設定影響——專案級正本本來就該隨專案而非隨使用者機器設定漂移。
- **設定頁**：Settings 頁「Agents」區塊新增路徑輸入欄（含瀏覽資料夾按鈕），對應 `settings.fields.agentsSourceRoot`；留空時顯示預設值提示。
- 向後相容：舊版設定檔缺少此欄位時，`serde(default)` 補空字串，行為與升級前一致。

## Risks / Trade-offs

- **Windows `canonicalize` 的 `\\?\` 前綴**：containment 檢查兩側都必須 canonicalize；寫入目標可能尚不存在 → canonicalize 最深的既有祖先 + 詞法拼接，並在拼接前拒絕含 `..` 的相對路徑；顯示路徑時去除前綴。
- **大型 repo 掃描成本**：深度上限 8 + 目錄數上限 + `truncated` 旗標 + 背景執行緒 + query 僅在分頁可見時啟用。
- **mtime 粒度與時鐘偏移**：一致性以 hash 為準；mtime 誤判的失敗模式是「多問一次使用者」，安全。
- **`.sessionhub/` 誤入版控**：預設不建立（`allowCreateProjectConfigDir` 預設 false）；建立後 UI 顯示 gitignore 提示。
- **symlink/junction**：`follow_links(false)`；skills 目錄複製沿用同一 walker，不跟隨連結。
- **連結模式的權限脆弱性**：一般 Windows 使用者預設無 `SeCreateSymbolicLinkPrivilege`；建立失敗必須有清楚的 fallback-to-copy 路徑與提示文案，避免使用者誤以為已連結但實際上仍是各自獨立副本。
- **連結模式的來源移動/刪除風險**：來源 skill 目錄被刪除或搬移時，指向它的 symlink 會變成失效連結；掃描時 SHALL 偵測並標示為錯誤狀態（非 in-sync），避免使用者誤判為正常。

### D9. `classify_file_status` 的 error 狀態誤用修正（2026-07 實地驗證發現）

實地以使用者真實全域環境驗證（`~/.claude/skills`、`~/.claude/commands` 等大量採用 symlink/junction 慣例，指向外部目錄 `D:\ching\AI tool setting\...`）時發現：Skills 矩陣的 symlink 判定（`inspect_directory_symlink`）與聯集顯示邏輯運作正確，`Linked`/`TargetMissing` 狀態皆如預期產生；但 **Commands 矩陣**因來源 `.agents/skills/command/` 尚未建立，name 由 target 端（如 codex 的 `.codex/prompts/opsx-apply.md`）反查得到後，其餘各 target（如 claude）若也缺少該檔案，`classify_file_status` 會因「來源、目標皆不存在」落入預設分支回傳 `SyncStatus::Error`。前端將 `error` 顯示為中性「錯誤」標籤（`agents.status.error` = 「錯誤」），使用者容易誤判為系統未正確識別該 command，而非「尚待同步」。

修正：`classify_file_status` 的「來源、目標皆不存在」分支改回傳 `SyncStatus::TargetMissing`（而非 `Error`），語意與既有「來源存在但此 target 未同步」情況一致。`Error` 狀態保留給真正的例外（如 symlink 解析失敗、讀檔錯誤等）。已於 `src-tauri/src/agents_config.rs::classify_file_status` 完成程式修正；詳見 `agents-commands-sync` spec 新增的對應 Scenario 與 `tasks.md` 8.5。
