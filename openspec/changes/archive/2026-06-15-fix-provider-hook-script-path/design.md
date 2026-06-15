## Context

Provider hook 重構引入了 `.sh` / `.ps1` 雙版本腳本與「腳本式」hook 指令。安裝流程分成兩段：

1. `ensure_*_hook_scripts_installed`：把嵌入的腳本（`include_str!`）寫出到磁碟。
2. `render_*_integration`：產生 `hooks.json` 中每個事件的 hook 指令字串（內含腳本絕對路徑）。

`settings.rs` 為此新增了兩組函式：
- `default_*_hook_scripts_root` → `~/.codex/hooks`、`~/.copilot/hooks`
- `bundled_*_hook_scripts_root` → `%APPDATA%\SessionHub\.codex\hooks`、`...\.copilot\hooks`

目前的缺陷：`ensure_*` 用 **bundled**（寫出位置），`render_*` 用 **default**（指令引用位置），兩者指向不同目錄。實測 `~/.codex/hooks/` 不存在，導致 codex 執行 `sh '~/.codex/hooks/on-*.sh'` 時找不到檔案而 `exit 1`。失敗的 hook 在每次工具呼叫被觸發，加上遞迴 watcher 偵測 session 檔變更而 refresh，形成回授迴圈，於掃描期間拖垮 UI。

此外 `is_sessionhub_hook_group` 只比對新版 `# sessionhub-provider-event-bridge` marker，無法辨識舊版 v4 PowerShell 內嵌 group，導致 `retain` 未清乾淨，`hooks.json` 出現新舊 group 並存。

## Goals / Non-Goals

**Goals**
- 安裝與寫入 config 兩處對 hook 腳本根目錄的解析收斂到單一來源，永遠一致。
- 腳本實際寫出於 `~/.codex/hooks`、`~/.copilot/hooks`（與整合檔指令引用一致）。
- 安裝/更新時清除舊版內嵌 hook group，保證每事件僅一個 SessionHub group。
- 移除 `bash.exe.stackdump` 殘留檔。

**Non-Goals**
- 不移除 sh 版本腳本（維持 sh + ps1 雙版本）。
- 不改動 Claude 既有安裝路徑邏輯（Claude 已正常）。
- 不重寫 watcher 架構；回授迴圈在 hook 修好後即自然消失。
- 不變更 jq 依賴策略（沿用既有 `_ensure_jq` 安全退出）。

## Decisions

### 決策 1：收斂為單一腳本根目錄來源（default_*_hook_scripts_root）

`ensure_codex_hook_scripts_installed` / `ensure_copilot_hook_scripts_installed` 改用 `default_*_hook_scripts_root()`，與 `render_*_integration` 取得腳本路徑的來源一致。

**為何選 default 而非 bundled**：`hooks.json` 由 codex 直接讀取並執行其中的絕對路徑指令，路徑必須是 codex 看得到的真實位置；`~/.codex/hooks` 與 codex 設定同層，語意清楚且就近。

**替代方案（已否決）**：讓 `render_*` 改用 bundled 路徑。否決原因——`%APPDATA%\SessionHub` 是本應用私有資料夾，把 codex 要執行的腳本塞進別的 app 的資料夾語意不佳，且 uninstall 時較難對齊清理。

**收斂手段**：移除 `bundled_codex_hook_scripts_root` / `bundled_copilot_hook_scripts_root` 的使用點（若無其他引用則一併刪除函式），避免日後再被誤用造成分歧。

### 決策 2：強化 is_sessionhub_hook_group 辨識舊版 group

`is_sessionhub_hook_group` 除比對新版 marker 外，新增辨識舊版內嵌指令特徵——`command` / `commandWindows` 字串包含 `provider = 'codex'`（v4 PowerShell 內嵌記錄指令的穩定特徵）。如此 `retain(|g| !is_sessionhub_hook_group(g))` 能同時清除新舊 group。

**替代方案（已否決）**：一次性 migration 掃描整份 `hooks.json` 刪除舊 group。否決原因——安裝流程本就會 `retain` + `push`，把辨識補齊即可自然完成升級，無需額外 migration 路徑。

### 決策 3：Copilot 對齊同樣修正

Copilot 安裝邏輯與 Codex 同構，套用相同的根目錄收斂與 group 清理修正，避免 copilot 日後重蹈覆轍。

## Risks / Trade-offs

- [既有錯誤 `hooks.json` 仍殘留舊 group] → 使用者下次安裝/更新時由強化後的 `is_sessionhub_hook_group` 自動清除；無需手動處理。
- [`~/.codex/hooks` 寫入權限問題] → 與 codex 自身設定同層，權限模型一致；寫入失敗時沿用既有 `build_install_failure_status` 回報。
- [sh 在 Windows 仍依賴 jq] → 非本次範圍；`record-event.sh` 既有 `_ensure_jq` 會安全退出（exit 0），不會再造成 exit 1。

## Migration Plan

1. 修正 `ensure_*_hook_scripts_installed` 改用 `default_*_hook_scripts_root`。
2. 收斂/移除 `bundled_*_hook_scripts_root` 使用點。
3. 強化 `is_sessionhub_hook_group`。
4. 刪除 `bash.exe.stackdump`。
5. 使用者重新對 Codex / Copilot 執行「安裝整合」即完成自我修復——新腳本寫到正確位置、舊 group 被清除。

回滾策略：本變更為純修正，回滾即還原至目前（壞掉）狀態；無資料遷移風險。

## Open Questions

- codex 在 Windows 上是否確實優先執行 `commandWindows` 而非 `command`？本次保留 sh 雙版本，但若 codex 在 Windows 仍走 `command`（sh），則需另案確認 Git Bash + jq 環境齊備；此問題不阻擋本次路徑一致性修正。
