## Why

目前所有終端啟動入口都假設 `terminal_path` 是「一個 shell 執行檔」，以 `CREATE_NEW_CONSOLE` 另開一個作業系統視窗，因此每開一個 session 就多一個 pwsh 視窗，難以集中管理。使用者已改用 herdr（終端多工器，本機已安裝 0.8.0-preview 且 server 執行中）作為日常終端環境，希望 SessionHub 能直接把 session 開進 herdr 的 tab／pane，而非持續產生零散的獨立視窗。

herdr 無法透過現有機制支援：它不是「shell + inline command」形式，而是兩段式流程（先建立 tab 取得 pane id，再對該 pane 送出指令），且需要讀取並解析 stdout 的 JSON，與現行 fire-and-forget 的 `spawn()` 模式根本不同。

## What Changes

- 新增「終端啟動器種類」概念：設定新增 `terminal_launcher` 欄位（`"shell"` | `"herdr"`），與既有 `terminal_path` 並存。預設為 `"shell"`，維持現有行為不變。
- 新增 herdr 啟動路徑：以 `herdr tab create --cwd <PATH> --label <TEXT> --focus` 建立 tab，解析回傳 JSON 取得 `result.root_pane.pane_id`，再以 `herdr pane run <PANE_ID> <COMMAND>` 送出該工具的啟動指令。純開終端（無 initial command）時只需建立 tab。
- 抽出共用啟動指令組裝函式，讓 `open_in_tool_internal`、`open_terminal_internal`、`resume_session_in_terminal_internal` 共用同一套 launcher 分派邏輯，取代目前約 8 處重複的 `file_stem` switch 區塊。
- 終端聚焦行為依 launcher 分流：herdr 模式下改以 `tab create --focus` 達成聚焦，跳過依視窗標題比對的 Win32 聚焦流程（herdr 只有單一視窗、多個 pane，標題比對必然找錯目標）。
- 終端路徑驗證依 launcher 分流：`terminal_launcher` 為 `"herdr"` 時允許 PATH 解析的裸指令名，不強制要求可瀏覽的檔案路徑。
- herdr 不可用時（未安裝、server 未執行、tab create 失敗）回傳明確錯誤訊息，由前端 toast 呈現，不靜默失敗。
- 設定頁新增 launcher 選擇控制項，並補上 zh-TW / en-US 兩份 locale 字串。

不在此變更範圍：tmux 或其他多工器支援（本機 WSL 未安裝 tmux，且需另做 Windows 路徑轉換，屬無法驗證的預測性開發）。

## Capabilities

### New Capabilities

無。本變更修改既有終端啟動與聚焦行為，不引入新的能力領域。

### Modified Capabilities

- `terminal-launcher`: 啟動邏輯從「單一 shell + file_stem 白名單」擴充為「先依 launcher 種類分派，再於 shell 模式沿用 file_stem 白名單」；新增 herdr 兩段式啟動流程與其失敗處理要求。
- `terminal-focus`: 新增依 launcher 分流的聚焦要求 — herdr 模式改由建立 tab 時聚焦，不套用既有的視窗標題比對邏輯。
- `app-settings`: 設定欄位定義新增 `terminal_launcher: Option<String>`，並調整終端路徑驗證要求使其依 launcher 分流。

## Impact

**後端（Rust）**
- `src-tauri/src/commands/tools.rs` — `open_in_tool_internal`（terminal/opencode/claude/codex/copilot/gemini/vscode shim 共 7 個分支）、`resume_session_in_terminal_internal`、`focus_terminal_window_internal`
- `src-tauri/src/sessions/copilot.rs` — `open_terminal_internal`
- `src-tauri/src/types/settings.rs` — `AppSettings` 新增 `terminal_launcher` 欄位
- `src-tauri/src/settings.rs` — 設定解析與預設值
- 新增 herdr 啟動模組（tab create／pane run 呼叫與 JSON 解析）

**前端（TypeScript）**
- `src/App.tsx` — `validate_terminal_path` 呼叫時機、傳遞 launcher 至 invoke 參數
- `src/components/SettingsView.tsx` — launcher 選擇控制項
- `src/types/index.ts`、`src/utils/appSettingsDefaults.ts`、`src/hooks/useAppSettingsForm.ts` — 型別與表單預設值
- `src/locales/zh-TW.ts`、`src/locales/en-US.ts` — 新增文案

**相依性**
- 外部 CLI：herdr（僅在使用者選擇 herdr launcher 時需要；未安裝時降級為錯誤提示）
- 既有設定檔向後相容：`terminal_launcher` 缺漏時視為 `"shell"`
