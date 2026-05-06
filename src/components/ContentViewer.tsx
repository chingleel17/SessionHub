import DOMPurify from "dompurify";
import { marked } from "marked";
import { useEffect, useMemo, useRef } from "react";
import { useI18n } from "../i18n/I18nProvider";
import type { TreeNode } from "../types";

type Props = {
  content: string | null;
  filePath: string | null;
  filePathType: TreeNode["filePathType"] | null;
  isLoading: boolean;
  error: string | null;
  isTaskSaving: boolean;
  onToggleTask: (filePath: string, taskIndex: number, checked: boolean) => Promise<void>;
};

function getDisplayPath(filePath: string): string {
  const parts = filePath.replace(/\\/g, "/").split("/");
  return parts.slice(-2).join("/");
}

function isMarkdownFile(filePath: string | null): boolean {
  return Boolean(filePath && /\.md$/i.test(filePath));
}

function isInteractiveTaskFile(filePath: string | null, filePathType: TreeNode["filePathType"] | null): boolean {
  return filePathType === "openspec" && Boolean(filePath && /(^|[\\/])tasks\.md$/i.test(filePath));
}

function stripParagraphWrapper(html: string): string {
  const trimmed = html.trim();
  const matched = trimmed.match(/^<p>([\s\S]*)<\/p>$/);
  return matched ? matched[1] : trimmed;
}

function renderMarkdownHtml(content: string, interactiveTasks: boolean): string {
  let taskIndex = 0;
  const renderer = new marked.Renderer();
  if (interactiveTasks) {
    const defaultListItem = renderer.listitem.bind(renderer);
    renderer.listitem = function(item) {
      if (!item.task) {
        return defaultListItem(item);
      }

      const currentTaskIndex = taskIndex++;
      const body = stripParagraphWrapper(this.parser.parse(item.tokens, !!item.loose));
      const checkedClass = item.checked ? " explorer-task-content--checked" : "";

      return [
        `<li class="explorer-task-list-item">`,
        `<button`,
        ` type="button"`,
        ` class="explorer-task-toggle"`,
        ` data-task-index="${currentTaskIndex}"`,
        ` role="checkbox"`,
        ` aria-checked="${item.checked ? "true" : "false"}"`,
        ` aria-label="Toggle task ${currentTaskIndex + 1}"`,
        `>`,
        `<span class="explorer-task-toggle-box${item.checked ? " explorer-task-toggle-box--checked" : ""}" aria-hidden="true"></span>`,
        `</button>`,
        `<div class="explorer-task-content${checkedClass}">${body}</div>`,
        `</li>`,
      ].join("");
    };
  }

  return DOMPurify.sanitize(
    marked.parse(content, { async: false, gfm: true, renderer }),
    {
      ADD_TAGS: ["button", "span"],
      ADD_ATTR: ["class", "data-task-index", "type", "role", "aria-checked", "aria-hidden", "aria-label"],
    },
  );
}

export function ContentViewer({
  content,
  filePath,
  filePathType,
  isLoading,
  error,
  isTaskSaving,
  onToggleTask,
}: Props) {
  const { t } = useI18n();
  const contentBodyRef = useRef<HTMLDivElement>(null);
  const markdownFile = isMarkdownFile(filePath);
  const interactiveTaskFile = isInteractiveTaskFile(filePath, filePathType);
  const markdownHtml = useMemo(
    () => (content === null || !markdownFile ? null : renderMarkdownHtml(content, interactiveTaskFile)),
    [content, interactiveTaskFile, markdownFile],
  );

  useEffect(() => {
    const contentBody = contentBodyRef.current;
    if (!contentBody || !interactiveTaskFile || !filePath) return undefined;

    const handleClick = async (event: Event) => {
      const target = event.target;
      if (!(target instanceof Element)) {
        return;
      }

      const button = target.closest<HTMLButtonElement>(".explorer-task-toggle");
      if (!button) {
        return;
      }

      const taskIndex = Number(button.dataset.taskIndex);
      const previousChecked = button.getAttribute("aria-checked") === "true";
      const nextChecked = !previousChecked;
      if (Number.isNaN(taskIndex) || isTaskSaving) {
        return;
      }

      button.disabled = true;
      try {
        await onToggleTask(filePath, taskIndex, nextChecked);
      } catch {
        button.setAttribute("aria-checked", previousChecked ? "true" : "false");
      } finally {
        button.disabled = false;
      }
    };

    contentBody.addEventListener("click", handleClick);
    return () => {
      contentBody.removeEventListener("click", handleClick);
    };
  }, [filePath, interactiveTaskFile, isTaskSaving, onToggleTask]);

  if (isLoading) {
    return (
      <div className="explorer-content">
        {filePath ? (
          <div className="explorer-content-header">{getDisplayPath(filePath)}</div>
        ) : null}
        <div className="explorer-content-loading">
          {t("plansSpecs.loading")}
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="explorer-content">
        {filePath ? (
          <div className="explorer-content-header">{getDisplayPath(filePath)}</div>
        ) : null}
        <div className="explorer-error-banner">{error}</div>
      </div>
    );
  }

  if (content === null) {
    return (
      <div className="explorer-content">
        <div className="explorer-content-empty">
          {t("plansSpecs.explorer.selectPrompt")}
        </div>
      </div>
    );
  }

  return (
    <div className="explorer-content">
      {filePath ? (
        <div className="explorer-content-header">{getDisplayPath(filePath)}</div>
      ) : null}
      <div
        ref={contentBodyRef}
        className={[
          "explorer-content-body",
          markdownFile ? "explorer-content-body--markdown" : "explorer-content-body--plain",
          isTaskSaving ? "explorer-content-body--busy" : "",
        ].filter(Boolean).join(" ")}
        dangerouslySetInnerHTML={markdownFile && markdownHtml !== null
          ? { __html: markdownHtml }
          : undefined}
      >
        {!markdownFile ? content : undefined}
      </div>
    </div>
  );
}
