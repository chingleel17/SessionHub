# SessionHub 元件與版面範式

實際可套用的寫法。所有值引用 token（見 `tokens.md`）。這些範式都已在 Agents / Project 頁落地，複製時保持一致。

## 目錄
- [去卡片容器](#去卡片容器)
- [Row / List（去卡片列表）](#row--list去卡片列表)
- [Tab（Linear 式）](#tablinear-式)
- [Button 三階層](#button-三階層)
- [收折分區 + 標題行整合](#收折分區--標題行整合)
- [Modal / 浮層（glass）](#modal--浮層glass)
- [長內容 Modal（滾動不溢出圓角）](#長內容-modal滾動不溢出圓角)
- [Scrollbar](#scrollbar)

## 去卡片容器

把「卡片」改為透明容器，靠留白 + hairline 分界：

```css
/* Before（卡片）：border + radius + shadow + 背景色塊 */
.some-panel {
  border: 1px solid var(--color-border-subtle);
  border-radius: 22px;
  background: rgba(255, 255, 255, 0.9);
  box-shadow: 0 18px 40px rgba(18, 30, 56, 0.06);
}

/* After（去卡片）：透明，需要分界時只留一條 hairline */
.some-panel {
  border: none;
  border-radius: 0;
  background: transparent;
  box-shadow: none;
}
```

若原本靠 `[data-theme="dark"]` 覆寫深色底色，去卡片後**一併刪掉該覆寫**（透明不需要深色覆寫）。

## Row / List（去卡片列表）

不要每列一張卡。改 row + hover + hairline：

```css
.xxx-list {
  display: flex;
  flex-direction: column;
  border: none;
  border-top: 1px solid var(--color-border-subtle);
  border-radius: 0;
  background: transparent;
}
.xxx-list-item {
  padding: 10px 14px;
  border: none;
  border-bottom: 1px solid var(--color-border-subtle);
  background: transparent;
  transition: background var(--motion-fast) var(--motion-ease);
}
.xxx-list-item:hover {
  background: var(--color-action-primary-subtle-bg);
}
```

## Tab（Linear 式）

active = 底部 accent line；inactive = 無背景、次要文字色、hover 轉深。**不要**帶框/帶背景塊的 Bootstrap tab。

```css
.sub-tab-item {
  min-height: 36px;
  padding: 0 14px;
  border: none;
  border-bottom: 2px solid transparent;
  border-radius: 0;
  background: transparent;
  color: var(--color-text-secondary);
  transition:
    color var(--motion-fast) var(--motion-ease),
    border-color var(--motion-fast) var(--motion-ease);
}
.sub-tab-item:hover { color: var(--color-text-primary); }
.sub-tab-item--active {
  color: var(--color-text-primary);
  border-bottom-color: var(--color-action-primary);
}
```

搜尋框等與 tab 同列的控制項，高度對齊 tab（`height: 36px`），並覆寫全域 `input { min-height: 42px }`：

```css
.xxx-search-field { height: 36px; padding: 0 12px; border-radius: var(--radius-button); }
.xxx-search-field input { min-height: 0; height: 100%; padding: 0; border: none; background: transparent; }
```

## Button 三階層

```css
/* Primary：filled 藍，無框 */
.primary-button {
  height: 40px; padding: 0 18px;
  border: 1px solid transparent;
  border-radius: var(--radius-button);
  background: var(--color-action-primary);
  color: var(--color-action-primary-text);
}
/* Secondary：ghost，透明底 hover 淡背景 */
.ghost-button {
  border: 1px solid var(--color-border-subtle);
  background: var(--color-surface-panel);
  color: var(--color-text-primary);
  border-radius: var(--radius-button);
}
/* Tertiary：純文字 */
.inline-link { border: 0; background: transparent; color: var(--color-action-primary); }
```

盡量減少 outlined button。icon-only 用 `.icon-button`（見 App.css）。

## 收折分區 + 標題行整合

`CollapsibleSection`（`src/components/CollapsibleSection.tsx`）支援 `titleMeta`（標題旁小字）與 `actions`（標題列操作按鈕）：

```tsx
<CollapsibleSection
  title={`${label} (${count})`}
  expanded={expanded}
  onToggle={toggle}
  titleMeta={configPath}          // 標題右側小字（可截斷）
  actions={<HeaderActions … />}   // 標題列最右操作按鈕（不參與收折點擊）
>
  {body}
</CollapsibleSection>
```

**單一 scope（無收折）情境**：不要退回卡片工具列，改內容頂部一列內嵌操作列：

```tsx
<div className="xxx-inline-header">
  {path ? <span className="xxx-inline-header-path">{path}</span> : null}
  <HeaderActions … />
</div>
```

```css
.xxx-inline-header { display: flex; align-items: center; gap: 12px; padding: 2px 2px 6px; }
.xxx-inline-header-path {
  flex: 1; min-width: 0;
  overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
  color: var(--color-text-secondary); font-size: 12px;
}
```

收折標題行的主操作（如「新增」）在收折狀態被點時，先展開再執行（用一個遞增 signal 傳給內容元件觸發）。

**縮排**：收折標題與展開內容左緣要對齊——`.collapsible-section-body` 左右 padding 歸零、header 貼齊左緣。

## Modal / 浮層（glass）

modal / dropdown / popover / context menu：

```css
.dialog-card {
  border: 1px solid var(--color-border-subtle);
  border-radius: var(--radius-modal);        /* dropdown 用 --radius-input */
  background: var(--glass-bg);
  backdrop-filter: blur(var(--glass-blur));
  -webkit-backdrop-filter: blur(var(--glass-blur));
  box-shadow: var(--shadow-float);
}
```

## 長內容 Modal（滾動不溢出圓角）

若 modal 內含可捲動的長內容（如檔案預覽），別讓 modal 本身 `overflow-y:auto`——scrollbar 會溢出圓角外緣。改成固定高度 + 內容區內部滾動，並在 header 放關閉鈕：

```css
.preview-modal {
  height: min(88vh, 880px);
  display: flex;
  flex-direction: column;
  overflow: hidden;           /* modal 不滾動 */
}
.preview-modal .modal-header { flex-shrink: 0; border-radius: var(--radius-modal) var(--radius-modal) 0 0; }
.preview-modal .modal-body { flex: 1; min-height: 0; overflow-y: auto; }  /* 只有這裡滾動 */
```

header 右側務必有關閉（×）按鈕，hover 可轉 `--color-status-error` 示意。

## Scrollbar

全站已有樣式（`src/App.css` 底部），新捲動區自動繼承。若需在特定容器強化，仍用 token：

```css
*::-webkit-scrollbar { width: 10px; height: 10px; }
*::-webkit-scrollbar-thumb {
  background: var(--color-scrollbar-thumb);
  border-radius: 999px;
  border: 3px solid transparent;
  background-clip: padding-box;
}
*::-webkit-scrollbar-thumb:hover { background: var(--color-scrollbar-thumb-hover); background-clip: padding-box; }
```

不要把捲軸覆寫回系統預設的粗灰條。
