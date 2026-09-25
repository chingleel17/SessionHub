import { describe, expect, test } from "bun:test";
import { createElement, useState } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { act } from "react";
import { createRoot } from "react-dom/client";
import { Window } from "happy-dom";

import { AnalyticsTrendChart } from "../src/components/AnalyticsTrendChart";
import { I18nProvider } from "../src/i18n/I18nProvider";
import { filterSelectOptions, Select } from "../src/components/ui/Select";
import type { AnalyticsMetricValue, AnalyticsSeriesPoint } from "../src/types";

const metric = (knownValue: number | null): AnalyticsMetricValue => ({
  knownValue,
  eligibleEventCount: Number(knownValue !== null),
  missingFieldEventCount: Number(knownValue === null),
  priceVersions: [],
});

describe("AnalyticsTrendChart", () => {
  test("renders with the React 19 component tree and provides a text data table", () => {
    const originalStorage = Object.getOwnPropertyDescriptor(globalThis, "localStorage");
    Object.defineProperty(globalThis, "localStorage", {
      configurable: true,
      value: { getItem: () => "en-US", setItem: () => undefined },
    });

    const point: AnalyticsSeriesPoint = {
      label: "2026-04-10",
      startAt: "2026-04-09T16:00:00.000000000Z",
      endAt: "2026-04-10T16:00:00.000000000Z",
      values: {
        totalTokens: metric(120),
        inputTokens: metric(80),
        outputTokens: metric(40),
        cacheReadTokens: metric(10),
        cacheWriteTokens: metric(5),
        reasoningTokens: metric(12),
        estimatedUsd: metric(null),
        copilotPoints: metric(null),
      },
      interactionCount: 1,
      sessionCount: 1,
    };

    try {
      const markup = renderToStaticMarkup(createElement(
        I18nProvider,
        null,
        createElement(AnalyticsTrendChart, {
          title: "Usage trend",
          data: [point],
          metric: "tokens",
          tokenField: "total",
          locale: "en-US",
          unitLabel: "Total tokens",
          onSelectBucket: () => undefined,
        }),
      ));

      expect(markup).toContain("analytics-trend-chart");
      expect(markup).toContain("recharts-responsive-container");
      expect(markup).toContain("Usage trend");
      expect(markup).toContain("analytics-chart-data-details");
      expect(markup).toContain("2026-04-10");
    } finally {
      if (originalStorage) {
        Object.defineProperty(globalThis, "localStorage", originalStorage);
      } else {
        Reflect.deleteProperty(globalThis, "localStorage");
      }
    }
  });

  test("ResponsiveContainer observes and redraws after ResizeObserver size changes", async () => {
    const browserWindow = new Window({ url: "http://localhost" });
    const originals = new Map<string, PropertyDescriptor | undefined>();
    const replaceGlobal = (key: string, value: unknown) => {
      originals.set(key, Object.getOwnPropertyDescriptor(globalThis, key));
      Object.defineProperty(globalThis, key, { configurable: true, value });
    };
    const observers: Array<{ resize: (width: number, height: number) => void }> = [];
    class TestResizeObserver {
      private target: Element | null = null;
      private readonly callback: ResizeObserverCallback;

      constructor(callback: ResizeObserverCallback) {
        this.callback = callback;
        observers.push({
          resize: (width, height) => {
            if (!this.target) return;
            const entry = {
              target: this.target,
              contentRect: new browserWindow.DOMRect(0, 0, width, height),
            } as unknown as ResizeObserverEntry;
            this.callback([entry], this as unknown as ResizeObserver);
          },
        });
      }

      observe(target: Element) { this.target = target; }
      unobserve() { this.target = null; }
      disconnect() { this.target = null; }
    }

    replaceGlobal("window", browserWindow);
    replaceGlobal("document", browserWindow.document);
    replaceGlobal("navigator", browserWindow.navigator);
    replaceGlobal("localStorage", { getItem: () => "en-US", setItem: () => undefined });
    replaceGlobal("HTMLElement", browserWindow.HTMLElement);
    replaceGlobal("Element", browserWindow.Element);
    replaceGlobal("Node", browserWindow.Node);
    replaceGlobal("MutationObserver", browserWindow.MutationObserver);
    replaceGlobal("ResizeObserver", TestResizeObserver);
    replaceGlobal("IS_REACT_ACT_ENVIRONMENT", true);
    const originalGetBoundingClientRect = Object.getOwnPropertyDescriptor(
      browserWindow.HTMLElement.prototype,
      "getBoundingClientRect",
    );
    Object.defineProperty(browserWindow.HTMLElement.prototype, "getBoundingClientRect", {
      configurable: true,
      value: () => new browserWindow.DOMRect(0, 0, 720, 280),
    });
    const container = browserWindow.document.createElement("div");
    browserWindow.document.body.appendChild(container);
    const root = createRoot(container as unknown as Element);

    try {
      await act(async () => {
        root.render(createElement(
          I18nProvider,
          null,
          createElement(AnalyticsTrendChart, {
            title: "Usage trend",
            data: [{
              label: "2026-04-10",
              startAt: "2026-04-09T16:00:00.000000000Z",
              endAt: "2026-04-10T16:00:00.000000000Z",
              values: {
                totalTokens: metric(120),
                inputTokens: metric(80),
                outputTokens: metric(40),
                cacheReadTokens: metric(10),
                cacheWriteTokens: metric(5),
                reasoningTokens: metric(12),
                estimatedUsd: metric(null),
                copilotPoints: metric(null),
              },
              interactionCount: 1,
              sessionCount: 1,
            }],
            metric: "tokens",
            tokenField: "total",
            locale: "en-US",
            unitLabel: "Total tokens",
            onSelectBucket: () => undefined,
          }),
        ));
      });

      expect(observers.length).toBeGreaterThan(0);
      await act(async () => {
        observers.forEach((observer) => observer.resize(720, 280));
      });
      expect(container.querySelector("svg.recharts-surface")?.getAttribute("width")).toBe("720");
      expect(container.querySelector("svg.recharts-surface")?.getAttribute("height")).toBe("280");
    } finally {
      await act(async () => root.unmount());
      if (originalGetBoundingClientRect) {
        Object.defineProperty(
          browserWindow.HTMLElement.prototype,
          "getBoundingClientRect",
          originalGetBoundingClientRect,
        );
      }
      browserWindow.close();
      for (const [key, descriptor] of originals) {
        if (descriptor) Object.defineProperty(globalThis, key, descriptor);
        else Reflect.deleteProperty(globalThis, key);
      }
    }
  });

  test("filterable multiple Select keeps search inside the menu and toggles selections", async () => {
    const browserWindow = new Window({ url: "http://localhost" });
    const originals = new Map<string, PropertyDescriptor | undefined>();
    const replaceGlobal = (key: string, value: unknown) => {
      originals.set(key, Object.getOwnPropertyDescriptor(globalThis, key));
      Object.defineProperty(globalThis, key, { configurable: true, value });
    };
    replaceGlobal("window", browserWindow);
    replaceGlobal("document", browserWindow.document);
    replaceGlobal("navigator", browserWindow.navigator);
    replaceGlobal("HTMLElement", browserWindow.HTMLElement);
    replaceGlobal("Element", browserWindow.Element);
    replaceGlobal("Node", browserWindow.Node);
    replaceGlobal("MutationObserver", browserWindow.MutationObserver);
    replaceGlobal("Event", browserWindow.Event);
    replaceGlobal("IS_REACT_ACT_ENVIRONMENT", true);

    let selectedValues: string[] = [];
    function SelectHarness() {
      const [values, setValues] = useState<string[]>([]);
      return createElement(
        Select,
        {
          multiple: true,
          value: values,
          filterable: true,
          filterPlaceholder: "Search projects",
          filterAriaLabel: "Search projects",
          filterEmptyLabel: "No matches",
          "aria-label": "Projects",
          onChange: (event) => {
            const nextValues = Array.from(event.currentTarget.selectedOptions)
              .map((option) => option.value)
              .filter(Boolean);
            selectedValues = nextValues;
            setValues(nextValues);
          },
        },
        createElement("option", { value: "" }, "All projects"),
        createElement("option", { value: "social-platform" }, "social-platform"),
        createElement("option", { value: "knowledge-docs" }, "knowledge-academy-docs"),
      );
    }

    const container = browserWindow.document.createElement("div");
    browserWindow.document.body.appendChild(container);
    const root = createRoot(container as unknown as Element);
    try {
      await act(async () => root.render(createElement(SelectHarness)));
      const trigger = container.querySelector(".ui-select-trigger") as unknown as HTMLButtonElement | null;
      expect(trigger).not.toBeNull();
      await act(async () => trigger?.click());
      expect(container.querySelector(".ui-select-filter")).not.toBeNull();

      const search = container.querySelector(".ui-select-filter") as unknown as HTMLInputElement | null;
      expect(search).not.toBeNull();
      await act(async () => {
        if (!search) return;
        const valueSetter = Object.getOwnPropertyDescriptor(
          browserWindow.HTMLInputElement.prototype,
          "value",
        )?.set;
        valueSetter?.call(search, "social");
        search.dispatchEvent(new browserWindow.Event("input", { bubbles: true }) as unknown as Event);
        search.dispatchEvent(new browserWindow.Event("change", { bubbles: true }) as unknown as Event);
      });
      const filteredOptions = container.querySelectorAll('[role="option"]');
      expect(filteredOptions.length).toBe(3);
      expect(filterSelectOptions([
        { label: "All projects", value: "", disabled: false },
        { label: "social-platform", value: "D:/social-platform", disabled: false },
        { label: "knowledge-academy-docs", value: "D:/knowledge-academy-docs", disabled: false },
      ], "social").map((option) => option.label)).toEqual(["social-platform"]);
      await act(async () => {
        (container.querySelector('[role="option"][aria-selected="false"]') as unknown as HTMLButtonElement | null)?.click();
      });
      expect(selectedValues).toEqual(["social-platform"]);
      expect(container.querySelector('[role="option"][aria-selected="true"]')?.textContent).toBe("social-platform");
    } finally {
      await act(async () => root.unmount());
      browserWindow.close();
      for (const [key, descriptor] of originals) {
        if (descriptor) Object.defineProperty(globalThis, key, descriptor);
        else Reflect.deleteProperty(globalThis, key);
      }
    }
  });
});
