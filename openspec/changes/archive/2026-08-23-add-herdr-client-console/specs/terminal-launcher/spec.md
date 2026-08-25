## ADDED Requirements

### Requirement: herdr client console 生命週期

在 herdr 啟動器模式下，系統 SHALL 確保每次啟動時存在一個由本應用程式建立的 herdr TUI client console 程序，使建立的 tab 對使用者可見。

herdr 為 server（headless）+ client（TUI）雙程序架構，socket API 呼叫只影響 server 狀態，若無 client 程序則使用者看不到任何畫面。

#### Scenario: 以 socket API 判定是否已有 client 附著

- **WHEN** 啟動器為 herdr 且系統準備啟動
- **THEN** 系統以 socket API 詢問 server 是否已有 client 附著
- **AND** 此判定涵蓋使用者手動開啟的 TUI 及本應用程式重啟前建立的 client
- **AND** 該判定所用的方法 SHALL 為冪等且不留下殘留副作用

#### Scenario: 已有 client 附著時不建立

- **WHEN** socket API 回報已有 client 附著
- **THEN** 系統不建立新的 client console
- **AND** 直接進入 tab 建立流程

#### Scenario: 尚無 client 附著時建立

- **WHEN** socket API 回報無 client 附著
- **THEN** 系統以獨立 console 視窗 spawn herdr 裸指令，使其啟動或附著至持久 session
- **AND** 保留該子程序控制代碼供後續存活判定

#### Scenario: socket 判定無法取得時的退路

- **WHEN** 系統無法自 socket API 取得 client 附著狀態
- **THEN** 系統改以本程序自建 client 的子程序存活狀態決定是否建立
- **AND** 不因判定失敗而中斷啟動流程

#### Scenario: 清除巢狀啟動環境變數

- **WHEN** 系統 spawn herdr client console
- **THEN** 系統先清除繼承自 herdr session 的巢狀偵測環境變數
- **AND** 此為必要步驟，否則 herdr 會以「nested herdr is disabled by default」拒絕啟動並立即結束

#### Scenario: client console 啟動後隨即結束

- **WHEN** client console 程序 spawn 成功但在啟動確認時間內即結束
- **THEN** 系統回傳指出該視窗啟動後隨即結束的錯誤訊息，並附上結束代碼
- **AND** 不記錄該子程序，亦不繼續建立 tab

#### Scenario: 已有 client console 時重用

- **WHEN** 啟動器為 herdr 且系統持有的 client console 程序仍存活
- **THEN** 系統不再建立新的 console 視窗
- **AND** 沿用該既有 console 承載本次啟動的 tab

#### Scenario: client console 已被關閉

- **WHEN** 系統持有的 client console 程序已結束（例如使用者關閉該視窗）
- **THEN** 系統視為不存在並重新建立一個 client console

#### Scenario: 辨識使用者手動開啟的 client

- **WHEN** 使用者自行以終端執行 herdr 開啟 TUI，而系統未持有自建的 client console 程序
- **THEN** 系統經 socket API 判定已有 client 附著，不再另外建立

#### Scenario: 應用程式重啟後仍能辨識既有 client

- **WHEN** 本應用程式重新啟動後首次以 herdr 啟動器觸發啟動，且既有 client 仍附著
- **THEN** 系統經 socket API 判定已有 client 附著，不因自建追蹤狀態遺失而重複建立

#### Scenario: 退路判定造成的重複建立

- **WHEN** socket 判定無法取得，且本應用程式重啟後未持有自建 client 的追蹤狀態
- **THEN** 系統另外建立一個 client console
- **AND** 多個 client 同時附著於同一 session 不影響運作，系統 SHALL NOT 因此中斷啟動流程

### Requirement: herdr server 就緒等待

系統 SHALL 在建立 client console 後、送出 tab 建立請求前，等待 herdr server 進入可用狀態，避免對尚未建立的 socket 發出請求。

#### Scenario: 冷啟動等待就緒

- **WHEN** 系統建立 client console 且該次啟動前 herdr server 未執行
- **THEN** 系統以固定間隔輪詢 server 狀態直到其回報執行中
- **AND** 就緒後才建立 tab

#### Scenario: 就緒等待逾時

- **WHEN** 輪詢達到上限時間後 herdr server 仍未回報執行中
- **THEN** 系統回傳指出 server 未能啟動的錯誤訊息
- **AND** 不再嘗試建立 tab

#### Scenario: server 已執行時不等待

- **WHEN** 啟動前 herdr server 已在執行中
- **THEN** 系統不進行就緒輪詢，直接進入 tab 建立流程

