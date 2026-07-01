import { useCallback, useEffect, useMemo, useState } from "react";
import { useI18n } from "../i18n/I18nProvider";
import type {
  AnalyticsDataPoint,
  AnalyticsGroupBy,
  IdeLauncherType,
  OpenSpecData,
  ProjectGroup,
  ProjectSubTabState,
  SessionActivityStatus,
  SessionInfo,
  SessionStats,
  SessionTodo,
  SisyphusData,
  SortKey,
  ToolAvailability,
} from "../types";
import { ProjectAnalyticsTab } from "./ProjectAnalyticsTab";
import { DeleteIcon, PinIcon, UnpinIcon } from "./Icons";
import { PlanEditor } from "./PlanEditor";
import { PlansSpecsView } from "./PlansSpecsView";
import { ProjectStatsBanner } from "./ProjectStatsBanner";
import { SessionCard } from "./SessionCard";
import { SessionTodosTab } from "./SessionTodosTab";
import { getProviderLabel } from "../utils/providerLabel";

const FILTER_EXPANDED_STORAGE_KEY = "sessionFilterExpanded";

type SessionUpdatedRange = "all" | "week" | "month";

const SESSIONS_PAGE_SIZE = 20;

function getUpdatedRangeStart(range: SessionUpdatedRange): number | null {
  const now = new Date();
  if (range === "week") {
    const start = new Date(now);
    start.setDate(now.getDate() - ((now.getDay() + 6) % 7));
    start.setHours(0, 0, 0, 0);
    return start.getTime();
  }

  if (range === "month") {
    return new Date(now.getFullYear(), now.getMonth(), 1).getTime();
  }

  return null;
}

function isSessionInUpdatedRange(session: SessionInfo, range: SessionUpdatedRange): boolean {
  const rangeStart = getUpdatedRangeStart(range);
  if (rangeStart === null) return true;
  if (!session.updatedAt) return false;
  const updatedAtTime = Date.parse(session.updatedAt);
  return !Number.isNaN(updatedAtTime) && updatedAtTime >= rangeStart;
}

type Props = {
  project: ProjectGroup;
  showArchived: boolean;
  hideEmptySessions: boolean;
  onHideEmptySessionsChange: (value: boolean) => void;
  totalEmptySessions: number;
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
  sessionTodos: Record<string, SessionTodo[] | undefined>;
  sessionTodosLoading: Record<string, boolean | undefined>;
  sessionsLoading: boolean;
  sisyphusData: SisyphusData | undefined;
  openspecData: OpenSpecData | undefined;
  plansSpecsLoading: boolean;
  plansSpecsRefreshing: boolean;
  onReadFileContent: (filePath: string) => Promise<string>;
  onReadOpenspecFile: (projectCwd: string, relativePath: string) => Promise<string>;
  onWriteOpenspecFile: (projectCwd: string, relativePath: string, content: string) => Promise<void>;
  onRefreshPlansSpecs: () => Promise<void>;
  plansSpecsRefreshToken: string;
  activityStatusMap: Map<string, SessionActivityStatus>;
  onOpenInTool: (session: SessionInfo, tool: IdeLauncherType) => void;
  onFocusTerminal: (session: SessionInfo) => void;
  defaultLauncher: string | null;
  toolAvailability: ToolAvailability | null;
  // Plan sub-tab props (IPC handled by App.tsx, state flows through here)
  activePlanSessionId: string | null;
  onActivePlanChange: (sessionId: string | null) => void;
  planDraft: string;
  planPreviewHtml: string;
  onPlanDraftChange: (value: string) => void;
  onSavePlan: () => void;
  onOpenPlanExternal: (session: SessionInfo) => void;
  // Controlled sub-tab state (lifted to App.tsx for cross-project persistence)
  openDetailKeys: string[];
  activeSubTab: string;
  onSubTabStateChange: (state: ProjectSubTabState) => void;
  onFetchAnalytics: (
    cwd: string | null,
    startDate: string,
    endDate: string,
    groupBy: AnalyticsGroupBy,
  ) => Promise<AnalyticsDataPoint[] | null>;
};

