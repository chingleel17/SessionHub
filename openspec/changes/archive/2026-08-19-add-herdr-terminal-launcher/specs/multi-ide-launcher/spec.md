## ADDED Requirements

### Requirement: herdr 可用性偵測

系統 SHALL 在偵測工具可用性時，額外檢查 PATH 中是否存在 `herdr` 可執行檔，並將結果納入可用性資料。

#### Scenario: 偵測 herdr 是否安裝

- **WHEN** 系統執行工具可用性偵測
- **THEN** 回傳的可用性資料 SHALL 包含 `herdr` 布林值，代表其是否存在於 PATH

#### Scenario: 未安裝時的可用性結果

- **WHEN** PATH 中不存在 `herdr` 可執行檔
- **THEN** 可用性資料中的 `herdr` 為 false

### Requirement: herdr 服務狀態偵測

系統 SHALL 區分「herdr 未安裝」與「herdr 已安裝但服務未執行」兩種狀態，因兩者的補救方式不同。

#### Scenario: 已安裝且服務執行中

- **WHEN** `herdr` 存在於 PATH 且查詢服務狀態回報執行中
- **THEN** 系統視 herdr 為可正常使用

#### Scenario: 已安裝但服務未執行

- **WHEN** `herdr` 存在於 PATH 但查詢服務狀態未回報執行中
- **THEN** 系統回報「已安裝但服務未執行」狀態
- **AND** 相關提示訊息指引使用者啟動 herdr，而非重新安裝

#### Scenario: 未安裝

- **WHEN** `herdr` 不存在於 PATH
- **THEN** 系統回報「未偵測到」狀態
- **AND** 相關提示訊息指引使用者安裝 herdr

### Requirement: herdr 可用性快取更新時機

系統 SHALL 讓使用者在安裝或啟動 herdr 後，無需重啟應用程式即可讓設定頁反映最新可用性。

#### Scenario: 重新偵測 herdr

- **WHEN** 使用者於設定頁觸發重新偵測
- **THEN** 系統重新查詢 herdr 可用性與服務狀態
- **AND** 設定頁依最新結果更新 launcher 選項的可用狀態

#### Scenario: 儲存設定後更新可用性

- **WHEN** 使用者儲存設定
- **THEN** 系統使工具可用性快取失效，使後續讀取取得最新結果
