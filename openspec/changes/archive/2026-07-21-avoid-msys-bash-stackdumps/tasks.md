## 1. MSYS 環境組態

- [x] 1.1 在 Rust 工具啟動領域新增合併 `MSYS` 選項的純函式，缺少 `error_start` 時追加 `error_start:`，並保留既有值
- [x] 1.2 新增大小寫不敏感、既有除錯器、其他 MSYS 選項、空值及重複套用的單元測試
- [x] 1.3 新增 Windows `Command` 環境組態 helper，並讓非 Windows 建置保持 no-op

## 2. 啟動路徑整合

- [x] 2.1 在 `open_terminal_internal` 啟動終端前套用共用 MSYS stackdump 緩解環境
- [x] 2.2 在 `open_in_tool_internal` 的 terminal 與 AI coding CLI 分支套用緩解環境，維持 VS Code 與 Explorer 分支不變
- [x] 2.3 在 `resume_session_in_terminal_internal` 套用相同環境，確認 provider 命令、cwd 與 console flags 不變

## 3. 驗證

- [x] 3.1 新增 Windows 測試子程序，驗證 SessionHub 設定的 `MSYS` 可由後代程序繼承
- [x] 3.2 執行 `cargo test`，確認既有終端、工具啟動與 provider 測試無回歸
- [x] 3.3 在 Windows 從 SessionHub 啟動至少一個使用 Git Bash 的 CLI 工作流程，確認新產生的程序環境包含緩解選項且不新增 `bash.exe.stackdump`
