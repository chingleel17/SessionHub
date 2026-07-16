## ADDED Requirements

### Requirement: 側欄收折展開平滑過渡

側欄收折與展開 SHALL 以平滑動畫過渡（寬度變化約 150–250ms），文字內容以淡出/淡入方式隱藏與顯示，不得瞬間跳變；在 `prefers-reduced-motion` 環境下 SHALL 停用動畫並立即切換。

#### Scenario: 使用者收折側欄

- **WHEN** 使用者點擊收折按鈕
- **THEN** 側欄寬度以平滑動畫縮小至收折寬度
- **AND** 導覽文字與面板內容淡出，不產生內容擠壓變形

#### Scenario: 使用者展開側欄

- **WHEN** 使用者於收折狀態點擊展開按鈕
- **THEN** 側欄寬度以平滑動畫恢復，文字內容淡入

#### Scenario: 減少動態偏好

- **WHEN** 作業系統啟用 reduce motion
- **THEN** 收折/展開立即切換，不播放過渡動畫

### Requirement: 收折按鈕位置固定

收折/展開按鈕 SHALL 在展開與收折兩種狀態下維持相同的固定位置；展開狀態下不得移動至品牌 icon 旁。

#### Scenario: 切換收折狀態時按鈕不位移

- **WHEN** 使用者連續切換收折與展開
- **THEN** 收折按鈕的螢幕位置維持不變，使用者無需移動滑鼠即可連續點擊

### Requirement: 收折狀態 icon 對齊一致

收折狀態下，品牌 app icon 與導覽項目 icon SHALL 使用相同的水平置中基準；收折與展開切換時，icon SHALL 不產生可見的水平跳位。

#### Scenario: 收折後 icon 對齊

- **WHEN** 側欄處於收折狀態
- **THEN** 品牌 icon 與所有導覽 icon 在同一垂直軸線上置中對齊

#### Scenario: 切換時 icon 不跳位

- **WHEN** 使用者收折或展開側欄
- **THEN** 導覽 icon 的水平位置平滑過渡或維持不變，無瞬間跳動
