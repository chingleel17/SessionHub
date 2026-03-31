import { useMemo, useState } from "react";
import { useI18n } from "../i18n/I18nProvider";
import type { ProjectGroup, SessionInfo, SessionStats, SortKey } from "../types";
import { DeleteIcon, PinIcon, UnpinIcon } from "./Icons";
import { ProjectStatsBanner } from "./ProjectStatsBanner";
import { SessionCard } from "./SessionCard";

type Props = {
  project: ProjectGroup;
  showArchived: boolean;
  onToggleArchived: (value: boolean) => void;
  onOpenTerminal: (session: SessionInfo) => void;
  onCopyCommand: (sessionId: string) => void;
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
};

function filterAndSortSessions(
  sessions: SessionInfo[],
  searchTerm: string,
  sortKey: SortKey,
  selectedTags: string[],
  hideEmpty: boolean,
) {
  const normalizedSearchTerm = searchTerm.trim().toLowerCase();

  const filtered = sessions.filter((session) => {
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
}: Props) {
  const { t } = useI18n();
  const [searchTerm, setSearchTerm] = useState("");
  const [sortKey, setSortKey] = useState<SortKey>("updatedAt");
  const [selectedTags, setSelectedTags] = useState<string[]>([]);
  const [hideEmpty, setHideEmpty] = useState(false);

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
    () => filterAndSortSessions(project.sessions, searchTerm, sortKey, selectedTags, hideEmpty),
    [project.sessions, searchTerm, sortKey, selectedTags, hideEmpty],
  );

  const hiddenCount = useMemo(() => {
    const withoutHide = filterAndSortSessions(project.sessions, searchTerm, sortKey, selectedTags, false);
    const withHide = filterAndSortSessions(project.sessions, searchTerm, sortKey, selectedTags, true);
    return withoutHide.length - withHide.length;
  }, [project.sessions, searchTerm, sortKey, selectedTags]);

  return (
    <section className="project-page">
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
        {filteredSessions.map((session) => (
          <SessionCard
            key={session.id}
            session={session}
            onOpenTerminal={onOpenTerminal}
            onCopyCommand={onCopyCommand}
            onEditNotes={onEditNotes}
            onEditTags={onEditTags}
            onOpenPlan={onOpenPlan}
            onArchive={onArchive}
            onUnarchive={onUnarchive}
            onDelete={onDelete}
            stats={sessionStats[session.id]}
            statsLoading={Boolean(sessionStatsLoading[session.id])}
          />
        ))}
      </div>
    </section>
  );
}
