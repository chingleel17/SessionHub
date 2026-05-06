import type { OpenSpecChange, OpenSpecData, SisyphusData, TreeNode } from "../types";
import type { MessageKey } from "../locales/zh-TW";

type TranslateFn = (key: MessageKey) => string;

function changeToNodes(change: OpenSpecChange, basePath: string, t: TranslateFn): TreeNode[] {
  const artifacts: TreeNode[] = [];
  if (change.hasProposal) {
    artifacts.push({
      id: `change:${change.name}:proposal`,
      label: "proposal.md",
      filePath: `${basePath}/proposal.md`,
      filePathType: "openspec",
    });
  }
  if (change.hasDesign) {
    artifacts.push({
      id: `change:${change.name}:design`,
      label: "design.md",
      filePath: `${basePath}/design.md`,
      filePathType: "openspec",
    });
  }
  if (change.hasTasks) {
    artifacts.push({
      id: `change:${change.name}:tasks`,
      label: "tasks.md",
      filePath: `${basePath}/tasks.md`,
      filePathType: "openspec",
    });
  }
  if (change.specs.length > 0) {
    artifacts.push({
      id: `change:${change.name}:specs`,
      label: `${t("plansSpecs.openspec.specs")} (${change.specs.length})`,
      defaultOpen: false,
      children: change.specs.map((spec) => ({
        id: `change:${change.name}:spec:${spec.name}`,
        label: spec.name,
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
      defaultOpen: true,
      children: data.plans.map((plan) => ({
        id: `sisyphus:plan:${plan.path}`,
        label: plan.title ?? plan.name,
        filePath: plan.path,
        filePathType: "absolute" as const,
      })),
    });
  }

  if (data.notepads.length > 0) {
    sections.push({
      id: "sisyphus:notepads",
      label: `${t("plansSpecs.sisyphus.notepads")} (${data.notepads.length})`,
      defaultOpen: false,
      children: data.notepads.map((np) => ({
        id: `sisyphus:notepad:${np.name}`,
        label: np.name,
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
      defaultOpen: false,
      children: data.evidenceFiles.map((f) => ({
        id: `sisyphus:evidence:${f}`,
        label: f.split(/[\\/]/).pop() ?? f,
        filePath: f,
        filePathType: "absolute" as const,
      })),
    });
  }

  if (data.draftFiles.length > 0) {
    sections.push({
      id: "sisyphus:drafts",
      label: `${t("plansSpecs.sisyphus.drafts")} (${data.draftFiles.length})`,
      defaultOpen: false,
      children: data.draftFiles.map((f) => ({
        id: `sisyphus:draft:${f}`,
        label: f.split(/[\\/]/).pop() ?? f,
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
      defaultOpen: true,
      children: data.activeChanges.map((change) => ({
        id: `openspec:change:${change.name}`,
        label: change.name,
        defaultOpen: false,
        children: changeToNodes(change, `changes/${change.name}`, t),
      })),
    });
  }

  if (data.archivedChanges.length > 0) {
    sections.push({
      id: "openspec:archived-changes",
      label: `${t("plansSpecs.openspec.archivedChanges")} (${data.archivedChanges.length})`,
      defaultOpen: false,
      children: data.archivedChanges.map((change) => ({
        id: `openspec:archived:${change.name}`,
        label: change.name,
        defaultOpen: false,
        children: changeToNodes(change, `changes/archive/${change.name}`, t),
      })),
    });
  }

  if (data.specs.length > 0) {
    sections.push({
      id: "openspec:specs",
      label: `${t("plansSpecs.openspec.specs")} (${data.specs.length})`,
      defaultOpen: true,
      children: data.specs.map((spec) => ({
        id: `openspec:spec:${spec.path}`,
        label: spec.name,
        filePath: spec.path,
        filePathType: "absolute" as const,
      })),
    });
  }

  return sections;
}
