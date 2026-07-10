## Context

各 AI CLI 工具的資料目錄預設都在 C 槽使用者目錄：

| 工具 | 預設位置 | 官方環境變數 |
|---|---|---|
| Claude Code | `%USERPROFILE%\.claude` | `CLAUDE_CONFIG_DIR`（整個 config 目錄） |
| Codex CLI | `%USERPROFILE%\.codex` | `CODEX_HOME`（整個目錄） |
| Copilot CLI | `%USERPROFILE%\.copilot` | `COPILOT_HOME`（整個目錄，官方建議） |
| opencode | `%USERPROFILE%\.local\share\opencode` | `XDG_DATA_HOME`（資料改存 `$XDG_DATA_HOME\opencode`） |
| SessionHub | `%APPDATA%\SessionHub` | `COPILOT_SESSION_MANAGER_APPDATA_OVERRIDE`（既有機制） |

SessionHub 目前的 `settings.rs` 中 `default_*_root()` 一律以 `USERPROFILE` 組路徑，未考慮上述環境變數；使用者若已透過環境變數改位置，SessionHub 會讀錯目錄。設定頁已有各 provider root 的手動輸入欄位（`AppSettings.claude_root` 等），但沒有任何資料位置現況檢視或搬遷輔助。

## Goals / Non-Goals

**Goals:**

- 設定頁一眼看出每個工具的資料實際放在哪、佔多少空間、是否已改到非預設位置
- 提供安全的引導式搬遷：複製資料 → 設定使用者層級環境變數 → 同步 SessionHub root 設定
- `default_*_root()` 解析納入官方環境變數，與 CLI 工具實際行為一致

**Non-Goals:**

- 不做跨電腦的匯出/匯入打包功能（另案處理）
- 不自動刪除舊目錄（由使用者確認新位置正常後自行刪除）
- 不處理 macOS/Linux 的環境變數持久化（本產品目標平台為 Windows）
- 不搬遷各工具的登入憑證有效性問題（憑證檔案照常複製，若綁定機器需使用者重新登入，僅以提示告知）

## Decisions

### D1: 用官方環境變數而非 NTFS junction

**選項 A（採用）**：寫入使用者層級環境變數（registry `HKCU\Environment`），各 CLI 工具以官方支援的方式讀取新位置。
**選項 B**：建立 NTFS junction 把 `~/.claude` 等指向新位置。

採用 A：官方支援、行為可預期；junction 對 file watcher（SessionHub 自身的 notify watcher）與部分工具的路徑正規化有相容性風險，且使用者不易察覺目錄其實被重導。寫入 `HKCU\Environment` 不需管理員權限。寫入後需廣播 `WM_SETTINGCHANGE` 讓 Explorer 及後續新啟動的程序讀到新值；既有終端機需重開，以完成提示告知。

### D2: 搬遷採「複製後保留來源」而非「移動」

複製完成並驗證（比對檔案數與總大小）後才寫環境變數與更新設定；來源目錄保留不動。任一步驟失敗即中止並保持原狀（不寫環境變數、不改設定），使用者資料零風險。代價是搬遷期間磁碟需容納兩份資料，屬可接受。

### D3: 資料位置解析的優先序

各 provider 的實際資料根目錄解析順序：

1. `AppSettings` 中使用者手動填寫的 root（既有行為，優先權最高）
2. 對應的官方環境變數（`CLAUDE_CONFIG_DIR` / `CODEX_HOME` / `COPILOT_HOME` / `XDG_DATA_HOME`）
3. `USERPROFILE` 預設路徑

現況檢視同時回報「值來自哪一層」，UI 據此標示「預設 / 環境變數 / 手動設定」。此修改落在 `settings.rs` 的 `default_*_root()`，全域生效（session 掃描、quota、hook 安裝共用同一解析）。

### D4: 目錄大小計算為非同步 command 且結果快取於前端

目錄大小以遞迴 `walkdir` 計算，`.claude`/`.codex` 可能達數 GB、數萬檔案，故獨立為一個 async Tauri command，由前端進入「資料位置」區塊時觸發、顯示 loading，結果留在前端 state 不落地。不做背景常駐計算（YAGNI）。

### D5: 搬遷進度以 Tauri event 回報

複製大量檔案期間，後端每複製固定批次即 emit `data-migration-progress` 事件（含已複製檔案數/總數、bytes），前端顯示進度條並提供取消。取消時停止複製、刪除已複製到目的地的部分內容、保持原狀。

### D6: 沿用集中式 IPC 架構

所有新 command 的 `invoke()` 集中在 `src/App.tsx`，設定頁子元件維持純顯示元件；新 Rust commands 依現有模組慣例放在 `src-tauri/src/commands/` 下新檔 `data_location.rs`。

## Risks / Trade-offs

- [使用者在其他 CLI 工具執行中搬遷，工具持續寫入舊目錄造成複製不完整] → 搬遷前 UI 提示關閉相關 CLI 工具；複製完成後做檔案數/大小驗證，不符則警告並允許重試。
- [環境變數寫入 registry 後，已開啟的終端機與常駐程序仍讀舊值] → 廣播 `WM_SETTINGCHANGE` 並在完成畫面明確提示需重開終端機；SessionHub 自身立即改用新路徑（透過 AppSettings root 同步更新，不依賴自身程序環境變數）。
- [`XDG_DATA_HOME` 是共用變數，影響 opencode 以外遵循 XDG 的程式] → UI 對 opencode 搬遷特別註明此副作用，並顯示目前 `XDG_DATA_HOME` 是否已被設定；若已設定為其他值則不允許覆寫，改為指示使用者手動處理。
- [Claude Code 的 `CLAUDE_CONFIG_DIR` 在部分場景（IDE 整合）有已知相容性問題] → 引導文案標註此限制與上游 issue，讓使用者知情後再執行。
- [複製期間磁碟空間不足] → 搬遷前檢查目的地磁碟可用空間 ≥ 來源目錄大小，不足即拒絕開始。
- [SessionHub 自身資料目錄搬遷需重啟才生效] → `COPILOT_SESSION_MANAGER_APPDATA_OVERRIDE` 在程序啟動時讀取，完成畫面提示需重啟 SessionHub。

## Migration Plan

功能屬新增，無資料結構變更、無破壞性修改；`default_*_root()` 加入環境變數判斷後，對未設定這些環境變數的既有使用者行為完全不變。回退即移除設定頁區塊與新 commands。

## Open Questions

- Copilot CLI 的 `COPILOT_HOME` 於較舊版本可能不支援（早期僅 `XDG_CONFIG_HOME`）：實作時以目前安裝版本驗證，必要時在 UI 顯示最低版本需求。
- opencode 在 Windows 上的 XDG 路徑解析有已知不一致（上游 issue #8235）：實作前先以實機驗證 `XDG_DATA_HOME` 生效行為。
