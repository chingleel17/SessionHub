import { TerminalIcon } from "session-hub";

const row: React.CSSProperties = { display: "flex", alignItems: "center", gap: 24, padding: 8 };
const cell: React.CSSProperties = { display: "flex", flexDirection: "column", alignItems: "center", gap: 6, fontSize: 11, color: "#6b7280" };

export const Sizes = () => (
  <div style={row}>
    <div style={cell}><TerminalIcon size={16} /><span>16</span></div>
    <div style={cell}><TerminalIcon size={24} /><span>24</span></div>
    <div style={cell}><TerminalIcon size={32} /><span>32</span></div>
  </div>
);

export const InButton = () => (
  <div style={row}>
    <button type="button" className="icon-button" aria-label="TerminalIcon"><TerminalIcon size={14} /></button>
    <button type="button" className="ghost-button"><TerminalIcon size={14} /> TerminalIcon</button>
  </div>
);
