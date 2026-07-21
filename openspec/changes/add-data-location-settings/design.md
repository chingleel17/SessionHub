## Context

各 AI CLI 工具的資料目錄預設都在 C 槽使用者目錄：

| 工具 | 預設位置 |
|---|---|
| Claude Code | `%USERPROFILE%\.claude` |
| Codex CLI | `%USERPROFILE%\.codex` |
| Copilot CLI | `%USERPROFILE%\.copilot` |
| opencode | `%USERPROFILE%\.local\share\opencode` |
| SessionHub | `%APPDATA%\SessionHub`（可由 `COPILOT_SESSION_MANAGER_APPDATA_OVERRIDE` 覆寫） |

原提案考慮用各工具的官方環境變數（`CLAUDE_CONFIG_DIR` / `CODEX_HOME` / `COPILOT_HOME` / `XDG_DATA_HOME`）改變資料位置，但查證後發現支援程度不一致：Codex 的 `CODEX_HOME`、Copilot 的 `COPILOT_HOME` 是官方文件記載的穩定變數；但 Claude Code 的 `CLAUDE_CONFIG_DIR` 未寫入官方文件，且已知 VS Code 擴充套件不遵守它（只有 CLI 本體遵守）；opencode 對 `XDG_DATA_HOME` 的遵循也不完全符合 XDG 規範（上游 issue #18633）。四個工具行為不一致，且環境變數方案只解決「搬到新位置」，不天然支援「跨電腦持續共用同一份資料」的情境。

改採目錄 symlink：把預設路徑本身變成指向新位置（例如雲端同步資料夾）的連結，讓所有工具維持讀寫原本熟悉的預設路徑，不受各家環境變數支援度差異影響。已用 `notify` 8.x（SessionHub 現行版本）實測驗證：

- watcher 監聽 symlink 路徑，對其連結目標寫入檔案，事件正常觸發（Create/Modify）
- 事件回報的路徑是 **symlink 側**的路徑，與 SessionHub 現有 `path.starts_with(&watch_root)` 判斷邏輯完全相容
- 遞迴監聽子目錄、`.jsonl` 副檔名過濾皆正常運作

## Goals / Non-Goals

**Goals:**

- 設定頁一眼看出每個工具的資料實際放在哪、佔多少空間、目前是否已是 symlink（及連結目標）
- 提供安全的引導式搬遷：複製資料 → 原路徑備份改名 → 建立 symlink 指向新位置
- 換電腦時，使用者只要在新機器對同一個雲端同步路徑重建 symlink，即可直接接續使用既有 session 資料

**Non-Goals:**

- 不做跨電腦的匯出/匯入打包功能（另案處理）
- 不自動刪除備份目錄（由使用者確認新位置正常後自行刪除）
- 不處理 macOS/Linux 平台（本產品目標平台為 Windows）
- 不做提權引導 UI（自用工具；無權限時直接中止並提示使用者自行開啟開發人員模式或以系統管理員身分重新啟動 SessionHub）
- 不搬遷各工具的登入憑證有效性問題（憑證檔案照常複製，若綁定機器需使用者重新登入，僅以提示告知）

## Decisions

### D1: 用目錄 symlink 而非官方環境變數

**選項 A**：寫入使用者層級環境變數（registry `HKCU\Environment`），各 CLI 工具以官方支援的方式讀取新位置。
**選項 B（採用）**：在原預設路徑建立目錄 symlink 指向新位置。

採用 B：四個工具中僅兩個（Codex、Copilot）的環境變數是官方穩定支援，Claude Code 的 `CLAUDE_CONFIG_DIR` 未文件化且 IDE 整合不吃這個變數，opencode 的 XDG 遵循亦不穩定 — 環境變數方案在四個工具上的可靠度不一致。Symlink 讓所有工具讀寫的路徑完全不變，行為對工具本身透明，且更符合「換電腦後直接接續使用」的目標（新機器上只要同一個同步路徑存在、重建 symlink 即可，不需要逐一設定四種不同環境變數）。已實測 `notify` crate 對 symlink 路徑監聽正常，排除了原先評估中「file watcher 相容性風險」的疑慮。

代價：建立目錄 symlink 需要 Windows 開發人員模式或系統管理員權限（junction 免權限但無法跨磁碟區，本情境常需跨磁碟/跨雲端同步資料夾，故不採用 junction）。見 D3。

### D2: 搬遷採「複製後改名備份」而非直接覆蓋或移動

