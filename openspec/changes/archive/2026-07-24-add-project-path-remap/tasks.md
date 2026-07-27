## 1. 後端資料層

- [x] 1.1 在 `src-tauri/src/db.rs` 的 `init_db()` 新增 `project_path_remap` 資料表（`old_path_key` PRIMARY KEY、`old_path`、`new_path`、`created_at`）
- [x] 1.2 在 `db.rs` 新增路徑正規化函式（反斜線統一、去除結尾分隔符、轉小寫）與其單元測試
- [x] 1.3 在 `db.rs` 新增規則存取函式：`list_path_remaps`、`upsert_path_remap`（以 `old_path_key` 做 upsert）、`delete_path_remap`
- [x] 1.4 為 upsert 撰寫單元測試，驗證舊路徑大小寫不同時覆寫既有規則而非新增

## 2. 後端改寫邏輯

- [x] 2.1 新增改寫函式：輸入規則清單與路徑，依目錄邊界前綴比對回傳改寫後路徑，無相符則回傳原值
- [x] 2.2 為改寫函式撰寫單元測試：完全相符、子路徑相符、僅字首相符（`D:\old\project2` 不得被改寫）、無相符、空規則清單
- [x] 2.3 在 `sessions/mod.rs` 的 `get_sessions_internal()` 中，於 git 補強步驟（第 444 行附近）之前套用改寫至 `cwd` 與 `repo_root`；規則表為空時整段跳過
- [x] 2.4 驗證改寫後的 session 能正確解析 git 分支與 repo 名稱（原本 `repo_root` 為空者會重跑 git）

## 3. 後端 Tauri commands

- [x] 3.1 在 `src-tauri/src/commands/` 新增規則 CRUD command：列出、新增/更新、刪除，皆回傳 `Result<T, String>`
- [x] 3.2 新增/確認目錄存在檢查 command，供前端建立規則前驗證新路徑
- [x] 3.3 新增規則時於後端檢查新路徑目錄存在，不存在則回傳錯誤字串
- [x] 3.4 在 `lib.rs` 的 invoke handler 註冊新增的 commands

## 4. 前端型別與顯示路徑

- [x] 4.1 在 `src/types/index.ts` 新增路徑對應規則型別（camelCase 對應後端 serde）
- [x] 4.2 在 `src/App.tsx` 新增顯示用 helper：僅將開頭 `^[a-z]:` 的磁碟機代號轉大寫，其餘原樣
- [x] 4.3 將該 helper 套用於 `getProjectDisplayPath()` 的回傳值，確認 `normalizePath()` 的分組比對邏輯不受影響
- [x] 4.4 確認專案標題（`getProjectTitle`）與側邊欄顯示在磁碟機代號大寫後仍正確

## 5. 前端規則管理與引導

- [x] 5.1 在 `App.tsx` 新增規則清單 query 與 CRUD mutation，成功後失效 session 相關 query 使列表重新載入
- [x] 5.2 在 `SettingsView.tsx` 新增「專案路徑對應」區塊：規則清單顯示舊/新路徑，支援新增、編輯、刪除，並以 dialog plugin 提供資料夾瀏覽
- [x] 5.3 新增/編輯時若新路徑不存在，顯示錯誤 toast 並拒絕儲存
- [x] 5.4 在 `App.tsx` 加入專案路徑存在性檢查，將結果以 prop 傳入 `ProjectView`
- [x] 5.5 在 `ProjectView.tsx` 新增路徑失效提示與「重新指定資料夾位置」動作，選定資料夾後建立規則並重新載入
- [x] 5.6 在 `src/locales/zh-TW.ts` 與 `en-US.ts` 新增所有相關翻譯 key，確認 JSX 無硬編中文
- [x] 5.7 在 `App.css` 新增所需樣式（BEM-like class names，遵循既有設計 token）

## 6. 驗證

- [x] 6.1 執行 `cargo test` 確認後端測試全數通過
- [x] 6.2 執行前端 lint 與 type check 確認無錯誤
- [x] 6.3 手動驗證：建立規則後，該專案的開啟資料夾、開啟終端機、resume 皆指向新路徑
- [x] 6.4 手動驗證：舊路徑與新路徑的 session 合併為同一群組，路徑標籤顯示新路徑且磁碟機代號為大寫
- [x] 6.5 手動驗證：刪除規則後，受影響 session 還原顯示原始路徑