function filterAndSortSessions(
  sessions: SessionInfo[],
  searchTerm: string,
  sortKey: SortKey,
  selectedTags: string[],
  hideEmpty: boolean,
  selectedProviders: string[],
  updatedRange: SessionUpdatedRange,
) {
  const normalizedSearchTerm = searchTerm.trim().toLowerCase();

  const filtered = sessions.filter((session) => {
    if (selectedProviders.length > 0 && !selectedProviders.includes(session.provider)) return false;
    if (hideEmpty && !session.hasEvents) return false;
    if (!isSessionInUpdatedRange(session, updatedRange)) return false;

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
  hideEmptySessions,
  onHideEmptySessionsChange,
  totalEmptySessions,
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
  sessionTodos,
  sessionTodosLoading,
  sessionsLoading,
  sisyphusData,
  openspecData,
  plansSpecsLoading,
  plansSpecsRefreshing,
  onReadFileContent,
  onReadOpenspecFile,
  onWriteOpenspecFile,
  onRefreshPlansSpecs,
  plansSpecsRefreshToken,
  activePlanSessionId,
  onActivePlanChange,
  planDraft,
  planPreviewHtml,
  onPlanDraftChange,
  onSavePlan,
  onOpenPlanExternal,
  openDetailKeys,
  activeSubTab,
  onSubTabStateChange,
  onFetchAnalytics,
  activityStatusMap,
  onOpenInTool,
  onFocusTerminal,
  defaultLauncher,
  toolAvailability,
}: Props) {
  const { t } = useI18n();
  const [searchTerm, setSearchTerm] = useState("");
  const [sortKey, setSortKey] = useState<SortKey>("updatedAt");
  const [selectedTags, setSelectedTags] = useState<string[]>([]);
  const [selectedProviders, setSelectedProviders] = useState<string[]>([]);
  const [selectedUpdatedRange, setSelectedUpdatedRange] = useState<SessionUpdatedRange>("all");
  const [currentPage, setCurrentPage] = useState(1);
  const [openLauncherSessionId, setOpenLauncherSessionId] = useState<string | null>(null);
  const [isFilterExpanded, setIsFilterExpanded] = useState(() => {
    return window.localStorage.getItem(FILTER_EXPANDED_STORAGE_KEY) === "true";
  });

  const handleToggleFilterExpanded = useCallback(() => {
    setIsFilterExpanded((prev) => {
      const next = !prev;
      window.localStorage.setItem(FILTER_EXPANDED_STORAGE_KEY, String(next));
      return next;
    });
  }, []);

  const handleToggleLauncher = useCallback((sessionId: string) => {
    setOpenLauncherSessionId((prev) => prev === sessionId ? null : sessionId);
  }, []);

  // 點擊選單外部關閉
  useEffect(() => {
    if (!openLauncherSessionId) return;
    const handler = (e: MouseEvent) => {
      if (!(e.target as Element).closest("[data-launcher-menu]")) {
        setOpenLauncherSessionId(null);
      }
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, [openLauncherSessionId]);

  const buildDetailTabKey = (kind: "plan" | "todos", sessionId: string) => `${kind}:${sessionId}`;
  const parseDetailTabKey = (value: string): { kind: "plan" | "todos"; sessionId: string } | null => {
    if (value.startsWith("plan:")) return { kind: "plan", sessionId: value.replace("plan:", "") };
    if (value.startsWith("todos:")) return { kind: "todos", sessionId: value.replace("todos:", "") };
    return null;
  };

  const setActiveSubTab = (next: string) => {
    onSubTabStateChange({ openDetailKeys, activeSubTab: next });
    if (!next.startsWith("plan:")) {
      onActivePlanChange(null);
    }
  };

  const handleOpenPlanSubTab = (session: SessionInfo) => {
    const planKey = buildDetailTabKey("plan", session.id);
    const nextKeys = openDetailKeys.includes(planKey) ? openDetailKeys : [...openDetailKeys, planKey];
    onSubTabStateChange({ openDetailKeys: nextKeys, activeSubTab: planKey });
    onOpenPlan(session);
  };

  const handleOpenTodosSubTab = (session: SessionInfo) => {
    const todosKey = buildDetailTabKey("todos", session.id);
    const nextKeys = openDetailKeys.includes(todosKey) ? openDetailKeys : [...openDetailKeys, todosKey];
    onSubTabStateChange({ openDetailKeys: nextKeys, activeSubTab: todosKey });
    onActivePlanChange(null);
  };

  const handleCloseDetailSubTab = (detailKey: string) => {
    const detail = parseDetailTabKey(detailKey);
    const nextKeys = openDetailKeys.filter((k) => k !== detailKey);
    const nextSubTab = activeSubTab === detailKey ? "sessions" : activeSubTab;
    onSubTabStateChange({ openDetailKeys: nextKeys, activeSubTab: nextSubTab });
    if (detail?.kind === "plan" && activePlanSessionId === detail.sessionId) {
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

  const filteredSessions = useMemo(
    () =>
      filterAndSortSessions(
        project.sessions,
        searchTerm,
        sortKey,
        selectedTags,
        hideEmptySessions,
        selectedProviders,
        selectedUpdatedRange,
      ),
    [project.sessions, searchTerm, sortKey, selectedTags, hideEmptySessions, selectedProviders, selectedUpdatedRange],
  );

  const hiddenCount = useMemo(() => {
    const withoutHide = filterAndSortSessions(
      project.sessions,
      searchTerm,
      sortKey,
      selectedTags,
      false,
      selectedProviders,
      selectedUpdatedRange,
    );
    const withHide = filterAndSortSessions(
      project.sessions,
      searchTerm,
      sortKey,
      selectedTags,
      true,
      selectedProviders,
      selectedUpdatedRange,
    );
    return withoutHide.length - withHide.length;
  }, [project.sessions, searchTerm, sortKey, selectedTags, selectedProviders, selectedUpdatedRange]);

  useEffect(() => {
    setCurrentPage(1);
  }, [searchTerm, sortKey, selectedTags, selectedProviders, selectedUpdatedRange, hideEmptySessions, filteredSessions.length]);

  const totalPages = Math.ceil(filteredSessions.length / SESSIONS_PAGE_SIZE);
  const paginatedSessions = useMemo(() => {
    const startIndex = (currentPage - 1) * SESSIONS_PAGE_SIZE;
    return filteredSessions.slice(startIndex, startIndex + SESSIONS_PAGE_SIZE);
  }, [currentPage, filteredSessions]);

  const pageStart = filteredSessions.length === 0 ? 0 : (currentPage - 1) * SESSIONS_PAGE_SIZE + 1;
  const pageEnd = filteredSessions.length === 0
    ? 0
    : Math.min(currentPage * SESSIONS_PAGE_SIZE, filteredSessions.length);

  return (
    <section className="project-page">
      <div className="sticky-project-header">
        <div className="sticky-project-shell">
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
            <button
              type="button"
              className={`sub-tab-item ${activeSubTab === "analytics" ? "sub-tab-item--active" : ""}`}
              onClick={() => setActiveSubTab("analytics")}
            >
              {t("project.subTab.analytics")}
            </button>
            {openDetailKeys.map((detailKey) => {
              const detail = parseDetailTabKey(detailKey);
              if (!detail) return null;
              const session = project.sessions.find((s) => s.id === detail.sessionId);
              if (!session) return null;
              const tabTitle = session.summary?.trim() || session.id.slice(0, 8);
              const prefix = detail.kind === "plan" ? t("plan.tab") : t("session.todos.tab");
              return (
                <div
                  key={detailKey}
                  className={`sub-tab-item sub-tab-item--closeable ${activeSubTab === detailKey ? "sub-tab-item--active" : ""}`}
                >
                  <button
                    type="button"
                    className="sub-tab-label"
                    onClick={() => {
                      setActiveSubTab(detailKey);
                      onActivePlanChange(detail.kind === "plan" ? detail.sessionId : null);
                    }}
                  >
                    {prefix} · {tabTitle}
                  </button>
                  <button
                    type="button"
                    className="sub-tab-close"
                    onClick={() => handleCloseDetailSubTab(detailKey)}
                    aria-label={`${t("tabs.close")} ${tabTitle}`}
                  >
                    ×
                  </button>
                </div>
              );
            })}
          </div>

          {activeSubTab === "sessions" ? (
            <div className="sticky-filter-header">
              <section className="toolbar-card">
                <div className="filter-bar-summary">
                  <ProjectStatsBanner
                    sessions={filteredSessions}
                    sessionStats={sessionStats}
                    sessionStatsLoading={sessionStatsLoading}
                  />

                  <div className="filter-bar-actions">
                    {availableProviders.length > 1 ? (
                      <>
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
                              {getProviderLabel(provider)}
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
                      disabled={totalEmptySessions === 0}
                      onClick={onDeleteEmptySessions}
                    >
                      <DeleteIcon size={16} />
                    </button>

                    <button
                      type="button"
                      className={`icon-button filter-toggle-btn ${isFilterExpanded ? "filter-toggle-btn--active" : ""}`}
                      title={t("session.filter.toggle")}
                      aria-label={t("session.filter.toggle")}
                      aria-expanded={isFilterExpanded}
                      onClick={handleToggleFilterExpanded}
                    >
                      <span className={`filter-toggle-chevron ${isFilterExpanded ? "filter-toggle-chevron--open" : ""}`}>▾</span>
                    </button>
                  </div>
                </div>

                {isFilterExpanded ? (
                  <div className="filter-bar">
                    <label className="field-group compact-field" style={{ flex: 2, minWidth: "160px" }}>
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

                    <label className="field-group compact-field">
                      <span>{t("session.filter.updatedRange")}</span>
                      <select
                        value={selectedUpdatedRange}
                        onChange={(event) => setSelectedUpdatedRange(event.currentTarget.value as SessionUpdatedRange)}
                      >
                        <option value="all">{t("session.filter.updatedRange.all")}</option>
                        <option value="week">{t("session.filter.updatedRange.week")}</option>
                        <option value="month">{t("session.filter.updatedRange.month")}</option>
                      </select>
                    </label>

                    <button
                      type="button"
                      className={`tag-filter-chip ${showArchived ? "active" : ""}`}
                      onClick={() => onToggleArchived(!showArchived)}
                    >
                      {t("project.showArchivedToggle")}
                    </button>

                    <button
                      type="button"
                      className={`tag-filter-chip ${hideEmptySessions ? "active" : ""}`}
                      onClick={() => onHideEmptySessionsChange(!hideEmptySessions)}
                    >
                      {t("session.filter.hideEmpty")}
                      {hideEmptySessions && hiddenCount > 0 ? (
                        <span className="hidden-count-hint">
                          {" "}({t("session.filter.hiddenCount").replace("{count}", String(hiddenCount))})
                        </span>
                      ) : null}
                    </button>
                  </div>
                ) : null}
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
            </div>
          ) : null}
        </div>
      </div>

      {activeSubTab === "sessions" ? (
        <div className="session-content">
          {!sessionsLoading ? (
            <div className="session-results-bar">
              <span className="session-results-summary">
                {t("session.pagination.summary")
                  .replace("{start}", String(pageStart))
                  .replace("{end}", String(pageEnd))
                  .replace("{total}", String(filteredSessions.length))}
              </span>
              {totalPages > 1 ? (
                <div className="session-pagination">
                  <button
                    type="button"
                    className="ghost-button session-pagination-btn"
                    disabled={currentPage <= 1}
                    onClick={() => setCurrentPage((page) => Math.max(1, page - 1))}
                  >
                    {t("session.pagination.prev")}
                  </button>
                  <span className="session-pagination-label">
                    {t("session.pagination.page")
                      .replace("{current}", String(currentPage))
                      .replace("{total}", String(totalPages))}
                  </span>
                  <button
                    type="button"
                    className="ghost-button session-pagination-btn"
                    disabled={currentPage >= totalPages}
                    onClick={() => setCurrentPage((page) => Math.min(totalPages, page + 1))}
                  >
                    {t("session.pagination.next")}
                  </button>
                </div>
              ) : null}
            </div>
          ) : null}

          <div className="session-list">
          {sessionsLoading ? (
            <>
              <div className="skeleton-card" />
              <div className="skeleton-card" />
              <div className="skeleton-card" />
            </>
          ) : paginatedSessions.length === 0 ? (
            <div className="session-list-empty">
              {t("session.filter.noResults")}
            </div>
          ) : (
            paginatedSessions.map((session) => (
              <SessionCard
                key={session.id}
                session={session}
                onOpenTerminal={onOpenTerminal}
                onCopyCommand={onCopyCommand}
                onEditNotes={onEditNotes}
                onEditTags={onEditTags}
                onOpenPlan={handleOpenPlanSubTab}
                onOpenTodos={handleOpenTodosSubTab}
                onArchive={onArchive}
                onUnarchive={onUnarchive}
                onDelete={onDelete}
                stats={sessionStats[session.id]}
                statsLoading={Boolean(sessionStatsLoading[session.id])}
                todos={sessionTodos[session.id] ?? []}
                todosLoading={Boolean(sessionTodosLoading[session.id])}
                activityStatus={activityStatusMap.get(session.id)}
                onOpenInTool={onOpenInTool}
                onFocusTerminal={onFocusTerminal}
                defaultLauncher={defaultLauncher}
                toolAvailability={toolAvailability}
                isLauncherOpen={openLauncherSessionId === session.id}
                onToggleLauncher={() => handleToggleLauncher(session.id)}
              />
            ))
          )}
          </div>
        </div>
      ) : activeSubTab === "analytics" ? (
        <ProjectAnalyticsTab
          sessions={project.sessions}
          sessionStats={sessionStats}
          onFetchAnalytics={onFetchAnalytics}
        />
      ) : activeSubTab === "plans-specs" ? (
        <PlansSpecsView
          sisyphusData={sisyphusData}
          openspecData={openspecData}
          isLoading={plansSpecsLoading}
          isRefreshing={plansSpecsRefreshing}
          onReadFileContent={onReadFileContent}
          onReadOpenspecFile={onReadOpenspecFile}
          onWriteOpenspecFile={onWriteOpenspecFile}
          onRefresh={onRefreshPlansSpecs}
          refreshToken={plansSpecsRefreshToken}
          projectCwd={project.pathLabel}
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
              onClose={() => handleCloseDetailSubTab(activeSubTab)}
            />
          );
        })()
      ) : activeSubTab.startsWith("todos:") ? (
        (() => {
          const sessionId = activeSubTab.replace("todos:", "");
          const todoSession = project.sessions.find((s) => s.id === sessionId);
          if (!todoSession) return null;
          return (
            <SessionTodosTab
              session={todoSession}
              todos={sessionTodos[todoSession.id] ?? []}
              isLoading={Boolean(sessionTodosLoading[todoSession.id])}
              onClose={() => handleCloseDetailSubTab(activeSubTab)}
            />
          );
        })()
      ) : null}
    </section>
  );
}
