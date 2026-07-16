import { useEffect, useRef, useState, type ReactNode } from "react";
import { createPortal } from "react-dom";

type DropdownMenuProps = {
  trigger: (props: { ref: React.RefObject<HTMLButtonElement | null>; onClick: () => void; open: boolean }) => ReactNode;
  children: ReactNode | ((props: { close: () => void }) => ReactNode);
  className?: string;
};

const MENU_MARGIN = 8;

/**
 * 共用下拉選單：以觸發元素為錨點用 `position: fixed` 透過 portal 渲染，
 * 並在計算位置時依視窗邊界自動夾擠，避免選單溢出可視區域造成跑版。
 */
export function DropdownMenu({ trigger, children, className }: DropdownMenuProps) {
  const [open, setOpen] = useState(false);
  const btnRef = useRef<HTMLButtonElement>(null);
  const menuRef = useRef<HTMLDivElement>(null);
  const [pos, setPos] = useState<{ top: number; left: number } | null>(null);

  useEffect(() => {
    if (!open || !btnRef.current) {
      setPos(null);
      return;
    }
    const rect = btnRef.current.getBoundingClientRect();
    const menuEl = menuRef.current;
    const menuWidth = menuEl?.offsetWidth ?? 180;
    const menuHeight = menuEl?.offsetHeight ?? 0;

    let left = rect.left;
    let top = rect.bottom + 2;

    if (left + menuWidth > window.innerWidth - MENU_MARGIN) {
      left = Math.max(MENU_MARGIN, window.innerWidth - menuWidth - MENU_MARGIN);
    }
    if (menuHeight && top + menuHeight > window.innerHeight - MENU_MARGIN) {
      top = Math.max(MENU_MARGIN, rect.top - menuHeight - 2);
    }

    setPos({ top, left });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open, menuRef.current?.offsetHeight, menuRef.current?.offsetWidth]);

  useEffect(() => {
    if (!open) return;
    const handler = (e: MouseEvent) => {
      if (!(e.target as Element).closest("[data-dropdown-menu]") && !(e.target as Element).closest("[data-dropdown-trigger]")) {
        setOpen(false);
      }
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, [open]);

  useEffect(() => {
    if (!open) return;
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key !== "Escape") return;
      event.preventDefault();
      setOpen(false);
      btnRef.current?.focus();
    };
    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [open]);

  useEffect(() => {
    if (!open) return;
    const close = () => setOpen(false);
    document.addEventListener("scroll", close, true);
    window.addEventListener("resize", close);
    return () => {
      document.removeEventListener("scroll", close, true);
      window.removeEventListener("resize", close);
    };
  }, [open]);

  return (
    <>
      <span data-dropdown-trigger="true">
        {trigger({ ref: btnRef, onClick: () => setOpen((v) => !v), open })}
      </span>
      {open ? createPortal(
        <div
          ref={menuRef}
          data-dropdown-menu="true"
          className={`dropdown-menu${className ? ` ${className}` : ""}`}
          role="menu"
          style={{
            position: "fixed",
            top: pos?.top ?? -9999,
            left: pos?.left ?? -9999,
            visibility: pos ? "visible" : "hidden",
            zIndex: 9999,
          }}
        >
          {typeof children === "function" ? children({ close: () => setOpen(false) }) : children}
        </div>,
        document.body
      ) : null}
    </>
  );
}
