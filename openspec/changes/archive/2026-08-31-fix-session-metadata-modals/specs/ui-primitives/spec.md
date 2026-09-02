## ADDED Requirements

### Requirement: 共用實心 Modal 容器

系統 SHALL 提供可重用的 Modal 容器，統一全畫面深色遮罩、面板邊框、圓角、陰影與內容承載方式。表單或長文字等內容密集的 Modal MUST 使用不透明的主題面板背景，不得透出後方頁面內容。

#### Scenario: 內容密集 Modal 維持可讀性

- **WHEN** 使用者開啟編輯表單或長文字預覽 Modal
- **THEN** 系統以深色遮罩降低背景干擾
- **AND** Modal 面板使用不透明的主題背景，後方頁面內容不會穿透面板

#### Scenario: 不同功能共用一致 Modal 外觀

- **WHEN** 使用者分別開啟 session 中繼資料編輯與 Skills 檢視 Modal
- **THEN** 兩者使用相同的遮罩、面板背景、邊框、圓角及陰影規則
- **AND** 各 Modal 仍可依內容設定自己的尺寸與內部版面

#### Scenario: 深色與淺色主題

- **WHEN** 使用者在任一支援的主題開啟共用 Modal
- **THEN** 遮罩及實心面板 SHALL 使用該主題的設計 token 呈現
- **AND** 標題、說明、輸入內容與操作按鈕保持清楚可辨
