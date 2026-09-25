import { useState } from "react";

import { useI18n } from "../i18n/I18nProvider";
import type { ManualModelPricingInput, ModelPricingEntry } from "../types";
import { formatAnalyticsSupplierLabel } from "../utils/analyticsProviderLabels";
import { DeleteIcon, EditNotesIcon, EyeIcon, EyeOffIcon } from "./Icons";
import { Button } from "./ui/Button";
import { Checkbox } from "./ui/Checkbox";
import { IconButton } from "./ui/IconButton";
import { Select } from "./ui/Select";

type Props = {
  entries: ModelPricingEntry[];
  isLoading: boolean;
  isSaving: boolean;
  errorMessage: string | null;
  onSave: (input: ManualModelPricingInput) => Promise<void>;
  onDelete: (provider: string, model: string) => void;
  onVisibilityChange: (provider: string, model: string, hidden: boolean) => void;
};

type Draft = {
  provider: string;
  model: string;
  prompt: string;
  completion: string;
  cacheRead: string;
  cacheWrite: string;
};

const EMPTY_DRAFT: Draft = {
  provider: "openai",
  model: "",
  prompt: "",
  completion: "",
  cacheRead: "",
  cacheWrite: "",
};

function sourceKey(source: string) {
  if (source === "user") return "pricing.source.user" as const;
  if (source === "builtin-official") return "pricing.source.builtin" as const;
  return "pricing.source.openrouter" as const;
}

