## Why

SessionHub 的專案路徑（`cwd` / `repoRoot`）全部來自各 AI CLI 工具自己寫下的 session 檔案，SessionHub 只讀不寫，因此有兩個使用者實際遇到的問題無法靠「改儲存方式」解決：

1. **顯示路徑大小寫不一致**：不同工具（或同一工具不同版本）寫入的磁碟機代號大小寫不同，畫面上同一個專案有時顯示 `d:\ching\SourceCode\Unity practise`、有時 `D:\...`。分組本身不受影響（`buildProjectGroups` 已用小寫正規化比對），純粹是專案標題下的路徑標籤看起來雜亂。
2. **資料夾搬移或改名後功能全數失效**：使用者把專案資料夾換位置或改名（例如把空格拿掉），舊 session 記錄的仍是舊路徑。此時「開啟資料夾」、「開啟終端機」、「resume session」、git 分支/repo 名稱解析、OpenSpec/AGENTS 掃描全部指向已不存在的目錄而失敗，且這些 session 會被歸到一個死掉的專案群組。

這兩件事必須在 SessionHub 端以「讀取後轉換」的方式處理，不能回頭改寫各工具的 session 資料。

## What Changes

- **顯示路徑正規化（僅磁碟機代號）**：專案路徑在顯示時一律將磁碟機代號轉為大寫（`d:\...` → `D:\...`），路徑其餘部分維持工具寫入的原始大小寫不變（`Unity practise` 不會被改成 `UNITY PRACTISE`）。此為顯示層一致性處理，不影響既有的分組比對邏輯。
- **新增專案路徑重新對應（remap）功能**：使用者可為某個專案群組指定「新路徑」，SessionHub 在讀取 session 後、進行 git 解析前，將符合舊路徑（或其子路徑）的 `cwd` / `repoRoot` 改寫為新路徑，使後續所有功能（分組、git 中繼資料、開啟資料夾/終端機、resume、OpenSpec 掃描）都作用在正確的位置。
- **重新對應規則的持久化與管理**：對應規則存於 SQLite（`metadata.db`）新資料表，以正規化後的舊路徑為 key（因此可同時吸收大小寫差異）。設定頁提供規則清單，可新增、編輯、刪除。
- **失效專案的引導入口**：當某專案群組的路徑實際上已不存在時，於專案畫面顯示提示與「重新指定資料夾位置」動作，讓使用者直接挑選新目錄建立對應規則。
- 新增後端指令：查詢/新增/刪除路徑對應規則、檢查目錄是否存在。

## Capabilities

### New Capabilities
- `project-path-remap`: 專案路徑重新對應 — 對應規則的資料模型與持久化、session 讀取後的路徑改寫（含子路徑前綴改寫）、失效路徑偵測，以及設定頁與專案畫面的規則管理與引導 UI。

### Modified Capabilities
- `session-grouping`: 新增「顯示路徑的磁碟機代號一律大寫」需求；並補充分組發生於路徑重新對應之後（對應後的路徑才是分組依據）。

## Impact

- **前端**：`src/App.tsx`（顯示路徑正規化 helper、remap 規則的 IPC 呼叫與 query 失效）、`src/components/SettingsView.tsx`（規則管理區塊）、`src/components/ProjectView.tsx`（失效路徑提示與重新指定入口）、`src/locales/*.ts`（新增翻譯 key）、`src/types/index.ts`（新增規則型別）。
- **後端**：`src-tauri/src/db.rs`（新增 `project_path_remap` 資料表與存取函式）、`src-tauri/src/sessions/mod.rs`（於 `get_sessions_internal` 的 git 補強步驟之前套用改寫）、`src-tauri/src/commands/`（新增規則 CRUD 與目錄存在檢查 command）。
- **快取**：session 列表快取（`sessions_cache`）存的是改寫前的原始路徑，規則變動後需讓前端重新取得 session 並重跑 git 解析。
- **已知限制**：重新對應只修正 SessionHub 這一側的導覽與啟動路徑；各工具內部仍以舊路徑為 key 保存自身歷史（例如 Claude 的 project 目錄），因此在新路徑下 resume 時該工具未必能列出舊路徑的歷史紀錄。
- **風險**：改寫規則若設定錯誤會讓正常專案被導到錯的目錄；需限定為前綴比對且在建立規則時檢查新目錄存在。
