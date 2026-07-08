import { useMemo, useState } from "react";
import { useI18n } from "../i18n/I18nProvider";
import type { SyncDirection, SyncItem } from "../types";

type ConflictEntry = {
  source: string;
  target: string;
  reason?: string | null;
};

type Props = {
  conflicts: ConflictEntry[];
  canRememberChoice: boolean;
  onResolve: (result: {
    items: SyncItem[];
    rememberChoice: boolean;
    rememberedChoice: "source-wins" | "target-wins" | null;
  }) => void;
  onCancel: () => void;
};

function basename(path: string): string {
  return path.replace(/\\/g, "/").split("/").filter(Boolean).pop() ?? path;
}

export function SyncConflictDialog({ conflicts, canRememberChoice, onResolve, onCancel }: Props) {
  const { t } = useI18n();
  const [decisions, setDecisions] = useState<Record<string, SyncDirection | "skip">>({});
  const [applyToAll, setApplyToAll] = useState(false);
  const [rememberChoice, setRememberChoice] = useState(false);

  const resolvedDecisions = useMemo(() => {
    const fallback = applyToAll
      ? Object.values(decisions).find((value) => value === "source-to-target" || value === "target-to-source")
      : undefined;

    return conflicts.map((conflict, index) => {
      const key = `${conflict.source}|${conflict.target}|${index}`;
      const decision = decisions[key] ?? fallback ?? "source-to-target";
      return { key, conflict, decision };
    });
  }, [applyToAll, conflicts, decisions]);

  return (
    <div className="dialog-backdrop" role="presentation">
      <div className="dialog-card sync-conflict-dialog" role="dialog" aria-modal="true" aria-labelledby="sync-conflict-title">
        <h3 id="sync-conflict-title">{t("agents.conflict.title")}</h3>
        <p>{t("agents.conflict.description")}</p>

        <div className="sync-conflict-list">
          {resolvedDecisions.map(({ key, conflict, decision }) => (
            <section key={key} className="sync-conflict-item">
              <div className="sync-conflict-paths">
                <strong>{basename(conflict.target)}</strong>
                <span>{conflict.source}</span>
                <span>{conflict.target}</span>
                {conflict.reason ? <small>{conflict.reason}</small> : null}
              </div>

              <div className="sync-conflict-options">
                {([
                  ["source-to-target", t("agents.conflict.option.sourceToTarget")],
                  ["target-to-source", t("agents.conflict.option.targetToSource")],
                  ["skip", t("agents.conflict.option.skip")],
                ] as const).map(([value, label]) => (
                  <label key={value} className="checkbox-group checkbox-group--inline">
                    <input
                      type="radio"
                      name={key}
                      checked={decision === value}
                      onChange={() => setDecisions((current) => ({ ...current, [key]: value }))}
                    />
                    <span>{label}</span>
                  </label>
                ))}
              </div>
            </section>
          ))}
        </div>

        <div className="sync-conflict-flags">
          <label className="checkbox-group">
            <input type="checkbox" checked={applyToAll} onChange={(event) => setApplyToAll(event.currentTarget.checked)} />
            <span>{t("agents.conflict.applyToAll")}</span>
          </label>
          {canRememberChoice ? (
            <label className="checkbox-group">
              <input type="checkbox" checked={rememberChoice} onChange={(event) => setRememberChoice(event.currentTarget.checked)} />
              <span>{t("agents.conflict.rememberChoice")}</span>
            </label>
          ) : null}
        </div>

        <div className="dialog-actions">
          <button type="button" className="ghost-button" onClick={onCancel}>{t("dialog.cancel")}</button>
          <button
            type="button"
            className="primary-button"
            onClick={() => {
              const items = resolvedDecisions.flatMap(({ conflict, decision }) => (
                decision === "skip"
                  ? []
                  : [{ source: conflict.source, target: conflict.target, direction: decision }]
              ));
              const rememberedChoice = rememberChoice
                ? resolvedDecisions[0]?.decision === "target-to-source"
                  ? "target-wins"
                  : "source-wins"
                : null;
              onResolve({ items, rememberChoice, rememberedChoice });
            }}
          >
            {t("agents.conflict.apply")}
          </button>
        </div>
      </div>
    </div>
  );
}
