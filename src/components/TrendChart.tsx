import { useMemo, useState } from "react";

import type { AnalyticsDataPoint } from "../types";

type LineKey = "outputTokens" | "interactionCount" | "costPoints";

export type TrendChartLine = {
  key: LineKey;
  label: string;
  colorClass: string;
};

type Props = {
  title: string;
  data: AnalyticsDataPoint[];
  lines: TrendChartLine[];
  emptyLabel: string;
  ariaLabel: string;
};

const WIDTH = 720;
const HEIGHT = 260;
const PADDING = { top: 16, right: 16, bottom: 40, left: 44 };

function formatAxisValue(value: number) {
  if (value >= 1_000_000) return `${(value / 1_000_000).toFixed(1)}M`;
  if (value >= 1_000) return `${(value / 1_000).toFixed(1)}K`;
  return Number.isInteger(value) ? String(value) : value.toFixed(1);
}

export function TrendChart({ title, data, lines, emptyLabel, ariaLabel }: Props) {
  const [enabledKeys, setEnabledKeys] = useState<LineKey[]>(lines.map((line) => line.key));

  const activeLines = useMemo(
    () => lines.filter((line) => enabledKeys.includes(line.key)),
    [enabledKeys, lines],
  );

  const chartValues = useMemo(
    () =>
      data.flatMap((point) =>
        activeLines.map((line) => {
          const value = point[line.key];
          return typeof value === "number" ? value : 0;
        }),
      ),
    [activeLines, data],
  );

  const maxValue = chartValues.length > 0 ? Math.max(...chartValues, 1) : 1;
  const innerWidth = WIDTH - PADDING.left - PADDING.right;
  const innerHeight = HEIGHT - PADDING.top - PADDING.bottom;

  const getX = (index: number) =>
    data.length <= 1
      ? PADDING.left + innerWidth / 2
      : PADDING.left + (index / (data.length - 1)) * innerWidth;
  const getY = (value: number) => PADDING.top + innerHeight - (value / maxValue) * innerHeight;

  const axisTicks = useMemo(() => {
    const tickCount = 4;
    return Array.from({ length: tickCount + 1 }, (_, index) => {
      const value = (maxValue / tickCount) * index;
      return {
        value,
        y: getY(value),
      };
    });
  }, [maxValue]);

  const toggleLine = (key: LineKey) => {
    setEnabledKeys((current) => {
      if (current.includes(key)) {
        return current.length === 1 ? current : current.filter((item) => item !== key);
      }
      return [...current, key];
    });
  };

  if (data.length === 0) {
    return (
      <section className="chart-card">
        <div className="chart-card-header">
          <h4>{title}</h4>
        </div>
        <div className="chart-empty">{emptyLabel}</div>
      </section>
    );
  }

  return (
    <section className="chart-card">
      <div className="chart-card-header">
        <h4>{title}</h4>
      </div>

      <div className="trend-chart-toggle-row">
        {lines.map((line) => {
          const active = enabledKeys.includes(line.key);
          return (
            <button
              key={line.key}
              type="button"
              className={`trend-chart-toggle ${active ? "trend-chart-toggle--active" : ""}`}
              onClick={() => toggleLine(line.key)}
            >
              <span className={`trend-chart-toggle-swatch ${line.colorClass}`} />
              {line.label}
            </button>
          );
        })}
      </div>

      <svg
        className="trend-chart"
        viewBox={`0 0 ${WIDTH} ${HEIGHT}`}
        role="img"
        aria-label={ariaLabel}
      >
        {axisTicks.map((tick) => (
          <g key={tick.value}>
            <line
              className="trend-chart-grid"
              x1={PADDING.left}
              y1={tick.y}
              x2={WIDTH - PADDING.right}
              y2={tick.y}
            />
            <text className="trend-chart-axis" x={PADDING.left - 8} y={tick.y + 4} textAnchor="end">
              {formatAxisValue(tick.value)}
            </text>
          </g>
        ))}

        {data.map((point, index) => (
          <text
            key={point.label}
            className="trend-chart-axis"
            x={getX(index)}
            y={HEIGHT - 10}
            textAnchor="middle"
          >
            {point.label}
          </text>
        ))}

        {activeLines.map((line) => {
          const points = data.map((point, index) => {
            const value = point[line.key];
            return `${getX(index)},${getY(typeof value === "number" ? value : 0)}`;
          });
          const lineClass = `trend-chart-path ${line.colorClass}`;

          if (data.length === 1) {
            const point = data[0];
            const value = point[line.key];
            return (
              <circle
                key={line.key}
                className={`trend-chart-point ${line.colorClass}`}
                cx={getX(0)}
                cy={getY(typeof value === "number" ? value : 0)}
                r={5}
              />
            );
          }

          return (
            <g key={line.key}>
              <polyline className={lineClass} points={points.join(" ")} />
              {data.map((point, index) => {
                const value = point[line.key];
                return (
                  <circle
                    key={`${line.key}-${point.label}`}
                    className={`trend-chart-point ${line.colorClass}`}
                    cx={getX(index)}
                    cy={getY(typeof value === "number" ? value : 0)}
                    r={4}
                  />
                );
              })}
            </g>
          );
        })}
      </svg>
    </section>
  );
}
