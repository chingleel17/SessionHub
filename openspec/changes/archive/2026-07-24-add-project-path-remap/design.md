## Context

SessionHub 的專案路徑來自各 provider 自己寫入的 session 檔案（opencode 的 `directory`、Claude/Codex/Copilot 的 session 記錄），SessionHub 只讀取不回寫。因此本次的兩個問題都必須以「讀取之後的轉換層」解決：

- **顯示大小寫**：`src/App.tsx:61` 的 `normalizePath()` 已在分組時將兩側轉小寫比對，所以大小寫差異**不會**造成專案群組分裂；`getProjectDisplayPath()`（App.tsx:85）直接取用第一筆 session 的原始字串，才是畫面上時大時小的原因。Windows 檔案系統本身大小寫不敏感，因此大小寫**從來不是**開啟資料夾/工具失敗的原因。
- **資料夾搬移/改名**：這才是造成開啟資料夾、開啟終端機、resume、git 解析、OpenSpec 掃描全部失敗的原因，需要真正的路徑改寫。

後端 session 組裝的匯流點在 `src-tauri/src/sessions/mod.rs` 的 `get_sessions_internal()`：所有 provider 的掃描結果在此匯入 `all_sessions`，其後（第 444 行起）才對 `repo_root` 為空的 session 執行 git 子程序。這個位置是套用改寫的唯一正確插入點。

## Goals / Non-Goals

**Goals:**
- 專案顯示路徑的磁碟機代號一致大寫，其餘部分不動。
- 使用者能為搬移或改名過的專案指定新路徑，讓 SessionHub 的導覽與啟動功能重新可用。
- 改寫在單一位置生效，讓分組、git 中繼資料、開啟資料夾/終端機、resume、OpenSpec/AGENTS 掃描全部一致受惠。

**Non-Goals:**
- 不改寫任何 provider 的 session 原始檔案。
- 不自動偵測資料夾被搬到哪裡（不做全碟搜尋或相似度猜測），一律由使用者指定。
- 不處理各工具內部以舊路徑為 key 的歷史資料（例如 Claude 的 project 目錄）。
- 不修改既有 `normalizePath()` 的分組比對語意。
- 不改寫即時事件（activity hint）payload 中的 `cwd`。這些事件由 hook 從「工具當下實際執行的目錄」發出，該目錄必然是搬移後的新路徑，不會是待改寫的舊路徑；且事件只用於查找既有 session 以更新活動狀態，比對不到僅是漏掉一次即時狀態閃示，下次批次載入即恢復正確。

## Decisions

### 決策一：改寫套用於後端 `get_sessions_internal()`，在 git 補強之前

在 `sessions/mod.rs` 第 444 行的 git 補強步驟**之前**插入改寫，改寫 `cwd` 與 `repo_root`。

- **為何不放前端**：前端只改顯示的話，`resolve_git_metadata()` 仍會對死路徑跑 git、`open_terminal`/`resume` 仍會用死路徑當 `current_dir`。功能性問題不會被解決。
- **涵蓋範圍限定於批次載入路徑**：`get_sessions_internal()` 的唯一出口在第 471 行（第 207–406 行皆為 provider 分支內的 `match` 分支，非提前返回），因此批次載入的所有 session 必經改寫。但即時事件路徑（`src/hooks/useSessionRealtimeEvents.ts`）另以 `normalizePath(payload.cwd)` 比對既有 session 來套用活動狀態，不經過本改寫。此處刻意不處理，理由見 Non-Goals。
- **為何要在 git 補強之前**：改寫後路徑才會指向真實存在的 git 儲存庫，git 分支與 repo 名稱才解析得出來。若放在之後，這些欄位仍是空的。
- **替代方案（否決）**：在每個 provider 的掃描函式各自改寫 — 需要改五個檔案且容易漏掉新 provider。

### 決策二：規則存於 SQLite `metadata.db`，以正規化舊路徑為主鍵

