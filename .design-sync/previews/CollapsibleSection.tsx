import { useState } from "react";
import { CollapsibleSection, RefreshIcon, SearchIcon } from "session-hub";

const listStyle: React.CSSProperties = { display: "flex", flexDirection: "column", gap: 8, padding: "4px 0" };

function DemoRows() {
  return (
    <div style={listStyle}>
      <div>AGENTS.md — synced 2 minutes ago</div>
      <div>skills/openspec — 4 files, in sync</div>
      <div>commands/review — target missing</div>
    </div>
  );
}

export const Expanded = () => (
  <CollapsibleSection title="Skills" expanded onToggle={() => {}} titleMeta="~/.claude/skills">
    <DemoRows />
  </CollapsibleSection>
);

export const Collapsed = () => (
  <CollapsibleSection title="Commands" expanded={false} onToggle={() => {}} titleMeta="~/.claude/commands">
    <DemoRows />
  </CollapsibleSection>
);

export const WithActions = () => (
  <CollapsibleSection
    title="AGENTS.md"
    expanded
    onToggle={() => {}}
    titleMeta="H:/Code/DIY/SessionHub"
    actions={
      <>
        <button type="button" className="icon-button" aria-label="Search">
          <SearchIcon size={14} />
        </button>
        <button type="button" className="icon-button" aria-label="Refresh">
          <RefreshIcon size={14} />
        </button>
      </>
    }
  >
    <DemoRows />
  </CollapsibleSection>
);

export const Interactive = () => {
  const [open, setOpen] = useState(true);
  return (
    <CollapsibleSection title="MCP Servers" expanded={open} onToggle={() => setOpen(!open)} titleMeta="3 servers">
      <DemoRows />
    </CollapsibleSection>
  );
};
