## Context

SessionHub 目前只能讀取 Copilot CLI 的 session（YAML 檔案掃描），架構是「單一 root 路徑 → 掃描 session-state 目錄」。OpenCode 使用完全不同的儲存方式——SQLite 資料庫（`opencode.db`），包含 `session`、`project`、`message` 等表。

OpenCode 資料庫結構（已驗證）：
- `project` 表：`id`(text PK), `worktree`(text, 對應 cwd), `name`(text)
- `session` 表：`id`(text PK), `project_id`(FK→project), `title`(text), `slug`(text), `directory`(text), `time_created`(integer, unix ms), `time_updated`(integer, unix ms), `time_archived`(integer, nullable), `summary_additions/deletions/files`(integer)
- `message` 表：`id`(text PK), `session_id`(FK→session), `data`(text JSON, 含 role/agent/model 等資訊)

此外，使用者安裝了 oh-my-opencode 的 Sisyphus plan agent，會在專案目錄下建立 `.sisyphus/` 目錄：
- `boulder.json`：active plan 狀態（含 `active_plan` 路徑、`session_ids` 關聯、`plan_name`、`agent`）
- `plans/*.md`：結構化 plan 文件（含 TL;DR、Context、Work Objectives、Tasks）
- `notepads/<topic>/`：每個 topic 下有 `issues.md`、`learnings.md`
- `evidence/*.txt`：task 執行證據紀錄
- `drafts/*.md`：草稿文件

使用者也使用 openspec 管理變更規格：
- `openspec/config.yaml`：schema 設定
- `openspec/changes/<name>/`：進行中的 change（含 proposal.md、design.md、tasks.md、specs/）
- `openspec/changes/archive/`：已封存的 change
- `openspec/specs/<name>/spec.md`：累積的規格文件

目前專案頁面（`ProjectView`）只有單一 session 列表，無法呈現 `.sisyphus` 與 `openspec` 等專案級資料。

## Goals / Non-Goals

**Goals:**
- 支援從 OpenCode SQLite 資料庫讀取 session 資料，映射至共用 `SessionInfo` 結構
- 讓不同平台的 session 以相同專案路徑（cwd）合併分組
- 在前端提供平台來源篩選與平台標籤顯示
- 讀取各專案目錄下的 `.sisyphus/` 與 `openspec/` 資料，在 UI 中呈現
- 重構專案頁面為子分頁架構（Sessions / Plans & Specs），清晰分離不同類型資訊
- 保持完全向後相容：未安裝 OpenCode 或不存在 `.sisyphus`/`openspec` 時應用如常運作
- 設計可擴展架構，未來新增平台或資料來源時改動最小

**Non-Goals:**
- 不支援寫入或修改 OpenCode 資料庫（唯讀）
- 不實作 OpenCode session 的封存/刪除操作（OpenCode 自有管理機制）
- 不解析 OpenCode message/part 表的詳細對話內容
- 不實作 OpenCode session 的 events/stats 分析（第一階段僅基礎資訊）
- 不實作 `.sisyphus` plan 的編輯功能（僅唯讀檢視）
- 不實作 `openspec` change 的建立/修改功能（僅唯讀瀏覽狀態）
- 不跨平台支援 Linux/macOS（維持 Windows only）

## Decisions

### D1: Provider 抽象模式 — Trait-based Provider

**選擇**：定義 `SessionProvider` trait，每個平台實作為獨立 provider。

```rust
enum SessionProviderType { Copilot, OpenCode }

trait SessionProvider {
    fn provider_type(&self) -> SessionProviderType;
    fn scan_sessions(&self, show_archived: bool, connection: &Connection) -> Result<Vec<SessionInfo>, String>;
    fn is_available(&self) -> bool;
}
```

**替代方案**：直接在 `scan_sessions` 中用 if-else 分支加入 OpenCode 邏輯。
**理由**：trait 模式可測試、可擴展，新增平台只需實作 trait。if-else 方式雖簡單但隨平台增加會迅速膨脹。Rust trait 零成本抽象，無效能疑慮。

### D2: OpenCode 資料庫存取 — 唯讀連線

**選擇**：使用 `rusqlite` 開啟 OpenCode 的 `opencode.db` 為獨立唯讀連線（`SQLITE_OPEN_READ_ONLY`）。

