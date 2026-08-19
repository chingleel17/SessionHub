## 1. 設定欄位與型別

- [x] 1.1 `src-tauri/src/types/settings.rs` 的 `AppSettings` 新增 `terminal_launcher: Option<String>`，加上 `#[serde(default)]`，確認舊 settings.json 缺該欄位時可正常讀入
- [x] 1.2 `src-tauri/src/settings.rs` 定義 launcher 常數（`"shell"` / `"herdr"`）與解析 helper，未知值一律視為 `shell`
- [x] 1.3 `src/types/index.ts` 的 settings 型別新增 `terminalLauncher?: string | null`
- [x] 1.4 `src/utils/appSettingsDefaults.ts` 新增 `terminalLauncher: "shell"` 預設值
- [x] 1.5 `src/hooks/useAppSettingsForm.ts` 的表單合併與送出邏輯納入 `terminalLauncher`

## 2. 抽出共用啟動分派

- [x] 2.1 定義 `TerminalLaunchSpec`（`cwd` / `command: Option<&str>` / `label`）與 `launch_terminal(launcher, terminal_path, spec)` 分派入口
- [x] 2.2 將現有 shell 啟動邏輯抽為 `launch_via_shell`，保留 file_stem 白名單、`current_dir`、`CREATE_NEW_CONSOLE` 與 `configure_msys_stackdump_suppression`。**shell 路徑的 argv 一律以 `sessions/copilot.rs:414-429` 的現行程式碼為準**；`terminal-launcher` 規格中的 `Set-Location -Path`、`--init-file`、`zsh` 屬既有舊文字（實際為 `cd '<cwd>'`、`-i`，且 `VALID_TERMINAL_STEMS` 不含 zsh），不得據以改動行為
- [x] 2.3 改寫 `sessions/copilot.rs` 的 `open_terminal_internal` 改為呼叫 `launch_terminal`
- [x] 2.4 改寫 `commands/tools.rs` 的 `open_in_tool_internal` 中 terminal / opencode / claude / codex / copilot / gemini 六個分支，各自只組出 command 字串後呼叫 `launch_terminal`
- [x] 2.5 改寫 `resume_session_in_terminal_internal` 改為呼叫 `launch_terminal`
- [x] 2.6 確認 `vscode` shim 與 `explorer` 分支維持既有行為，不走 launcher 分派
- [x] 2.7 逐一比對重構前後各分支產生的 command 字串完全一致

## 3. herdr 啟動實作

- [x] 3.1 新增 herdr 模組，實作 `herdr_tab_create(cwd, label, focus)`：以 `output()` 執行 `herdr tab create --cwd <PATH> --label <TEXT> --focus`，解析 stdout JSON 取 `result.root_pane.pane_id` 與 `result.tab.tab_id`
- [x] 3.2 實作 `herdr_pane_run(pane_id, command)`：執行 `herdr pane run <PANE_ID> <COMMAND>`
- [x] 3.3 實作 `launch_via_herdr`：無 command 時只建立 tab；有 command 時建立 tab 後送出指令
- [x] 3.4 錯誤處理：herdr 可執行檔找不到、非零 exit code、JSON 無法解析或缺 `pane_id` 時，回傳含成因的錯誤訊息（附 stderr 片段）
- [x] 3.5 錯誤訊息區分「未偵測到」（不在 PATH）與「已安裝但服務未執行」（`herdr status server` 非 running），分別指引安裝或啟動
- [x] 3.6 確認 herdr 失敗時不自動回退至 shell 啟動器
- [x] 3.7 tab 標籤以專案目錄名稱為主、對應工具時附加工具識別（見 D11）
- [x] 3.8 確認 herdr 路徑不套用 Windows console creation flags

## 4. 聚焦行為分流

- [x] 4.1 `commands/tools.rs` 的 `focus_terminal_window_internal` 入口加入 launcher 判斷
- [x] 4.2 於 AppState 新增記憶體內的 session → tab_id 對應表，啟動時寫入、聚焦時查詢（不持久化至 SQLite）
- [x] 4.3 實作 `herdr_tab_focus(tab_id)`：執行 `herdr tab focus <TAB_ID>`
- [x] 4.4 herdr 模式下不進入 EnumWindows 比對；查無對應 tab_id 或 tab 已關閉時回傳可辨識的錯誤訊息供前端 toast 呈現
- [x] 4.5 確認 shell 模式的既有 Win32 聚焦邏輯完全不變

## 5. 終端路徑驗證分流

