import type { ReactNode } from "react";
import { ChevronLeftIcon } from "./Icons";

type Props = {
  title: string;
  expanded: boolean;
  onToggle: () => void;
  children: ReactNode;
};

/** 可收折分區：標題列 + chevron，內容僅於展開時渲染（收折時不掛載、狀態不保留）。 */
export function CollapsibleSection({ title, expanded, onToggle, children }: Props) {
  return (
    <section className={`collapsible-section ${expanded ? "collapsible-section--expanded" : ""}`}>
      <button type="button" className="collapsible-section-header" onClick={onToggle} aria-expanded={expanded}>
        <span className={`collapsible-section-chevron ${expanded ? "collapsible-section-chevron--expanded" : ""}`}>
          <ChevronLeftIcon size={14} />
        </span>
        <strong>{title}</strong>
      </button>
      {expanded ? <div className="collapsible-section-body">{children}</div> : null}
    </section>
  );
}
