import { useMemo, useState } from "react";
import { useI18n } from "../i18n/I18nProvider";
import type { ProjectGroup, SessionInfo, SortKey } from "../types";
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
  onDelete: (session: SessionInfo) => void;
};

function filterAndSortSessions(
  sessions: SessionInfo[],
  searchTerm: string,
  sortKey: SortKey,
  selectedTags: string[],
) {
  const normalizedSearchTerm = searchTerm.trim().toLowerCase();

  const filtered = sessions.filter((session) => {
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
  onDelete,
}: Props) {
  const { t } = useI18n();
  const [searchTerm, setSearchTerm] = useState("");
  const [sortKey, setSortKey] = useState<SortKey>("updatedAt");
  const [selectedTags, setSelectedTags] = useState<string[]>([]);

  const availableTags = useMemo(
    () =>
      [...new Set(project.sessions.flatMap((s) => s.tags))]
        .filter(Boolean)
        .sort((a, b) => a.localeCompare(b)),
    [project.sessions],
  );

  const filteredSessions = useMemo(
    () => filterAndSortSessions(project.sessions, searchTerm, sortKey, selectedTags),
    [project.sessions, searchTerm, sortKey, selectedTags],
  );

  return (
    <section className="project-page">
      <section className="toolbar-card">
        <label className="field-group compact-field">
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
            onDelete={onDelete}
          />
        ))}
      </div>
    </section>
  );
}
