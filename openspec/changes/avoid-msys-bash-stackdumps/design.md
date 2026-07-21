## Context

使用者提供的 stack trace 全部位於 `msys-2.0.dll`，且與 Claude Code 已知議題 `anthropics/claude-code#37920` 的位址序列一致。該類檔案是 Git for Windows / MSYS2 Bash 在異常終止時寫出的 crash dump；檔案位置取決於 Bash 當時的工作目錄，因此會散落在各專案。這不代表 Claude、Codex、OpenCode 的 session 資料損毀，也不是 Rust 或 Node 的 stack trace。

SessionHub 目前在 `open_terminal_internal`、`open_in_tool_internal` 與 `resume_session_in_terminal_internal` 建立程序，並將專案 `cwd` 設為工作目錄。Windows 子程序會繼承 SessionHub 提供的環境，因此可在不更動第三方 CLI 的前提下，對從 SessionHub 啟動的整個程序樹套用 MSYS runtime 選項。

## Goals / Non-Goals

**Goals:**
- 避免從 SessionHub 啟動的 Windows CLI 工作流程持續在專案中產生 `bash.exe.stackdump`。
- 保留使用者既有的 `MSYS` 設定與自訂 `error_start` 除錯器。
- 讓所有相關啟動入口使用同一套可測試的環境組態。
- 維持現有命令、參數、工作目錄與 console 行為。

**Non-Goals:**
- 修復 Git for Windows、MSYS2 或第三方 CLI 的原生崩潰。
- 影響不是從 SessionHub 啟動的 Claude Code、Codex、OpenCode 或其他 Bash 程序。
- 掃描、忽略、搬移或刪除既有 stackdump 檔案。
- 變更 provider hooks、integration 格式或前端設定。

## Decisions

### 1. 在程序環境加入空 `error_start:` token

Windows 專用 helper 讀取目前的 `MSYS`，若不存在 `error_start` token則追加 `error_start:`，再透過 `std::process::Command::env` 設定於即將啟動的程序。MSYS2 runtime 官方原始碼將 `MSYS` 的 `error_start` 視為 fatal error handler 選項；已知 Claude Code 問題回報驗證空值可避免 stackdump 寫檔。

替代方案是事後刪除 `bash.exe.stackdump`，但那需要掃描任意專案、可能誤刪使用者保留的診斷資料，且會掩蓋持續產檔，因此不採用。另一替代方案是設定全域使用者環境變數，但 SessionHub 不應永久修改系統設定。

### 2. 不覆寫既有 `error_start`

以 ASCII 空白分隔 `MSYS` token，大小寫不敏感地檢查 token 名稱是否為 `error_start`（後接 `:` 或 `=`）。若使用者已指定除錯器，保留整個原值；若只有其他選項，於尾端追加空 token。這可保持冪等，並避免破壞 `winsymlinks` 等設定。

替代方案是無條件將 `MSYS` 設為 `error_start:`，但會覆蓋使用者的 symlink、globbing 或程序重試設定，因此不採用。

### 3. 在共用程序組態 helper 套用，而非各分支自行拼接

新增最接近工具啟動職責的 Rust helper，接受 `&mut Command` 並在 Windows 套用環境；非 Windows 實作為 no-op。`open_terminal_internal`、`open_in_tool_internal` 的 terminal/AI CLI 分支，以及 `resume_session_in_terminal_internal` 在 `spawn` 前呼叫。Explorer 與 VS Code 不套用，避免讓與問題無關的長生命週期 GUI 程序繼承特殊環境。

替代方案是在每個 match 分支直接呼叫 `.env()`，但容易在新增 provider 或 resume 路徑時遺漏，也增加測試重複。

## Risks / Trade-offs

- [空 `error_start:` 的行為屬 MSYS runtime 緩解措施，並未修復崩潰] → 文件與驗收只保證環境正確注入，不宣稱 Bash 不再崩潰。
- [第三方 CLI 可能清除或覆寫 `MSYS`] → 單元測試涵蓋 SessionHub 啟動邊界；超出該程序邊界的行為列為非目標。
- [使用者已有自訂 fatal-error debugger] → 偵測任何既有 `error_start` token後完全保留原值。
- [環境 token 解析不支援含空白的罕見 debugger 路徑] → 不重組已有 `error_start` 的值；只在不存在時追加固定 token。

## Migration Plan

1. 加入環境值合併與 `Command` 組態 helper及單元測試。
2. 將 helper 接到一般終端、AI CLI 與 session resume 啟動路徑。
3. 執行 Rust 單元測試，並在 Windows 以測試子程序確認環境繼承。
4. 此變更不修改持久化資料；回滾只需移除 helper 呼叫，不需資料遷移。

## Open Questions

無。若第三方工具未來修正子程序終止流程，可另行評估是否移除此緩解措施。
