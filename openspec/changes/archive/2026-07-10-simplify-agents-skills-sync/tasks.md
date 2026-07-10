# Tasks: simplify-agents-skills-sync

## 1. 後端：掃描與目標結構簡化

- [x] 1.1 `src-tauri/src/agents_config.rs`：`skill_target_roots` 改為情境式組裝——專案 scope `[("claude", <project>/.claude/skills)]`；全域未自訂正本 `[("claude", ~/.claude/skills)]`；全域自訂正本（≠ `~/.agents`）`[("agents", ~/.agents/skills), ("claude", ~/.claude/skills)]`
- [x] 1.2 `scan_agents_skills_internal`：反向探索來源僅剩 claude（與自訂情境的 agents）target，移除 codex/opencode/copilot 相關路徑；確認 `resolve_project_target_root`（.github/.copilot fallback）不再被 skills 使用（commands 仍用則保留）
- [x] 1.3 `ProjectAgentsPrefs.enabled_targets`：讀取時過濾僅剩 `agents`/`claude` 有效值，`default_enabled_targets`（skills 用途）調整；注意 commands 頁籤仍需四 provider——若 enabledTargets 為 skills/commands 共用，需拆分或僅於 skills 流程過濾，勿破壞 commands
- [x] 1.4 更新既有 Rust 測試：移除/改寫涉及 codex/opencode/copilot skills 目標的案例，新增「全域自訂正本時 targets 含 agents 與 claude」案例，`cargo test --lib agents_config` 通過

## 2. 後端：~/.agents 連結檢查與建立

- [x] 2.1 新增 `check_agents_root_link_internal`：回傳 `linked | not-linked | conflict | missing`（`~/.agents` 為指向自訂位置的 symlink → linked；不存在 → missing；為實體目錄 → conflict；symlink 指向他處 → not-linked）
- [x] 2.2 新增 `link_agents_root_internal`：`~/.agents` 不存在時建立 symlink 指向自訂位置；conflict 時回傳錯誤不覆蓋；沿用 `is_symlink_privilege_error` 產生權限提示訊息
- [x] 2.3 `src-tauri/src/commands/mod.rs` 與 `lib.rs`：註冊 `check_agents_root_link`、`link_agents_root` Tauri commands
- [x] 2.4 新增 Rust 測試：missing→建立成功、conflict 不覆蓋、已 linked 冪等

## 3. 前端：矩陣雙欄與 banner

- [x] 3.1 `src/components/AgentsConfigView.tsx`：Skills 矩陣改依後端 `targets` 動態渲染欄位；預設情境補渲染 agents 欄（由 `SkillEntry` source fingerprint/file_count 推導「正本／未收錄」，不可勾選、不參與同步）
- [x] 3.2 表頭加入相容說明列（`.agents 原生相容：codex / opencode / copilot（無需同步）`）
- [x] 3.3 欄標題：`title` 屬性顯示該欄根目錄完整路徑；點擊欄名以 opener 開啟目錄（不存在時 disabled）；claude 欄保留啟用勾選框
- [x] 3.4 全域 scope 且 `agentsSourceRoot` 有值時：載入 `check_agents_root_link` 結果渲染 banner（未連結→「建立連結」按鈕；conflict→合併指引；linked→已連結徽章）；IPC 呼叫依專案慣例集中於 `App.tsx` 傳 props
- [x] 3.5 `buildMatrixSyncRequest`：目標集合改為後端 targets（claude／agents），移除 codex/opencode/copilot 組裝路徑；Commands 頁籤流程不動

## 4. 文案

- [x] 4.1 `zh-TW.ts` / `en-US.ts`：新增 `agents.status.canonical`（正本／Canonical）、`agents.status.not-in-source`（未收錄／Not in source）、相容說明、banner 與連結操作文案
- [x] 4.2 修正 `agents.status.source-missing`：「僅存正本」→「僅存此端」（Source only → Only here；語意方向修正）
- [x] 4.3 `settings.fields.agentsSourceRootDesc` 補充：`.agents` 原生相容 codex/opencode/copilot、自訂位置需將 `~/.agents` 連結過去的說明

## 5. 品質檢查

- [x] 5.1 `cargo test --lib agents_config` 全數通過
- [x] 5.2 `tsc --noEmit` 通過
- [x] 5.3 手動走查：專案 scope 雙欄矩陣、全域 scope（預設正本）雙欄矩陣、全域 scope（自訂正本）三情境（未連結→建立連結→已連結）、欄標題開啟目錄、Commands 頁籤四欄不受影響
      （本次僅完成程式碼審閱、後端測試與 tsc 型別檢查；因環境無法啟動 Tauri 桌面視窗，尚未實際點擊走查 UI，需使用者另行驗證）
