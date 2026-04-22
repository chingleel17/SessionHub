import { useCallback, useEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { useI18n } from "../i18n/I18nProvider";
import type { IdeLauncherType, ProjectGroup, SessionActivityStatus, SessionInfo, ToolAvailability } from "../types";

type Props = {
  sessionsIsLoading: boolean;
  sessionsIsFetching: boolean;
  sessionsIsError: boolean;
  sessionsError: unknown;
  groupedProjects: ProjectGroup[];
  recentSessions: SessionInfo[];
  dashboardPeriod: "week" | "month";
  onPeriodChange: (period: "week" | "month") => void;
  filteredTotalOutputTokens: number;
  filteredTotalInteractions: number;
  filteredTotalCost: number;
  onOpenProject: (projectKey: string) => void;
  onOpenRecentSession: (session: SessionInfo) => void;
  activityStatusMap: Map<string, SessionActivityStatus>;
  onOpenInTool: (session: SessionInfo, tool: IdeLauncherType) => void;
  onFocusTerminal: (session: SessionInfo) => void;
  defaultLauncher: string | null;
  toolAvailability: ToolAvailability | null;
  viewMode: "list" | "kanban";
  onViewModeChange: (mode: "list" | "kanban") => void;
};

const RECENT_TITLE_MAX_LEN = 80;
const PROJECT_LAST_SESSION_MAX_LEN = 60;
const DONE_INITIAL_LIMIT = 10;
const DONE_LOAD_MORE_STEP = 10;
const KANBAN_COL_WIDTHS_KEY = "sessionhub.kanban.columnWidths";

function truncate(text: string, maxLen: number): string {
  return text.length > maxLen ? `${text.slice(0, maxLen)}…` : text;
}

function getSessionTitle(session: SessionInfo) {
  return session.summary?.trim() || session.id;
}

function getProjectShortName(cwd?: string | null): string | null {
  if (!cwd) return null;
  const parts = cwd.replace(/\//g, "\\").split("\\").filter(Boolean);
  return parts[parts.length - 1] ?? null;
}

function formatCompactNumber(value: number) {
  if (value >= 1_000_000) return `${(value / 1_000_000).toFixed(value >= 10_000_000 ? 0 : 1)}M`;
  if (value >= 1_000) return `${(value / 1_000).toFixed(value >= 10_000 ? 0 : 1)}K`;
  return String(value);
}

const LAUNCHER_OPTIONS: { type: IdeLauncherType; label: string; icon: string; availKey?: keyof ToolAvailability }[] = [
  { type: "terminal", label: "Terminal", icon: ">_" },
  { type: "copilot", label: "Copilot", icon: "C", availKey: "copilot" },
  { type: "opencode", label: "OpenCode", icon: "O", availKey: "opencode" },
  { type: "gemini", label: "Gemini", icon: "G", availKey: "gemini" },
  { type: "vscode", label: "VS Code", icon: "⌨", availKey: "vscode" },
  { type: "explorer", label: "Explorer", icon: "📁" },
];

function getDefaultLauncher(
  session: SessionInfo,
  toolAvailability: ToolAvailability | null,
  globalDefault: string | null,
): IdeLauncherType {
  if (globalDefault) {
    const ga = globalDefault as IdeLauncherType;
    const opt = LAUNCHER_OPTIONS.find((o) => o.type === ga);
    if (opt?.availKey && toolAvailability && !toolAvailability[opt.availKey]) return "terminal";
    return ga;
  }
  if (session.provider === "copilot") {
    if (!toolAvailability || toolAvailability.copilot) return "copilot";
    return "terminal";
  }
  if (session.provider === "opencode") {
    if (!toolAvailability || toolAvailability.opencode) return "opencode";
    return "terminal";
  }
  return "terminal";
}

function getActivityStatusLabel(t: (k: string) => string, status?: SessionActivityStatus): { label: string; cls: string } {
  if (!status) return { label: t("dashboard.kanban.status.idle"), cls: "kanban-status-idle" };
  switch (status.status) {
    case "active": return { label: t(`dashboard.kanban.detail.${status.detail ?? "working"}`), cls: "kanban-status-active" };
    case "waiting": return { label: t("dashboard.kanban.status.waiting"), cls: "kanban-status-waiting" };
    case "done": return { label: t("dashboard.kanban.status.done"), cls: "kanban-status-done" };
    default: return { label: t("dashboard.kanban.status.idle"), cls: "kanban-status-idle" };
  }
}

function loadColumnWidths(): number[] {
  try {
    const stored = localStorage.getItem(KANBAN_COL_WIDTHS_KEY);
    if (stored) {
      const parsed = JSON.parse(stored) as unknown;
      if (Array.isArray(parsed) && parsed.length === 4 && (parsed as unknown[]).every((n) => typeof n === "number")) {
        return parsed as number[];
      }
    }
  } catch { /* ignore */ }
  return [25, 25, 25, 25];
}

function saveColumnWidths(widths: number[]) {
  try { localStorage.setItem(KANBAN_COL_WIDTHS_KEY, JSON.stringify(widths)); } catch { /* ignore */ }
}

// ─── KanbanProjectCard ────────────────────────────────────────────────────────

const SESSIONS_PER_PROJECT_CARD = 3;

function KanbanProjectCard({
  projectName,
  projectKey,
  sessions,
  activityStatusMap,
  onOpenInTool,
  onFocusTerminal,
  onOpenProject,
  defaultLauncher,
  toolAvailability,
  openMenu,
  onToggleMenu,
}: {
  projectName: string;
  projectKey: string;
  sessions: SessionInfo[];
  activityStatusMap: Map<string, SessionActivityStatus>;
  onOpenInTool: (session: SessionInfo, tool: IdeLauncherType) => void;
  onFocusTerminal: (session: SessionInfo) => void;
  onOpenProject: (key: string) => void;
  defaultLauncher: string | null;
  toolAvailability: ToolAvailability | null;
  openMenu: { sessionId: string; rect: DOMRect } | null;
  onToggleMenu: (sessionId: string, rect: DOMRect) => void;
}) {
  const { t } = useI18n();

  const providers = [...new Set(sessions.map((s) => s.provider))];
  const lastUpdated = sessions
    .map((s) => s.updatedAt)
    .filter((v): v is string => Boolean(v))
    .sort()
    .pop();

  const visibleSessions = sessions.slice(0, SESSIONS_PER_PROJECT_CARD);
  const hiddenCount = sessions.length - visibleSessions.length;

  const isToolAvailable = (opt: typeof LAUNCHER_OPTIONS[number]) => {
    if (!opt.availKey || !toolAvailability) return true;
    return toolAvailability[opt.availKey];
  };

  return (
    <div className="kanban-project-card">
      <div
        className="kanban-project-card-header"
        role="button"
        tabIndex={0}
        onClick={() => onOpenProject(projectKey)}
        onKeyDown={(e) => { if (e.key === "Enter" || e.key === " ") onOpenProject(projectKey); }}
        title={t("dashboard.kanban.openProject")}
      >
        <strong className="kanban-project-name">{projectName}</strong>
        <span className="kanban-project-count">{sessions.length}</span>
        <div className="kanban-project-providers">
          {providers.map((p) => (
            <span key={p} className={`provider-tag provider-tag--${p}`}>
              {p === "opencode" ? "OC" : "CP"}
            </span>
          ))}
        </div>
        {lastUpdated ? <span className="kanban-project-time">{lastUpdated.slice(0, 10)}</span> : null}
        <span className="kanban-project-goto">›</span>
      </div>

      <div className="kanban-project-sessions">
        {visibleSessions.map((session) => {
          const activityStatus = activityStatusMap.get(session.id);
          const { label, cls } = getActivityStatusLabel(t as (k: string) => string, activityStatus);
          const sessionTitle = session.summary?.trim() || session.id;
          const launcher = getDefaultLauncher(session, toolAvailability, defaultLauncher);

          return (
            <div key={session.id} className="kanban-session-row">
              <span className={`kanban-activity-badge ${cls}`}>{label}</span>
              <span className="kanban-session-summary" title={sessionTitle}>
                {truncate(sessionTitle, 60)}
              </span>
              <div className="kanban-session-actions">
                <button
                  type="button"
                  className="kanban-action-btn"
                  onClick={(e) => { e.stopPropagation(); onOpenInTool(session, launcher); }}
                  title={t("session.actions.openTool")}
                >
                  ▶
                </button>
                <button
                  type="button"
                  className="kanban-action-btn"
                  onClick={(e) => {
                    e.stopPropagation();
                    onToggleMenu(session.id, e.currentTarget.getBoundingClientRect());
                  }}
                  title={t("session.actions.chooseTool")}
                >
                  ⋯
                </button>
                {openMenu?.sessionId === session.id ? createPortal(
                  <div
                    data-launcher-menu="true"
                    className="kanban-launcher-menu"
                    style={{
                      position: "fixed",
                      top: openMenu.rect.bottom + 2,
                      left: openMenu.rect.left,
                      zIndex: 9999,
                    }}
                  >
                    {LAUNCHER_OPTIONS.map((opt) => {
                      const available = isToolAvailable(opt);
                      return (
                        <button
                          key={opt.type}
                          type="button"
                          className={`kanban-launcher-option${!available ? " kanban-launcher-option--disabled" : ""}${opt.type === launcher ? " kanban-launcher-option--default" : ""}`}
                          disabled={!available}
                          onClick={(e) => {
                            e.stopPropagation();
                            onOpenInTool(session, opt.type);
                            onToggleMenu(session.id, e.currentTarget.getBoundingClientRect());
                          }}
                        >
                          <span className="launcher-option-icon">{opt.icon}</span>
                          {opt.label}
                          {opt.type === launcher ? <span className="launcher-default-tag"> ★</span> : null}
                          {!available ? <span className="launcher-option-unavail"> —</span> : null}
                        </button>
                      );
                    })}
                  </div>,
                  document.body
                ) : null}
                <button
                  type="button"
                  className="kanban-action-btn"
                  onClick={(e) => { e.stopPropagation(); onFocusTerminal(session); }}
                  title={t("session.actions.focusTerminal")}
                >
                  ⊙
                </button>
              </div>
            </div>
          );
        })}
        {hiddenCount > 0 ? (
          <button
            type="button"
            className="kanban-project-more-btn"
            onClick={() => onOpenProject(projectKey)}
          >
            +{hiddenCount} {t("dashboard.kanban.moreSessions")}
          </button>
        ) : null}
      </div>
    </div>
  );
}

// ─── KanbanBoard ──────────────────────────────────────────────────────────────

type ProjectBucket = { projectName: string; projectKey: string; sessions: SessionInfo[] };

function KanbanBoard({
  groupedProjects,
  activityStatusMap,
  onOpenInTool,
  onFocusTerminal,
  onOpenProject,
  defaultLauncher,
  toolAvailability,
}: {
  groupedProjects: ProjectGroup[];
  activityStatusMap: Map<string, SessionActivityStatus>;
  onOpenInTool: (session: SessionInfo, tool: IdeLauncherType) => void;
  onFocusTerminal: (session: SessionInfo) => void;
  onOpenProject: (key: string) => void;
  defaultLauncher: string | null;
  toolAvailability: ToolAvailability | null;
}) {
  const { t } = useI18n();
  const [columnWidths, setColumnWidths] = useState<number[]>(loadColumnWidths);
  const [doneLimit, setDoneLimit] = useState(DONE_INITIAL_LIMIT);
  const [openMenu, setOpenMenu] = useState<{ sessionId: string; rect: DOMRect } | null>(null);
  const boardRef = useRef<HTMLDivElement>(null);
  const widthsRef = useRef(columnWidths);
  const dragRef = useRef<{ colIndex: number; startX: number; startWidths: number[] } | null>(null);

  useEffect(() => { widthsRef.current = columnWidths; }, [columnWidths]);
  useEffect(() => { saveColumnWidths(columnWidths); }, [columnWidths]);

  const handleToggleMenu = useCallback((sessionId: string, rect: DOMRect) => {
    setOpenMenu((prev) => prev?.sessionId === sessionId ? null : { sessionId, rect });
  }, []);

  const handleCloseMenu = useCallback(() => setOpenMenu(null), []);

  // 點擊選單外部關閉
  useEffect(() => {
    if (!openMenu) return;
    const handler = (e: MouseEvent) => {
      if (!(e.target as Element).closest("[data-launcher-menu]")) {
        handleCloseMenu();
      }
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, [openMenu, handleCloseMenu]);

  // 捲動時關閉，避免選單位置偏移（用 capture 捕捉各欄內部的 overflow-y: auto 捲動）
  useEffect(() => {
    if (!openMenu) return;
    document.addEventListener("scroll", handleCloseMenu, true);
    return () => document.removeEventListener("scroll", handleCloseMenu, true);
  }, [openMenu, handleCloseMenu]);

  const allSessions = groupedProjects.flatMap((p) => p.sessions);

  const columns: { key: string; label: string; buckets: ProjectBucket[] }[] = [
    { key: "active", label: t("dashboard.kanban.column.active"), buckets: [] },
    { key: "waiting", label: t("dashboard.kanban.column.waiting"), buckets: [] },
    { key: "idle", label: t("dashboard.kanban.column.idle"), buckets: [] },
    { key: "done", label: t("dashboard.kanban.column.done"), buckets: [] },
  ];

  const uncategorized = t("dashboard.kanban.uncategorized");
  for (const s of allSessions) {
    const statusKey = s.isArchived ? "done" : (activityStatusMap.get(s.id)?.status ?? "idle");
    const col = columns.find((c) => c.key === statusKey) ?? columns[2];
    const projectName = getProjectShortName(s.cwd) ?? uncategorized;
    const projectKey = s.cwd ? s.cwd.toLowerCase() : uncategorized;
    let bucket = col.buckets.find((b) => b.projectKey === projectKey);
    if (!bucket) { bucket = { projectName, projectKey, sessions: [] }; col.buckets.push(bucket); }
    bucket.sessions.push(s);
  }

  for (const col of columns) {
    col.buckets.sort((a, b) => {
      const aTime = a.sessions.map((s) => s.updatedAt).filter(Boolean).sort().pop() ?? "";
      const bTime = b.sessions.map((s) => s.updatedAt).filter(Boolean).sort().pop() ?? "";
      return bTime.localeCompare(aTime);
    });
  }

  const colSessionCounts = columns.map((c) =>
    c.buckets.reduce((sum, b) => sum + b.sessions.length, 0),
  );

  const handleResizerMouseDown = useCallback((colIndex: number, e: React.MouseEvent) => {
    e.preventDefault();
    dragRef.current = { colIndex, startX: e.clientX, startWidths: [...widthsRef.current] };

    const onMouseMove = (me: MouseEvent) => {
      if (!dragRef.current || !boardRef.current) return;
      const { colIndex: ci, startX, startWidths } = dragRef.current;
      const boardWidth = boardRef.current.getBoundingClientRect().width;
      const deltaPercent = ((me.clientX - startX) / boardWidth) * 100;
      const MIN = 10;
      const newWidths = [...startWidths];
      const newLeft = Math.max(MIN, Math.min(startWidths[ci] + deltaPercent, 100 - MIN * (startWidths.length - 1 - ci)));
      const diff = newLeft - startWidths[ci];
      newWidths[ci] = newLeft;
      newWidths[ci + 1] = Math.max(MIN, startWidths[ci + 1] - diff);
      setColumnWidths(newWidths);
    };

    const onMouseUp = () => {
      dragRef.current = null;
      document.removeEventListener("mousemove", onMouseMove);
      document.removeEventListener("mouseup", onMouseUp);
    };

    document.addEventListener("mousemove", onMouseMove);
    document.addEventListener("mouseup", onMouseUp);
  }, []);

  return (
    <div ref={boardRef} className="kanban-board" style={{ overflow: "hidden" }}>
      {columns.flatMap((col, idx) => {
        const isDone = col.key === "done";
        const visibleBuckets = isDone ? col.buckets.slice(0, doneLimit) : col.buckets;
        const hasMore = isDone && col.buckets.length > doneLimit;

        const colEl = (
          <div
            key={col.key}
            className="kanban-column"
            style={{ width: `${columnWidths[idx]}%`, flexShrink: 0, flexGrow: 0, minWidth: 0, overflow: "hidden" }}
          >
            <div className="kanban-column-header">
              <span className="kanban-column-title">{col.label}</span>
              <span className="kanban-column-count">{colSessionCounts[idx]}</span>
            </div>
            <div className="kanban-column-cards">
              {visibleBuckets.map((bucket) => (
                <KanbanProjectCard
                  key={bucket.projectKey}
                  projectName={bucket.projectName}
                  projectKey={bucket.projectKey}
                  sessions={bucket.sessions}
                  activityStatusMap={activityStatusMap}
                  onOpenInTool={onOpenInTool}
                  onFocusTerminal={onFocusTerminal}
                  onOpenProject={onOpenProject}
                  defaultLauncher={defaultLauncher}
                  toolAvailability={toolAvailability}
                  openMenu={openMenu}
                  onToggleMenu={handleToggleMenu}
                />
              ))}
              {visibleBuckets.length === 0 ? (
                <p className="kanban-empty">{t("dashboard.kanban.empty")}</p>
              ) : null}
              {hasMore ? (
                <button
                  type="button"
                  className="kanban-load-more-btn"
                  onClick={() => setDoneLimit((v) => v + DONE_LOAD_MORE_STEP)}
                >
                  {t("dashboard.kanban.loadMore")} ({col.buckets.length - doneLimit})
                </button>
              ) : null}
            </div>
          </div>
        );

        if (idx < columns.length - 1) {
          return [
            colEl,
            <div
              key={`resizer-${idx}`}
              className="kanban-column-resizer"
              onMouseDown={(e) => handleResizerMouseDown(idx, e)}
            />,
          ];
        }
        return [colEl];
      })}
    </div>
  );
}

export function DashboardView({
  sessionsIsLoading,
  sessionsIsFetching,
  sessionsIsError,
  sessionsError,
  groupedProjects,
  recentSessions,
  dashboardPeriod,
  onPeriodChange,
  filteredTotalOutputTokens,
  filteredTotalInteractions,
  filteredTotalCost,
  onOpenProject,
  onOpenRecentSession,
  activityStatusMap,
  onOpenInTool,
  onFocusTerminal,
  defaultLauncher,
  toolAvailability,
  viewMode,
  onViewModeChange,
}: Props) {
  const { t } = useI18n();

  const totalSessionCount = groupedProjects.reduce((sum, p) => sum + p.sessions.length, 0);
  const activeProjectCount = groupedProjects.length;
  const allSessions = groupedProjects.flatMap((p) => p.sessions);
  const archivedCount = allSessions.filter((s) => s.isArchived).length;
  const parseErrorCount = allSessions.filter((s) => s.parseError).length;

  const loading = sessionsIsLoading;
  const fetching = sessionsIsFetching;

  return (
    <section className="dashboard-layout">
      {/* Stat cards row — toggles on right */}
      <div className="stat-cards-row">
        <div className="stat-card stat-card--sessions">
          <span className="stat-card-icon">🗂</span>
          <strong className="stat-card-value">{loading ? "…" : formatCompactNumber(totalSessionCount)}</strong>
          <span className="stat-card-label">{t("dashboard.stats.totalSessions")}</span>
        </div>
        <div className="stat-card stat-card--projects">
          <span className="stat-card-icon">📁</span>
          <strong className="stat-card-value">{loading ? "…" : activeProjectCount}</strong>
          <span className="stat-card-label">{t("dashboard.stats.activeProjects")}</span>
        </div>
        <div className="stat-card stat-card--archived">
          <span className="stat-card-icon">📦</span>
          <strong className="stat-card-value">{loading ? "…" : archivedCount}</strong>
          <span className="stat-card-label">{t("dashboard.stats.archivedSessions")}</span>
        </div>
        {parseErrorCount > 0 ? (
          <div className="stat-card stat-card--errors">
            <span className="stat-card-icon">⚠</span>
            <strong className="stat-card-value">{loading ? "…" : parseErrorCount}</strong>
            <span className="stat-card-label">{t("dashboard.stats.parseErrors")}</span>
          </div>
        ) : null}
        <div className="stat-card stat-card--tokens">
          <span className="stat-card-icon">🪙</span>
          <strong className="stat-card-value">{loading ? "…" : formatCompactNumber(filteredTotalOutputTokens)}</strong>
          <span className="stat-card-label">{t("dashboard.stats.totalTokens")}</span>
        </div>
        <div className="stat-card stat-card--interactions">
          <span className="stat-card-icon">💬</span>
          <strong className="stat-card-value">{loading ? "…" : formatCompactNumber(filteredTotalInteractions)}</strong>
          <span className="stat-card-label">{t("dashboard.stats.totalInteractions")}</span>
        </div>
        <div className="stat-card stat-card--tokens">
          <span className="stat-card-icon">🧾</span>
          <strong className="stat-card-value">{loading ? "…" : filteredTotalCost.toFixed(2).replace(/\.00$/, "")}</strong>
          <span className="stat-card-label">{t("dashboard.stats.totalCost")}</span>
        </div>

        {/* Toggles on the far right, stacked vertically */}
        <div className="stat-card-toggles">
          <div className="view-mode-toggle">
            <button
              type="button"
              className={`period-toggle-btn${viewMode === "kanban" ? " active" : ""}`}
              onClick={() => onViewModeChange("kanban")}
            >
              {t("dashboard.viewMode.kanban")}
            </button>
            <button
              type="button"
              className={`period-toggle-btn${viewMode === "list" ? " active" : ""}`}
              onClick={() => onViewModeChange("list")}
            >
              {t("dashboard.viewMode.list")}
            </button>
          </div>
          <div className="period-toggle">
            <button
              type="button"
              className={`period-toggle-btn${dashboardPeriod === "week" ? " active" : ""}`}
              onClick={() => onPeriodChange("week")}
            >
              {t("dashboard.stats.period.week")}
            </button>
            <button
              type="button"
              className={`period-toggle-btn${dashboardPeriod === "month" ? " active" : ""}`}
              onClick={() => onPeriodChange("month")}
            >
              {t("dashboard.stats.period.month")}
            </button>
          </div>
        </div>
      </div>

      {sessionsIsError ? (
        <article className="info-card status-card status-card-error">
          <h3>{t("dashboard.status.errorTitle")}</h3>
          <p className="placeholder-copy">
            {sessionsError instanceof Error
              ? sessionsError.message
              : t("dashboard.status.errorDescription")}
          </p>
        </article>
      ) : null}

      {viewMode === "kanban" ? (
        <KanbanBoard
          groupedProjects={groupedProjects}
          activityStatusMap={activityStatusMap}
          onOpenInTool={onOpenInTool}
          onFocusTerminal={onFocusTerminal}
          onOpenProject={onOpenProject}
          defaultLauncher={defaultLauncher}
          toolAvailability={toolAvailability}
        />
      ) : (
        <section className="content-grid">
          <article className="info-card">
            <div className="section-heading">
              <h3>{t("dashboard.projects.title")}</h3>
              <span>{t("dashboard.projects.subtitle")}</span>
            </div>

            <div className="project-list">
              {groupedProjects.map((project) => {
                const lastSession = project.sessions[0];
                const lastSessionTitle = lastSession?.summary?.trim() || null;
                return (
                  <button
                    key={project.key}
                    type="button"
                    className="project-item"
                    onClick={() => onOpenProject(project.key)}
                  >
                    <div className="project-item-info">
                      <strong>{project.title}</strong>
                      <p>{project.pathLabel}</p>
                      {lastSessionTitle ? (
                        <p className="project-last-session-title">
                          {truncate(lastSessionTitle, PROJECT_LAST_SESSION_MAX_LEN)}
                        </p>
                      ) : null}
                    </div>

                    <div className="project-meta">
                      <span>
                        {project.sessions.length} {t("dashboard.projects.sessionCountSuffix")}
                      </span>
                      <span>{project.updatedAtLabel}</span>
                    </div>
                  </button>
                );
              })}
            </div>
          </article>

          <article className="info-card">
            <div className="section-heading">
              <h3>{t("dashboard.recent.title")}</h3>
              <span className="recent-subtitle-row">
                <span>{fetching ? t("dashboard.status.scanning") : t("dashboard.recent.subtitle")}</span>
              </span>
            </div>

            <ul className="feature-list feature-list-tight">
              {recentSessions.map((session) => {
                const fullTitle = getSessionTitle(session);
                const projectName = getProjectShortName(session.cwd);
                return (
                  <li key={session.id}>
                    <div className="recent-session-item">
                      <button
                        type="button"
                        className="inline-link"
                        title={fullTitle}
                        onClick={() => onOpenRecentSession(session)}
                      >
                        {truncate(fullTitle, RECENT_TITLE_MAX_LEN)}
                      </button>
                      {projectName ? (
                        <span className="recent-session-project">{projectName}</span>
                      ) : null}
                    </div>
                  </li>
                );
              })}
            </ul>
          </article>
        </section>
      )}
    </section>
  );
}