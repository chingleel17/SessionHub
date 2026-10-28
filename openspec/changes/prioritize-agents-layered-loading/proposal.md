# Proposal

## Why

大型專案的 AGENTS.md 與 Skills 散落在子目錄，目前必須等完整探索及內容比對結束才能使用根目錄資料。延續 OpenSpec 分層載入的改善，讓根目錄先可操作，同時維持完整探索與正確同步狀態。

## What Changes

- 專案 AGENTS.md 與 Skills 分成 root / nested 兩階段，根層先顯示，再背景合併子目錄結果。
- 所有顯示項目仍先完成既有 fingerprint、描述與內容指標，不引入推測的同步狀態。
- 專案中的全域 AGENTS.md / Skills 首次載入延至對應專案掃描結束後，獨立全域頁照常直接載入。
- 背景補齊與失敗有持續狀態提示，檔案預覽不被背景掃描遮住；刷新保留已載入資料。
- 指示檔 hash 與內容指標改為同一次檔案讀取。

## Capabilities

### New Capabilities

無。

### Modified Capabilities

- `agents-md-sync`: 根目錄優先回傳、背景補齊子目錄，預覽獨立於掃描。
- `agents-skills-sync`: 依專案上下文分層，保留完整性、provider 與子專案身分。
- `agents-resource-cache`: 分層結果共用既有快取上下文、刷新／寫入一致性與全域延後載入。

## Impact

影響 Rust Agents 掃描器及薄 command、App.tsx 的 IPC 與查詢、Agents 顯示元件、型別、翻譯及相關測試。不新增套件，既有未提供分層參數的呼叫保持完整掃描相容。
