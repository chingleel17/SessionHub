## Why

全域範圍設定自訂 agents 正本位置後，Skills 子分頁的連結狀態檢查要求 `~/.agents` 整層必須是指向自訂位置的 symlink；只要 `~/.agents` 是實體目錄就一律回報 conflict（「~/.agents 已存在實體內容，無法自動連結」）並要求手動合併。但實體目錄是合法且常見的佈局——使用者可能以自有腳本逐項 symlink 子目錄（如 `instructions`、`skills` 個別連到正本）、或直接在 `~/.agents` 維護實體檔案。現行檢查對這些使用者是誤報，且提供的「先手動合併再整層連結」指引反而會破壞其既有佈局。

## What Changes

- `check_agents_root_link` 後端檢查放寬：`~/.agents` 為實體目錄時不再一律回報 `conflict`，改為檢查「內容等效性」——逐一檢視正本根目錄下的第一層子項目（目錄與檔案），若 `~/.agents` 內對應項目均為解析到正本對應路徑的 symlink，視為等效連結（`linked`）；部分對應、部分缺漏或指向他處時回報新狀態 `partial`；完全無對應時回報 `unlinked-physical`。
- 移除 `conflict` 作為阻擋性錯誤：實體目錄情境的 banner 由錯誤語氣降級為資訊性提示，說明目前佈局與正本的關係，不再顯示「無法自動連結、請先手動合併」。
- 「建立連結」自動化僅在 `~/.agents` 完全不存在（`missing`）時提供整層 symlink 建立；其餘狀態不提供自動改寫，避免破壞使用者既有佈局。
- 前端 `AgentsRootLinkStatus` 型別與 banner 文案（zh-TW / en-US）同步更新。

## Capabilities

### New Capabilities

（無）

### Modified Capabilities

- `agents-skills-sync`: 「全域自訂正本位置的 ~/.agents 連結檢查」requirement 修改——實體目錄不再視為 conflict，新增逐項等效性判定（`linked` / `partial` / `unlinked-physical`），「建立連結」僅於 `missing` 時提供。

## Impact

- 後端：`src-tauri/src/agents_config.rs` 的 `AgentsRootLinkStatus` enum、`check_agents_root_link_against`、`link_agents_root_to` 與對應單元測試
- 前端：`src/types/index.ts` 的 `AgentsRootLinkStatus` 型別、`src/components/AgentsConfigView.tsx` 的 banner 渲染分支
- 文案：`src/locales/zh-TW.ts`、`src/locales/en-US.ts` 的 `agents.rootLink.*` keys
- 不影響：專案範圍 agents、Skills/Commands 同步矩陣本身、`agentsSourceRoot` 設定行為