## MODIFIED Requirements

### Requirement: 終端啟動器種類分派

系統 SHALL 依設定的終端啟動器種類（`terminal_launcher`）決定啟動方式，所有終端啟動入口（一般終端、AI coding CLI 啟動、session resume）SHALL 共用同一套分派規則。

#### Scenario: 預設為 shell 啟動器

- **WHEN** 設定中 `terminal_launcher` 為 `"shell"` 或未設定
- **THEN** 系統沿用既有 shell 啟動邏輯（`terminal_path` 搭配 file_stem 白名單，並以獨立 console 視窗開啟）

#### Scenario: 選用 herdr 啟動器

- **WHEN** 設定中 `terminal_launcher` 為 `"herdr"`
- **THEN** 系統以 herdr 建立 tab／pane 承載該次啟動
- **AND** 系統至多維持一個由本應用程式建立的 herdr client console 視窗承載所有 tab，不因每次啟動而累積視窗

#### Scenario: 未知啟動器種類

- **WHEN** `terminal_launcher` 的值不在支援清單內
- **THEN** 系統以 `"shell"` 行為啟動，不中斷使用者操作

### Requirement: herdr 不可用時的錯誤處理

系統 SHALL 在 herdr 無法完成啟動時回傳明確錯誤訊息，不得靜默失敗或產生無回應的操作。

#### Scenario: herdr 未安裝

- **WHEN** 使用者選用 herdr 啟動器但系統找不到 herdr 可執行檔
- **THEN** 系統回傳指出 herdr 未偵測到的錯誤訊息，並指引安裝
- **AND** 前端以 toast 呈現該錯誤

#### Scenario: herdr 已安裝但服務未執行

- **WHEN** herdr 可執行檔存在但其服務未執行
- **THEN** 系統建立 client console 以啟動服務，不將此情形視為錯誤
- **AND** 僅在服務於等待上限內仍未就緒時才回報錯誤

#### Scenario: client console 無法建立

- **WHEN** 系統無法 spawn herdr client console 程序
- **THEN** 系統回傳包含失敗原因的錯誤訊息，且不再嘗試建立 tab

#### Scenario: 建立 tab 失敗或回應無法解析

- **WHEN** herdr 回報非成功狀態，或其輸出無法解析出 pane 識別碼
- **THEN** 系統回傳包含失敗原因的錯誤訊息，且不再嘗試送出後續指令

#### Scenario: 不自動回退至 shell

- **WHEN** herdr 啟動流程因任一原因失敗
- **THEN** 系統 SHALL NOT 自動改以 shell 啟動器啟動
- **AND** 由使用者依錯誤訊息決定後續處置

### Requirement: Windows 終端與 CLI 啟動環境一致性

系統 SHALL 讓一般終端啟動、多工具啟動及 session resume 共用相同的 Windows MSYS stackdump 緩解環境組態，避免任一啟動入口遺漏。此要求適用於所有由系統建立的 console 程序，包含 shell 啟動器所建立的終端與 herdr client console。

#### Scenario: 開啟一般終端
- **WHEN** `open_terminal` 在 Windows 啟動使用者設定的終端
- **THEN** 新程序套用 MSYS stackdump 緩解環境

#### Scenario: 開啟或恢復 AI coding CLI
- **WHEN** `open_in_tool` 或 `resume_session_in_terminal` 在 Windows 啟動受支援的 AI coding CLI
- **THEN** 新程序套用與一般終端相同的 MSYS stackdump 緩解環境

#### Scenario: 啟動參數維持不變
- **WHEN** 系統套用 MSYS stackdump 緩解環境
- **THEN** 各 terminal 與 provider 原有的命令、參數、工作目錄及 Windows console creation flags 維持不變

#### Scenario: herdr 模式的環境套用範圍

- **WHEN** 啟動器為 herdr 且系統建立 client console 程序
- **THEN** 該程序以 console creation flags 開啟獨立視窗
- **AND** 套用與 shell 啟動器相同的 MSYS stackdump 緩解環境

#### Scenario: herdr tab 建立不另開 console

- **WHEN** 啟動器為 herdr 且系統對既有 client console 建立新的 tab
- **THEN** 該次 tab 建立不產生新的 Windows console 程序，因此不套用 console creation flags

#### Scenario: herdr socket API 呼叫不顯示視窗

- **WHEN** 系統執行 herdr socket API 類指令以查詢狀態或操作 tab／pane
- **THEN** 該程序以隱藏視窗方式執行，不閃現 console 視窗
