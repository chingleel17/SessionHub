# Spec: integration-card-collapse

## ADDED Requirements

### Requirement: 整合卡片預設收起
「平台整合管理」中每張 provider 整合卡片 SHALL 預設為收起狀態；有 lastError 的卡片 SHALL 預設為展開狀態。收折狀態 SHALL 僅存於前端 UI state，不寫入設定檔。

#### Scenario: 進入設定頁時無錯誤卡片全部收起
- **WHEN** 使用者開啟設定頁且所有 providerIntegrations 皆無 lastError
- **THEN** 所有整合卡片以收起狀態顯示

#### Scenario: 有錯誤的卡片預設展開
- **WHEN** 使用者開啟設定頁且某 provider 的 lastError 有值
- **THEN** 該卡片以展開狀態顯示錯誤訊息，其餘無錯誤卡片維持收起

#### Scenario: 重新載入後回到預設
- **WHEN** 使用者手動切換部分卡片後重新整理或重開應用程式
- **THEN** 所有卡片回到預設狀態（無錯誤收起、有錯誤展開）

### Requirement: 收起摘要列內容
收起狀態的卡片 SHALL 僅顯示一列摘要，內容依序為：平台名 badge（沿用 provider-tag 樣式）、安裝狀態 badge（已安裝／需更新／未安裝／需手動設定／錯誤）、版本 badge（僅在 installedVersion 存在時）、最後事件時間。操作按鈕（安裝、更新、重新檢查、開啟、編輯、解除安裝）SHALL NOT 在收起狀態顯示。

#### Scenario: 已安裝且有事件的卡片收起顯示
- **WHEN** 某 provider 狀態為 installed、installedVersion 為 4、lastEventAt 有值且卡片為收起狀態
- **THEN** 摘要列顯示平台名 badge、「已安裝」badge、「v4」badge 與格式化後的最後事件時間

#### Scenario: 無事件時顯示替代文案
- **WHEN** 某 provider 的 lastEventAt 為空且卡片為收起狀態
- **THEN** 摘要列的時間位置顯示「尚無事件」文案（settings.integrations.values.noEvent）

#### Scenario: 無版本時省略版本 badge
- **WHEN** 某 provider 的 installedVersion 為 null 且卡片為收起狀態
- **THEN** 摘要列不顯示版本 badge，其餘項目正常顯示

### Requirement: 點擊標題列切換收折
使用者 SHALL 能以點擊卡片標題列切換該卡片的收起／展開狀態；展開後 SHALL 顯示現有完整內容（操作按鈕、設定/plugin 路徑、Bridge 路徑、最後事件時間、整合版本、錯誤訊息）。標題列 SHALL 帶有 aria-expanded 屬性反映目前狀態。

#### Scenario: 展開卡片
- **WHEN** 使用者點擊收起卡片的標題列
- **THEN** 該卡片展開並顯示完整內容，aria-expanded 變為 true

#### Scenario: 收起卡片
- **WHEN** 使用者點擊已展開卡片的標題列
- **THEN** 該卡片收起為摘要列，aria-expanded 變為 false

#### Scenario: 操作按鈕不觸發收折
- **WHEN** 使用者點擊展開卡片內的操作按鈕（如重新檢查）
- **THEN** 執行該操作且卡片維持展開狀態

#### Scenario: 各卡片獨立收折
- **WHEN** 使用者展開其中一張卡片
- **THEN** 其他卡片維持原本的收折狀態不受影響

### Requirement: 收起時保留錯誤視覺提示
當 provider 整合有 lastError 時，卡片被使用者手動收起後 SHALL 仍套用錯誤視覺樣式（provider-integration-card--error），使使用者不展開也能辨識異常。

#### Scenario: 錯誤卡片手動收起後可辨識
- **WHEN** 某 provider 的 lastError 有值且使用者手動將該卡片收起
- **THEN** 卡片外觀呈現錯誤樣式；重新展開後可見完整錯誤訊息
