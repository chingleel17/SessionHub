## Why

Plans & Specs 的 Explorer 目前把 Sisyphus 與 OpenSpec 的節點混在同一層呈現，導致三個實際可見的問題：

1. List 與 Cols 模式在組裝清單時直接跳過 `root:sisyphus` / `root:openspec` 兩個來源根節點，把底下的群組攤平在同一層，使用者無法分辨也無法各自收折；Tree 模式因為保留了根節點反而是唯一正確的。
2. Cols 模式的 `resolveChangeAction()` 對所有非 `spec` 項目一律套用 OpenSpec 專屬的 `待 propose` / `可 apply` / `可封存` 判斷，讓 Sisyphus 的 plans、notepads 被標上不存在的 OpenSpec 動作與 slash command。
3. Sisyphus 的 evidence 與 drafts 節點點擊必定失敗：後端 `list_files_with_ext()` 只回傳裸檔名（如 `a.txt`），前端卻以 `filePathType: "absolute"` 讀取。notepads 則因後端 `SisyphusNotepad` 完全沒有路徑欄位而無法開啟任何內容。

## What Changes

- **Explorer 來源分層**：List 與 Cols 模式改為保留 Sisyphus / OpenSpec 來源層級，各自可獨立展開或收折，與 Tree 模式的階層語意一致。Cols 維持兩欄（不新增第三欄），來源層以可折疊標頭形式包住其狀態群組。
- **OpenSpec 專屬邏輯限縮**：`resolveChangeAction()` 與 `proposal`/`design`/`tasks` badge 僅套用於 OpenSpec change 項目；Sisyphus 項目不顯示任何 OpenSpec 動作徽章或 slash command。
- **修正 Sisyphus 檔案路徑**：後端 `list_files_with_ext()` 回傳完整絕對路徑，讓 drafts 可正常開啟。
- **notepads 可開啟**：後端 `SisyphusNotepad` 補上 `issues.md` / `learnings.md` 的絕對路徑，前端將 notepad 展開為可點擊的子節點。
- **排除 evidence**：`.sisyphus/evidence/` 為 `.txt` 任務進度輸出，不適合以 markdown 呈現且樣本不足，本次自 Explorer 全面移除（含 `hasSisyphus` 判定條件），後端保留掃描結果不動。
- **狀態初始值去除寫死深度**：`columnsOpenGroups` 預設值與 selectedNode 同步時的節點路徑 offset 改為依來源層級推導，不再假設 `path[1]` 必為狀態群組。

## Capabilities

### New Capabilities
- `explorer-source-grouping`: Plans & Specs Explorer 以資料來源（Sisyphus / OpenSpec）分層呈現，且該分層在 Tree、List、Cols 三種模式語意一致、可各自展開收折。

### Modified Capabilities
- `plans-specs-explorer-layout`: List 與 Cols 模式的階層契約新增來源層；Cols 第一欄不再是扁平的狀態群組清單；OpenSpec 專屬動作徽章的套用範圍由「非 spec 項目」改為「OpenSpec change 項目」。
- `sisyphus-reader`: Sisyphus 檔案節點須提供可開啟的絕對路徑；notepads 須可讀取 issues/learnings 內容；evidence 不再作為可瀏覽節點。

## Impact

- 前端：`src/components/PlansSpecsView.tsx`（List/Cols 渲染、`resolveChangeAction`、`allColsStatusGroups`、`columnsOpenGroups`、selectedNode 同步 effect、`hasSisyphus`）、`src/utils/buildTree.ts`（`buildSisyphusTree`）、`src/types/index.ts`（`SisyphusNotepad`）
- 後端：`src-tauri/src/sisyphus.rs`（`list_files_with_ext`、`scan_sisyphus_internal`）、`src-tauri/src/types/sisyphus_openspec.rs`（`SisyphusNotepad`）
- 測試：`src-tauri/src/lib.rs` 內 `scan_sisyphus_*` 測試的斷言需隨路徑格式更新
- 樣式：`src/styles/` 內 `explorer-list-*` / `explorer-cols-*` 需新增來源層標頭樣式
- 無 breaking change 對外 API；`read_file_content` command 介面不變
