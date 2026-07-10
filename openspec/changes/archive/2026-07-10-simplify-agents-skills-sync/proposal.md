# Proposal: simplify-agents-skills-sync

## Why

經查證官方文件（2026-07），codex CLI、opencode、Copilot CLI 三者皆已原生讀取 `.agents/skills`（專案級與全域 `~/.agents/skills`），且 codex 並不存在 `~/.codex/skills` 路徑——目前 Skills 矩陣把 skills 複製到 codex/opencode/copilot 各自目錄的同步機制不僅多餘，還會誤導使用者（顯示「未安裝」但實際上工具可直接讀取正本）。Skills 只需維護 `.agents`（正本）與 `.claude`（Claude Code 專用）兩處即可。

查證依據：

- codex：`$CWD/.agents/skills`、`$REPO_ROOT/.agents/skills`、`~/.agents/skills`、ADMIN 級 `/etc/codex/skills`、內建 SYSTEM 級；無 `~/.codex/skills`
- opencode：專案級 `.opencode/skills`、`.claude/skills`、`.agents/skills`；全域 `~/.config/opencode/skills`、`~/.claude/skills`、`~/.agents/skills`
- Copilot CLI：專案級 `.github/skills`、`.claude/skills`、`.agents/skills`；全域 `~/.copilot/skills`、`~/.agents/skills`
- codex custom prompts 已棄用且僅讀 `~/.codex/prompts`、opencode commands 僅讀自家目錄、Copilot CLI 無 prompt files 機制 → Commands 頁籤不在本次簡化範圍

## What Changes

- **BREAKING（UI 行為）** Skills 矩陣目標欄由 claude / codex / opencode / copilot 四欄改為 **agents / claude** 兩欄：agents 欄呈現正本狀態、claude 欄為唯一同步目標；不再對 codex/opencode/copilot 產生同步項目
- Skills 表頭新增相容性說明：「.agents 原生相容：codex / opencode / copilot」
- 矩陣欄標題 hover 顯示該欄目錄完整路徑，點擊欄標題可於檔案總管開啟該目錄
- Skills 掃描來源改為聯集偵測 `.agents/skills` + `.claude/skills`（全域 scope 且設定了自訂正本位置時，額外納入自訂位置）
- 全域 scope 設定自訂正本位置時，檢查 `~/.agents` 是否已 symlink 至自訂位置；未連結時顯示狀態與「建立連結」操作
- 各工具舊目錄（`~/.codex/skills`、`~/.config/opencode/skills`、`~/.copilot/skills` 等）不再掃描、不偵測、不清理
- AGENTS.md 頁籤與 Commands 頁籤維持現狀

## Capabilities

### New Capabilities

（無）

### Modified Capabilities

- `agents-skills-sync`：目標矩陣由四 provider 欄改為 agents/claude 雙欄；掃描來源聯集改為 `.agents` + `.claude`（+ 全域自訂位置）；新增全域自訂正本位置的 `~/.agents` link 狀態檢查與建立連結操作；欄標題路徑提示與開啟目錄

## Impact

- `src-tauri/src/agents_config.rs`：`skill_target_roots`、`scan_agents_skills_internal`、相關掃描與同步請求組裝；新增 `~/.agents` link 狀態檢查與建立連結 command
- `src-tauri/src/commands/mod.rs`、`src-tauri/src/lib.rs`：若新增 Tauri command 需註冊
- `src/components/AgentsConfigView.tsx`：Skills 矩陣渲染（雙欄）、相容性說明、欄標題 hover/點擊開啟目錄、link 狀態列
- `src/locales/zh-TW.ts`、`src/locales/en-US.ts`：新增/調整文案
- 既有 Rust 測試（`agents_config` 模組）需隨掃描邏輯調整
- Commands 頁籤程式路徑不動（`command_target_roots` 等維持四目標）
