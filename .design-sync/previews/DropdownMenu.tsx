import { useEffect, useRef } from "react";
import { DropdownMenu } from "session-hub";

const noop = () => {};

/** 自動展開的下拉選單：掛載後模擬點擊 trigger，讓截圖能呈現開啟狀態。 */
function AutoOpenMenu() {
  const apiRef = useRef(null);

  useEffect(() => {
    if (apiRef.current && !apiRef.current.open) {
      apiRef.current.onClick();
    }
  }, []);

  return (
    <div style={{ padding: "16px" }}>
      <DropdownMenu
        trigger={({ ref, onClick, open }) => {
          apiRef.current = { onClick, open };
          return (
            <button ref={ref} type="button" className="icon-button" title="選擇開啟工具" onClick={onClick}>
              ⋯
            </button>
          );
        }}
      >
        {({ close }) => (
          <>
            <button type="button" className="dropdown-menu-item dropdown-menu-item--default" onClick={close}>
              <span className="launcher-option-icon">▸</span>
              Claude Code
            </button>
            <button type="button" className="dropdown-menu-item" onClick={close}>
              <span className="launcher-option-icon">▸</span>
              VS Code
            </button>
            <button type="button" className="dropdown-menu-item" onClick={close}>
              <span className="launcher-option-icon">▸</span>
              檔案總管
            </button>
            <button type="button" className="dropdown-menu-item" disabled onClick={close}>
              <span className="launcher-option-icon">▸</span>
              OpenCode（未安裝）
            </button>
          </>
        )}
      </DropdownMenu>
    </div>
  );
}

export const OpenLauncherMenu = () => <AutoOpenMenu />;

export const ClosedTrigger = () => (
  <div style={{ padding: "16px" }}>
    <DropdownMenu
      trigger={({ ref, onClick }) => (
        <button ref={ref} type="button" className="icon-button" title="選擇開啟工具" onClick={onClick}>
          ⋯
        </button>
      )}
    >
      <button type="button" className="dropdown-menu-item" onClick={noop}>
        Claude Code
      </button>
    </DropdownMenu>
  </div>
);
