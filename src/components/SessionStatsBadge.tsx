import { useI18n } from "../i18n/I18nProvider";
import type { SessionInfo, SessionStats, SessionTodo } from "../types";
import { formatDuration } from "../utils/formatDuration";

type Props = {
  session: SessionInfo;
  stats?: SessionStats;
  isLoading: boolean;
  todos: SessionTodo[];
  todosLoading: boolean;
  onOpenTodos: (session: SessionInfo) => void;
};

function formatCompactNumber(value: number) {
  if (value >= 1_000_000) return `${(value / 1_000_000).toFixed(value >= 10_000_000 ? 0 : 1)}M`;
  if (value >= 1_000) return `${(value / 1_000).toFixed(value >= 10_000 ? 0 : 1)}K`;
  return String(value);
}

function getTodoStatusLabel(
  t: (key: string) => string,
  status: string,
) {
  switch (status) {
    case "pending":
      return t("session.todos.status.pending");
    case "in_progress":
      return t("session.todos.status.in_progress");
    case "done":
      return t("session.todos.status.done");
    case "blocked":
      return t("session.todos.status.blocked");
    default:
      return status.replace(/_/g, " ");
  }
}

export function SessionStatsBadge({
  session,
  stats,
  isLoading,
  todos,
  todosLoading,
  onOpenTodos,
}: Props) {
  const { t } = useI18n();
  const hasStats = Boolean(stats && (stats.outputTokens > 0 || stats.inputTokens > 0 || stats.interactionCount > 0 || stats.durationMinutes > 0));

  const todoCounts = todos.reduce<Record<string, number>>((acc, todo) => {
    const key = todo.status?.trim() || "pending";
    acc[key] = (acc[key] ?? 0) + 1;
    return acc;
  }, {});
  const builtInOrder = ["in_progress", "pending", "done", "blocked"];
  const builtInEntries = builtInOrder
    .filter((status) => (todoCounts[status] ?? 0) > 0)
    .map((status) => ({ status, count: todoCounts[status] }));
  const customEntries = Object.entries(todoCounts)
    .filter(([status, count]) => count > 0 && !builtInOrder.includes(status))
    .sort(([left], [right]) => left.localeCompare(right))
    .map(([status, count]) => ({ status, count }));
  const statusEntries = [...builtInEntries, ...customEntries];
  const shouldShowTodos = todos.length > 0;
  const todoSummary = shouldShowTodos
    ? [
      t("session.todos.badge.total").replace("{count}", String(todos.length)),
      ...statusEntries.map(({ status, count }) => `${getTodoStatusLabel(t as (key: string) => string, status)} ${count}`),
    ].join(" · ")
    : null;

  if (isLoading && !shouldShowTodos && !todosLoading) {
    return <div className="stats-badge-row"><span className="stats-badge stats-badge-loading">...</span></div>;
  }

  if (!hasStats && !shouldShowTodos && !todosLoading) {
    return <div className="stats-badge-row"><span className="stats-badge">{t("stats.noData")}</span></div>;
  }

  return (
    <div className="stats-badge-row">
      {hasStats && stats ? (
        <>
          <span className="stats-badge">{formatCompactNumber(stats.interactionCount)} {t("stats.turns")}</span>
          <span className="stats-badge">{formatCompactNumber(stats.outputTokens + stats.inputTokens)} {t("stats.tokens")}</span>
          <span className="stats-badge">{formatDuration(stats.durationMinutes)}</span>
          {stats.isLive ? <span className="stats-badge stats-badge-live"><span className="stats-badge-live-dot" />LIVE</span> : null}
        </>
      ) : null}
      {shouldShowTodos && todoSummary ? (
        <button
          type="button"
          className="stats-badge stats-badge-button stats-badge-entry stats-badge-todo-summary"
          onClick={() => onOpenTodos(session)}
        >
          {todoSummary}
        </button>
      ) : null}
    </div>
  );
}
