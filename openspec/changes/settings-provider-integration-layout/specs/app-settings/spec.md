## ADDED Requirements

### Requirement: 設定頁在寬畫面展開 provider integration 管理區塊
系統 SHALL 在設定頁的桌面寬畫面中，將 provider integration 管理區塊配置於可有效利用主內容寬度的獨立版面區域，而非與一般設定欄位共同擠在單一窄卡片中。

#### Scenario: 桌面寬畫面顯示 provider integration
- **WHEN** 使用者在桌面寬畫面開啟設定頁
- **THEN** 系統將 provider integration 顯示在獨立卡片或等效寬版區塊中
- **AND** 該區塊寬度 SHALL 明顯大於一般設定欄位區塊

#### Scenario: provider integration 包含多筆資訊與操作
- **WHEN** provider integration 卡片顯示狀態、操作按鈕、設定檔路徑、bridge 路徑與最後事件時間
- **THEN** 系統 SHALL 以分區排版呈現這些資訊
- **AND** 長路徑與操作按鈕不應因單一卡片欄寬過窄而長期處於擁擠換行狀態

### Requirement: 設定頁 provider integration 版面需具備響應式回退
系統 SHALL 在較窄視窗或空間不足時，讓 provider integration 管理區塊回退為可閱讀的堆疊式布局，但仍需保留完整的狀態、路徑與操作能力。

#### Scenario: 視窗寬度不足
- **WHEN** 設定頁可用寬度不足以容納寬版排版
- **THEN** 系統將 provider integration 內容回退為堆疊式布局
- **AND** 使用者仍可看到並操作所有 provider integration 功能
