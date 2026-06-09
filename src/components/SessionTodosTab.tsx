import { useI18n } from "../i18n/I18nProvider";
import type { SessionInfo, SessionTodo } from "../types";
import { SessionTodosPanel } from "./SessionTodosPanel";

type Props = {
  session: SessionInfo;
  todos: SessionTodo[];
  isLoading: boolean;
  onClose: () => void;
};

function getSessionTitle(session: SessionInfo) {
  return session.summary?.trim() || session.id;
}

export function SessionTodosTab({ session, todos, isLoading, onClose }: Props) {
  const { t } = useI18n();

  return (
    <article className="info-card plan-editor-card">
      <div className="section-heading">
        <h3>{t("session.todos.tab")}</h3>
        <span>{getSessionTitle(session)}</span>
      </div>

      <SessionTodosPanel todos={todos} isLoading={isLoading} />

      <div className="settings-actions">
        <button type="button" className="ghost-button" onClick={onClose}>
          {t("plan.actions.close")}
        </button>
      </div>
    </article>
  );
}