流程：複製資料到新目的地並驗證（比對檔案數與總大小）→ 驗證通過後才將原目錄 rename 為 `<原名>.bak` → 在原路徑建立 symlink 指向新目的地。任一步驟失敗即中止並保持原狀（不 rename、不建 symlink），使用者資料零風險。`.bak` 備份預設保留，由使用者確認新位置運作正常後自行刪除。

### D3: symlink 建立權限的判斷與處理

建立前呼叫後端 command 偵測目前程序是否具備建立目錄 symlink 的能力：

- 檢查是否以系統管理員身分執行（`IsUserAnAdmin` 或等效判斷），或
- 檢查登錄機碼 `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\AppModelUnlock\AllowDevelopmentWithoutDevLicense` 是否為 1（開發人員模式已啟用）

兩者皆否，直接中止搬遷流程，UI 顯示提示文字與可複製的指令（`start ms-settings:developers` 開啟開發人員模式設定頁），請使用者自行處理後重試。不做 UAC 提權彈窗或其他自動化繞過（自用工具，不需要為單一使用情境過度設計）。

### D4: 目錄大小計算為非同步 command 且結果快取於前端

目錄大小以遞迴 `walkdir` 計算，`.claude`/`.codex` 可能達數 GB、數萬檔案，故獨立為一個 async Tauri command，由前端進入「資料位置」區塊時觸發、顯示 loading，結果留在前端 state 不落地。不做背景常駐計算（YAGNI）。

### D5: 搬遷進度以 Tauri event 回報

複製大量檔案期間，後端每複製固定批次即 emit `data-migration-progress` 事件（含已複製檔案數/總數、bytes），前端顯示進度條並提供取消。取消時停止複製、刪除已複製到目的地的部分內容、保持原狀（不 rename 原目錄、不建 symlink）。

### D6: 沿用集中式 IPC 架構

所有新 command 的 `invoke()` 集中在 `src/App.tsx`，設定頁子元件維持純顯示元件；新 Rust commands 依現有模組慣例放在 `src-tauri/src/commands/` 下新檔 `data_location.rs`。

## Risks / Trade-offs

- [使用者在其他 CLI 工具執行中搬遷，工具持續寫入舊目錄造成複製不完整] → 搬遷前 UI 提示關閉相關 CLI 工具；複製完成後做檔案數/大小驗證，不符則警告並允許重試。
- [無建立 symlink 權限] → 搬遷前主動偵測並中止，明確提示開啟開發人員模式的路徑；不嘗試自動提權。
- [rename 原目錄為 `.bak` 後，若建立 symlink 失敗] → 立即將 `.bak` 改回原名，回復原狀；此步驟需做例外處理與復原測試。
- [複製期間磁碟空間不足] → 搬遷前檢查目的地磁碟可用空間 ≥ 來源目錄大小，不足即拒絕開始。
- [SessionHub 自身資料目錄搬遷需重啟才生效] → `COPILOT_SESSION_MANAGER_APPDATA_OVERRIDE` 在程序啟動時讀取，完成畫面提示需重啟 SessionHub；SessionHub 自身資料目錄的搬遷同樣走 symlink 流程，不走環境變數覆寫。
- [雲端同步資料夾（OneDrive 等）對 symlink 內容的同步行為未知] → 屬於 Open Questions，實作前需以實機驗證常見雲端同步工具是否正確同步 symlink 指向的實際內容（而非把 symlink 本身當作特殊檔案同步）。

## Migration Plan

功能屬新增，無資料結構變更、無破壞性修改；未執行搬遷的既有使用者行為完全不變（路徑仍是預設 `USERPROFILE` 路徑，非 symlink）。回退即移除設定頁區塊與新 commands；已建立的 symlink 不受回退影響，使用者可自行用「還原」流程（刪除 symlink、將 `.bak` 改回原名）復原。

## Open Questions

- 雲端同步工具（OneDrive/Dropbox/Google Drive 等）對「同步資料夾內含 symlink，且 symlink 指向同步資料夾外部路徑」或「symlink 本身在同步資料夾內」這兩種佈局的實際行為需要實機驗證，決定搬遷精靈的建議佈局（引導使用者把新目的地選在同步資料夾內，還是把 symlink 建在同步資料夾內指回本機）。
- 開發人員模式的偵測登錄機碼路徑已於本機驗證存在且可讀（`AllowDevelopmentWithoutDevLicense`），但未驗證其在所有 Windows 10/11 版本上的一致性，實作時建議兩種判斷方式（管理員身分 / 登錄機碼）並用，任一為真即視為具備權限。
