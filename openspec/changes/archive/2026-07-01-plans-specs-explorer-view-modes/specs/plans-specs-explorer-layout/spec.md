## MODIFIED Requirements

### Requirement: 左側 Explorer 樹狀導覽

系統 SHALL 在左側面板提供可切換的 Explorer 導覽模式，用於顯示所有可瀏覽的 Sisyphus 與 OpenSpec 文件節點。

#### Scenario: 顯示 explorer 檢視模式切換

- **WHEN** PlansSpecsView 有可顯示的資料
- **THEN** 左側面板標題列顯示 explorer 檢視模式切換控制
- **AND** 使用者可在 `Tree`、`List`、`Cols` 三種模式間切換
- **AND** 切換模式時右側內容面板保持在同一個 Plans & Specs 檢視流程中

#### Scenario: 群組預設折疊狀態

- **WHEN** 任一檢視模式首次載入某專案的 explorer 資料
- **THEN** 僅 `Active Changes` 群組預設展開
- **AND** `Archived Changes`、`Specs` 等其他群組與子群組預設折疊
- **AND** 使用者手動展開或折疊群組後，該狀態在當前頁面 session 內維持

#### Scenario: Tree 模式顯示階層導覽

- **WHEN** 使用者切換到 `Tree` 模式
- **THEN** 左側以多層樹狀結構顯示根節點、群組節點與葉節點
- **AND** 群組節點仍可獨立展開或折疊
- **AND** 點擊可讀取的葉節點時，系統在右側載入對應文件內容並高亮該節點

#### Scenario: Tree 模式僅 artifact 葉節點顯示 icon

- **WHEN** Tree 模式渲染節點
- **THEN** 系統僅為 `proposal.md`、`design.md`、`tasks.md` 三種 artifact 葉節點顯示固定 icon
- **AND** 根節點、群組節點與 change 節點不顯示前置圓點或字母 icon
- **AND** 群組與 change 節點僅以展開/折疊指示符與文字標籤呈現

#### Scenario: List 模式以列表列顯示 change 區塊

- **WHEN** 使用者切換到 `List` 模式
- **THEN** 左側以列表列（row）方式呈現群組內容，而非帶陰影的卡片
- **AND** 每個 OpenSpec change 為一個獨立列區塊，列之間以間距區分
- **AND** change 列第一行顯示 change 名稱，並在該行最右側顯示所屬 spec 數量（如 `2 specs`）
- **AND** change 列第二行顯示其具備的 `proposal`、`design`、`tasks` 可點擊 badge，缺少的 artifact 不顯示對應 badge
- **AND** `tasks` badge 顯示 `done/total` 進度數字與依狀態著色的狀態指示
- **AND** 點擊任一 badge 在右側載入對應文件並高亮該列

#### Scenario: Cols 模式以兩欄逐層選取且單一狀態展開

- **WHEN** 使用者切換到 `Cols` 模式
- **THEN** 左側以兩欄呈現：第一欄為狀態群組清單，第二欄為所選群組內的項目
- **AND** 第一欄一次僅展開一個狀態群組（預設 `Active Changes`），選取其他群組時收合前一個
- **AND** 第二欄每個 change 列顯示 `done/total` 進度 badge 與進度條
- **AND** 點擊最終可讀取檔案後，右側面板顯示對應內容

#### Scenario: OpenSpec artifact 顯示固定 icon

- **WHEN** 左側顯示 OpenSpec change 內的 `proposal.md`、`design.md`、`tasks.md`
- **THEN** 系統為這三種 artifact 顯示可區分的固定 icon
- **AND** 不同檢視模式中的 icon 語意保持一致

#### Scenario: tasks 與 change 顯示進度 badge

- **WHEN** change 具有可用的 task progress summary
- **THEN** 左側在 `tasks.md` 與所屬 change 項目上顯示 `done/total` badge
- **AND** badge 或狀態標記依 progress 狀態顯示不同色彩
- **AND** 未開始、進行中、已完成三種狀態須可被視覺區分

### Requirement: 記住各專案最後使用的檢視模式

系統 SHALL 以每個專案為單位記住使用者最後選用的 explorer 檢視模式，並在重新選取該專案時自動還原。

#### Scenario: 切換專案時還原該專案上次的檢視模式

- **WHEN** 使用者在 A 專案選用 `Cols` 模式後切換到 B 專案
- **THEN** B 專案還原其自身上次使用的檢視模式，不受 A 專案影響
- **AND** 使用者再切回 A 專案時，仍呈現 `Cols` 模式

#### Scenario: 首次開啟專案的預設模式

- **WHEN** 某專案尚無已記錄的檢視模式偏好
- **THEN** 系統以 `Tree` 作為該專案的預設檢視模式
- **AND** 使用者選用任一模式後即成為該專案的最後使用模式

### Requirement: 左側面板寬度可調整

系統 SHALL 允許使用者透過拖曳分隔線調整左側 Explorer 面板的寬度，並讓新版標題列與三種模式都能正常顯示。

#### Scenario: 拖曳調整寬度

- **WHEN** 使用者按住分隔線並拖曳
- **THEN** 左側面板寬度隨滑鼠移動即時調整
- **AND** 左側最小寬度須足以同時容納模式切換、icon 與進度 badge
- **AND** 右側內容面板保持可閱讀的最小寬度

#### Scenario: 折疊左側面板

- **WHEN** 使用者點擊折疊切換按鈕
- **THEN** 左側面板縮小至折疊狀態
- **AND** 右側面板佔用釋放的空間
- **AND** 使用者可再次展開並恢復 explorer 導覽

### Requirement: Explorer 佈局高度與標題列對齊

系統 SHALL 確保 Explorer 佈局的高度受限於視窗可視區域，且左側與右側標題列在新版設計下維持一致的對齊關係。

#### Scenario: Explorer 整體高度限制

- **WHEN** PlansSpecsView 在任意視窗大小下渲染
- **THEN** Explorer 佈局高度受限於可視區域
- **AND** 左右兩側面板可獨立捲動
- **AND** 不因 explorer 模式切換而產生頁面級捲動

#### Scenario: 兩側標題列對齊

- **WHEN** Explorer 面板與內容面板同時顯示
- **THEN** 左右標題列保持固定高度並水平對齊
- **AND** 左側可容納模式切換與操作按鈕
- **AND** 右側仍顯示目前文件路徑作為標題列內容
