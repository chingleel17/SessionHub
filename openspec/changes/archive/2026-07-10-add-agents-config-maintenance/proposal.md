# Proposal: add-agents-config-maintenance

## Why

AI 開發環境的 agent 指示檔與擴充設定散落各處：全域目錄（`~/.agents`、`~/.claude`、`~/.codex`、`~/.copilot`、`~/.config/opencode`）與各專案子目錄底下都可能有 AGENTS.md、CLAUDE.md、skills、commands。目前只能靠檔案總管或各 agent 的 TUI 分別查看，維護不便：不知道哪些子目錄有指示檔、AGENTS.md 與 CLAUDE.md 內容是否一致、`.agents/skills` 來源改了之後哪些 agent 的 skills 目錄尚未更新。使用者已有 [agents-sync](https://github.com/chingleel17/agents-sync) CLI 解決 AGENTS.md → CLAUDE.md 的複製，但缺乏可視化的狀態總覽與同步操作介面，且 skills / commands 的同步尚無工具支援。

## What Changes

- **新增「Agents」全域頁面**：Sidebar 新增獨立導覽項目，內含 AGENTS.md、Skills、Commands 三個分頁，管理全域範圍的 agent 設定（`~/.agents/skills`、`~/.claude`、`~/.codex`、`~/.copilot`、`~/.config/opencode` 等固定已知位置）。
- **新增專案「Agents」子分頁**：ProjectView 的 sub-tab bar 新增 "agents" 分頁，與全域頁面共用同一元件（以 scope prop 區分），管理該專案目錄下的 AGENTS.md/CLAUDE.md、`.agents/skills`、`.claude/commands` 等。
- **AGENTS.md/CLAUDE.md 掃描與同步**：Rust 內建移植 agents-sync 語意——遞迴掃描（忽略 node_modules、.git 等）、AGENTS.md 為來源複製至同目錄 CLAUDE.md；以 Tree 呈現各目錄的同步狀態（一致 / 缺 CLAUDE.md / 內容不同 / 僅有 CLAUDE.md），支援檢視（markdown 渲染）、編輯、以外部編輯器開啟、在檔案總管顯示、dry-run 預覽與逐項勾選套用。
- **Skills 同步**：`.agents/skills/<name>/`（專案）與 `~/.agents/skills/`（全域）為來源，以狀態矩陣（每列一個 skill、每欄一個 agent 目標）顯示 `.claude/.codex/.opencode/.copilot` 各 skills 目錄的同步狀態並可勾選同步。支援兩種同步模式：**複製**（各目標保留獨立實體檔案）與**連結**（於目標位置建立 symlink 指向來源，達成永遠一致；Windows 無建立目錄 symlink 權限時自動 fallback 為複製並提示）。
- **Commands 同步**：`.agents/skills/command/*.md` 為來源，分發至各 agent 的 command 目錄（`.claude/commands/`、`.codex/prompts/`、`.opencode/command/`、`.copilot/prompts/`），v1 為純複製，保留日後格式轉換的擴充點。
- **衝突處理**：預設來源獲勝；若目標檔比來源新、或來源缺失（僅存在目標端），一律視為衝突並跳出對話框詢問覆蓋方向（來源→目標 / 目標→來源 / 略過），可勾選「記住此專案的選擇」。
- **專案級偏好設定檔**：記住的衝突選擇、忽略路徑、啟用的同步目標存於 `<project>/.sessionhub/agents.json`；新增全域設定 `allowCreateProjectConfigDir`（預設關閉，**僅控制是否允許新建/寫入**該檔案，不影響既有檔案的讀取）。關閉時若專案內尚無該檔案，偏好改存 `%APPDATA%\SessionHub\project-agents\<hash>.json`；若專案內已存在該檔案（例如由他人建立或先前已建立），無論開關狀態一律讀取並使用。開關切換不會自動搬移既有偏好內容，兩處各自獨立。

## Capabilities

### New Capabilities

- `agents-md-sync`: AGENTS.md/CLAUDE.md 的遞迴掃描、同步狀態 Tree、檢視/編輯/開啟操作、dry-run 與同步套用、衝突對話框。
- `agents-skills-sync`: Skills 來源掃描、per-target 狀態矩陣、複製或連結（symlink）同步、目標啟用開關。
- `agents-commands-sync`: Commands 來源掃描、per-agent 目錄對映、複製同步與格式轉換擴充點。

### Modified Capabilities

- `app-settings`: 新增 `allowCreateProjectConfigDir` 設定與專案級 agents 偏好（`.sessionhub/agents.json` 或 APPDATA fallback）的持久化行為。

## Impact

- **後端**：新增 `src-tauri/src/agents_config.rs`（掃描器、sha256 指紋、同步引擎、prefs 持久化）與 `src-tauri/src/commands/agents_config.rs`（Tauri 指令包裝）；`src-tauri/src/lib.rs` 註冊指令；`Cargo.toml` 新增 `walkdir`、`sha2`；`AppSettings`（`types.rs` + `settings.rs` + 兩處字面值建構）新增欄位。
- **前端**：新增 `src/components/AgentsConfigView.tsx`（共用外殼，三分頁）、`src/components/SyncConflictDialog.tsx`；`src/utils/buildTree.ts` 新增 `buildAgentsMdTree`；`src/App.tsx`（TanStack Query、IPC handlers、`activeView === "agents-global"` 分支）；`src/components/Sidebar.tsx`（新導覽按鈕）、`ProjectView.tsx`（新 sub-tab）、`Icons.tsx`、`SettingsView.tsx`（新設定區塊）；`src/types/index.ts` 鏡射型別；`src/locales/zh-TW.ts` / `en-US.ts` 新增 `agents.*` 翻譯鍵；`src/App.css` 矩陣表格與對話框樣式。
- **設定**：`AppSettings.allowCreateProjectConfigDir`（新欄位，預設 false）；專案內可能新增 `.sessionhub/` 資料夾（僅在使用者啟用時）。
- **規格**：新增 `agents-md-sync`、`agents-skills-sync`、`agents-commands-sync` spec；`app-settings` 需要 delta spec。
