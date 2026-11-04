# Design: 核對 SessionHub 終端中的 OpenCode 全域 skills

## 結案紀錄（2026-11-04）

使用者回報手動終端與 SessionHub 終端測得的 OpenCode skills 均為 38 個，Claude Code 的檢查結果亦相同，Codex 目前未觀察到問題。這表示舊版記載的 7／21 個落差目前**無法重現**；沒有證據可確認當年的根因或是哪次修改使現象消失。尚未完成兩側 `(name, location)` 集合及 herdr 情境的完整比對，不能宣稱所有技能來源均經驗證一致。

依使用者決定，停止此變更並保留未完成任務作為歷史紀錄，不修改 SessionHub 程式碼，也不將尚未實作的 delta spec 同步至正式規格。日後如有具體 issue 與重現步驟，再另立變更調查。

## Context

### 歷史觀察與目前證據

先前在 OpenCode 1.18.4 曾觀察到 SessionHub 終端 7 個、手動終端 21 個 skills；舊版反組譯與 log 分析指出專案、全域來源可能分別失效，但沒有定位到 SessionHub 的實際根因。當時跨磁碟的 skill symlink／junction 是假說，不能當成既定缺陷。過去的 log run ID、固定數量與路徑只作歷史線索，不作現在的驗收基準。

2026-11-04，在本專案目錄直接執行 `opencode.exe --version` 得到 1.19.5；`opencode.exe debug skill` 成功輸出 JSON 陣列，以 `cmd.exe /d /c "opencode.exe debug skill | jq.exe length"` 取得 **38**。這次測試並非從 SessionHub 開出的終端，不能推論兩者已一致，也不能推論 38 個全來自全域目錄。原文件記載的 `planbyproject.md` 設定錯誤目前沒有阻擋這次執行；不應逕行修改使用者的全域設定。

現有 `launch_terminal` 分派 shell 或 herdr：shell 在 `sessions/terminal.rs` 透過 `Command::new` 啟動，未清除環境，只合併注入 `MSYS`；一般終端、工具啟動、session resume 皆使用此分派。herdr 先確保 client，再透過 herdr API 建立 tab 並送入命令；pane 可能由已存在的 herdr server 建立，不能假設繼承本次 SessionHub 程序的環境。SessionHub 的資源列示另有獨立的 `resource_discovery.rs` CLI 探索流程，不能拿其結果代替終端內的測試。

## Goals / Non-Goals

### Goals

- 先以同版 CLI、同工作目錄與相同有效設定，比對兩種啟動來源的 skill 名稱與來源路徑；必要時另測 herdr。
- 若差異可重現，隔離 SessionHub shell 繼承、herdr server／pane、OpenCode 設定或連結遍歷的責任邊界，僅針對已證實原因修正並驗證。
- 若結果一致，記錄版本、模式及比對結論，停止未必要的程式碼變更。

### Non-Goals

- 不修改 OpenCode CLI 或使用者全域設定作為本變更的修正手段。
- 不以固定數字、手動建立的候選 `SKILL.md` 檔案數，或 SessionHub 資源管理頁面的列示，取代 OpenCode 終端實測。
- 不預先更換 skills 連結型別、重建使用者目錄或重寫全部子程序環境。

## Decisions

### D1：以實際載入清單建立可重現基準

在同一個專案目錄、相近時間，於手動 shell 與 SessionHub shell 執行相同的 `opencode debug skill`，分別記錄 CLI 版本、工作目錄、exit code、JSON 筆數及由 `(name, location)` 組成的集合；比較來源路徑時對同一台機器的路徑作等價正規化，不用固定 7／21／38 作通過條件。CLI 目前輸出可由 `cmd.exe /d /c "opencode.exe debug skill | jq.exe length"` 計數；其 JSON 格式非穩定的自動化契約，若 CLI 升級改變格式，先調整診斷方法。來源集合比對仍必須獨立於計數。

### D2：先排除量測污染

比較前先確認兩側 CLI 可成功載入設定、`opencode debug paths` 及有效設定／停用旗標是否相符；以唯讀方式確認程序來源與是否連到既有 OpenCode server。不得逕自終止使用者執行中的 OpenCode 程序、修改或移開全域設定。若存在常駐服務，改以可識別的新工作階段重測，或先取得使用者同意再清除污染。測試結果若僅有筆數而無來源資訊，保留為初步線索，不宣稱問題已修復。

### D3：區分 shell 與 herdr 的環境邊界

shell 子程序預設繼承 SessionHub 當下的環境，且 `MSYS` 合併不得取代使用者原值。手動終端與 SessionHub 母程序原本就可能有不同環境，不能要求二者所有環境變數逐一相同。herdr 已有 client／server 時，pane 環境以該服務的生命週期為準；驗證時另記錄 server 啟動來源及冷／暖啟狀態。只有在證實某一路徑造成來源落差時，才實作該路徑必要的修正。

### D4：有證據才進入實作

若兩側 `(name, location)` 集合一致且預期全域來源存在，就以「目前無法重現」結案，所有程式碼任務不適用。若不一致，先重現並定位：版本／cwd／設定 → SessionHub 母程序與 shell 子程序環境 → herdr server／pane → symlink／junction 可視性及 CLI 日誌。只有當根因指向 SessionHub 啟動路徑才修改程式碼；保留原有 MSYS 緩解、啟動命令、cwd 與 console flags。若根因是外部設定或 CLI 行為，記錄證據並另案處理，不偽裝成本變更的修正。

## Risks / Trade-offs

- OpenCode 版本與 JSON 格式可能變動：診斷時以當機 `--version` 與 `debug skill` 實測為準，不依賴舊版反組譯內容。
- 不同 shell 啟動檔、使用者設定或既有 herdr server 可造成合法差異：比較時標記這些條件，避免以環境變數不同直接推斷故障。
- 相同數量可能掩蓋不同來源：必須同時比較名稱與 `location`；若命令失敗或缺少來源資料，驗收不得通過。

## 驗證與回復

先執行唯讀對照，必要時記錄差異與最小重現；若須修正 SessionHub，先加對應回歸測試，再執行 `cargo test` 與 `bun run build`，最後在相同條件下實測 shell 與使用中的 herdr 路徑。無可重現差異時不改程式碼，無須回復；若有實作，僅回復該次受影響的啟動路徑變更。

## 待實測事項

1. 使用者剛才的「都有列出來」是在哪個 SessionHub 啟動模式、哪個專案目錄與哪個 OpenCode 版本觀察？與手動 shell 的 `(name, location)` 集合是否相同？
2. 若 shell 與手動 shell 不一致，SessionHub 母程序的啟動來源及有效環境是否不同？
3. 若僅 herdr 不一致，既有 server／client 是由誰啟動、是否需要冷啟重測？

上述均是診斷步驟，不是預先要求使用者決定的產品政策。
