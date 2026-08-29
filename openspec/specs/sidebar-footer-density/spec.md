## Purpose

讓 Sidebar 底部的次要導覽、版本與即時狀態資訊以更緊湊的版面呈現，在不犧牲可讀性與操作能力的前提下釋放更多垂直空間給主要專案導覽。

## Requirements

### Requirement: Sidebar footer 使用緊湊垂直版面

Sidebar footer SHALL 壓縮 Agents、設定、版本號、即時狀態與刷新操作之間的列高、內距、區塊間距及底部留白，使 footer 整體垂直占用約為變更前基準的一半，且不得移除既有資訊或操作。

#### Scenario: 展開狀態顯示緊湊 footer

- **WHEN** Sidebar 處於展開狀態
- **THEN** Agents 與設定導覽列 SHALL 使用較緊湊但可辨識的列高與間距
- **AND** 版本號、即時更新狀態、時間與刷新操作 SHALL 保持可見
- **AND** footer 下方不得保留明顯多餘的空白區塊

#### Scenario: 收折狀態顯示緊湊 footer

- **WHEN** Sidebar 處於收折狀態
- **THEN** footer SHALL 維持與展開狀態一致的緊湊垂直節奏
- **AND** 既有收折狀態要求保留的 icon、版本資訊與即時狀態點不得被裁切或遮蔽

#### Scenario: Footer 功能保持不變

- **WHEN** 使用者在緊湊 footer 點擊 Agents、設定或刷新操作
- **THEN** 系統 SHALL 執行與變更前相同的導覽或刷新行為
- **AND** 即時狀態文字與最後同步時間 SHALL 繼續反映目前狀態

### Requirement: 緊湊 footer 維持可讀與可操作

Footer 密度調整 SHALL 保持文字與 icon 可辨識、滑鼠點擊目標可可靠操作、鍵盤焦點清楚可見，並在淺色、深色及窄視窗版面中維持內容邊界。

#### Scenario: 鍵盤操作 footer

- **WHEN** 使用者以鍵盤依序聚焦 Agents、設定與刷新操作
- **THEN** 每個可互動項目 SHALL 顯示清楚的 focus-visible 狀態
- **AND** 壓縮後的控制項不得重疊或使焦點外框遭到裁切

#### Scenario: 長即時狀態內容

- **WHEN** 即時狀態與同步時間超過 footer 可用寬度
- **THEN** 文字 SHALL 以既有截斷行為維持單列緊湊顯示
- **AND** 刷新操作不得被文字擠出 Sidebar 邊界

#### Scenario: 雙主題與窄視窗

- **WHEN** 使用者切換淺色或深色主題，或應用程式進入窄視窗布局
- **THEN** footer 文字、icon、狀態點與互動回饋 SHALL 保持足夠對比
- **AND** footer 不得產生水平溢出、垂直碰撞或內容裁切
