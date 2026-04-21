import { useI18n } from "../i18n/I18nProvider";

type Props = {
  content: string | null;
  filePath: string | null;
  isLoading: boolean;
  error: string | null;
};

function getDisplayPath(filePath: string): string {
  const parts = filePath.replace(/\\/g, "/").split("/");
  return parts.slice(-2).join("/");
}

export function ContentViewer({ content, filePath, isLoading, error }: Props) {
  const { t } = useI18n();

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
      <div className="explorer-content-body">{content}</div>
    </div>
  );
}
