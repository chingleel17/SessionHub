## Requirements

### Requirement: 路徑對應規則的資料模型
系統 SHALL 以 SQLite 資料表持久化專案路徑對應規則，每筆規則包含舊路徑、新路徑與建立時間，並以「正規化後的舊路徑」作為唯一鍵。正規化 SHALL 為：反斜線統一、去除結尾分隔符、轉為小寫。

#### Scenario: 建立規則後重啟應用程式仍存在
- **WHEN** 使用者建立一筆從 `D:\old\proj` 對應到 `D:\new\proj` 的規則並關閉應用程式
- **THEN** 重新啟動後該規則仍存在且持續生效

#### Scenario: 舊路徑大小寫不同視為同一筆規則
- **WHEN** 已存在舊路徑為 `D:\old\proj` 的規則，使用者再以 `d:\OLD\proj` 建立規則
- **THEN** 系統覆寫既有規則而非新增第二筆

### Requirement: Session 路徑改寫
系統 SHALL 在讀取各 provider 的 session 之後、進行 git 中繼資料解析之前，依對應規則改寫 session 的 `cwd` 與 `repoRoot`。比對 SHALL 使用正規化後的路徑前綴，且前綴必須落在路徑分隔邊界上。

#### Scenario: 完全相符的路徑被改寫
- **WHEN** 存在 `D:\old\proj` → `D:\new\proj` 的規則，且某 session 的 `cwd` 為 `D:\old\proj`
- **THEN** 該 session 回傳給前端的 `cwd` 為 `D:\new\proj`

#### Scenario: 子路徑一併被改寫
- **WHEN** 存在 `D:\old\proj` → `D:\new\proj` 的規則，且某 session 的 `cwd` 為 `D:\old\proj\src\app`
- **THEN** 該 session 的 `cwd` 被改寫為 `D:\new\proj\src\app`

#### Scenario: 僅字首相符但非目錄邊界不得改寫
- **WHEN** 存在 `D:\old\proj` → `D:\new\proj` 的規則，且某 session 的 `cwd` 為 `D:\old\project2`
- **THEN** 該 session 的 `cwd` 維持不變

#### Scenario: 改寫後才解析 git 中繼資料
- **WHEN** 某 session 的原始 `cwd` 已不存在但經規則改寫後指向存在的 git 儲存庫
- **THEN** 系統以改寫後的路徑解析 git 分支與 repo 名稱並成功取得結果

#### Scenario: 無相符規則時保持原樣
- **WHEN** 某 session 的 `cwd` 不符合任何規則
- **THEN** 該 session 的 `cwd` 與 `repoRoot` 維持原值

### Requirement: 規則變更後重新載入 session
系統 SHALL 在新增、編輯或刪除對應規則後，使 session 列表重新取得並重新解析 git 中繼資料，讓改寫結果立即反映於畫面。

#### Scenario: 新增規則後畫面立即更新
- **WHEN** 使用者為某失效專案新增對應規則
- **THEN** 該專案的路徑標籤更新為新路徑，且分組依新路徑重新計算

### Requirement: 失效專案路徑偵測與引導
系統 SHALL 判斷專案群組的路徑在檔案系統上是否存在；當路徑不存在時，於專案畫面顯示提示並提供「重新指定資料夾位置」動作，讓使用者選擇新目錄後建立對應規則。

#### Scenario: 路徑不存在時顯示提示
- **WHEN** 使用者開啟某專案，而該專案路徑在檔案系統上已不存在
- **THEN** 專案畫面顯示路徑失效提示與「重新指定資料夾位置」動作

#### Scenario: 透過引導建立規則
- **WHEN** 使用者在失效提示中選擇一個存在的新資料夾
- **THEN** 系統以該專案原路徑為舊路徑、選定資料夾為新路徑建立對應規則，並重新載入 session

#### Scenario: 路徑存在時不顯示提示
- **WHEN** 使用者開啟某專案，而該專案路徑存在
- **THEN** 專案畫面不顯示路徑失效提示

### Requirement: 對應規則管理介面
設定頁 SHALL 提供對應規則清單，顯示每筆規則的舊路徑與新路徑，並支援新增、編輯與刪除。建立或編輯規則時，系統 SHALL 檢查新路徑對應的目錄存在，不存在則拒絕儲存並顯示錯誤。

#### Scenario: 檢視現有規則
- **WHEN** 使用者開啟設定頁的路徑對應區塊
- **THEN** 系統列出所有已建立的規則及其舊路徑與新路徑

#### Scenario: 新路徑不存在時拒絕儲存
- **WHEN** 使用者輸入的新路徑在檔案系統上不存在並嘗試儲存
- **THEN** 系統拒絕儲存並顯示錯誤訊息

#### Scenario: 刪除規則後還原原始路徑
- **WHEN** 使用者刪除某筆對應規則
- **THEN** 受該規則影響的 session 於重新載入後顯示其原始路徑