export function ModelPricingView({ entries, isLoading, isSaving, errorMessage, onSave, onDelete, onVisibilityChange }: Props) {
  const { t, locale } = useI18n();
  const [search, setSearch] = useState("");
  const [provider, setProvider] = useState("all");
  const [showHidden, setShowHidden] = useState(false);
  const [draft, setDraft] = useState<Draft | null>(null);
  const formatter = new Intl.NumberFormat(locale, { maximumFractionDigits: 6 });
  const filtered = entries.filter((entry) => {
    const providerMatches = provider === "all" || entry.provider === provider;
    const visibilityMatches = showHidden || !entry.hidden;
    const searchMatches = `${entry.provider} ${entry.model}`.toLocaleLowerCase()
      .includes(search.trim().toLocaleLowerCase());
    return providerMatches && visibilityMatches && searchMatches;
  });
  const edit = (entry: ModelPricingEntry) => setDraft({
    provider: entry.provider,
    model: entry.model,
    prompt: entry.promptUsdPerMillion?.toString() ?? "",
    completion: entry.completionUsdPerMillion?.toString() ?? "",
    cacheRead: entry.cacheReadUsdPerMillion?.toString() ?? "",
    cacheWrite: entry.cacheWriteUsdPerMillion?.toString() ?? "",
  });
  const submit = async () => {
    if (!draft) return;
    const optionalNumber = (value: string) => value.trim() === "" ? null : Number(value);
    await onSave({
      provider: draft.provider,
      model: draft.model.trim(),
      promptUsdPerMillion: Number(draft.prompt),
      completionUsdPerMillion: Number(draft.completion),
      cacheReadUsdPerMillion: optionalNumber(draft.cacheRead),
      cacheWriteUsdPerMillion: optionalNumber(draft.cacheWrite),
    });
    setDraft(null);
  };

  return (
    <section className="pricing-view">
      <div className="pricing-toolbar">
        <div className="pricing-filters">
          <input
            type="search"
            value={search}
            placeholder={t("pricing.search")}
            aria-label={t("pricing.search")}
            onChange={(event) => setSearch(event.currentTarget.value)}
          />
          <Select value={provider} aria-label={t("pricing.provider")} onChange={(event) => setProvider(event.currentTarget.value)}>
            <option value="all">{t("pricing.allProviders")}</option>
            <option value="openai">OpenAI</option>
            <option value="anthropic">Anthropic</option>
          </Select>
          <Checkbox checked={showHidden} onChange={(event) => setShowHidden(event.currentTarget.checked)}>
            {t("pricing.showHidden")}
          </Checkbox>
        </div>
        <Button variant="primary" onClick={() => setDraft(EMPTY_DRAFT)}>{t("pricing.add")}</Button>
      </div>
      <p className="pricing-note">{t("pricing.note")}</p>
      {errorMessage ? <p className="analytics-error-banner" role="alert">{errorMessage}</p> : null}
      {isLoading ? <p className="analytics-empty-state">{t("analytics.actions.loading")}</p> : (
        <div className="pricing-table-wrap">
          <table className="pricing-table">
            <thead><tr>
              <th>{t("pricing.provider")}</th><th>{t("pricing.model")}</th>
              <th>{t("pricing.input")}</th><th>{t("pricing.output")}</th>
              <th>{t("pricing.cacheRead")}</th><th>{t("pricing.cacheWrite")}</th>
              <th>{t("pricing.source")}</th><th className="pricing-actions-heading">{t("pricing.actions")}</th>
            </tr></thead>
            <tbody>{filtered.map((entry) => (
              <tr key={`${entry.provider}:${entry.model}`} className={entry.hidden ? "pricing-row-hidden" : undefined}>
                <td>{formatAnalyticsSupplierLabel(entry.provider)}</td>
                <td><strong>{entry.model}</strong>{entry.status !== "success" ? <small>{entry.errorKind ?? entry.status}</small> : null}</td>
                {[entry.promptUsdPerMillion, entry.completionUsdPerMillion, entry.cacheReadUsdPerMillion, entry.cacheWriteUsdPerMillion]
                  .map((value, index) => <td key={index}>{value === null ? "—" : `$${formatter.format(value)}`}</td>)}
                <td>{t(sourceKey(entry.source))}</td>
                <td className="pricing-actions-cell">
                  <div className="pricing-actions">
                    <IconButton label={t("pricing.edit")} onClick={() => edit(entry)}>
                      <EditNotesIcon />
                    </IconButton>
                    {entry.source === "user" ? (
                      <IconButton danger label={t("pricing.delete")} onClick={() => onDelete(entry.provider, entry.model)}>
                        <DeleteIcon />
                      </IconButton>
                    ) : null}
                    <IconButton
                      label={t(entry.hidden ? "pricing.unhide" : "pricing.hide")}
                      onClick={() => onVisibilityChange(entry.provider, entry.model, !entry.hidden)}
                    >
                      {entry.hidden ? <EyeIcon /> : <EyeOffIcon />}
                    </IconButton>
                  </div>
                </td>
              </tr>
            ))}</tbody>
          </table>
          {filtered.length === 0 ? <p className="analytics-empty-state">{t("pricing.empty")}</p> : null}
        </div>
      )}
      {draft ? (
        <div className="modal-backdrop" role="presentation" onMouseDown={(event) => { if (event.target === event.currentTarget) setDraft(null); }}>
          <section className="pricing-dialog" role="dialog" aria-modal="true" aria-labelledby="pricing-dialog-title">
            <header><h3 id="pricing-dialog-title">{t("pricing.dialogTitle")}</h3></header>
            <div className="pricing-form">
              <label><span>{t("pricing.provider")}</span><Select value={draft.provider} onChange={(event) => setDraft({ ...draft, provider: event.currentTarget.value })}><option value="openai">OpenAI</option><option value="anthropic">Anthropic</option></Select></label>
              <label><span>{t("pricing.model")}</span><input value={draft.model} onChange={(event) => setDraft({ ...draft, model: event.currentTarget.value })} /></label>
              <label><span>{t("pricing.input")}</span><input type="number" min="0" step="any" value={draft.prompt} onChange={(event) => setDraft({ ...draft, prompt: event.currentTarget.value })} /></label>
              <label><span>{t("pricing.output")}</span><input type="number" min="0" step="any" value={draft.completion} onChange={(event) => setDraft({ ...draft, completion: event.currentTarget.value })} /></label>
              <label><span>{t("pricing.cacheRead")}</span><input type="number" min="0" step="any" value={draft.cacheRead} onChange={(event) => setDraft({ ...draft, cacheRead: event.currentTarget.value })} /></label>
              <label><span>{t("pricing.cacheWrite")}</span><input type="number" min="0" step="any" value={draft.cacheWrite} onChange={(event) => setDraft({ ...draft, cacheWrite: event.currentTarget.value })} /></label>
            </div>
            <footer><Button variant="ghost" onClick={() => setDraft(null)}>{t("dialog.cancel")}</Button><Button variant="primary" loading={isSaving} disabled={!draft.model.trim() || draft.prompt === "" || draft.completion === ""} onClick={() => void submit()}>{t("pricing.save")}</Button></footer>
          </section>
        </div>
      ) : null}
    </section>
  );
}
