import { useMemo, useState } from "react";
import { useI18n } from "../i18n/I18nProvider";
import type {
  OpenSpecData,
  ProjectGroup,
  SessionInfo,
  SessionStats,
  SisyphusData,
  SortKey,
} from "../types";
import { DeleteIcon, PinIcon, UnpinIcon } from "./Icons";
import { PlanEditor } from "./PlanEditor";
import { PlansSpecsView } from "./PlansSpecsView";
import { ProjectStatsBanner } from "./ProjectStatsBanner";
import { SessionCard } from "./SessionCard";

type SubTabState = { openPlanKeys: string[]; activeSubTab: string };

type Props = {
  project: ProjectGroup;
  showArchived: boolean;
  onToggleArchived: (value: boolean) => void;
  onOpenTerminal: (session: SessionInfo) => void;
  onCopyCommand: (session: SessionInfo) => void;
  onEditNotes: (session: SessionInfo) => void;
  onEditTags: (session: SessionInfo) => void;
  onOpenPlan: (session: SessionInfo) => void;
  onArchive: (session: SessionInfo) => void;
  onUnarchive: (session: SessionInfo) => void;
  onDelete: (session: SessionInfo) => void;
  onDeleteEmptySessions: () => void;
  isPinned: boolean;
  onTogglePin: () => void;
  sessionStats: Record<string, SessionStats | undefined>;
  sessionStatsLoading: Record<string, boolean | undefined>;
  sessionsLoading: boolean;
  sisyphusData: SisyphusData | undefined;
  openspecData: OpenSpecData | undefined;
  plansSpecsLoading: boolean;
  onReadFileContent: (filePath: string) => Promise<string>;
  // Plan sub-tab props (IPC handled by App.tsx, state flows through here)
  activePlanSessionId: string | null;
  onActivePlanChange: (sessionId: string | null) => void;
  planDraft: string;
  planPreviewHtml: string;
  onPlanDraftChange: (value: string) => void;
  onSavePlan: () => void;
  onOpenPlanExternal: (session: SessionInfo) => void;
  // Controlled sub-tab state (lifted to App.tsx for cross-project persistence)
  openPlanKeys: string[];
  activeSubTab: string;
  onSubTabStateChange: (state: SubTabState) => void;
};

function filterAndSortSessions(
  sessions: SessionInfo[],
  searchTerm: string,
  sortKey: SortKey,
  selectedTags: string[],
  hideEmpty: boolean,
  selectedProviders: string[],
) {
  const normalizedSearchTerm = searchTerm.trim().toLowerCase();

  const filtered = sessions.filter((session) => {
    if (selectedProviders.length > 0 && !selectedProviders.includes(session.provider)) return false;
    if (hideEmpty && !session.hasEvents) return false;

    const matchesTags =
      selectedTags.length === 0 || selectedTags.every((tag) => session.tags.includes(tag));

    if (!matchesTags) return false;
    if (!normalizedSearchTerm) return true;

    const haystacks = [
      session.id,
      session.cwd ?? "",
      session.summary ?? "",
      session.notes ?? "",
      session.tags.join(" "),
    ];

    return haystacks.some((value) => value.toLowerCase().includes(normalizedSearchTerm));
  });

  return filtered.sort((left, right) => {
    switch (sortKey) {
      case "createdAt":
        return (right.createdAt ?? "").localeCompare(left.createdAt ?? "");
      case "summaryCount":
        return (right.summaryCount ?? 0) - (left.summaryCount ?? 0);
      case "summary": {
        const getTitle = (s: SessionInfo) => s.summary?.trim() || s.id;
        return getTitle(left).localeCompare(getTitle(right));
      }
      case "updatedAt":
      default:
        return (right.updatedAt ?? "").localeCompare(left.updatedAt ?? "");
    }
  });
}