- [x] 5.1 `validate_terminal_path` command 新增 launcher 參數（依 D7，此為唯一由前端傳入 launcher 的 command，因驗證對象是尚未儲存的表單值），並於 `commands/mod.rs`、`lib.rs` 同步登記
- [x] 5.2 herdr 模式跳過 `VALID_TERMINAL_STEMS` 白名單，改以 PATH 解析或檔案存在判定
- [x] 5.3 herdr 指令無法解析時回報驗證失敗並提示 herdr 不可用
- [x] 5.4 確認既有測試 `validate_terminal_path_returns_true_for_existing_file` 仍通過

## 6. herdr 可用性偵測

- [x] 6.1 `src-tauri/src/types.rs` 的 `ToolAvailability` 新增 `herdr: bool` 欄位
- [x] 6.2 `check_tool_availability_internal` 以 `which_exists("herdr")` 填入該欄位
- [x] 6.3 新增服務狀態偵測：已安裝時執行 `herdr status server`，解析輸出判定是否 running，回傳可區分的狀態值
- [x] 6.4 `src/types/index.ts` 的 `ToolAvailability` 同步新增對應欄位
- [x] 6.5 儲存設定時使 `check_tool_availability` 查詢快取失效（`App.tsx:1095`）
- [x] 6.6 設定頁提供手動重新偵測入口，比照終端機路徑欄位既有的「自動偵測」按鈕（`SettingsView.tsx:239-245`）

## 7. 前端設定頁

- [x] 7.1 `src/components/SettingsView.tsx` 新增終端啟動器選擇控制項（受控元件，由 props 驅動），置於終端機路徑欄位鄰近位置
- [x] 7.2 launcher 選項在 herdr 不可用時停用並標示「未偵測到」；服務未執行時標示對應狀態（見 D8 / D9）
- [x] 7.3 確認當前已選取的 launcher 值一律照常渲染，即使該 launcher 已不可用，使用者仍可切回 shell
- [x] 7.4 確認終端機路徑欄位在 herdr 模式下仍顯示且可編輯
- [x] 7.5 provider 勾選區於資料根目錄旁顯示偵測狀態提示（重用 `check_directory_exists`），勾選框維持可用
- [x] 7.6 確認 provider 勾選可用性未被 CLI 安裝狀態影響，且 `onProviderAction(id, "install")` 流程不受影響（見 D10）
- [x] 7.7 `src/App.tsx` 僅於 `validate_terminal_path` 的 `invoke()` 傳入當前表單的 launcher；`open_in_tool`、`resume_session_in_terminal`、`focus_terminal_window` 三個呼叫的簽章維持不變（launcher 由後端讀設定，見 D7）
- [x] 7.8 `src/locales/zh-TW.ts` 新增啟動器與偵測狀態相關文案
- [x] 7.9 `src/locales/en-US.ts` 新增對應英文文案
- [x] 7.10 確認 JSX 中無硬編中文，所有文案透過 `t("key")` 取得

## 8. 測試與驗證

- [x] 8.1 新增單元測試：launcher 解析（未知值回退為 shell、缺漏欄位預設為 shell）
- [x] 8.2 新增單元測試：herdr `tab create` 回應 JSON 解析（成功取得 pane_id 與 tab_id、缺欄位時回錯誤）
- [x] 8.3 新增單元測試：`herdr status server` 輸出解析（running 與非 running 兩種情境）
- [x] 8.4 執行 `cargo test` 確認後端測試全數通過
- [x] 8.5 執行前端品質檢查（lint / type-check / build）
- [x] 8.6 手動驗證 shell 模式：開終端、啟動 AI CLI、resume session、聚焦終端行為皆與變更前一致
- [x] 8.7 手動驗證 herdr 模式：開終端建立 tab、啟動 AI CLI 於 pane 執行、tab 標籤可辨識、聚焦既有 tab 成功切換、tab 已關閉時錯誤訊息正確呈現
- [x] 8.8 手動驗證「未安裝」情境：暫時移除 herdr 於 PATH 的解析，確認設定頁標示未偵測到且選項停用
- [x] 8.9 手動驗證「服務未執行」情境：以 `herdr server stop` 停止服務，確認錯誤訊息指引啟動而非安裝
- [x] 8.10 手動驗證已選 herdr 但其不可用時，設定頁仍可切回 shell
- [x] 8.11 更新 `src-tauri/src/AGENTS.md` 的 command 清單（若有新增或調整 command 簽章）
