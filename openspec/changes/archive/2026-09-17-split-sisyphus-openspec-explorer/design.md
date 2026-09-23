## Context

`PlansSpecsView.tsx` 已在 `rootNodes` 中建立兩個來源根節點（`root:sisyphus`、`root:openspec`），Tree 模式直接渲染 `rootNodes` 因此階層正確。問題出在兩處攤平：

- `renderListView()`（L591-596）與 `allColsStatusGroups`（L443-451）以相同的雙層迴圈 `for rootNode of rootNodes → for child of rootNode.children` 收集群組，捨棄 `rootNode` 本身，兩個來源的群組因此併入同一個扁平陣列。
- `resolveChangeAction()`（L31-55）以 `entryNode.id.replace(/^openspec:(change|archived):/, "")` 取名，對 Sisyphus 節點 id（如 `sisyphus:plan:<path>`）不會匹配而原樣保留，再加上呼叫端僅以 `entryNode.icon === "spec"` 排除，使 plan/notepad 一律落入 `!hasProposal` 分支而顯示「待 propose」。

另有兩處寫死的深度假設會在新增來源層後失效：`columnsOpenGroups` 初始值寫死 `new Set(["openspec:active-changes"])`（L257-259）；selectedNode 同步 effect 以 `path[1]` 當狀態群組、`path[2]` 當 change（L454-472）。

後端 `sisyphus.rs` 的 `list_files_with_ext()` 只 push `path.file_name()`，`buildSisyphusTree()` 卻以 `filePathType: "absolute"` 使用它，因此 drafts 與 evidence 節點必定讀取失敗。`SisyphusNotepad` 無任何路徑欄位，notepad 節點本就無 `filePath`。

## Goals / Non-Goals

**Goals:**
- List 與 Cols 模式保留來源層級，兩來源可各自展開收折
- OpenSpec 專屬邏輯（動作徽章、slash command、artifact chip）僅套用於 OpenSpec change
- 修正 Sisyphus drafts 路徑、補上 notepad 檔案路徑，使其可開啟
- 自 Explorer 移除 evidence
- 消除 `columnsOpenGroups` 與 selectedNode 同步中的寫死深度

**Non-Goals:**
- 不為 Cols 新增第三欄；維持兩欄
- 不解析或美化 evidence `.txt` 內容（後端掃描結果保留但前端不使用）
- 不變更 Tree 模式現有渲染邏輯與 `ExplorerTree` 元件介面
- 不調整排序（`sortChanges`）行為

## Decisions

### 決策 1：以 `sourceKind` 標記節點，而非依深度推導

在 `TreeNode` 新增選用欄位 `sourceKind?: "sisyphus" | "openspec"`，由 `buildSisyphusTree()` / `buildOpenSpecTree()` 標記其產出的**所有**節點（來源根節點、群組節點、entry 節點與 artifact 葉節點），而非僅群組層。

**理由**：目前各處以節點深度或 id 前綴字串比對判斷歸屬，脆弱且分散。單一顯式欄位讓 `resolveChangeAction` 的呼叫條件、來源層渲染、狀態同步三處共用同一判準。

必須標到葉節點而非僅群組層，是因為 `renderListChangeRow(item)` 透過 `ListGroup` 的 `renderItem: (item: TreeNode) => ReactNode` 簽名只收到 entry 節點本身，取不到其群組或來源；若僅標群組層，List 模式將無法判斷該項目屬於哪個來源。全面標記亦讓 Cols 第二欄與 selectedNode 反查等處可直接自 entry 節點取得來源，無須額外傳參。

**替代方案**：將 `renderItem` 簽名改為 `(item, sourceKind)` 由 `ListGroup` 傳遞。已否決 — 僅解決 List 一處，其餘需要自 entry 反推來源的位置仍得各自處理。

**替代方案**：繼續以 `id.startsWith("openspec:")` 判斷。已否決 — id 格式散落在 buildTree 與 View 兩側，任一處改名即靜默失效。

### 決策 2：List / Cols 的階層契約改為 source → group → entry → artifact

以一個共用的 `sourceSections` memo 取代兩處重複的攤平迴圈：

```
sourceSections: Array<{ sourceNode: TreeNode; groups: TreeNode[] }>
```

`renderListView()` 與 `renderColumnsPanel()` 皆消費此結構，各自在最外層渲染可折疊的來源標頭。

