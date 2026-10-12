# Tasks

## 1. 子專案技能探索與資料模型

- [x] 1.1 在正式 provider-aware Skills 掃描加入實體專案上下文探索並套用既有 provider 入口規則；以根目錄、兩個 sibling 子專案、超過八層的子專案及入口內 `team/demo` 測試驗證皆可找到。
- [x] 1.2 加入固定忽略目錄、專案 ignored paths、provider 設定子樹剪枝，以及 20,000 個有效目錄預算；沿用 SkillsScanResult.diagnostics 回傳部分成功診斷，以排除子樹不被進入、到達預算回傳成功與診斷、可讀入口仍有結果的測試驗證。
- [x] 1.3 保持技能入口／技能目錄連結探索，避免沿一般子目錄 symlink／junction 擴大搜尋，並回報損壞或迴圈；以 Windows 連結與迴圈 fixture 測試驗證掃描可終止且不混入一般外部專案。
- [x] 1.4 單次掃描共用相同上下文入口的遍歷結果，保留 provider 歸屬與原 CLI discovery 呼叫範圍；以計數測試驗證共用 `.agents/skills` 遍歷一次且結果仍含各啟用 provider。
- [x] 1.5 同步擴充 Rust／TypeScript 的 `projectRelativeDir` 與技能目標 `targetRootExists`，依上下文計算 source／target；以正式掃描測試驗證 sibling 同名技能、子專案 Claude 目標與根目錄／全域相容性，前端型別檢查通過。

## 2. 前端資源快取與查詢生命週期

- [x] 2.1 依 design D2 的 per-kind sourcePaths 表建立穩定資源 query key，包含 scope、正規化路徑、provider 集合及 Skills ignored paths，等設定／prefs 確定後首次載入；以來源隔離、provider 排序等價、無關主題／Session root 設定不重掃及初始載入一次的測試驗證。
- [x] 2.2 移除 Skills／Commands signature polling 及 signature key 成分，為 project／global Skills、Commands、MCP 與相關 prefs、連結、Codex trust 設定程序生命週期記憶體快取與停用隱式重取；以切換頁籤／專案、focus、reconnect、超過原 staleTime、空結果及未開啟頁籤的 mock IPC 計數驗證不重掃、不預熱。
- [x] 2.3 實作首次失敗逐 key attempted 狀態及同 key single-flight，允許明確手動重試，不因 disabled→enabled 重試；以首次失敗後切換、重複刷新及手動重試測試驗證呼叫次數。
- [x] 2.4 移除跨 key 沿用上一份清單的 placeholder，保留同 key 刷新期間最後成功資料，延用共享全域 query；以 A→B→A、專案全域分區→全域頁及刷新失敗測試驗證來源不混用、資料不清空。

## 3. 清單識別、重新整理與操作後更新

- [x] 3.1 更新技能清單分組、搜尋、預覽、選取與 React key 使用子專案上下文及 raw name，顯示可辨識的子專案名稱；以兩個同名技能可獨立選取與預覽測試驗證，依 SessionHub Minimal UI 與既有 i18n 慣例呈現。
- [x] 3.2 更新技能同步 modal 使用列級目標存在狀態及上下文 targetRoot，而非根目錄 TargetInfo 或顯示標籤；以 dry-run 驗證只指向子專案 `.claude/skills/<name>`，根目錄目標不存在不會停用有效子專案目標。
- [x] 3.3 將重新整理 handlers 限縮至按鈕 scope、當前資源及相依顯示資料，prefs 變更時只刷新最終 key，執行中停用重複刷新並透過 toast 回報失敗；以各頁籤 mock IPC 計數驗證僅對應資源一次且無關 scope／kind 為零。
- [x] 3.4 成功 MCP 寫入／複製、Skills／Commands 實際同步及全域連結建立後，只更新受影響 active cache、標記受影響 inactive cache，避免預熱未載入資料；以 mutation 測試驗證跨畫面一致、dry-run／全部 skip／失敗／HTTP 測試不觸發重掃。
- [x] 3.5 固定操作當下 scope/key 並處理寫入與既有掃描競爭，確保寫入後再取得資料而非接受舊結果；以 deferred promises 驗證等待期間切到另一專案不污染清單，最後結果為寫入後狀態。
- [x] 3.6 在 Skills 清單上方持續呈現快取診斷，包含掃描不完整、上限或失敗路徑與原因，遵循 i18n 與既有 Minimal UI；以部分結果快取再進入、成功刷新替換及失敗刷新保留診斷的測試驗證。

## 4. 整合驗證

- [x] 4.1 在 `src-tauri` 執行 `cargo test agents_config::tests::` 及本案新增探索測試，確認正式掃描與既有 Skills／Commands／同步相關回歸通過。
- [x] 4.2 使用既有 Bun／happy-dom 執行新增的資源快取與 mock IPC 測試，驗證規格中的首次、命中、手動刷新、錯誤與 mutation 呼叫次數；執行 `bun run lint` 與 `bun run build` 並修正本案造成的失敗。
- [x] 4.3 在桌面應用程式驗證專案／全域三種資源首次載入、反覆切換、手動刷新、外部修改後刷新、子專案同名技能同步預覽與 MCP 編輯後更新；記錄快取命中不等待掃描、外部變更僅手動刷新反映的結果。
- [x] 4.4 執行 `openspec validate fix-project-skills-discovery-and-resource-cache --strict`，確認規格通過並僅於實際驗證成功後勾選已完成任務。

## 驗收紀錄

2026-10-12：使用者回報已驗收通過，確認能掃描子目錄 Skills；封存前另明確確認 4.3 的桌面整合驗證皆已完成，並同意同步主規格後封存。4.3 依使用者的實際驗收結果標記完成。
