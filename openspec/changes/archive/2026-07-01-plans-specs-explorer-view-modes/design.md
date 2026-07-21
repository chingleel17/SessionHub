## Context

Plans & Specs 目前使用單一左側樹狀 explorer，資料來源同時包含 Sisyphus 與 OpenSpec。這個區域已經具備雙面板佈局、可折疊左欄、可拖曳寬度，以及右側 markdown 內容檢視與 `tasks.md` 勾選寫回能力，但左欄節點資訊密度偏低，無法支援使用者在大量 change/spec 文件之間快速切換。新的設計稿要求保留現有右側內容體驗，同時讓左欄支援不同導覽模型、文件 icon 與任務進度摘要。

此變更同時跨前後端。前端需要在 `PlansSpecsView` 與 `ExplorerTree` 增加多種導覽模式與新的節點 metadata；後端 OpenSpec 掃描流程需要在不新增 IPC 命令的前提下，提供 `tasks.md` 的完成數、總數與狀態。

## Goals / Non-Goals

**Goals:**

- 在同一個 Plans & Specs 畫面內提供 `Tree`、`List`、`Cols` 三種左側導覽模式。
- 讓 OpenSpec change 與 `tasks.md` 顯示進度 badge 與狀態色彩。
- 為 `proposal.md`、`design.md`、`tasks.md` 與其他節點提供穩定的 icon metadata，讓不同檢視模式共用。
- 延續既有右側內容檢視、檔案讀取、錯誤顯示與 `tasks.md` 寫回機制。

**Non-Goals:**

- 不新增新的後端 command，也不改變 Plans & Specs 資料來源。
- 不重做整個 ProjectView 或右側 markdown renderer。
- 不處理 OpenSpec 以外格式的待辦進度，例如 Sisyphus plan 內容的勾選統計。

## Decisions

### 1. 擴充既有 `OpenSpecChange` 回傳資料，而不是新增獨立 progress API

後端在掃描 `openspec/changes/*/tasks.md` 時直接解析 markdown task list，回傳 `taskProgress = { done, total, status }`。這讓前端在同一次 `get_project_specs` 查詢中就能拿到左欄所需資訊，不需要額外讀檔或再發第二個 IPC。

替代方案：
- 在前端讀取 `tasks.md` 後自行計算 progress：會導致左側清單需要額外逐檔讀取，載入成本高且更新同步複雜。
- 新增獨立 command 查詢 progress：增加 IPC 面積，但資料仍來自同一個掃描來源，收益不高。

### 2. 以 `TreeNode` metadata 驅動三種檢視，而不是分開維護三份資料模型

前端沿用既有 `TreeNode` 作為單一中介模型，新增 `icon`、`tone`、`progress`、`badge` 等欄位。`Tree`、`List`、`Cols` 只是在同一份節點資料上使用不同渲染器，右側內容讀取仍由被選取的可讀檔案節點觸發。

替代方案：
- 各檢視模式建立不同 view model：容易出現選取邏輯、badge 規則與色彩狀態不一致。
- 讓 `List`/`Cols` 直接綁 OpenSpecData 與 SisyphusData：會把視圖邏輯分散到多個元件，不利維護。

### 3. 以 view-mode switcher 保留單一左欄容器與寬度調整能力

左欄仍使用同一個可折疊、可拖曳寬度的 explorer panel，只在內容區切換 `tree`、`list`、`cols`。這讓使用者切換檢視時不失去整體版面習慣，也避免三種模式各自管理不同寬度與折疊狀態。

替代方案：
- 為每種模式建立不同 panel：視覺切換成本高，也會讓 collapsed/expanded 行為更複雜。

### 4. `Cols` 模式採兩欄並一次展開單一狀態群組

依新的視覺稿，`Cols` 模式固定為兩欄：第一欄為狀態群組清單，第二欄顯示所選群組內的 change 列（含進度 badge 與進度條）。第一欄一次僅展開一個狀態群組，預設 `Active Changes`，避免三欄在平坦資料下浪費空間，也讓欄式選取聚焦在單一狀態。

替代方案：
- 三欄漸進（原方向）：與設計稿不符，平坦資料浪費空間。
- 第一欄同時展開多個群組：會讓第二欄內容來源不明確，違反「一次一狀態」的設計意圖。

### 5. 以每個專案為單位持久化最後使用的檢視模式

view-mode 偏好以 `projectId → mode` 對應持久化（沿用既有專案層級設定存放方式），切換專案時還原該專案上次模式；尚無偏好時以 `Tree` 為預設。

替代方案：
- 全域單一模式：實作較簡單，但切換專案會丟失各專案的檢視習慣，不符需求。
- 僅存當前頁面狀態不持久化：重新進入專案會回到預設，無法達成「切回 B 專案仍是 Cols」。

## Risks / Trade-offs

- [Markdown task 解析僅支援標準 checklist 語法] → 掃描器只計算 `- [ ]`、`- [x]`、`1. [ ]` 等常見格式，避免過度複雜；若未來格式更多，再擴充 parser。
- [左欄寬度與模式切換後的版面密度不同] → 保留拖曳調整，並提高最小寬度以容納 badge、icon 與 switcher。
- [三種檢視共享選取狀態，實作較集中] → 把選取與內容讀取仍集中在 `PlansSpecsView`，避免分散到子元件。
- [規格與目前實作細節可能同步變動] → 在 spec 中聚焦互動與資訊呈現要求，不把純視覺 token 寫死成過度細節的 CSS 常數。

## Migration Plan

- 此變更屬於本地桌面 UI 與掃描資料模型調整，無資料庫 migration。
- 先擴充 Rust 掃描結構與 TypeScript 型別，再更新左欄渲染元件與樣式。
- 若需要回滾，可移除 `taskProgress` 欄位與新檢視模式，恢復單一 tree 渲染器。

## Open Questions

- `Cols` 模式第二欄各群組最後一次選取的 change 是否需要持久化；本次先不持久化，只記住檢視模式本身。
- 是否要把 archived changes 的 badge 色彩與 active changes 完全一致；本次先依 task status 顯示，不另加 archived 專屬色彩語意。
