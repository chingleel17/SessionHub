import DOMPurify from "dompurify";
import { marked } from "marked";
import { useCallback, useEffect, useMemo, useRef, useState, type KeyboardEvent, type MouseEvent, type ReactNode } from "react";
import { useI18n } from "../i18n/I18nProvider";
import type { TreeNode } from "../types";
import { formatDate } from "../utils/formatDate";
import { prepareMarkdownForPreview } from "../utils/splitFrontmatter";
import { ContextMenu } from "./ContextMenu";

type Props = {
  content: string | null;
  filePath: string | null;
  filePathType: TreeNode["filePathType"] | null;
  createdAt?: string | null;
  isLoading: boolean;
  error: string | null;
  isTaskSaving: boolean;
  onToggleTask: (filePath: string, taskIndex: number, checked: boolean) => Promise<void>;
  headerMeta?: ReactNode;
  /** 右鍵選單「複製絕對路徑 / 以預設程式開啟」使用的實際檔案路徑 */
  absolutePath?: string | null;
  /** 右鍵選單「複製相對路徑」使用；未提供時不顯示該項目 */
  relativePath?: string | null;
  onOpenExternal?: (path: string) => void;
};

type ContextMenuState = {
  x: number;
  y: number;
  selectedText: string;
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
    // 統一略過原生核取方塊，包含鬆散清單中巢狀於段落內的 checkbox token。
    renderer.checkbox = () => "";
    const defaultListItem = renderer.listitem.bind(renderer);
    renderer.listitem = function(item) {
      if (!item.task) {
        return defaultListItem(item);
      }

      const currentTaskIndex = taskIndex++;
      const body = stripParagraphWrapper(this.parser.parse(item.tokens));
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
    marked.parse(prepareMarkdownForPreview(content), { async: false, gfm: true, renderer }),
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
  createdAt,
  isLoading,
  error,
  isTaskSaving,
  onToggleTask,
  headerMeta,
  absolutePath,
  relativePath,
  onOpenExternal,
}: Props) {
  const { t, locale } = useI18n();
  const contentBodyRef = useRef<HTMLDivElement>(null);
  const [contextMenu, setContextMenu] = useState<ContextMenuState | null>(null);
  const closeContextMenu = useCallback(() => setContextMenu(null), []);

  const selectAllContent = () => {
    const contentBody = contentBodyRef.current;
    if (!contentBody) return;
    window.getSelection()?.selectAllChildren(contentBody);
  };

  // Ctrl+A 只選取檢視器內容，而非整個應用程式畫面
  const handleBodyKeyDown = (event: KeyboardEvent<HTMLDivElement>) => {
    if ((event.ctrlKey || event.metaKey) && !event.altKey && event.key.toLowerCase() === "a") {
      event.preventDefault();
      selectAllContent();
    }
  };

  // 在事件當下記錄選取文字，避免點擊選單項目時選取被清除
  const handleContextMenu = (event: MouseEvent<HTMLDivElement>) => {
    event.preventDefault();
    setContextMenu({
      x: event.clientX,
      y: event.clientY,
      selectedText: window.getSelection()?.toString() ?? "",
    });
  };

  const runMenuAction = (action: () => void) => {
    closeContextMenu();
    action();
  };

  const copyText = (text: string) => {
    void navigator.clipboard.writeText(text).catch(() => undefined);
  };

  const renderContextMenu = () => {
    if (!contextMenu) return null;
    const hasPathActions = Boolean(absolutePath || relativePath);
    return (
      <ContextMenu x={contextMenu.x} y={contextMenu.y} onClose={closeContextMenu}>
        <button
          type="button"
          role="menuitem"
          className="dropdown-menu-item"
          disabled={!contextMenu.selectedText}
          onClick={() => runMenuAction(() => copyText(contextMenu.selectedText))}
        >
          {t("contentViewer.menu.copy")}
        </button>
        <button
          type="button"
          role="menuitem"
          className="dropdown-menu-item"
          onClick={() => runMenuAction(selectAllContent)}
        >
          {t("contentViewer.menu.selectAll")}
        </button>
        {hasPathActions ? <div className="context-menu-separator" role="separator" /> : null}
        {relativePath ? (
          <button
            type="button"
            role="menuitem"
            className="dropdown-menu-item"
            onClick={() => runMenuAction(() => copyText(relativePath))}
          >
            {t("contentViewer.menu.copyRelativePath")}
          </button>
        ) : null}
        {absolutePath ? (
          <button
            type="button"
            role="menuitem"
            className="dropdown-menu-item"
            onClick={() => runMenuAction(() => copyText(absolutePath))}
          >
            {t("contentViewer.menu.copyAbsolutePath")}
          </button>
        ) : null}
        {absolutePath && onOpenExternal ? (
          <>
            <div className="context-menu-separator" role="separator" />
            <button
              type="button"
              role="menuitem"
              className="dropdown-menu-item"
              onClick={() => runMenuAction(() => onOpenExternal(absolutePath))}
            >
              {t("contentViewer.menu.openExternal")}
            </button>
          </>
        ) : null}
        <div className="context-menu-separator" role="separator" />
        <button
          type="button"
          role="menuitem"
          className="dropdown-menu-item"
          onClick={() => runMenuAction(() => window.location.reload())}
        >
          {t("app.actions.refresh")}
        </button>
      </ContextMenu>
    );
  };

  const renderHeader = (path: string) => (
    <div className="explorer-content-header">
      <span className="explorer-content-header-path path-text path-text--truncate">{getDisplayPath(path)}</span>
      {createdAt ? (
        <span className="explorer-content-header-date">{formatDate(createdAt, locale)}</span>
      ) : null}
      {headerMeta}
    </div>
  );
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
        {filePath ? renderHeader(filePath) : null}
        <div className="explorer-content-loading">
          {t("plansSpecs.loading")}
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="explorer-content">
        {filePath ? renderHeader(filePath) : null}
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
    <div className="explorer-content" onContextMenu={handleContextMenu}>
      {filePath ? renderHeader(filePath) : null}
      <div
        ref={contentBodyRef}
        tabIndex={0}
        onKeyDown={handleBodyKeyDown}
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
      {renderContextMenu()}
    </div>
  );
}