**替代方案**：ATTACH DATABASE 到現有 metadata.db 連線。
**理由**：獨立唯讀連線更安全，避免意外寫入或 lock 衝突。OpenCode 可能在同時使用中，唯讀模式可與 OpenCode 的 WAL 模式共存。

### D3: SessionInfo 擴展 — 新增 `provider` 欄位

**選擇**：在 `SessionInfo` struct 新增 `provider: String` 欄位，值為 `"copilot"` 或 `"opencode"`。

**替代方案**：使用 enum `SessionProviderType`。
**理由**：String 對前端序列化更友善，且未來新增平台時不需修改 enum 定義。前端可直接用於 UI 顯示與 CSS class 判斷。

### D4: OpenCode 欄位映射

| OpenCode 欄位 | SessionInfo 欄位 | 轉換邏輯 |
|---|---|---|
| `session.id` | `id` | 直接使用 |
| `project.worktree` | `cwd` | 直接映射 |
| `session.title` | `summary` | 直接映射 |
| `session.time_created` (unix ms) | `createdAt` | 轉換為 ISO 8601 字串 |
| `session.time_updated` (unix ms) | `updatedAt` | 轉換為 ISO 8601 字串 |
| `session.directory` | `sessionDir` | 直接使用（用於唯一識別） |
| `session.time_archived` | `isArchived` | `IS NOT NULL` 判斷 |
| N/A | `parseError` | 恆為 `false`（SQLite 讀取不會有解析問題） |
| N/A | `hasPlan` | 恆為 `false`（OpenCode 無 plan 概念） |
| N/A | `hasEvents` | 恆為 `false`（OpenCode events 在 DB 中，非 jsonl） |
| `session.slug` | （可作為副標題顯示） | 直接使用 |

### D5: AppSettings 擴展

新增欄位（皆有 `#[serde(default)]` 確保向後相容）：

```rust
struct AppSettings {
    // ...existing...
    #[serde(default = "default_opencode_root")]
    opencode_root: String,
    #[serde(default = "default_enabled_providers")]
    enabled_providers: Vec<String>,  // ["copilot", "opencode"]
}
```

