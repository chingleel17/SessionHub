import { useState } from "react";
import type { TreeNode } from "../types";

type Props = {
  nodes: TreeNode[];
  selectedId: string | null;
  onSelect: (node: TreeNode) => void;
};

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
      <span className="tree-leaf-icon">📄</span>
      <span style={{ overflow: "hidden", textOverflow: "ellipsis" }}>{node.label}</span>
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
        <span style={{ overflow: "hidden", textOverflow: "ellipsis" }}>{node.label}</span>
        {node.badge ? (
          <span style={{ marginLeft: "auto", fontSize: 11, opacity: 0.7 }}>{node.badge}</span>
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
