# Tasks

> 2026-11-04 結案：使用者回報 OpenCode 在手動與 SessionHub 終端均列出 38 個、Claude Code 結果相同且 Codex 未見異常。舊問題目前無法重現；以下勾選框保留未完成狀態，不代表已執行或已驗證。未來若有具體 issue，再另立變更。

## 1. 重新確認現況

- [ ] 1.1 在同一專案目錄與相近時間，於手動終端及 SessionHub shell 終端記錄 `opencode --version`、`opencode debug paths`、`opencode debug skill` 的 exit code、清單筆數與 `(name, location)` 集合；可用 `cmd.exe /d /c "opencode.exe debug skill | jq.exe length"` 快速計數，但不以此取代來源比對。
- [ ] 1.2 確認兩側 CLI 能成功載入設定；記錄有效的 OpenCode 停用旗標、工作目錄與程序來源。不要修改使用者全域設定或強制關閉既有 OpenCode 程序；若量測遭常駐服務污染，隔離工作階段後再測。
- [ ] 1.3 若設定使用 herdr，另從 SessionHub herdr pane 取得相同資料，標記 server／client 的啟動來源與冷／暖啟狀態；不要假定已存在的 pane 繼承這次 SessionHub 的環境。
- [ ] 1.4 記錄名稱與來源集合的比對結論：若一致且包含預期全域來源，標示問題目前無法重現，第 2 至 4 節標為不適用，不改 SessionHub 程式碼。

## 2. 差異可重現時才進行根因定位

- [ ] 2.1 比對 CLI 版本、工作目錄、有效設定與 SessionHub 母程序／shell 子程序環境；區分使用者自訂停用旗標與 SessionHub 注入的 `MSYS`。
- [ ] 2.2 針對缺漏來源檢查 skills 連結能否遍歷及 OpenCode 診斷日誌；若只在 herdr 發生，檢查現有 server／pane 繼承邊界，另行以不影響現有使用者程序的方式重測。
- [ ] 2.3 寫回 `design.md`：最小重現步驟、兩側 `(name, location)` 差集、證實的根因與受影響啟動路徑；若根因屬外部設定或 OpenCode CLI，記錄並停止本變更的程式實作。

## 3. 僅在證實 SessionHub 原因後修正

- [ ] 3.1 只修改 2.3 所定位的 shell、herdr 或資源探索路徑；保留使用者設定、MSYS 合併語意、既有命令／參數／cwd／console flags，不硬編 skills 位置或注入 home 相關變數。
- [ ] 3.2 針對已重現的差異增加最小必要回歸測試，涵蓋受影響入口與使用者已設定／未設定 OpenCode 停用旗標的行為。

## 4. 僅在修改程式碼後驗證

- [ ] 4.1 執行 `cargo test`（工作目錄 `src-tauri`）及 `bun run build`，並確認既有 MSYS 測試通過。
- [ ] 4.2 以與 1.1 相同條件重新比較來源集合；若使用 herdr，另核對已重現的 herdr 情境。確認未更動使用者既有未提交工作。
