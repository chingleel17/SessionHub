import { useEffect, useLayoutEffect, useRef, useState, type ReactNode } from "react";
import { createPortal } from "react-dom";

type ContextMenuProps = {
  x: number;
  y: number;
  onClose: () => void;
  children: ReactNode;
};

const MENU_MARGIN = 8;

/**
 * 依滑鼠座標定位的右鍵選單：透過 portal 以 `position: fixed` 渲染，
 * 並依視窗邊界夾擠位置；點擊外部、Esc、捲動或視窗縮放時關閉。
 */
export function ContextMenu({ x, y, onClose, children }: ContextMenuProps) {
  const menuRef = useRef<HTMLDivElement>(null);
  const [pos, setPos] = useState<{ top: number; left: number } | null>(null);

  useLayoutEffect(() => {
    const menuEl = menuRef.current;
    const menuWidth = menuEl?.offsetWidth ?? 180;
    const menuHeight = menuEl?.offsetHeight ?? 0;
    setPos({
      left: Math.max(MENU_MARGIN, Math.min(x, window.innerWidth - menuWidth - MENU_MARGIN)),
      top: Math.max(MENU_MARGIN, Math.min(y, window.innerHeight - menuHeight - MENU_MARGIN)),
    });
  }, [x, y]);

  useEffect(() => {
    const handleMouseDown = (event: MouseEvent) => {
      if (!menuRef.current?.contains(event.target as Node)) {
        onClose();
      }
    };
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key !== "Escape") return;
      event.preventDefault();
      onClose();
    };
    document.addEventListener("mousedown", handleMouseDown);
    document.addEventListener("keydown", handleKeyDown);
    document.addEventListener("scroll", onClose, true);
    window.addEventListener("resize", onClose);
    window.addEventListener("blur", onClose);
    return () => {
      document.removeEventListener("mousedown", handleMouseDown);
      document.removeEventListener("keydown", handleKeyDown);
      document.removeEventListener("scroll", onClose, true);
      window.removeEventListener("resize", onClose);
      window.removeEventListener("blur", onClose);
    };
  }, [onClose]);

  return createPortal(
    <div
      ref={menuRef}
      className="dropdown-menu context-menu"
      role="menu"
      onContextMenu={(event) => {
        // portal 內的 React 事件仍會冒泡到元件樹上層，需阻止以免重新開啟選單
        event.preventDefault();
        event.stopPropagation();
      }}
      style={{
        position: "fixed",
        top: pos?.top ?? -9999,
        left: pos?.left ?? -9999,
        visibility: pos ? "visible" : "hidden",
        zIndex: 9999,
      }}
    >
      {children}
    </div>,
    document.body,
  );
}