`default_opencode_root`：`%USERPROFILE%\.local\share\opencode\`
`default_enabled_providers`：`["copilot", "opencode"]`

### D6: 專案分組合併策略

**選擇**：正規化路徑後以字串比對合併。

兩個平台的 cwd 可能有細微差異（如大小寫、尾部斜線），採用 `PathBuf::canonicalize()` 或 `dunce::canonicalize()` 正規化後比對。同一路徑的 Copilot + OpenCode session 會歸入同一個 `ProjectGroup`。

### D7: FS Watcher 策略 — 監聽 OpenCode DB WAL

**選擇**：對 OpenCode 資料庫檔案使用 `notify` 監聽 `opencode.db-wal` 變更，偵測到變更時發出 `sessions-updated` 事件。

**替代方案**：定時輪詢（如每 30 秒查詢一次）。
**理由**：`notify` 監聽 WAL 檔案變更比輪詢更即時，且已有 watcher 基礎建設可複用。WAL 檔在每次寫入時都會變更，是可靠的變更指標。

### D8: 前端篩選狀態管理

**選擇**：`enabledProviders` 存入 `AppSettings`（持久化），前端透過現有 settings query/mutation 管理。

**替代方案**：純前端 state（不持久化）。
**理由**：持久化讓使用者偏好在重啟後保留。且 `enabledProviders` 直接傳入後端 `get_sessions`，可在 Rust 端篩選，減少無用資料傳輸。

### D9: 專案頁面子分頁架構

**選擇**：在 `ProjectView` 內新增子分頁（sub-tabs）機制，預設兩個分頁：

1. **Sessions**：現有的 session 列表（含搜尋、篩選、排序、provider filter）
2. **Plans & Specs**：`.sisyphus` 與 `openspec` 的唯讀瀏覽

```
ProjectView
├── Sub-tab bar: [Sessions] [Plans & Specs]
├── Sessions (default)
│   ├── toolbar (search, sort, provider filter, tag filter)
│   └── SessionCard list
└── Plans & Specs
    ├── .sisyphus section
    │   ├── Active Plan banner (from boulder.json)
    │   ├── Plans list (plans/*.md)
    │   ├── Notepads list (notepads/*)
    │   └── Evidence / Drafts (collapsible)
    └── openspec section
        ├── Active Changes (changes/*)
        ├── Archived Changes (changes/archive/*)
        └── Specs list (specs/*)
```

**替代方案A**：三個獨立分頁（Sessions / Sisyphus Plans / OpenSpec）。
**替代方案B**：全部混在同一頁面用 section 分隔。
**理由**：`.sisyphus` 與 `openspec` 都屬於「專案的開發計畫與規格」範疇，語義上自然歸為同組。兩個分頁（Sessions vs Plans & Specs）比三個分頁更精簡，又比混在同頁面更清晰。分頁內再用 section header 區分 sisyphus 與 openspec。

### D10: .sisyphus / openspec 資料讀取策略

**選擇**：新增獨立的 Tauri commands，按需讀取（lazy load）。

```rust
#[tauri::command]
fn get_project_plans(project_dir: String) -> Result<ProjectPlansData, String> { ... }

#[tauri::command]
fn read_plan_content(plan_path: String) -> Result<String, String> { ... }

#[tauri::command]
fn get_project_specs(project_dir: String) -> Result<ProjectSpecsData, String> { ... }
```

`.sisyphus` 與 `openspec` 資料僅在使用者切換到 "Plans & Specs" 分頁時才讀取，避免在 session 列表載入時增加不必要的 I/O。

**資料結構**：

```rust
struct SisyphusData {
    active_plan: Option<SisyphusBoulder>,
    plans: Vec<SisyphusPlan>,
    notepads: Vec<SisyphusNotepad>,
    evidence_files: Vec<String>,
    draft_files: Vec<String>,
}

struct SisyphusBoulder {
    active_plan: Option<String>,
    plan_name: Option<String>,
    agent: Option<String>,
    session_ids: Vec<String>,
    started_at: Option<String>,
}

struct SisyphusPlan {
    name: String,
    path: String,
    title: Option<String>,      // 從 Markdown # heading 取得
    tldr: Option<String>,       // 從 ## TL;DR section 取得
    is_active: bool,            // 是否為 boulder.json 的 active_plan
}

struct SisyphusNotepad {
    name: String,
    has_issues: bool,
    has_learnings: bool,
}

struct OpenSpecData {
    schema: Option<String>,
    active_changes: Vec<OpenSpecChange>,
    archived_changes: Vec<OpenSpecChange>,
    specs: Vec<OpenSpecSpec>,
}

struct OpenSpecChange {
    name: String,
    has_proposal: bool,
    has_design: bool,
    has_tasks: bool,
    specs_count: usize,
}

struct OpenSpecSpec {
    name: String,
    path: String,
}
```

### D11: Markdown 內容檢視

**選擇**：點擊 plan/spec 條目時，在 detail panel 中顯示 Markdown 原始內容（複用現有的 `PlanEditor` 唯讀模式或類似的 Markdown 預覽元件）。

**替代方案**：使用外部編輯器開啟。
**理由**：app 內預覽更流暢，且已有 `PlanEditor` 的 Markdown 渲染基礎可複用。同時保留「以外部編輯器開啟」作為輔助操作。

## Risks / Trade-offs

- **[OpenCode DB Lock 衝突]** → 使用 `SQLITE_OPEN_READ_ONLY` + `PRAGMA journal_mode` 檢查。WAL 模式下唯讀連線不會阻塞寫入者。
- **[大量 session 效能]** → OpenCode 已有 522 個 session，加上 Copilot 的量可能上千。在 Rust 端篩選 `enabled_providers` 避免全量傳送；前端 React Query 已有快取。
- **[OpenCode 資料庫格式變更]** → OpenCode 仍在快速迭代，schema 可能變更。以 try-catch 方式讀取，失敗時靜默忽略而非崩潰。
- **[路徑正規化]** → Windows 路徑可能有 `\\?\` 前綴或大小寫差異，使用 `dunce::canonicalize` 處理。
- **[時間格式差異]** → Copilot 使用 ISO 8601 字串，OpenCode 使用 unix timestamp (ms)。統一轉為 ISO 8601 字串在 `SessionInfo` 中。
- **[.sisyphus 不存在]** → 大多數專案不會有 `.sisyphus` 目錄。Plans & Specs 分頁需優雅處理空狀態（顯示「此專案尚無 plan 或 spec」提示）。
- **[ProjectView 重構複雜度]** → 現有 `ProjectView` 250 行，拆分為 sub-tabs 架構需仔細保持既有功能不變。策略：先將現有邏輯原封不動包入 Sessions sub-tab，再新增 Plans & Specs sub-tab。
