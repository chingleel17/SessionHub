import { useState } from "react";
import type { TreeNode } from "../types";

type Props = {
  nodes: TreeNode[];
  selectedId: string | null;
  onSelect: (node: TreeNode) => void;
};

function getNodeIconLabel(icon: TreeNode["icon"]): string {
  switch (icon) {
    case "proposal":
      return "P";
    case "design":
      return "D";
    case "tasks":
      return "T";
    case "spec":
      return "S";
    case "change":
      return "C";
    case "plan":
      return "PL";
    case "note":
      return "N";
    case "evidence":
      return "EV";
    case "draft":
      return "DR";
    default:
      return "•";
  }
}

function TreeLeaf({
  node,
  selectedId,
  onSelect,
  depth,
}: {
  node: TreeNode;
  selectedId: string | null;
  onSelect: (node: TreeNode) => void;
  depth: number;
}) {
  const isSelectable = !!node.filePath;
  const isSelected = selectedId === node.id;
  const paddingLeft = 8 + depth * 12;

  return (
    <button
      type="button"
      className={`tree-leaf-btn${isSelected ? " tree-leaf-btn--selected" : ""}${!isSelectable ? " tree-leaf-btn--disabled" : ""}`}
      style={{ paddingLeft }}
      onClick={() => {
        if (isSelectable) onSelect(node);
      }}
      disabled={!isSelectable}
      title={node.label}
    >
      <span className={`tree-leaf-icon tree-leaf-icon--${node.icon ?? "folder"} tree-leaf-icon--${node.tone ?? "neutral"}`}>
        {getNodeIconLabel(node.icon)}
      </span>
      <span className="tree-node-label">{node.label}</span>
      {node.badge ? (
        <span className={`tree-node-badge tree-node-badge--${node.tone ?? "neutral"}`}>{node.badge}</span>
      ) : null}
    </button>
  );
}

function TreeGroup({
  node,
  selectedId,
  onSelect,
  depth,
}: {
  node: TreeNode;
  selectedId: string | null;
  onSelect: (node: TreeNode) => void;
  depth: number;
}) {
  const [isOpen, setIsOpen] = useState(node.defaultOpen ?? false);
  const paddingLeft = 4 + depth * 12;

  return (
    <div>
      <button
        type="button"
        className="tree-subgroup-btn"
        style={{ paddingLeft }}
        onClick={() => setIsOpen((v) => !v)}
      >
        <span className={`tree-group-arrow${isOpen ? " tree-group-arrow--open" : ""}`}>▶</span>
        <span className="tree-node-label">{node.label}</span>
        {node.badge ? (
          <span className={`tree-node-badge tree-node-badge--${node.tone ?? "neutral"}`}>{node.badge}</span>
        ) : null}
      </button>
      {isOpen && node.children ? (
        <div>
          {node.children.map((child) => (
            <TreeNodeItem
              key={child.id}
              node={child}
              selectedId={selectedId}
              onSelect={onSelect}
              depth={depth + 1}
            />
          ))}
        </div>
      ) : null}
    </div>
  );
}

function TreeNodeItem({
  node,
  selectedId,
  onSelect,
  depth,
}: {
  node: TreeNode;
  selectedId: string | null;
  onSelect: (node: TreeNode) => void;
  depth: number;
}) {
  if (node.children !== undefined) {
    return (
      <TreeGroup
        node={node}
        selectedId={selectedId}
        onSelect={onSelect}
        depth={depth}
      />
    );
  }
  return (
    <TreeLeaf
      node={node}
      selectedId={selectedId}
      onSelect={onSelect}
      depth={depth}
    />
  );
}

export function ExplorerTree({ nodes, selectedId, onSelect }: Props) {
  return (
    <div className="explorer-tree">
      {nodes.map((node) => (
        <TreeNodeItem
          key={node.id}
          node={node}
          selectedId={selectedId}
          onSelect={onSelect}
          depth={0}
        />
      ))}
    </div>
  );
}