新增資料表，沿用 `db.rs` 現有的 `CREATE TABLE IF NOT EXISTS` 慣例（此專案未使用版本化 migration）：

```sql
CREATE TABLE IF NOT EXISTS project_path_remap (
    old_path_key TEXT PRIMARY KEY,   -- 正規化後的舊路徑（比對用）
    old_path     TEXT NOT NULL,      -- 原始大小寫（顯示用）
    new_path     TEXT NOT NULL,
    created_at   TEXT NOT NULL
)
```

以正規化路徑當主鍵，讓「同一目錄但大小寫不同」自然收斂為一筆規則 — 這是兩個功能的交會點：大小寫差異在此被規則比對自動吸收。

- **為何不放 `settings.json`**：規則屬於機器本地的資料狀態（與 session 快取同性質），而非使用者偏好設定；且 `db.rs` 已是 session 相關中繼資料的既有歸屬。

### 決策三：前綴比對必須落在目錄邊界

改寫條件為：正規化後的 `cwd` 等於規則舊路徑，**或**以「規則舊路徑 + 路徑分隔符」為開頭。避免 `D:\old\proj` 的規則誤傷 `D:\old\project2`。改寫時保留 session 原始路徑的後半段（子路徑部分維持其原始大小寫）。

規則不做鏈式解析（A→B 且 B→C 不會推導出 A→C），只比對一輪，取第一個相符的規則。理由是 YAGNI 且可避免環狀規則。

### 決策四：顯示層磁碟機代號大寫，與分組比對脫鉤

在 `App.tsx` 新增獨立的顯示用 helper（例如 `formatDisplayPath()`），只將開頭符合 `^[a-z]:` 的字元轉大寫，其餘原樣輸出，套用於 `getProjectDisplayPath()` 的回傳值。

- **明確不做**：不把整條路徑轉大寫。使用者的訴求是「硬碟槽」（磁碟機代號），把 `Unity practise` 變成 `UNITY PRACTISE` 會破壞可讀性。
- 既有的 `normalizePath()`（全小寫）維持不變，繼續負責分組 key 與 pinned key 的比對，兩者職責分離。

### 決策五：規則變更後強制重新載入 session

`sessions_cache` 存的是改寫前的原始路徑，改寫發生在讀取快取之後，因此快取不需失效或重建。但前端必須重新取得 session 列表，且需讓 git 中繼資料重新解析（原本 `repo_root` 為空的判斷會自然重跑）。規則 CRUD 成功後由 `App.tsx` 失效 session 相關 query 即可。

### 決策六：選擇資料夾沿用既有 dialog plugin

專案已引入 `@tauri-apps/plugin-dialog` / `tauri-plugin-dialog`（`lib.rs:362` 已註冊），失效專案引導與設定頁的「瀏覽」皆使用其資料夾選取，不新增相依套件。

## Risks / Trade-offs

- **規則設定錯誤把正常專案導向錯誤目錄** → 限定前綴比對於目錄邊界；建立/編輯時後端檢查新路徑存在，不存在即拒絕；設定頁可隨時刪除規則還原。
- **改寫後 resume 時工具找不到舊歷史**（各工具內部以舊路徑為 key） → 屬已知限制，記於 proposal；本次不處理，必要時於 UI 提示使用者。
- **每次 `get_sessions_internal()` 都要查一次規則表** → 規則數量極少（個位數），一次查詢後在記憶體比對，成本可忽略；若規則表為空則整段跳過。
- **舊路徑與新路徑的 session 合併後，group key 由改寫後路徑決定** → 已釘選（pinned）的專案若 key 變動會失去釘選狀態。可接受：使用者重新釘選即可，不為此加相容層。

## Migration Plan

資料表以 `CREATE TABLE IF NOT EXISTS` 在 `init_db()` 建立，既有使用者升級後規則表為空、行為完全不變（無規則即不改寫）。無需資料遷移，回退僅需移除改寫呼叫，資料表留著不影響舊版本運作。

## Open Questions

無。所有決策已依現有程式碼結構與使用者訴求確定。
