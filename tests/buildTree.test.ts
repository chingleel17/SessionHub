import { describe, expect, test } from "bun:test";

import { buildOpenSpecTree, buildSisyphusTree } from "../src/utils/buildTree";
import type { OpenSpecData, SisyphusData, TreeNode } from "../src/types";

const t = (key: string) => key;

function flatten(nodes: TreeNode[]): TreeNode[] {
  return nodes.flatMap((node) => [node, ...flatten(node.children ?? [])]);
}

describe("buildSisyphusTree", () => {
  const data: SisyphusData = {
    activePlan: null,
    plans: [{
      name: "plan-a",
      path: "C:/project/.sisyphus/plans/plan-a.md",
      title: "Plan A",
      tldr: null,
      isActive: true,
    }],
    notepads: [{
      name: "notes",
      hasIssues: true,
      hasLearnings: false,
      issuesPath: "C:/project/.sisyphus/notepads/notes/issues.md",
      learningsPath: null,
    }],
    evidenceFiles: ["C:/project/.sisyphus/evidence/task.txt"],
    draftFiles: ["C:/project/.sisyphus/drafts/draft.md"],
  };

  test("標記所有節點來源並展開 notepad 檔案", () => {
    const nodes = buildSisyphusTree(data, t);
    const allNodes = flatten(nodes);
    expect(allNodes.every((node) => node.sourceKind === "sisyphus")).toBe(true);
    expect(allNodes.some((node) => node.id.includes("evidence"))).toBe(false);
    expect(allNodes.find((node) => node.id.endsWith(":issues"))?.filePath).toBe(data.notepads[0].issuesPath);
    expect(allNodes.find((node) => node.id.endsWith(":learnings"))).toBeUndefined();
    expect(allNodes.find((node) => node.id === "sisyphus:draft:C:/project/.sisyphus/drafts/draft.md")?.filePath)
      .toBe(data.draftFiles[0]);
  });
});

describe("buildOpenSpecTree", () => {
  test("標記來源根、change、artifact 與 spec 葉節點", () => {
    const data: OpenSpecData = {
      schema: "spec-driven",
      activeChanges: [{
        name: "change-a",
        hasProposal: true,
        hasDesign: false,
        hasTasks: true,
        taskProgress: { done: 1, total: 2, status: "in_progress" },
        specsCount: 1,
        specs: [{ name: "feature", path: "C:/project/openspec/specs/feature/spec.md" }],
        createdAt: null,
      }],
      archivedChanges: [],
      specs: [],
    };

    const nodes = buildOpenSpecTree(data, t);
    expect(flatten(nodes).every((node) => node.sourceKind === "openspec")).toBe(true);
  });
});
