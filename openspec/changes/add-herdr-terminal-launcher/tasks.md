## 1. 設定欄位與型別

- [ ] 1.1 `src-tauri/src/types/settings.rs` 的 `AppSettings` 新增 `terminal_launcher: Option<String>`，加上 `#[serde(default)]`，確認舊 settings.json 缺該欄位時可正常讀入
- [ ] 1.2 `src-tauri/src/settings.rs` 定義 launcher 常數（`"shell"` / `"herdr"`）與解析 helper，未知值一律視為 `shell`
- [ ] 1.3 `src/types/index.ts` 的 settings 型別新增 `terminalLauncher?: string | null`
- [ ] 1.4 `src/utils/appSettingsDefaults.ts` 新增 `terminalLauncher: "shell"` 預設值
- [ ] 1.5 `src/hooks/useAppSettingsForm.ts` 的表單合併與送出邏輯納入 `terminalLauncher`

## 2. 抽出共用啟動分派

- [ ] 2.1 定義 `TerminalLaunchSpec`（`cwd` / `command: Option<&str>` / `label`）與 `launch_terminal(launcher, terminal_path, spec)` 分派入口
- [ ] 2.2 將現有 shell 啟動邏輯抽為 `launch_via_shell`，保留 file_stem 白名單、`current_dir`、`CREATE_NEW_CONSOLE` 與 `configure_msys_stackdump_suppression`。**shell 路徑的 argv 一律以 `sessions/copilot.rs:414-429` 的現行程式碼為準**；`terminal-launcher` 規格中的 `Set-Location -Path`、`--init-file`、`zsh` 屬既有舊文字（實際為 `cd '<cwd>'`、`-i`，且 `VALID_TERMINAL_STEMS` 不含 zsh），不得據以改動行為
- [ ] 2.3 改寫 `sessions/copilot.rs` 的 `open_terminal_internal` 改為呼叫 `launch_terminal`
- [ ] 2.4 改寫 `commands/tools.rs` 的 `open_in_tool_internal` 中 terminal / opencode / claude / codex / copilot / gemini 六個分支，各自只組出 command 字串後呼叫 `launch_terminal`
- [ ] 2.5 改寫 `resume_session_in_terminal_internal` 改為呼叫 `launch_terminal`
- [ ] 2.6 確認 `vscode` shim 與 `explorer` 分支維持既有行為，不走 launcher 分派
- [ ] 2.7 逐一比對重構前後各分支產生的 command 字串完全一致

## 3. herdr 啟動實作

- [ ] 3.1 新增 herdr 模組，實作 `herdr_tab_create(cwd, label, focus)`：以 `output()` 執行 `herdr tab create --cwd <PATH> --label <TEXT> --focus`，解析 stdout JSON 取 `result.root_pane.pane_id` 與 `result.tab.tab_id`
- [ ] 3.2 實作 `herdr_pane_run(pane_id, command)`：執行 `herdr pane run <PANE_ID> <COMMAND>`
- [ ] 3.3 實作 `launch_via_herdr`：無 command 時只建立 tab；有 command 時建立 tab 後送出指令
- [ ] 3.4 錯誤處理：herdr 可執行檔找不到、非零 exit code、JSON 無法解析或缺 `pane_id` 時，回傳含成因的錯誤訊息（附 stderr 片段）
- [ ] 3.5 確認 herdr 路徑不套用 Windows console creation flags

## 4. 聚焦行為分流

- [ ] 4.1 `commands/tools.rs` 的 `focus_terminal_window_internal` 入口加入 launcher 判斷
- [ ] 4.2 於 AppState 新增記憶體內的 session → tab_id 對應表，啟動時寫入、聚焦時查詢（不持久化至 SQLite）
- [ ] 4.3 實作 `herdr_tab_focus(tab_id)`：執行 `herdr tab focus <TAB_ID>`
- [ ] 4.4 herdr 模式下不進入 EnumWindows 比對；查無對應 tab_id 或 tab 已關閉時回傳可辨識的錯誤訊息供前端 toast 呈現
- [ ] 4.5 確認 shell 模式的既有 Win32 聚焦邏輯完全不變

## 5. 終端路徑驗證分流

- [ ] 5.1 `validate_terminal_path` command 新增 launcher 參數（依 D7，此為唯一由前端傳入 launcher 的 command，因驗證對象是尚未儲存的表單值），並於 `commands/mod.rs`、`lib.rs` 同步登記
- [ ] 5.2 herdr 模式跳過 `VALID_TERMINAL_STEMS` 白名單，改以 PATH 解析或檔案存在判定
- [ ] 5.3 herdr 指令無法解析時回報驗證失敗並提示 herdr 不可用
- [ ] 5.4 確認既有測試 `validate_terminal_path_returns_true_for_existing_file` 仍通過

## 6. 前端設定頁

- [ ] 6.1 `src/components/SettingsView.tsx` 新增終端啟動器選擇控制項（受控元件，由 props 驅動）
- [ ] 6.2 `src/App.tsx` 僅於 `validate_terminal_path` 的 `invoke()` 傳入當前表單的 launcher；`open_in_tool`、`resume_session_in_terminal`、`focus_terminal_window` 三個呼叫的簽章維持不變（launcher 由後端讀設定，見 D7）
- [ ] 6.3 `src/locales/zh-TW.ts` 新增啟動器相關文案
- [ ] 6.4 `src/locales/en-US.ts` 新增對應英文文案
- [ ] 6.5 確認 JSX 中無硬編中文，所有文案透過 `t("key")` 取得

## 7. 測試與驗證

- [ ] 7.1 新增單元測試：launcher 解析（未知值回退為 shell、缺漏欄位預設為 shell）
- [ ] 7.2 新增單元測試：herdr `tab create` 回應 JSON 解析（成功取得 pane_id、缺欄位時回錯誤）
- [ ] 7.3 執行 `cargo test` 確認後端測試全數通過
- [ ] 7.4 執行前端品質檢查（lint / type-check / build）
- [ ] 7.5 手動驗證 shell 模式：開終端、啟動 AI CLI、resume session、聚焦終端行為皆與變更前一致
- [ ] 7.6 手動驗證 herdr 模式：開終端建立 tab、啟動 AI CLI 於 pane 執行、tab 標籤可辨識、聚焦既有 tab 成功切換、tab 已關閉時錯誤訊息正確呈現
- [ ] 7.7 手動驗證 herdr 不可用情境（暫停 herdr server 或指定錯誤指令）錯誤訊息清楚
- [ ] 7.8 更新 `src-tauri/src/AGENTS.md` 的 command 清單（若有新增或調整 command 簽章）
