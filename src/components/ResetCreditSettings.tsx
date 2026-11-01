import { useI18n } from "../i18n/I18nProvider";
import type { AppSettings, ResetCreditAutomationStatus } from "../types";
import { formatDateTime } from "../utils/formatDate";
import { resetCreditMessage } from "../utils/resetCreditAutomation";
import { Button } from "./ui/Button";

type ResetCreditSettingsProps = {
  settings: AppSettings;
  onChange: (next: AppSettings) => void;
  status?: ResetCreditAutomationStatus;
  onRefresh?: () => void;
  isSaving?: boolean;
};

export function ResetCreditSettings({ settings, onChange, status, onRefresh, isSaving = false }: ResetCreditSettingsProps) {
  const { t, locale } = useI18n();
  const reminderEnabled = settings.codexResetCreditReminderEnabled ?? false;
  const reminderReason = status?.reminderReason ?? status?.reason;
  const autoUseReason = status?.autoUseReason ?? status?.reason;
  const format = (value: string | null) => value ? formatDateTime(value, locale) : t("resetCredit.notScheduled");
  return (
    <article className="info-card reset-credit-settings" aria-labelledby="reset-credit-heading">
      <div className="section-heading">
        <h3 id="reset-credit-heading">{t("resetCredit.title")}</h3>
        <Button variant="secondary" onClick={onRefresh}>{t("resetCredit.refresh")}</Button>
      </div>
      <p className="settings-field-desc">{t("resetCredit.description")}</p>
      <label className="checkbox-group">
        <input type="checkbox" checked={reminderEnabled} disabled={isSaving} onChange={(e) => onChange({ ...settings, codexResetCreditReminderEnabled: e.currentTarget.checked })} />
        <span>{t("resetCredit.reminder")}</span>
      </label>
      <div className="reset-credit-settings-inputs">
        <div className="settings-field settings-field--stacked">
          <label htmlFor="reset-credit-time">{t("resetCredit.time")}</label>
          <input id="reset-credit-time" type="time" step={60} required disabled={!reminderEnabled || isSaving}
            value={settings.codexResetCreditReminderTime ?? "09:00"}
            onChange={(e) => onChange({ ...settings, codexResetCreditReminderTime: e.currentTarget.value })} />
        </div>
        <div className="settings-field settings-field--stacked">
          <label htmlFor="reset-credit-days">{t("resetCredit.days")}</label>
          <input id="reset-credit-days" type="number" min={0} max={30} step={1} required disabled={!reminderEnabled || isSaving}
            value={Number.isFinite(settings.codexResetCreditReminderDaysBefore ?? 0) ? settings.codexResetCreditReminderDaysBefore ?? 0 : ""}
            aria-describedby="reset-credit-days-description"
            onChange={(e) => onChange({ ...settings, codexResetCreditReminderDaysBefore: e.currentTarget.valueAsNumber })} />
        </div>
      </div>
      <p id="reset-credit-days-description" className="settings-field-desc">{t("resetCredit.daysDescription")}</p>
      <label className="checkbox-group">
        <input type="checkbox" checked={settings.codexResetCreditAutoUseEnabled ?? false} disabled={isSaving}
          onChange={(e) => onChange({ ...settings, codexResetCreditAutoUseEnabled: e.currentTarget.checked })} />
        <span>{t("resetCredit.autoUse")}<small className="settings-field-desc">{t("resetCredit.autoUseDescription")}</small></span>
      </label>
      <p className="settings-field-desc">{t("resetCredit.runtimeDescription")}</p>
      <div className="reset-credit-status" aria-live="polite" aria-atomic="true">
        <span className="settings-field-desc">{t("resetCredit.savedStatus")}</span>
        {status ? <>
          <dl>
            <div><dt>{t("resetCredit.reminder")}</dt><dd>{resetCreditMessage(t, status.reminderState)}
              {status.reminderState === "paused" && reminderReason ? <small className="settings-field-desc">{resetCreditMessage(t, reminderReason)}</small> : null}
            </dd></div>
            <div><dt>{t("resetCredit.autoUse")}</dt><dd>{status.autoUseState === "ready" ? t("resetCredit.state.scheduled") : resetCreditMessage(t, status.autoUseState)}
              {status.autoUseState === "paused" && autoUseReason ? <small className="settings-field-desc">{resetCreditMessage(t, autoUseReason)}</small> : null}
            </dd></div>
            <div><dt>{t("resetCredit.nextCheck")}</dt><dd>{format(status.nextReminderAt)}</dd></div>
            {status.notificationStatus ? <div><dt>{t("resetCredit.lastNotice")}</dt><dd>{resetCreditMessage(t, status.notificationError ?? status.notificationStatus)}</dd></div> : null}
            {status.lastAttemptAt ? <div><dt>{t("resetCredit.lastAttempt")}</dt><dd>{format(status.lastAttemptAt)}</dd></div> : null}
            {status.lastResult ? <div><dt>{t("resetCredit.lastResult")}</dt><dd>{resetCreditMessage(t, status.lastResult)}</dd></div> : null}
          </dl>
        </> : <p className="settings-field-desc">{t("resetCredit.loading")}</p>}
      </div>
    </article>
  );
}
