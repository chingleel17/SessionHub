import { useCallback, useEffect, useState } from "react";
import { useI18n } from "../i18n/I18nProvider";
import { ContextMenu } from "./ContextMenu";

/** 可編輯元素保留原生右鍵選單，才能使用貼上等編輯功能 */
const NATIVE_MENU_SELECTOR = "input, textarea, select, [contenteditable]:not([contenteditable='false'])";

/**
 * 全域右鍵選單：取代 WebView 預設選單。
 * 其他元件已自行處理（呼叫 preventDefault）的右鍵事件不會再顯示此選單。
 */
export function GlobalContextMenu() {
  const { t } = useI18n();
  const [position, setPosition] = useState<{ x: number; y: number } | null>(null);
  const close = useCallback(() => setPosition(null), []);

  useEffect(() => {
    const handleContextMenu = (event: MouseEvent) => {
      if (event.defaultPrevented) return;
      if (event.target instanceof Element && event.target.closest(NATIVE_MENU_SELECTOR)) return;
      event.preventDefault();
      setPosition({ x: event.clientX, y: event.clientY });
    };
    document.addEventListener("contextmenu", handleContextMenu);
    return () => document.removeEventListener("contextmenu", handleContextMenu);
  }, []);

  if (!position) return null;

  return (
    <ContextMenu x={position.x} y={position.y} onClose={close}>
      <button
        type="button"
        role="menuitem"
        className="dropdown-menu-item"
        onClick={() => {
          close();
          window.location.reload();
        }}
      >
        {t("app.actions.refresh")}
      </button>
    </ContextMenu>
  );
}
