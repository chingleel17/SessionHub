# SessionHub Design System — 使用約定

## 必要包裹（Provider）

所有元件都要包在 `I18nProvider` 內（多數元件呼叫 `useI18n()`，缺了會直接 throw）；主題切換需要 `ThemeProvider`（透過 `data-theme` 屬性驅動 dark mode）：

```jsx
import { I18nProvider, ThemeProvider, SessionCard } from "session-hub";

<I18nProvider>
  <ThemeProvider>
    <SessionCard session={session} /* … */ />
  </ThemeProvider>
</I18nProvider>
```

語言由 `useI18n().setLocale("zh-TW" | "en-US")` 切換；主題由 `useTheme().setTheme("light" | "dark")` 切換（dark 樣式掛在 `:root[data-theme="dark"]`）。

## 樣式慣用法：CSS 類別 + design token（無 utility framework）

這套 DS 用**普通 CSS class + CSS 自訂屬性**，沒有 Tailwind、沒有 styled-props。自己的版面膠水請直接用 token 寫 inline style 或既有 class，**不要發明新的視覺常數**。

核心 token（完整定義見 `styles.css` 的 import closure，即 `_ds_bundle.css` 開頭的 theme 區塊）：

- 表面：`--color-surface-app`（app 底）、`--color-surface-panel`（面板白）、`--color-surface-subtle`
- 文字：`--color-text-primary`、`--color-text-secondary`
- 邊框：`--color-border-subtle`（極淡，介面像一整塊畫布）
- 主色：`--color-action-primary`（#2563eb 藍）
- 圓角系統（不得混用其他值）：`--radius-button: 10px`、`--radius-input: 12px`、`--radius-card: 16px`、`--radius-modal: 20px`
- 陰影：`--shadow-panel`（輕）、`--shadow-float`（浮層）
- 動畫：`--motion-fast/base/slow` + `--motion-ease`（150–250ms ease-out）
- 字型：`--font-family-sans`（系統字型棧，刻意不 ship 字型檔）

## 可重用 class 詞彙（直接可用）

- 按鈕：`primary-button`、`ghost-button`（modifier：`--active`、`--danger`）、`icon-button`（modifier：`--danger`）
- Chips：`session-chip`、`session-chip-button`、`session-chip-row`、`muted-chip`、`error-chip`、`tag-chip`
- Provider 品牌標籤：`provider-tag provider-tag--claude|copilot|opencode|codex|antigravity`
- 活動狀態徽章：`activity-badge activity-badge--active|idle|waiting|done`
- 對話框：`dialog-backdrop` > `dialog-card` > `dialog-form`/`dialog-actions`（圓角用 `--radius-modal`）
- 下拉選單：`dropdown-menu` > `dropdown-menu-item`

## 設計語言基調

Linear/Vercel 式極簡：冷灰中性色、去卡片化（少框少影）、極淡邊框、單色連續畫布背景。收折分區用 `CollapsibleSection` 元件而非自製。列表 hover 用柔和背景（`--color-surface-subtle`），不用粗框陰影。

## 真相來源

- 樣式全文：`styles.css` → `_ds_bundle.css`（含兩個 theme 的全部 token 與元件 CSS）
- 每個元件的 API：`components/general/<Name>/<Name>.d.ts`；用法示例：`<Name>.prompt.md`
- 資料形狀（SessionInfo、QuotaSnapshot、TreeNode 等）：見各元件 `.d.ts` 的 props 型別

## 一個道地的組合範例

```jsx
<div style={{ background: "var(--color-surface-panel)", borderRadius: "var(--radius-card)", border: "1px solid var(--color-border-subtle)", padding: 16 }}>
  <div className="session-chip-row">
    <span className="provider-tag provider-tag--claude">Claude Code</span>
    <span className="activity-badge activity-badge--active">working</span>
  </div>
  <CollapsibleSection title="Sessions" expanded onToggle={() => {}}>
    <SessionCard session={session} stats={stats} statsLoading={false} todos={[]} todosLoading={false} /* callbacks */ />
  </CollapsibleSection>
</div>
```
