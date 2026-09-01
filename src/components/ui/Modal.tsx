import type { ReactNode } from "react";

type ModalProps = {
  children: ReactNode;
  panelClassName?: string;
  ariaLabel?: string;
};

/** 共用內容密集型 Modal 外殼，提供一致遮罩、語意與實心面板。 */
export function Modal({ children, panelClassName, ariaLabel }: ModalProps) {
  return (
    <div className="dialog-backdrop modal-backdrop">
      <article
        className={`dialog-card dialog-card--solid modal-panel${panelClassName ? ` ${panelClassName}` : ""}`}
        role="dialog"
        aria-modal="true"
        aria-label={ariaLabel}
      >
        {children}
      </article>
    </div>
  );
}
