# Design: simplify-agents-skills-sync

## Context

現行 Skills 矩陣（`add-agents-config-maintenance`）將 `.agents/skills` 視為正本，對 claude / codex / opencode / copilot 四個目標逐一計算同步狀態並執行複製或連結。經 2026-07 查證官方文件，codex、opencode、Copilot CLI 皆原生讀取 `.agents/skills`（專案級與 `~/.agents/skills`），codex 甚至不存在 `~/.codex/skills`；opencode 與 Copilot CLI 另外還直接讀 `.claude/skills`。因此對這三家的同步是無效工，矩陣顯示的「未安裝」也是誤導。唯一真正需要同步的目標是 Claude Code 的 `.claude/skills`（Claude Code 不讀 `.agents`）。

Commands 不適用同一簡化：codex custom prompts 已棄用且僅讀 `~/.codex/prompts`、opencode 僅讀自家 commands 目錄、Copilot CLI 無 prompt files 機制。

## Goals / Non-Goals

**Goals:**

- Skills 矩陣簡化為 agents / claude 兩欄，消除誤導性的「未安裝」狀態
- 掃描與同步邏輯只處理 `.agents` 與 `.claude` 兩處（＋全域自訂正本位置）
- 全域自訂正本位置時，提供 `~/.agents` → 自訂位置的 symlink 狀態檢查與建立操作
- 欄標題提供目錄路徑提示與開啟目錄捷徑
- 表頭明示 `.agents` 的原生相容平台

**Non-Goals:**

- 不清理、不偵測各工具舊目錄（`~/.codex/skills`、`~/.config/opencode/skills`、`~/.copilot/skills`）中先前複製的內容
- 不改動 AGENTS.md 頁籤與 Commands 頁籤（`command_target_roots` 維持四目標）
- 不處理 codex ADMIN 級（`/etc/codex/skills`）與 SYSTEM 級 skills 的顯示

## Decisions

### D1：欄位語意 — agents 欄是「正本狀態」，claude 欄是唯一同步目標

矩陣固定兩欄：

- **agents 欄**：呈現該 skill 於正本（canonical source）的狀態。預設情境（正本＝`.agents`）下：skill 存在於正本 → 顯示「正本」；skill 僅從 `.claude` 端探索到 → 顯示「未收錄」（可透過既有 target-to-source 回補流程收錄）。
- **claude 欄**：`.claude/skills/<name>` 相對正本的同步狀態，沿用既有 SyncStatus（一致／內容不同／未安裝／已連結／連結失效），為唯一可勾選的同步目標。

全域 scope 且設定自訂正本位置時，canonical source＝自訂位置，agents 欄改呈現 `~/.agents/skills/<name>` 相對自訂正本的狀態（已連結／一致／內容不同／未安裝），且 agents 欄也成為可同步目標（把自訂正本佈署到 `~/.agents` 供三家工具讀取）。

替代方案：保留三家唯讀欄顯示「原生讀取」。否決——每格資訊固定不變，佔空間無資訊量，改以表頭一行相容說明取代。

### D2：後端 target 結構 — targets 依情境組裝，agents 欄預設由 source fingerprint 推導

`skill_target_roots` 改為：

- 專案 scope：`[("claude", <project>/.claude/skills)]`
- 全域 scope（未自訂正本）：`[("claude", ~/.claude/skills)]`
- 全域 scope（自訂正本 ≠ `~/.agents`）：`[("agents", ~/.agents/skills), ("claude", ~/.claude/skills)]`

預設情境的 agents 欄不需後端 target——`SkillEntry.file_count`／source fingerprint 已足以讓前端判定「正本／未收錄」，避免把 source 同時當 target 造成自我比對。`TargetInfo` 沿用，前端據 `targets` 陣列動態渲染欄位，無 schema 變更。

替代方案：後端永遠回傳 agents 虛擬 target。否決——預設情境下 source 與 target 同路徑，狀態恆為 in-sync，徒增比對成本與語意混淆。

### D3：自訂正本的 link 檢查放在「根目錄層級」，以 banner 呈現

檢查對象是 `~/.agents` 整個目錄（或至少 `~/.agents/skills`）是否為指向自訂位置的 symlink，而非逐 skill 檢查。新增 Tauri command `check_agents_root_link`（回傳 `linked | not-linked | conflict(實體目錄已存在) | missing`）與 `link_agents_root`（建立 symlink；`~/.agents` 不存在時直接建立、已存在實體目錄時回報衝突不強制覆蓋）。掃描結果之上以 banner 顯示狀態與「建立連結」按鈕；已連結時 agents 欄自然全部呈現「已連結」。

選擇 `~/.agents` 整體 link 而非 `~/.agents/skills`：一次涵蓋 skills 與 instructions 等未來內容；若 `~/.agents` 已有實體內容則退而建議 `~/.agents/skills` 層級連結，由衝突訊息引導使用者手動處理，不自動搬移。

### D4：欄標題互動

欄標題（agents / claude）hover 以 `title` 屬性顯示該欄根目錄完整路徑；點擊欄標題名稱以既有 `revealItemInDir`/`openPath` 於檔案總管開啟該目錄（目錄不存在時 disabled 並於 tooltip 註明）。claude 欄保留啟用勾選框（enabledTargets 持久化沿用，僅剩 claude／agents 兩個有效值；舊偏好中的 codex/opencode/copilot 值讀取時忽略）。

### D5：文案

- 表頭說明：`.agents 原生相容：codex / opencode / copilot（無需同步）`，並附上「僅 Claude Code 需同步至 .claude」的補充。
- agents 欄新狀態鍵：`agents.status.canonical`（「正本」）、`agents.status.not-in-source`（「未收錄」）。
- `agents.status.source-missing` 現行譯文「僅存正本」語意方向錯誤（該狀態實為「此端有、正本沒有」），一併修正為「僅存此端」。
- 設定頁 `agentsSourceRootDesc` 補充相容平台說明。

### D6：同步模式預設「連結」

`AgentsConfigView` 的 syncMode 預設值已改為 `link`（前次修改完成），本 change 的 spec delta 一併把「預設為複製」的既有敘述改為「預設為連結」。

## Risks / Trade-offs

- [使用者已依賴舊行為把 skills 複製到 codex/opencode/copilot 目錄] → 不清理舊目錄，工具端行為不變；矩陣不再顯示這些欄，功能面無破壞。若 `.agents` 與舊目錄內容分歧，以工具各自的優先序為準（超出本 app 管轄）。
- [`~/.agents` 已存在實體內容時無法直接 link] → `link_agents_root` 回報 conflict，不自動搬移或覆蓋；banner 顯示指引讓使用者手動合併後再連結。
- [舊 enabledTargets 偏好含已移除的 target id] → 讀取時過濾無效值，寫回時只保留 agents/claude，向前相容。
- [Windows symlink 權限不足] → 沿用既有 `is_symlink_privilege_error` 偵測與開發者模式提示；根層級 link 失敗不自動退回複製（正本佈署用複製會失去單一正本意義），僅提示。

## Migration Plan

1. 後端調整 `skill_target_roots` 與新增 link 檢查 commands，既有測試同步更新
2. 前端矩陣改雙欄渲染與 banner，locales 補文案
3. 無資料遷移；`agents.json` 偏好向前相容（過濾無效 target id）

## Open Questions

（無——欄位語意、link 層級、舊目錄處置已於提案階段與使用者確認）
