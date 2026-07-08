import type { AgentsMdScanResult, OpenSpecChange, OpenSpecData, SisyphusData, TreeNode } from "../types";
import type { MessageKey } from "../locales/zh-TW";

type TranslateFn = (key: MessageKey) => string;

function progressTone(status: string | null | undefined): TreeNode["tone"] {
  switch (status) {
    case "not_started":
    case "in_progress":
    case "done":
      return status;
    default:
      return "neutral";
  }
}

function changeToNodes(change: OpenSpecChange, basePath: string, t: TranslateFn): TreeNode[] {
  const artifacts: TreeNode[] = [];
  if (change.hasProposal) {
    artifacts.push({
      id: `change:${change.name}:proposal`,
      label: "proposal.md",
      icon: "proposal",
      filePath: `${basePath}/proposal.md`,
      filePathType: "openspec",
    });
  }
  if (change.hasDesign) {
    artifacts.push({
      id: `change:${change.name}:design`,
      label: "design.md",
      icon: "design",
      filePath: `${basePath}/design.md`,
      filePathType: "openspec",
    });
  }
  if (change.hasTasks) {
    artifacts.push({
      id: `change:${change.name}:tasks`,
      label: "tasks.md",
      badge: change.taskProgress ? `${change.taskProgress.done}/${change.taskProgress.total}` : undefined,
      icon: "tasks",
      progress: change.taskProgress ?? null,
      tone: progressTone(change.taskProgress?.status),
      filePath: `${basePath}/tasks.md`,
      filePathType: "openspec",
    });
  }
  if (change.specs.length > 0) {
    artifacts.push({
      id: `change:${change.name}:specs`,
      label: `${t("plansSpecs.openspec.specs")} (${change.specs.length})`,
      icon: "folder",
      defaultOpen: false,
      children: change.specs.map((spec) => ({
        id: `change:${change.name}:spec:${spec.name}`,
        label: spec.name,
        icon: "spec",
        filePath: spec.path,
        filePathType: "absolute" as const,
      })),
    });
  }
  return artifacts;
}

export function buildSisyphusTree(data: SisyphusData, t: TranslateFn): TreeNode[] {
  const sections: TreeNode[] = [];

  if (data.plans.length > 0) {
    sections.push({
      id: "sisyphus:plans",
      label: `${t("plansSpecs.sisyphus.plans")} (${data.plans.length})`,
      icon: "section",
      defaultOpen: true,
      children: data.plans.map((plan) => ({
        id: `sisyphus:plan:${plan.path}`,
        label: plan.title ?? plan.name,
        icon: "plan",
        filePath: plan.path,
        filePathType: "absolute" as const,
      })),
    });
  }

  if (data.notepads.length > 0) {
    sections.push({
      id: "sisyphus:notepads",
      label: `${t("plansSpecs.sisyphus.notepads")} (${data.notepads.length})`,
      icon: "section",
      defaultOpen: false,
      children: data.notepads.map((np) => ({
        id: `sisyphus:notepad:${np.name}`,
        label: np.name,
        icon: "note",
        badge: [np.hasIssues ? "issues" : null, np.hasLearnings ? "learnings" : null]
          .filter(Boolean)
          .join(", ") || undefined,
      })),
    });
  }

  if (data.evidenceFiles.length > 0) {
    sections.push({
      id: "sisyphus:evidence",
      label: `${t("plansSpecs.sisyphus.evidence")} (${data.evidenceFiles.length})`,
      icon: "section",
      defaultOpen: false,
      children: data.evidenceFiles.map((f) => ({
        id: `sisyphus:evidence:${f}`,
        label: f.split(/[\\/]/).pop() ?? f,
        icon: "evidence",
        filePath: f,
        filePathType: "absolute" as const,
      })),
    });
  }

  if (data.draftFiles.length > 0) {
    sections.push({
      id: "sisyphus:drafts",
      label: `${t("plansSpecs.sisyphus.drafts")} (${data.draftFiles.length})`,
      icon: "section",
      defaultOpen: false,
      children: data.draftFiles.map((f) => ({
        id: `sisyphus:draft:${f}`,
        label: f.split(/[\\/]/).pop() ?? f,
        icon: "draft",
        filePath: f,
        filePathType: "absolute" as const,
      })),
    });
  }

  return sections;
}

