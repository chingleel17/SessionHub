import { useState } from "react";
import { useI18n } from "../i18n/I18nProvider";
import type {
  OpenSpecChange,
  OpenSpecData,
  OpenSpecSpec,
  SisyphusData,
  SisyphusPlan,
} from "../types";

type Props = {
  sisyphusData: SisyphusData | undefined;
  openspecData: OpenSpecData | undefined;
  isLoading: boolean;
  onReadFileContent: (filePath: string) => Promise<string>;
};

// --- 可折疊區塊 ---

function CollapsibleSection({
  title,
  defaultOpen = false,
  children,
}: {
  title: string;
  defaultOpen?: boolean;
  children: React.ReactNode;
}) {
  const [isOpen, setIsOpen] = useState(defaultOpen);

  return (
    <div>
      <button
        type="button"
        className="collapsible-header"
        onClick={() => setIsOpen((v) => !v)}
      >
        <span className={`collapsible-arrow ${isOpen ? "collapsible-arrow--open" : ""}`}>
          &#9654;
        </span>
        {title}
      </button>
      {isOpen ? children : null}
    </div>
  );
}

// --- Sisyphus 區塊 ---

function SisyphusSection({
  data,
  previewPath,
  previewContent,
  previewLoading,
  onSelectFile,
}: {
  data: SisyphusData;
  previewPath: string | null;
  previewContent: string;
  previewLoading: boolean;
  onSelectFile: (path: string) => void;
}) {
  const { t } = useI18n();

  return (
    <section className="plans-specs-section">
      <h3>{t("plansSpecs.sisyphus.title")}</h3>

      {/* Active Plan 橫幅 */}
      {data.activePlan ? (
        <div className="plans-specs-item plans-specs-item-active" style={{ cursor: "default" }}>
          <div>
            <strong>{t("plansSpecs.sisyphus.activePlan")}</strong>
            <span style={{ marginLeft: 8 }}>{data.activePlan.planName ?? data.activePlan.activePlan ?? "—"}</span>
            {data.activePlan.agent ? (
              <span className="plans-specs-item-meta" style={{ marginLeft: 12 }}>
                agent: {data.activePlan.agent}
              </span>
            ) : null}
          </div>
          {data.activePlan.sessionIds.length > 0 ? (
            <span className="plans-specs-item-meta">
              {data.activePlan.sessionIds.length} session(s)
            </span>
          ) : null}
        </div>
      ) : null}

      {/* Plans 列表 */}
      {data.plans.length > 0 ? (
        <CollapsibleSection title={`${t("plansSpecs.sisyphus.plans")} (${data.plans.length})`} defaultOpen>
          <div className="plans-specs-list" style={{ marginTop: 8 }}>
            {data.plans.map((plan) => (
              <PlanItem
                key={plan.path}
                plan={plan}
                isSelected={previewPath === plan.path}
                onSelect={() => onSelectFile(plan.path)}
              />
            ))}
          </div>
        </CollapsibleSection>
      ) : null}

      {/* Notepads */}
      {data.notepads.length > 0 ? (
        <CollapsibleSection title={`${t("plansSpecs.sisyphus.notepads")} (${data.notepads.length})`}>
          <div className="plans-specs-list" style={{ marginTop: 8 }}>
            {data.notepads.map((np) => (
              <div key={np.name} className="plans-specs-item" style={{ cursor: "default" }}>
                <span>{np.name}</span>
                <span className="plans-specs-item-meta">
                  {[np.hasIssues ? "issues" : null, np.hasLearnings ? "learnings" : null]
                    .filter(Boolean)
                    .join(", ")}
                </span>
              </div>
            ))}
          </div>
        </CollapsibleSection>
      ) : null}

      {/* Evidence */}
      {data.evidenceFiles.length > 0 ? (
        <CollapsibleSection title={`${t("plansSpecs.sisyphus.evidence")} (${data.evidenceFiles.length})`}>
          <div className="plans-specs-list" style={{ marginTop: 8 }}>
            {data.evidenceFiles.map((file) => {
              const fileName = file.split(/[\\/]/).pop() ?? file;
              return (
                <button
                  key={file}
                  type="button"
                  className={`plans-specs-item ${previewPath === file ? "plans-specs-item-active" : ""}`}
                  onClick={() => onSelectFile(file)}
                >
                  <span>{fileName}</span>
                </button>
              );
            })}
          </div>
        </CollapsibleSection>
      ) : null}

      {/* Drafts */}
      {data.draftFiles.length > 0 ? (
        <CollapsibleSection title={`${t("plansSpecs.sisyphus.drafts")} (${data.draftFiles.length})`}>
          <div className="plans-specs-list" style={{ marginTop: 8 }}>
            {data.draftFiles.map((file) => {
              const fileName = file.split(/[\\/]/).pop() ?? file;
              return (
                <button
                  key={file}
                  type="button"
                  className={`plans-specs-item ${previewPath === file ? "plans-specs-item-active" : ""}`}
                  onClick={() => onSelectFile(file)}
                >
                  <span>{fileName}</span>
                </button>
              );
            })}
          </div>
        </CollapsibleSection>
      ) : null}

      {/* 內容預覽 */}
      {previewPath ? (
        <div className="plans-specs-preview">
          {previewLoading ? "..." : previewContent}
        </div>
      ) : null}
    </section>
  );
}

function PlanItem({
  plan,
  isSelected,
  onSelect,
}: {
  plan: SisyphusPlan;
  isSelected: boolean;
  onSelect: () => void;
}) {
  return (
    <button
      type="button"
      className={`plans-specs-item ${isSelected ? "plans-specs-item-active" : ""} ${plan.isActive ? "plans-specs-item-active" : ""}`}
      onClick={onSelect}
    >
      <div>
        <span>{plan.title ?? plan.name}</span>
        {plan.tldr ? (
          <div className="plans-specs-item-meta">{plan.tldr}</div>
        ) : null}
      </div>
      <span className="plans-specs-item-meta">{plan.name}</span>
    </button>
  );
}

