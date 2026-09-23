import type { MessageKey } from "../locales/zh-TW";

export function quotaPlanLabel(
  provider: string,
  plan: string,
  t: (key: MessageKey, params?: Record<string, string | number>) => string,
): { label: string; title: string } {
  const normalized = plan.trim().toLowerCase();
  if (provider === "copilot" && normalized === "individual") {
    return {
      label: t("quota.plan.copilotPro"),
      title: t("quota.plan.communityMapping", { type: plan }),
    };
  }
  if (provider === "copilot" && normalized === "individual_pro") {
    return {
      label: t("quota.plan.copilotProPlus"),
      title: t("quota.plan.communityMapping", { type: plan }),
    };
  }
  if (provider === "copilot" && normalized.startsWith("individual_")) {
    return { label: t("quota.plan.individual"), title: t("quota.plan.copilotLimit") };
  }
  if (provider === "codex" && normalized === "self_serve_business_prolite") {
    return {
      label: t("quota.plan.businessPremium"),
      title: t("quota.plan.inferredType", { type: plan }),
    };
  }
  if (provider === "codex" && normalized.startsWith("self_serve_business_")) {
    return {
      label: t("quota.plan.businessAccount"),
      title: t("quota.plan.internalType", { type: plan }),
    };
  }
  return { label: plan.replace(/_/g, " "), title: t("quota.plan") };
}