export function buildOpenSpecTree(data: OpenSpecData, t: TranslateFn): TreeNode[] {
  const sections: TreeNode[] = [];

  if (data.activeChanges.length > 0) {
    sections.push({
      id: "openspec:active-changes",
      label: `${t("plansSpecs.openspec.activeChanges")} (${data.activeChanges.length})`,
      icon: "section",
      defaultOpen: true,
      children: data.activeChanges.map((change) => ({
        id: `openspec:change:${change.name}`,
        label: change.name,
        badge: change.taskProgress ? `${change.taskProgress.done}/${change.taskProgress.total}` : undefined,
        icon: "change",
        progress: change.taskProgress ?? null,
        tone: progressTone(change.taskProgress?.status),
        defaultOpen: false,
        children: changeToNodes(change, `changes/${change.name}`, t),
      })),
    });
  }

  if (data.archivedChanges.length > 0) {
    sections.push({
      id: "openspec:archived-changes",
      label: `${t("plansSpecs.openspec.archivedChanges")} (${data.archivedChanges.length})`,
      icon: "section",
      defaultOpen: false,
      children: data.archivedChanges.map((change) => ({
        id: `openspec:archived:${change.name}`,
        label: change.name,
        badge: change.taskProgress ? `${change.taskProgress.done}/${change.taskProgress.total}` : undefined,
        icon: "change",
        progress: change.taskProgress ?? null,
        tone: progressTone(change.taskProgress?.status),
        defaultOpen: false,
        children: changeToNodes(change, `changes/archive/${change.name}`, t),
      })),
    });
  }

  if (data.specs.length > 0) {
    sections.push({
      id: "openspec:specs",
      label: `${t("plansSpecs.openspec.specs")} (${data.specs.length})`,
      icon: "section",
      defaultOpen: false,
      children: data.specs.map((spec) => ({
        id: `openspec:spec:${spec.path}`,
        label: spec.name,
        icon: "spec",
        filePath: spec.path,
        filePathType: "absolute" as const,
      })),
    });
  }

  return sections;
}

function agentsStatusTone(status: string): TreeNode["tone"] {
  switch (status) {
    case "in-sync":
    case "linked":
      return "done";
    case "target-missing":
    case "source-missing":
      return "not_started";
    case "differs":
    case "link-broken":
      return "in_progress";
    default:
      return "neutral";
  }
}

function getAgentsEntryLabel(entry: AgentsMdScanResult["entries"][number], root: string, t: TranslateFn): string {
  const normalized = entry.relDir.replace(/\\/g, "/").replace(/^\.?\/?/, "");
  if (!normalized) {
    return root.replace(/\\/g, "/").split("/").filter(Boolean).pop() ?? t("agents.tree.root");
  }

  const segments = normalized.split("/").filter(Boolean);
  return segments[segments.length - 1] ?? normalized;
}

export function buildAgentsMdTree(result: AgentsMdScanResult, t: TranslateFn): TreeNode[] {
  const groups = new Map<string, TreeNode>();
  const roots: TreeNode[] = [];

  const ensureGroup = (parentKey: string | null, segment: string, fullKey: string): TreeNode => {
    const existing = groups.get(fullKey);
    if (existing) return existing;

    const next: TreeNode = {
      id: `agents-md-group:${fullKey}`,
      label: segment,
      icon: "folder",
      defaultOpen: true,
      children: [],
    };

    groups.set(fullKey, next);
    if (parentKey) {
      groups.get(parentKey)?.children?.push(next);
    } else {
      roots.push(next);
    }
    return next;
  };

  for (const entry of result.entries) {
    const normalized = entry.relDir.replace(/\\/g, "/").replace(/^\.?\/?/, "");
    const segments = normalized ? normalized.split("/").filter(Boolean) : [];
    let parentKey: string | null = null;
    let currentKey = "";

    for (const segment of segments.slice(0, -1)) {
      currentKey = currentKey ? `${currentKey}/${segment}` : segment;
      ensureGroup(parentKey, segment, currentKey);
      parentKey = currentKey;
    }

    const leaf: TreeNode = {
      id: `agents-md:${entry.dir}`,
      label: getAgentsEntryLabel(entry, result.root, t),
      badge: t(`agents.status.${entry.status}` as MessageKey),
      icon: "agents",
      tone: agentsStatusTone(entry.status),
      filePath: entry.source.exists ? entry.source.path : entry.target.path,
      filePathType: "absolute",
    };

    if (parentKey) {
      groups.get(parentKey)?.children?.push(leaf);
    } else {
      roots.push(leaf);
    }
  }

  return roots;
}
