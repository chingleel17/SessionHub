import type { ReactNode } from "react";
import { ChevronLeftIcon } from "./Icons";

type Props = {
  title: string;
  expanded: boolean;
  onToggle: () => void;
  children: ReactNode;
  /** 標題右側的小字 meta（如路徑），與標題同列，維持小字。 */
  titleMeta?: ReactNode;
  /** 標題列最右側的操作按鈕群（不參與收折點擊）。 */
  actions?: ReactNode;
};

/** 可收折分區：標題列 + chevron，內容僅於展開時渲染（收折時不掛載、狀態不保留）。
 *  標題列右側可選放置 titleMeta（小字）與 actions（操作按鈕）。 */
export function CollapsibleSection({ title, expanded, onToggle, children, titleMeta, actions }: Props) {
  return (
    <section className={`collapsible-section ${expanded ? "collapsible-section--expanded" : ""}`}>
      <div className="collapsible-section-header">
        <button type="button" className="collapsible-section-toggle" onClick={onToggle} aria-expanded={expanded}>
          <span className={`collapsible-section-chevron ${expanded ? "collapsible-section-chevron--expanded" : ""}`}>
            <ChevronLeftIcon size={14} />
          </span>
          <strong>{title}</strong>
          {titleMeta ? <span className="collapsible-section-meta">{titleMeta}</span> : null}
        </button>
        {actions ? <div className="collapsible-section-actions">{actions}</div> : null}
      </div>
      {expanded ? <div className="collapsible-section-body">{children}</div> : null}
    </section>
  );
}