// --- OpenSpec 區塊 ---

function OpenSpecSection({
  data,
  previewPath,
  previewContent,
  previewLoading,
  onSelectFile,
}: {
  data: OpenSpecData;
  previewPath: string | null;
  previewContent: string;
  previewLoading: boolean;
  onSelectFile: (path: string) => void;
}) {
  const { t } = useI18n();

  return (
    <section className="plans-specs-section">
      <h3>{t("plansSpecs.openspec.title")}</h3>

      {/* Active Changes */}
      {data.activeChanges.length > 0 ? (
        <CollapsibleSection
          title={`${t("plansSpecs.openspec.activeChanges")} (${data.activeChanges.length})`}
          defaultOpen
        >
          <div className="plans-specs-list" style={{ marginTop: 8 }}>
            {data.activeChanges.map((change) => (
              <ChangeItem key={change.name} change={change} />
            ))}
          </div>
        </CollapsibleSection>
      ) : null}

      {/* Archived Changes */}
      {data.archivedChanges.length > 0 ? (
        <CollapsibleSection
          title={`${t("plansSpecs.openspec.archivedChanges")} (${data.archivedChanges.length})`}
        >
          <div className="plans-specs-list" style={{ marginTop: 8 }}>
            {data.archivedChanges.map((change) => (
              <ChangeItem key={change.name} change={change} />
            ))}
          </div>
        </CollapsibleSection>
      ) : null}

      {/* Specs */}
      {data.specs.length > 0 ? (
        <CollapsibleSection title={`${t("plansSpecs.openspec.specs")} (${data.specs.length})`} defaultOpen>
          <div className="plans-specs-list" style={{ marginTop: 8 }}>
            {data.specs.map((spec) => (
              <SpecItem
                key={spec.path}
                spec={spec}
                isSelected={previewPath === spec.path}
                onSelect={() => onSelectFile(spec.path)}
              />
            ))}
          </div>
        </CollapsibleSection>
      ) : null}

      {/* 內容預覽 */}
      {previewPath ? (
        <div className="plans-specs-preview">
          {previewLoading ? "..." : previewContent}
        </div>
      ) : null}
    </section>
  );
}

function ChangeItem({ change }: { change: OpenSpecChange }) {
  const artifacts = [
    change.hasProposal ? "proposal" : null,
    change.hasDesign ? "design" : null,
    change.hasTasks ? "tasks" : null,
    change.specsCount > 0 ? `${change.specsCount} spec(s)` : null,
  ]
    .filter(Boolean)
    .join(" / ");

  return (
    <div className="plans-specs-item" style={{ cursor: "default" }}>
      <span>{change.name}</span>
      <span className="plans-specs-item-meta">{artifacts}</span>
    </div>
  );
}

function SpecItem({
  spec,
  isSelected,
  onSelect,
}: {
  spec: OpenSpecSpec;
  isSelected: boolean;
  onSelect: () => void;
}) {
  return (
    <button
      type="button"
      className={`plans-specs-item ${isSelected ? "plans-specs-item-active" : ""}`}
      onClick={onSelect}
    >
      <span>{spec.name}</span>
    </button>
  );
}

// --- 主元件 ---

export function PlansSpecsView({
  sisyphusData,
  openspecData,
  isLoading,
  onReadFileContent,
}: Props) {
  const { t } = useI18n();
  const [previewPath, setPreviewPath] = useState<string | null>(null);
  const [previewContent, setPreviewContent] = useState("");
  const [previewLoading, setPreviewLoading] = useState(false);

  const handleSelectFile = async (filePath: string) => {
    if (previewPath === filePath) {
      setPreviewPath(null);
      setPreviewContent("");
      return;
    }
    setPreviewPath(filePath);
    setPreviewLoading(true);
    try {
      const content = await onReadFileContent(filePath);
      setPreviewContent(content);
    } catch {
      setPreviewContent("Failed to read file.");
    } finally {
      setPreviewLoading(false);
    }
  };

  if (isLoading) {
    return (
      <div className="plans-specs-empty">
        {t("plansSpecs.loading")}
      </div>
    );
  }

  const hasSisyphus = sisyphusData && (
    sisyphusData.plans.length > 0 ||
    sisyphusData.notepads.length > 0 ||
    sisyphusData.evidenceFiles.length > 0 ||
    sisyphusData.draftFiles.length > 0 ||
    sisyphusData.activePlan !== null
  );

  const hasOpenSpec = openspecData && (
    openspecData.activeChanges.length > 0 ||
    openspecData.archivedChanges.length > 0 ||
    openspecData.specs.length > 0
  );

  if (!hasSisyphus && !hasOpenSpec) {
    return (
      <div className="plans-specs-empty">
        {t("plansSpecs.empty")}
      </div>
    );
  }

  return (
    <div className="plans-specs-layout">
      {hasSisyphus && sisyphusData ? (
        <SisyphusSection
          data={sisyphusData}
          previewPath={previewPath}
          previewContent={previewContent}
          previewLoading={previewLoading}
          onSelectFile={handleSelectFile}
        />
      ) : null}

      {hasOpenSpec && openspecData ? (
        <OpenSpecSection
          data={openspecData}
          previewPath={previewPath}
          previewContent={previewContent}
          previewLoading={previewLoading}
          onSelectFile={handleSelectFile}
        />
      ) : null}
    </div>
  );
}
