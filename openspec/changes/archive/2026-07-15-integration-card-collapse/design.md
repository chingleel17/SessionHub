# Design: Integration Card Collapse

## Context

「平台整合管理」位於 `SettingsView.tsx`（`settings.integrations.title` 區塊），以 `provider-integration-card` 逐一渲染 `settingsForm.providerIntegrations`。每張卡片包含：

- header：badges（`provider-tag`、狀態 `session-chip`、`provider-version-badge`）＋ 操作按鈕（主要動作、重新檢查、開啟、編輯、解除安裝）
- grid：設定/plugin 路徑、Bridge 路徑（各自為 `<details>` 收折）、最後事件時間、整合版本
- `lastError` 錯誤區塊（存在時卡片加 `provider-integration-card--error`）

SettingsView 為純顯示元件、props 驅動；IPC 集中於 App.tsx。專案樣式使用 plain BEM-like class、遵循 sessionhub-minimal-ui 設計 token。

## Goals / Non-Goals

**Goals:**
- 卡片預設收起（有 lastError 者預設展開），收起時單列摘要：平台名 badge、狀態 badge、版本 badge、最後事件時間。
- 點擊標題列切換收折；展開時呈現現有完整內容。
- 錯誤卡片被手動收起時仍能辨識錯誤狀態。

**Non-Goals:**
- 不持久化收折狀態（重新整理／重開 App 回到預設收起）。
- 不改動後端、AppSettings、ProviderIntegrationStatus 型別。
- 不提供「全部展開／全部收起」批次操作（YAGNI，之後有需要再加）。

## Decisions

### D1: 收折 state 放在 SettingsView 內部（`useState<Record<string, boolean>>`）

以 `expandedProviders` record（key = provider id，預設空物件）管理。單張卡片的展開判定為 `expandedProviders[provider] ?? Boolean(integration.lastError)` — 未被手動切換過時，一般卡片收起、錯誤卡片展開；使用者手動切換後以 record 中的值為準。不上提到 App.tsx，因為這是純視圖狀態，與 IPC 無關；也不寫入 settings（Non-Goal）。

替代方案：每張卡片用原生 `<details>` — 但 header 內已有多個 button（`<summary>` 內嵌互動元素的事件行為與樣式控制較差），且需要 chevron 旋轉、動畫等自訂樣式，故採受控 state。

### D2: 收起摘要重用現有 header 列，附加摘要資訊

收起時不另造一個「summary card」，而是同一個 header 列：

- 左側：現有 badges（provider-tag、狀態 chip、版本 badge）＋ 最後事件時間（`provider-integration-summary-time`，muted 文字，格式沿用 `formatDateTime`；無事件顯示 `settings.integrations.values.noEvent`）。
- 右側：僅保留 chevron 展開指示（收起時隱藏操作按鈕，避免誤觸與視覺噪音；展開後操作按鈕如現狀顯示）。

版本 badge 已存在（`provider-version-badge`），收起與展開共用，符合「(版本)」需求。

替代方案：收起時仍顯示全部操作按鈕 — 與截圖需求「僅顯示平台名/狀態/版本/最後事件時間」不符，捨棄。

### D3: 點擊互動與可及性

- header 列本身可點擊切換（`onClick` + `role="button"` 或直接用 `<button>` 包裹左側區域）；採用整列 `div` + `onClick`，並於內部互動元素（操作按鈕）`stopPropagation`。
- 加 `aria-expanded` 與 `title`（新增 locale key：`settings.integrations.actions.expand` / `collapse`）。
- chevron 用 inline SVG，展開時 `transform: rotate(90deg)`，過渡動畫遵循設計 token。

### D4: 錯誤狀態呈現

有 `lastError` 的卡片**預設展開**，直接呈現錯誤訊息與操作按鈕，便於立即處理；使用者仍可手動收起。`provider-integration-card--error` class 保留在卡片根節點，被收起時邊框/底色提示仍可見。

## Risks / Trade-offs

- [收起時隱藏「重新檢查／安裝」按鈕，多一次點擊才能操作] → 摘要列資訊已能回答多數查看需求；需操作時點一下展開，可接受。
- [header 整列 onClick 與內部按鈕事件衝突] → 內部按鈕容器 `stopPropagation`；操作按鈕僅在展開時渲染，衝突面縮小。
- [`<details>` 路徑收折巢狀在卡片收折內，層級稍多] → 展開後行為與現狀相同，不另改。
