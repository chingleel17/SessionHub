import { describe, expect, test } from "bun:test";
import DOMPurify from "dompurify";
import { Window } from "happy-dom";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";

import { ContentViewer } from "../src/components/ContentViewer";
import { I18nProvider } from "../src/i18n/I18nProvider";

function renderContent(content: string, filePath = "change/tasks.md") {
  const window = new Window();
  const originalSanitize = DOMPurify.sanitize;
  const originalStorage = Object.getOwnPropertyDescriptor(globalThis, "localStorage");
  DOMPurify.sanitize = DOMPurify(window as unknown as Parameters<typeof DOMPurify>[0]).sanitize;
  Object.defineProperty(globalThis, "localStorage", {
    configurable: true,
    value: { getItem: () => "en-US", setItem: () => undefined },
  });
  try {
    const html = renderToStaticMarkup(createElement(I18nProvider, null,
      createElement(ContentViewer, {
        content, filePath, filePathType: "openspec", isLoading: false,
        error: null, isTaskSaving: false, onToggleTask: async () => undefined,
      }),
    ));
    const container = window.document.createElement("div");
    container.innerHTML = html;
    return container;
  } finally {
    DOMPurify.sanitize = originalSanitize;
    if (originalStorage) {
      Object.defineProperty(globalThis, "localStorage", originalStorage);
    } else {
      Reflect.deleteProperty(globalThis, "localStorage");
    }
  }
}

describe("ContentViewer task checkboxes", () => {
  for (const [name, content] of [
    ["tight list", "- [x] First task\n- [ ] Second task"],
    ["loose list", "- [x] First task\n\n- [ ] Second task"],
    ["nested tasks and paragraphs", "- [x] First task\n\n  Extra details\n\n  - [ ] Second task"],
  ]) {
    test(`${name} renders one interactive checkbox per task`, () => {
      const container = renderContent(content);
      const toggles = container.querySelectorAll('button[role="checkbox"]');
      expect(toggles.length).toBe(2);
      expect(container.querySelectorAll('input[type="checkbox"]').length).toBe(0);
      expect(toggles[0].getAttribute("data-task-index")).toBe("0");
      expect(toggles[0].getAttribute("aria-checked")).toBe("true");
      expect(toggles[1].getAttribute("data-task-index")).toBe("1");
      expect(toggles[1].getAttribute("aria-checked")).toBe("false");
      expect(container.textContent).toContain("First task");
      expect(container.textContent).toContain("Second task");
    });
  }

  test("ordinary Markdown keeps its native disabled checkboxes", () => {
    const container = renderContent("- [x] First task\n\n- [ ] Second task", "change/spec.md");
    expect(container.querySelectorAll('button[role="checkbox"]').length).toBe(0);
    expect(container.querySelectorAll('input[type="checkbox"][disabled]').length).toBe(2);
  });
});
