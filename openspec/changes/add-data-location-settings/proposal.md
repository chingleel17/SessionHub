## Why

各 AI CLI 工具（Claude Code、Codex、Copilot CLI、opencode）的 session 與設定資料預設全部存放在 C 槽使用者目錄下，使用者換電腦或想把資料放到雲端同步資料夾（OneDrive/Dropbox 等）時，難以掌握資料位置與搬遷方式。原本考慮用各工具的官方環境變數（`CLAUDE_CONFIG_DIR`、`CODEX_HOME`、`COPILOT_HOME`、`XDG_DATA_HOME`）改位置，但實測發現 Claude Code 的 `CLAUDE_CONFIG_DIR` 未寫入官方文件且 VS Code 擴充套件不遵守它、opencode 對 `XDG_DATA_HOME` 的遵循也不穩定，四個工具的支援程度不一致。改用目錄 symlink（將預設路徑本身變成指向新位置的連結）可以讓所有工具維持讀寫原本的預設路徑，不受各家環境變數支援度差異影響，也更貼近「跨電腦共用 session」的使用情境（新電腦上重建同一個 symlink 指到雲端同步資料夾即可接續使用）。已實測 Windows 目錄 symlink 對 `notify` crate 的檔案監聽完全透明（建立/修改/遞迴子目錄/副檔名過濾皆正常），技術上可行。

## What Changes

- 設定頁新增「資料位置」區塊：顯示各已啟用 provider 的資料目錄現況 — 目前實際路徑、是否為 symlink（若是，顯示連結目標）、目錄佔用大小。
- 同時顯示 SessionHub 自身資料目錄（`%APPDATA%\SessionHub`，可由 `COPILOT_SESSION_MANAGER_APPDATA_OVERRIDE` 覆寫）現況。
- 新增「搬遷引導」流程：使用者為某 provider 選擇新目錄後，SessionHub 依序執行 —
  1. 檢查是否具備建立目錄 symlink 的權限（見下方權限判斷）；不具備則中止並提示
  2. 複製既有資料至新目錄（複製成功並驗證後才進行下一步，來源目錄暫不刪除）
  3. 將原資料目錄改名為 `.bak` 備份，於原路徑建立指向新目錄的 symlink
  4. 驗證 symlink 可正常讀取後，提示使用者確認無誤可自行刪除 `.bak` 備份
- **權限判斷**：建立目錄 symlink 需要 Windows 開發人員模式或系統管理員權限。搬遷流程開始前先偵測目前程序是否具備建立 symlink 的能力（開發人員模式已開啟，或以系統管理員身分執行）：
  - 具備權限：直接執行搬遷
  - 不具備權限：中止搬遷，顯示「開啟開發人員模式」的設定路徑或指令（`start ms-settings:developers`），請使用者自行開啟後重試；不做提權引導（自用工具，不需要過度設計）
- 引導完成後提示使用者：已開啟的相關 CLI 工具/終端機建議重啟；換電腦時只要在新機器上對同一個雲端同步路徑重建 symlink 即可接續使用。
- 新增後端指令：查詢各 provider 實際資料路徑與是否為 symlink、計算目錄大小、偵測 symlink 建立權限、執行資料複製與 symlink 建立。

## Capabilities

### New Capabilities
- `data-location-settings`: 設定頁的資料位置檢視與搬遷引導 — 各 provider 與 SessionHub 自身資料目錄的現況顯示（路徑、是否為 symlink、大小）、搬遷流程（權限檢查、複製資料、建立 symlink）與完成後的提示。

### Modified Capabilities
- `app-settings`: 設定頁新增「資料位置」區塊入口。

## Impact

- **前端**：`src/App.tsx` 新增 IPC 呼叫；設定頁（Settings view）新增「資料位置」區塊與搬遷引導 UI；新增翻譯 key。
- **後端**：新增 commands（查詢資料位置現況、計算目錄大小、偵測 symlink 權限、執行搬遷）；`settings.rs` 的 `default_*_root` 維持現行 `USERPROFILE` 解析邏輯不變（symlink 對呼叫端透明，不需改動路徑解析）。
- **系統層**：透過 Windows 目錄 symlink（`std::os::windows::fs::symlink_dir` 或 `mklink /D`）建立，需要開發人員模式或系統管理員權限；不寫入任何環境變數，不影響其他程式或其他使用者。
- **風險**：搬移中若複製失敗需保持原狀（來源不動、不建 symlink）；大目錄複製需有進度回報與取消機制；無建立 symlink 權限時需明確中止並給出開啟方式。
