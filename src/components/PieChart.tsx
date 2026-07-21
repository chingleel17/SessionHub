type Slice = {
  label: string;
  value: number;
  color: string;
};

type Props = {
  title: string;
  slices: Slice[];
  emptyLabel: string;
  ariaLabel: string;
};

const SIZE = 220;
const RADIUS = 88;
const CENTER = SIZE / 2;

function polarToCartesian(angleInDegrees: number) {
  const angleInRadians = ((angleInDegrees - 90) * Math.PI) / 180;
  return {
    x: CENTER + RADIUS * Math.cos(angleInRadians),
    y: CENTER + RADIUS * Math.sin(angleInRadians),
  };
}

function describeArc(startAngle: number, endAngle: number) {
  if (endAngle - startAngle >= 360) {
    return [
      `M ${CENTER} ${CENTER - RADIUS}`,
      `A ${RADIUS} ${RADIUS} 0 1 1 ${CENTER - 0.01} ${CENTER - RADIUS}`,
      `Z`,
    ].join(" ");
  }

  const start = polarToCartesian(endAngle);
  const end = polarToCartesian(startAngle);
  const largeArcFlag = endAngle - startAngle <= 180 ? "0" : "1";

  return [
    `M ${CENTER} ${CENTER}`,
    `L ${start.x} ${start.y}`,
    `A ${RADIUS} ${RADIUS} 0 ${largeArcFlag} 0 ${end.x} ${end.y}`,
    "Z",
  ].join(" ");
}

export function PieChart({ title, slices, emptyLabel, ariaLabel }: Props) {
  const total = slices.reduce((sum, slice) => sum + slice.value, 0);

  if (total <= 0) {
    return (
      <section className="chart-card">
        <div className="chart-card-header">
          <h4>{title}</h4>
        </div>
        <div className="chart-empty">{emptyLabel}</div>
      </section>
    );
  }

  let currentAngle = 0;
  const normalizedSlices = slices
    .filter((slice) => slice.value > 0)
    .map((slice) => {
      const angle = (slice.value / total) * 360;
      const segment = {
        ...slice,
        percentage: (slice.value / total) * 100,
        startAngle: currentAngle,
        endAngle: currentAngle + angle,
      };
      currentAngle += angle;
      return segment;
    });

  return (
    <section className="chart-card">
      <div className="chart-card-header">
        <h4>{title}</h4>
      </div>

      <div className="pie-chart-layout">
        <svg className="pie-chart" viewBox={`0 0 ${SIZE} ${SIZE}`} role="img" aria-label={ariaLabel}>
          {normalizedSlices.map((slice) => (
            <path
              key={slice.label}
              d={describeArc(slice.startAngle, slice.endAngle)}
              fill={slice.color}
              stroke="var(--color-surface-panel)"
              strokeWidth="2"
            />
          ))}
        </svg>

        <div className="pie-chart-legend">
          {normalizedSlices.map((slice) => (
            <div key={slice.label} className="pie-chart-legend-item">
              <span className="pie-chart-legend-swatch" style={{ backgroundColor: slice.color }} />
              <span className="pie-chart-legend-label">{slice.label}</span>
              <span className="pie-chart-legend-value">{slice.percentage.toFixed(1)}%</span>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