**理由**：兩處攤平邏輯完全相同，抽出後修一次即可，也避免未來再次分歧。

### 決策 3：Cols 維持兩欄，來源層作為第一欄內的折疊標頭

第一欄結構為：來源標頭（可折疊）→ 其狀態群組（手風琴，全域單選）。第二欄不變。

**理由**：使用者訴求是「各自展開或收折」，不是多一層導覽。加第三欄會在已受限的 260–560px 面板寬度內進一步壓縮可讀性。

### 決策 4：`columnsOpenGroups` 預設值由資料推導

改為：優先取 OpenSpec 的 `openspec:active-changes`；若該群組不存在，取第一個可見群組。以 `useEffect` 在 `sourceSections` 就緒後套用，而非 `useState` 初始值寫死。

### 決策 5：selectedNode 同步改以 `findNodePath` 結果反查

不再使用 `path[1]` / `path[2]` 固定 offset，改為在回傳路徑中尋找「第一個屬於已知狀態群組清單的節點」作為 statusGroup，其後一個節點作為 change。

**理由**：來源層插入後 offset 位移；反查方式對層數變動免疫。

### 決策 6：後端 `list_files_with_ext` 回傳絕對路徑

`list_files_with_ext()` 改 push `path.to_string_lossy().to_string()`。前端 `buildSisyphusTree()` 顯示時仍以 `f.split(/[\\/]/).pop()` 取檔名作為 label，該行為已存在無需改動。

**理由**：型別 `Vec<String>` 不變，僅內容語意由檔名改為完整路徑，與 `SisyphusPlan.path` 一致。既有 Rust 測試斷言需同步更新。

**替代方案**：前端組路徑。已否決 — 前端無 `.sisyphus` 根目錄資訊，需額外傳遞專案路徑並重複 join 邏輯。

### 決策 7：`SisyphusNotepad` 補 `issuesPath` / `learningsPath`

新增兩個 `Option<String>` 欄位承載絕對路徑，保留既有 `has_issues` / `has_learnings` 布林欄位以免影響現有使用端。前端將 notepad 由葉節點改為帶 children 的群組節點，僅為存在的檔案產生子節點。

### 決策 8：evidence 在前端層移除

`buildSisyphusTree()` 刪除 evidence 區塊。後端 `evidence_files` 欄位與掃描邏輯保留，未來若決定支援可直接接回。

**理由**：移除範圍最小且可逆。若連後端一併刪除，日後恢復成本較高。

### 決策 9：`hasSisyphus` / `hasOpenSpec` 改由 builder 產出推導

兩個旗標不再各自維護一份欄位條件清單（如 `evidenceFiles.length > 0 || activePlan !== null`），改為直接判斷對應 builder 是否產出節點：`sisyphusNodes.length > 0` / `openspecNodes.length > 0`。

**理由**：現行條件與 builder 實際會渲染的內容脫鉤。移除 evidence 後，只有 `boulder.json`、其餘皆空的專案會滿足 `activePlan !== null` 而判定為「有資料」，卻因 `buildSisyphusTree()` 回傳 `[]` 而不產生任何節點，導致面板全白且不顯示空狀態提示。由產出推導可一次消除這整類不同步，也讓 spec 中「僅有 evidence 的專案視為無 Sisyphus 可顯示資料」的 scenario 成立。

## Risks / Trade-offs

- **既有 localStorage 狀態相容性**：`explorer-view-mode:<cwd>` 與 `explorer-sort:<cwd>` 的值格式不變，不受影響。`columnsOpenGroups` 不落地儲存，無遷移問題。
- **`hasSisyphus` 條件變更的邊界**：僅含 evidence 的專案將由「顯示空的 Sisyphus 層」變為「完全不顯示」。此為預期行為，已寫入 spec scenario。
- **Rust 測試需同步更新**：`lib.rs` 內 `scan_sisyphus_*` 測試斷言 `vec!["a.txt", ...]`，改為絕對路徑後需改以 `ends_with` 或組合 `project_dir` 比對，避免斷言綁定臨時目錄名稱。
- **前端無自動化測試覆蓋**：`tests/` 目前僅有 `DashboardView.test.ts`，`buildTree` 與 `PlansSpecsView` 皆無測試。本次為 `buildSisyphusTree` 補上單元測試（來源標記、notepad 展開、evidence 排除），Cols/List 的互動行為則以手動驗證為主。
