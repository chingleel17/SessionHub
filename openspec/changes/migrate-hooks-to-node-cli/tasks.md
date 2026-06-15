> 實作備註：因專案 `package.json` 含 `"type": "module"`，`.js` 會被當 ESM 而 `require` 失效，故 Node 主軌腳本一律改用 `.cjs` 副檔名強制 CommonJS（規格僅要求 Node.js CLI hook，副檔名屬實作細節）。

## 1. Copilot Node 主軌（試點）

- [x] 1.1 新增 `hooks/copilot/modules/record-event.cjs`：實作 stdin payload 讀取、`JSON.parse` 解析、欄位擷取（sessionId/cwd/transcriptPath 多候選鍵）、bridge record 組裝（version 4）、append 寫入與 retry 退避，timestamp 用 `toISOString()`
- [x] 1.2 新增 Copilot 各事件 `.cjs` 入口腳本（on-session-start / on-session-end / on-user-prompt-submitted / on-pre-tool-use / on-post-tool-use / on-error-occurred），接受 `--bridge-path` / `--provider`，空 payload 或缺 bridge-path 時 exit 0
- [x] 1.3 `src-tauri/src/provider/copilot.rs`：`include_str!` 改嵌入 `.cjs` 與 `record-event.cjs`、更新 `hook_script_entries()`、移除 ps1 嵌入與清單項
- [x] 1.4 `copilot.rs`：`render_copilot_hook_command` 改產生 `node <script.cjs>` 主軌命令置於 `command` 欄；移除 PowerShell 命令產生函式與設定檔 `powershell` 欄位（sh 腳本檔仍寫至磁碟作手動退路）
- [x] 1.5 升 `HOOK_SCRIPT_VERSION` 為 3 以觸發既有安裝重新寫出

## 2. Copilot 驗證

- [x] 2.1 `cargo build` 通過（僅既有 dead-code 警告，無本次改動相關錯誤）
- [x] 2.4 以 `node` 直接驗證各事件 hook 寫出正確 version 4 record（timestamp ms 轉 ISO、prompt 截斷 80 字、toolResult failure 判定、空 payload / 缺 bridge-path 皆 exit 0）
- [ ] 2.2 （需 app 手動驗證）安裝 Copilot 整合，確認設定檔主命令為 `node <script.cjs>`、無 `powershell` 欄位
- [ ] 2.3 （需 app 手動驗證）確認 provider 原生 hook 目錄已寫出 `.cjs` 與 `record-event.cjs`，且無殘留 `.ps1`

## 3. 套用至 Codex

- [x] 3.1 新增 `hooks/codex/modules/record-event.cjs` 與各事件 `.cjs` 入口腳本（比照 Copilot）
- [x] 3.2 `src-tauri/src/provider/codex.rs`：`include_str!` 改嵌 cjs、更新 `hook_script_entries()`、移除 ps1
- [x] 3.3 `codex.rs`：hook 命令改 node 主軌置於 `command` 欄、移除 `commandWindows` PowerShell 欄位（sh 腳本檔仍寫至磁碟；`is_sessionhub_hook_group` 保留舊 commandWindows 偵測以清理升級前的 group）
- [x] 3.4 以 `node` 直接驗證 Codex 各事件 cjs 寫入 bridge 正確（session.started/source、session.stop/stop_reason）

## 4. 套用至 Claude

- [x] 4.1 新增 `.claude/hooks/modules/record-event.cjs` 與各事件 `.cjs` 入口腳本（record-event.cjs 自帶 retry，不再需要 db-ops/task-queue 模組）
- [x] 4.2 `src-tauri/src/provider/claude.rs`：`include_str!` 改嵌 cjs、更新 `hook_script_entries()`、移除 ps1/psm1
- [x] 4.3 `claude.rs` `managed_hook_group`：主命令改 `node <script.cjs>` 置於 `command` 欄、移除 `commandWindows` 欄位（sh 腳本檔仍寫至磁碟）
- [x] 4.4 cargo test 驗證 Claude 冪等（重複安裝不重複 group）與 node 命令格式；以 node 直接驗證 cjs 寫入正確

## 5. 收尾

- [x] 5.1 移除三 provider 下所有 `.ps1` / `.psm1` 檔案（共 21 個）
- [x] 5.2 `mod.rs` 經檢視無 PowerShell 命令產生邏輯需清理（`install_hook_scripts` 為通用寫檔；測試 fixture 的 ps1 字串測「保留使用者檔案」行為與副檔名無關，保留）
- [x] 5.3 全專案搜尋並更新 lib.rs 測試 fixture / 斷言為 node + cjs；保留 codex.rs/claude.rs 對舊 commandWindows group 的相容偵測（升級時清理用）；terminal 的 pwsh 與 hook 無關保留
- [x] 5.4 `cargo test --lib`（73 passed）與 `npm run build`（tsc + vite）皆通過

## 6. 需 app 手動端到端驗證（非自動化範疇）

- [x] 6.1 於 app 安裝各 provider 整合，確認設定檔主命令為 `node <script.cjs>`、無 `powershell` / `commandWindows` 欄位
- [x] 6.2 確認 provider 原生 hook 目錄已寫出 `.cjs` 與 `record-event.cjs`，且舊 `.ps1` 經升版（version 3）重裝後被清除
- [x] 6.3 實際觸發各 provider session，確認 bridge events.jsonl 即時新增 record、app 活動狀態正確更新
