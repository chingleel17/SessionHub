## ADDED Requirements

### Requirement: 統一操作與導覽圖示來源

系統 SHALL 透過單一圖示出口提供所有操作與導覽圖示，使用同一套 outline icon family、`currentColor`、一致的 line cap 與 line join；元件不得為一般操作自行新增 inline SVG、Emoji 或文字符號圖示。

#### Scenario: 畫面渲染操作圖示
- **WHEN** 任一畫面渲染開啟、儲存、刪除、搜尋、切換、導覽等一般操作
- **THEN** 圖示來自集中管理的統一圖示出口

#### Scenario: 新增操作圖示
- **WHEN** 開發者新增一般操作或導覽圖示
- **THEN** 開發者使用既有圖示出口或在該出口新增對應映射
- **AND** 不得在業務元件內直接定義該 SVG path

### Requirement: 圖示尺寸與專用 SVG 邊界

一般操作圖示 SHALL 預設為 16px，Sidebar/主要導覽得使用 18px，主要動作得使用 20px；資料視覺化、quota ring、品牌 logo 與 provider 品牌資產得保留專用 SVG。

#### Scenario: 資料視覺化渲染
- **WHEN** 系統渲染 chart、progress ring 或品牌標誌
- **THEN** 元件可使用其專用 SVG
- **AND** 不受一般操作圖示來源限制

#### Scenario: Provider 額度頁顯示供應商識別
- **WHEN** provider quota 或 session UI 需要顯示 provider 識別
- **THEN** 顯示統一的 ProviderIcon 或文字 badge
- **AND** 不得以 Emoji 作為唯一識別方式