export function ProjectView({
  project,
  showArchived,
  onToggleArchived,
  onOpenTerminal,
  onCopyCommand,
  onEditNotes,
  onEditTags,
  onOpenPlan,
  onArchive,
  onUnarchive,
  onDelete,
  onDeleteEmptySessions,
  isPinned,
  onTogglePin,
  sessionStats,
  sessionStatsLoading,
  sessionsLoading,
  sisyphusData,
  openspecData,
  plansSpecsLoading,
  onReadFileContent,
  activePlanSessionId,
  onActivePlanChange,
  planDraft,
  planPreviewHtml,
  onPlanDraftChange,
  onSavePlan,
  onOpenPlanExternal,
  openPlanKeys,
  activeSubTab,
  onSubTabStateChange,
}: Props) {
  const { t } = useI18n();
  const [searchTerm, setSearchTerm] = useState("");
  const [sortKey, setSortKey] = useState<SortKey>("updatedAt");
  const [selectedTags, setSelectedTags] = useState<string[]>([]);
  const [hideEmpty, setHideEmpty] = useState(false);
  const [selectedProviders, setSelectedProviders] = useState<string[]>([]);

  const setActiveSubTab = (next: string) => {
    onSubTabStateChange({ openPlanKeys, activeSubTab: next });
  };

  const handleOpenPlanSubTab = (session: SessionInfo) => {
    const planKey = `plan:${session.id}`;
    const nextKeys = openPlanKeys.includes(planKey) ? openPlanKeys : [...openPlanKeys, planKey];
    onSubTabStateChange({ openPlanKeys: nextKeys, activeSubTab: planKey });
    onOpenPlan(session);
  };

  const handleClosePlanSubTab = (planKey: string) => {
    const sessionId = planKey.replace("plan:", "");
    const nextKeys = openPlanKeys.filter((k) => k !== planKey);
    const nextSubTab = activeSubTab === planKey ? "sessions" : activeSubTab;
    onSubTabStateChange({ openPlanKeys: nextKeys, activeSubTab: nextSubTab });
    if (activePlanSessionId === sessionId) {
      onActivePlanChange(null);
    }
  };

  const availableProviders = useMemo(
    () => [...new Set(project.sessions.map((s) => s.provider))].sort(),
    [project.sessions],
  );

  const availableTags = useMemo(
    () =>
      [...new Set(project.sessions.flatMap((s) => s.tags))]
        .filter(Boolean)
        .sort((a, b) => a.localeCompare(b)),
    [project.sessions],
  );

  const emptySessions = useMemo(
    () => project.sessions.filter((s) => !s.hasEvents),
    [project.sessions],
  );

  const filteredSessions = useMemo(
    () => filterAndSortSessions(project.sessions, searchTerm, sortKey, selectedTags, hideEmpty, selectedProviders),
    [project.sessions, searchTerm, sortKey, selectedTags, hideEmpty, selectedProviders],
  );

  const hiddenCount = useMemo(() => {
    const withoutHide = filterAndSortSessions(project.sessions, searchTerm, sortKey, selectedTags, false, selectedProviders);
    const withHide = filterAndSortSessions(project.sessions, searchTerm, sortKey, selectedTags, true, selectedProviders);
    return withoutHide.length - withHide.length;
  }, [project.sessions, searchTerm, sortKey, selectedTags, selectedProviders]);

  return (
    <section className="project-page">
      {/* Sub-tab bar */}
      <div className="sub-tab-bar">
        <button
          type="button"
          className={`sub-tab-item ${activeSubTab === "sessions" ? "sub-tab-item--active" : ""}`}
          onClick={() => setActiveSubTab("sessions")}
        >
          {t("project.subTab.sessions")}
        </button>
        <button
          type="button"
          className={`sub-tab-item ${activeSubTab === "plans-specs" ? "sub-tab-item--active" : ""}`}
          onClick={() => setActiveSubTab("plans-specs")}
        >
          {t("project.subTab.plansSpecs")}
        </button>
        {openPlanKeys.map((planKey) => {
          const sessionId = planKey.replace("plan:", "");
          const session = project.sessions.find((s) => s.id === sessionId);
          if (!session) return null;
          const tabTitle = session.summary?.trim() || session.id.slice(0, 8);
          return (
            <div
              key={planKey}
              className={`sub-tab-item sub-tab-item--closeable ${activeSubTab === planKey ? "sub-tab-item--active" : ""}`}
            >
              <button
                type="button"
                className="sub-tab-label"
                onClick={() => {
                  setActiveSubTab(planKey);
                  onActivePlanChange(sessionId);
                }}
              >
                {t("plan.tab")} · {tabTitle}
              </button>
              <button
                type="button"
                className="sub-tab-close"
                onClick={() => handleClosePlanSubTab(planKey)}
                aria-label={`${t("tabs.close")} ${tabTitle}`}
              >
                ×
              </button>
            </div>
          );
        })}
      </div>

      {activeSubTab === "sessions" ? (
        <>
          <section className="toolbar-card">
            <ProjectStatsBanner
              sessions={filteredSessions}
              sessionStats={sessionStats}
              sessionStatsLoading={sessionStatsLoading}
            />

            <div className="filter-bar">
              <label className="field-group compact-field" style={{ flex: 2, minWidth: '160px' }}>
                <span>{t("session.search")}</span>
                <input
                  value={searchTerm}
                  onChange={(event) => setSearchTerm(event.currentTarget.value)}
                  placeholder={t("session.searchPlaceholder")}
                />
              </label>

              <label className="field-group compact-field">
                <span>{t("session.sort")}</span>
                <select
                  value={sortKey}
                  onChange={(event) => setSortKey(event.currentTarget.value as SortKey)}
                >
                  <option value="updatedAt">{t("session.sortUpdatedAt")}</option>
                  <option value="createdAt">{t("session.sortCreatedAt")}</option>
                  <option value="summaryCount">{t("session.sortSummaryCount")}</option>
                  <option value="summary">{t("session.sortSummary")}</option>
                </select>
              </label>

              <label className="checkbox-group compact-checkbox">
                <input
                  type="checkbox"
                  checked={showArchived}
                  onChange={(event) => onToggleArchived(event.currentTarget.checked)}
                />
                <span>{t("project.showArchivedToggle")}</span>
              </label>

              <label className="checkbox-group compact-checkbox">
                <input
                  type="checkbox"
                  checked={hideEmpty}
                  onChange={(event) => setHideEmpty(event.currentTarget.checked)}
                />
                <span>
                  {t("session.filter.hideEmpty")}
                  {hideEmpty && hiddenCount > 0 ? (
                    <span className="hidden-count-hint">
                      {" "}({t("session.filter.hiddenCount").replace("{count}", String(hiddenCount))})
                    </span>
                  ) : null}
                </span>
              </label>

              <div className="filter-bar-actions">
                {availableProviders.length > 1 ? (
                  <>
                    <span className="session-meta-label">{t("session.providerFilter")}</span>
                    {availableProviders.map((provider) => {
                      const isActive = selectedProviders.length === 0 || selectedProviders.includes(provider);
                      return (
                        <button
                          key={provider}
                          type="button"
                          className={`tag-filter-chip ${isActive ? "active" : ""}`}
                          onClick={() =>
                            setSelectedProviders((current) => {
                              if (current.length === 0) {
                                return [provider];
                              }
                              if (current.includes(provider)) {
                                const next = current.filter((p) => p !== provider);
                                return next.length === 0 ? [] : next;
                              }
                              const next = [...current, provider];
                              return next.length === availableProviders.length ? [] : next;
                            })
                          }
                        >
                          {provider === "copilot" ? "Copilot" : provider === "opencode" ? "OpenCode" : provider}
                        </button>
                      );
                    })}
                  </>
                ) : null}

                <button
                  type="button"
                  className="icon-button"
                  title={isPinned ? t("project.actions.unpin") : t("project.actions.pin")}
                  aria-label={isPinned ? t("project.actions.unpin") : t("project.actions.pin")}
                  onClick={onTogglePin}
                >
                  {isPinned ? <UnpinIcon size={16} /> : <PinIcon size={16} />}
                </button>

                <button
                  type="button"
                  className="icon-button icon-button--danger"
                  title={t("session.actions.deleteEmpty")}
                  aria-label={t("session.actions.deleteEmpty")}
                  disabled={emptySessions.length === 0}
                  onClick={onDeleteEmptySessions}
                >
                  <DeleteIcon size={16} />
                </button>
              </div>
            </div>
          </section>

          {availableTags.length > 0 ? (
            <section className="tag-filter-bar">
              <span className="session-meta-label">{t("session.tagFilter")}</span>
              <div className="session-chip-row">
                {availableTags.map((tag) => {
                  const isActive = selectedTags.includes(tag);
                  return (
                    <button
                      key={tag}
                      type="button"
                      className={`tag-filter-chip ${isActive ? "active" : ""}`}
                      onClick={() =>
                        setSelectedTags((current) =>
                          current.includes(tag)
                            ? current.filter((item) => item !== tag)
                            : [...current, tag],
                        )
                      }
                    >
                      #{tag}
                    </button>
                  );
                })}
              </div>
            </section>
          ) : null}

          <div className="session-list">
            {sessionsLoading ? (
              <>
                <div className="skeleton-card" />
                <div className="skeleton-card" />
                <div className="skeleton-card" />
              </>
            ) : (
              filteredSessions.map((session) => (
                <SessionCard
                  key={session.id}
                  session={session}
                  onOpenTerminal={onOpenTerminal}
                  onCopyCommand={onCopyCommand}
                  onEditNotes={onEditNotes}
                  onEditTags={onEditTags}
                  onOpenPlan={handleOpenPlanSubTab}
                  onArchive={onArchive}
                  onUnarchive={onUnarchive}
                  onDelete={onDelete}
                  stats={sessionStats[session.id]}
                  statsLoading={Boolean(sessionStatsLoading[session.id])}
                />
              ))
            )}
          </div>
        </>
      ) : activeSubTab === "plans-specs" ? (
        <PlansSpecsView
          sisyphusData={sisyphusData}
          openspecData={openspecData}
          isLoading={plansSpecsLoading}
          onReadFileContent={onReadFileContent}
        />
      ) : activeSubTab.startsWith("plan:") ? (
        (() => {
          const sessionId = activeSubTab.replace("plan:", "");
          const planSession = project.sessions.find((s) => s.id === sessionId);
          if (!planSession) return null;
          return (
            <PlanEditor
              session={planSession}
              planDraft={planDraft}
              planPreviewHtml={planPreviewHtml}
              onDraftChange={onPlanDraftChange}
              onSave={onSavePlan}
              onOpenExternal={onOpenPlanExternal}
              onClose={() => handleClosePlanSubTab(activeSubTab)}
            />
          );
        })()
      ) : null}
    </section>
  );
}
