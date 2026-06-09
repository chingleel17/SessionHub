import { useI18n } from "../i18n/I18nProvider";
import type { SessionTodo } from "../types";

type Props = {
  todos: SessionTodo[];
  isLoading: boolean;
};

function getStatusLabel(
  t: (
    key:
      | "session.todos.status.pending"
      | "session.todos.status.in_progress"
      | "session.todos.status.done"
      | "session.todos.status.blocked"
  ) => string,
  status: SessionTodo["status"] | string,
) {
  switch (status) {
    case "in_progress":
      return t("session.todos.status.in_progress");
    case "done":
      return t("session.todos.status.done");
    case "blocked":
      return t("session.todos.status.blocked");
    case "pending":
      return t("session.todos.status.pending");
    default:
      return String(status).replace(/_/g, " ");
  }
}

export function SessionTodosPanel({ todos, isLoading }: Props) {
  const { t } = useI18n();

  return (
    <section className="session-todos-panel">
      <div className="session-todos-panel-header">
        <strong>{t("session.todos.title")}</strong>
      </div>

      {isLoading ? (
        <span className="session-todos-empty">{t("session.todos.loading")}</span>
      ) : todos.length === 0 ? (
        <span className="session-todos-empty">{t("session.todos.empty")}</span>
      ) : (
        <div className="session-todos-list">
          {todos.map((todo) => (
            <article key={todo.id} className="session-todo-item">
              <div className="session-todo-main">
                <span className={`session-todo-badge session-todo-badge--${todo.status}`}>
                  {getStatusLabel(t, todo.status)}
                </span>
                <div className="session-todo-copy">
                  <strong>{todo.title}</strong>
                  {todo.description ? <span>{todo.description}</span> : null}
                </div>
              </div>
            </article>
          ))}
        </div>
      )}
    </section>
  );
}
