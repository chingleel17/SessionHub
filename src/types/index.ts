export type SessionInfo = {
    id: string;
    provider: string;
    cwd?: string | null;
    repoRoot?: string | null;
    repoName?: string | null;
    gitBranch?: string | null;
    summary?: string | null;
    summaryCount?: number | null;
    createdAt?: string | null;
    updatedAt?: string | null;
    sessionDir: string;
    parseError: boolean;
    isArchived: boolean;
    notes?: string | null;
    tags: string[];
    hasPlan: boolean;
    hasEvents: boolean;
};

export type ProjectPathRemap = {
    oldPath: string;
    newPath: string;
    createdAt: string;
};

export type SessionTodoStatus = "pending" | "in_progress" | "done" | "blocked" | (string & {});

export type SessionTodo = {
    id: string;
    title: string;
    status: SessionTodoStatus;
    description?: string | null;
    updatedAt?: string | null;
};

export type ProjectSubTabState = {
    openDetailKeys: string[];
    activeSubTab: string;
};

export type AppSettings = {
    copilotRoot: string;
    opencodeRoot: string;
    codexRoot: string;
    terminalPath?: string | null;
    terminalLauncher?: string | null;
    externalEditorPath?: string | null;
    showArchived: boolean;
    pinnedProjects?: string[];
    enabledProviders: string[];
    providerIntegrations?: ProviderIntegrationStatus[];
    defaultLauncher?: string | null;
    enableInterventionNotification?: boolean;
    enableSessionEndNotification?: boolean;
    showStatusBar?: boolean;
    analyticsRefreshInterval?: 10 | 30;
    analyticsPanelCollapsed?: boolean;
    minimizeToTray?: boolean;
    launchOnStartup?: boolean;
    startMinimizedOnStartup?: boolean;
    claudeRoot?: string;
    antigravityRoot?: string;
    hookScriptsPath?: string;
    claudeQuotaResetDay?: number;
    claudeMonthlyLimitTokens?: number | null;
    claudeMonthlyLimitUsd?: number | null;
    enableQuotaMonitoring?: boolean;
    quotaEnabledProviders?: string[];
    allowCreateProjectConfigDir?: boolean;
    agentsSourceRoot?: string;
    trayQuotaMode?: TrayQuotaMode;
    trayQuotaPrimaryProvider?: string | null;
    trayQuotaPanelEnabled?: boolean;
    quotaOverlayEnabled?: boolean;
    quotaOverlayLocked?: boolean;
    quotaOverlayOpacity?: number;
    quotaOverlayProviders?: string[];
    quotaOverlayTheme?: OverlayTheme;
    quotaOverlayStyle?: OverlayStyle;
};

export type TrayQuotaMode = "icon_only" | "percentage" | "bar" | "hidden";
export type OverlayTheme = "dark" | "light";
export type OverlayStyle = "full" | "compact";

export type AgentsScope =
    | { kind: "global" }
    | { kind: "project"; projectCwd: string };

export type SyncStatus =
    | "in-sync"
    | "target-missing"
    | "differs"
    | "source-missing"
    | "linked"
    | "link-broken"
    | (string & {});

export type SyncDirection = "source-to-target" | "target-to-source";

export type SyncMode = "copy" | "link";

export type AgentsRootLinkStatus =
    | { status: "linked" }
    | { status: "partial"; unmatchedItems: string[] }
    | { status: "unlinked-physical" }
    | { status: "not-linked" }
    | { status: "missing" };

export type FileFingerprint = {
    path: string;
    exists: boolean;
    hash?: string | null;
    mtimeMs?: number | null;
    size?: number | null;
};

export type FileContentMetrics = {
    characterCount: number;
    estimatedTokens: number;
    estimatorVersion: string;
};

export type AgentsMdEntry = {
    dir: string;
    relDir: string;
    source: FileFingerprint;
    target: FileFingerprint;
    sourceMetrics?: FileContentMetrics | null;
    targetMetrics?: FileContentMetrics | null;
    status: SyncStatus;
    targetNewer: boolean;
};

export type AgentsMdScanResult = {
    root: string;
    entries: AgentsMdEntry[];
    truncated: boolean;
    scannedDirs: number;
};

export type TargetInfo = {
    targetId: string;
    root: string;
    rootExists: boolean;
};

export type TargetStatus = {
    targetId: string;
    targetRoot: string;
    status: SyncStatus;
    targetNewer: boolean;
};

export type SkillEntry = {
    name: string;
    sourceDir: string;
    skillMdPath: string;
    fileCount: number;
    targets: TargetStatus[];
  description?: string | null;
  providerId?: string | null;
  locations: ResourceLocation[];
  effectivePath?: string | null;
  cliState?: ResourceState | null;
  cliSource?: DiscoverySource | null;
  metrics?: FileContentMetrics | null;
};

export type SkillsScanResult = {
    sourceRoot: string;
    skills: SkillEntry[];
  targets: TargetInfo[];
  enabledProviders: string[];
  discoveries: ResourceDiscovery[];
  diagnostics: DiscoveryDiagnostic[];
};

export type CommandEntry = {
    name: string;
    sourcePath: string;
    syncSourcePath: string;
    targets: TargetStatus[];
  description?: string | null;
  providerId?: string | null;
  locations: ResourceLocation[];
  effectivePath?: string | null;
  cliState?: ResourceState | null;
  cliSource?: DiscoverySource | null;
  metrics?: FileContentMetrics | null;
};

export type CommandsScanResult = {
    sourceRoot: string;
    commands: CommandEntry[];
  targets: TargetInfo[];
  enabledProviders: string[];
  discoveries: ResourceDiscovery[];
  diagnostics: DiscoveryDiagnostic[];
};

export type ResourceKind = "skill" | "command" | "mcp";
export type DiscoveryScope = "global" | "project" | "effective";
export type DiscoverySource = "cli" | "file";
export type ResourceState = "available" | "configured" | "disabled";
export type ResourceLocation = {
  providerId: string;
  scope: DiscoveryScope;
  root: string;
  path: string;
};
export type ResourceDiscovery = {
  providerId: string;
  kind: ResourceKind;
  scope: DiscoveryScope;
  locations: ResourceLocation[];
  effectivePath?: string | null;
  source: DiscoverySource;
  state: ResourceState;
  editable: boolean;
};
export type DiscoveryDiagnostic = {
  providerId: string;
  kind: ResourceKind;
  scope: DiscoveryScope;
  message: string;
};

export type SyncItem = {
    source: string;
    target: string;
    itemKind?: "file" | "directory";
    direction?: SyncDirection | null;
    targetId?: string | null;
};

export type SyncRequest = {
    items: SyncItem[];
    dryRun: boolean;
    force: boolean;
    mode: SyncMode;
    projectCwd?: string | null;
};

export type SyncActionResult = {
    source: string;
    target: string;
    action:
        | "create"
        | "overwrite"
        | "skip-in-sync"
        | "conflict"
        | "error"
        | "link-fallback-copy"
        | (string & {});
    reason?: string | null;
    bytes?: number | null;
};

export type SyncReport = {
    dryRun: boolean;
    actions: SyncActionResult[];
    conflicts: number;
    errors: number;
};

export type ProjectAgentsPrefs = {
    conflictChoice?: "source-wins" | "target-wins" | null;
    ignoredPaths: string[];
    enabledTargets: string[];
};

export type SaveProjectAgentsPrefsResult = {
    storedPath: string;
    createdProjectConfigDir: boolean;
};

export type McpScope =
    | { kind: "global" }
    | { kind: "project"; projectCwd: string };

export type McpServerEntry = {
    name: string;
    enabled: boolean;
  configJson: string;
  effective?: boolean | null;
  source?: string | null;
  scope?: DiscoveryScope;
  editable: boolean;
};

export type McpProviderConfig = {
    providerId: string;
    configPath: string;
    configExists: boolean;
    servers: McpServerEntry[];
  error?: string | null;
  enabled: boolean;
  diagnostics: DiscoveryDiagnostic[];
};

export type AnalyticsGroupBy = "day" | "week" | "month";

export type AnalyticsMetric = "tokens" | "estimated_usd" | "copilot_points";
export type AnalyticsTokenField = "total" | "input" | "output";
export type AnalyticsBreakdown = "total" | "provider" | "model" | "token_type";
export type AnalyticsCoverageStatus =
    | "complete"
    | "partial"
    | "pending"
    | "unsupported"
    | "error"
    | "summary_only";
export type AnalyticsDetailStatus = "ready" | "stale_revision";
export type AnalyticsSessionSort = "event_time_desc";

export type AnalyticsQuery = {
    startDate: string;
    endDate: string;
    timeZone: string;
    groupBy: AnalyticsGroupBy;
    providers: string[];
    cwd: string | null;
    model: string | null;
    cwds: string[];
    models: string[];
    includeArchived: boolean;
    metric: AnalyticsMetric;
    tokenField: AnalyticsTokenField;
    breakdown: AnalyticsBreakdown;
};

export type AnalyticsMetricValue = {
    knownValue: number | null;
    eligibleEventCount: number;
    missingFieldEventCount: number;
    priceVersions: string[];
};

export type AnalyticsValues = {
    totalTokens: AnalyticsMetricValue;
    inputTokens: AnalyticsMetricValue;
    outputTokens: AnalyticsMetricValue;
    cacheReadTokens: AnalyticsMetricValue;
    cacheWriteTokens: AnalyticsMetricValue;
    reasoningTokens: AnalyticsMetricValue;
    estimatedUsd: AnalyticsMetricValue;
    copilotPoints: AnalyticsMetricValue;
};

export type AnalyticsSeriesPoint = {
    label: string;
    startAt: string;
    endAt: string;
    values: AnalyticsValues;
    interactionCount: number;
    sessionCount: number;
};

export type AnalyticsMetricCoverage = {
    eligibleEventCount: number;
    missingFieldEventCount: number;
    status: AnalyticsCoverageStatus;
};

export type AnalyticsProviderCoverage = {
    provider: string;
    status: AnalyticsCoverageStatus;
    completeSessionCount: number;
    partialSessionCount: number;
    pendingSessionCount: number;
    unsupportedSessionCount: number;
    errorSessionCount: number;
    summaryOnlySessionCount: number;
    periodUnknownSessionCount: number;
};

export type AnalyticsCoverage = {
    metrics: Record<string, AnalyticsMetricCoverage>;
    providers: AnalyticsProviderCoverage[];
};

export type AnalyticsRankingEntry = {
    key: string;
    label: string;
    value: AnalyticsMetricValue;
    sessionCount: number;
    shareOfKnownValue: number | null;
};

export type AnalyticsSessionSummaryOnly = {
    provider: string;
    sessionId: string;
    cwd: string | null;
    model: string | null;
    activeFrom: string | null;
    activeUntil: string | null;
    values: AnalyticsValues;
    status: AnalyticsCoverageStatus;
};

export type AnalyticsReport = {
    summary: AnalyticsValues;
    previousSummary: AnalyticsValues | null;
    comparison: AnalyticsComparison;
    series: AnalyticsSeriesPoint[];
    projectRanking: AnalyticsRankingEntry[];
    modelRanking: AnalyticsRankingEntry[];
    platformRanking: AnalyticsRankingEntry[];
    supplierRanking: AnalyticsRankingEntry[];
    coverage: AnalyticsCoverage;
    sessionSummaryOnly: AnalyticsSessionSummaryOnly[];
    revision: number;
    generatedAt: string;
    timeZone: string;
};

export type AnalyticsComparisonStatus =
    | "comparable"
    | "new_usage"
    | "no_change"
    | "not_comparable";

export type AnalyticsComparison = {
    status: AnalyticsComparisonStatus;
    percentChange: number | null;
    reason: string | null;
};

export type AnalyticsSessionDetail = {
    provider: string;
    modelProviderIds: string[];
    sessionId: string;
    cwd: string | null;
    model: string | null;
    firstEventAt: string | null;
    lastEventAt: string | null;
    values: AnalyticsValues;
    sessionSummaryOnly: AnalyticsValues | null;
    status: AnalyticsCoverageStatus;
};

export type AnalyticsSessionPage = {
    status: AnalyticsDetailStatus;
    revision: number;
    page: number;
    pageSize: number;
    totalCount: number;
    items: AnalyticsSessionDetail[];
};

export type AnalyticsSessionPageQuery = {
    query: AnalyticsQuery;
    revision: number;
    page: number;
    pageSize: number;
    sort: AnalyticsSessionSort;
};

export type UsageIndexProgress = {
    totalSessions: number;
    indexedSessions: number;
    pendingSessions: number;
    unsupportedSessions: number;
    errorSessions: number;
    unindexedSessions: number;
};

export type ModelPricingEntry = {
    provider: string;
    model: string;
    status: string;
    promptUsdPerMillion: number | null;
    completionUsdPerMillion: number | null;
    cacheReadUsdPerMillion: number | null;
    cacheWriteUsdPerMillion: number | null;
    source: string;
    fetchedAt: number;
    expiresAt: number;
    errorKind: string | null;
    hidden: boolean;
};

export type ManualModelPricingInput = {
    provider: string;
    model: string;
    promptUsdPerMillion: number;
    completionUsdPerMillion: number;
    cacheReadUsdPerMillion: number | null;
    cacheWriteUsdPerMillion: number | null;
};

export type SessionActivityStatus = {
    sessionId: string;
    /** "idle" | "active" | "waiting" | "done" */
    status: "idle" | "active" | "waiting" | "done";
    /** "thinking" | "tool_call" | "file_op" | "sub_agent" | "working" | "completed" */
    detail?: string | null;
    lastActivityAt?: string | null;
};

export type IdeLauncherType =
    | "terminal"
    | "opencode"
    | "claude"
    | "codex"
    | "copilot"
    | "vscode"
    | "gemini"
    | "explorer";

export type ToolAvailability = {
    copilot: boolean;
    opencode: boolean;
    claude: boolean;
    codex: boolean;
    gemini: boolean;
    vscode: boolean;
    herdr: boolean;
    herdrServerRunning: boolean;
};

export type ProviderIntegrationState =
    | "installed"
    | "outdated"
    | "missing"
    | "manual_required"
    | "error";

export type ProviderIntegrationStatus = {
    provider: string;
    status: ProviderIntegrationState;
    configPath?: string | null;
    bridgePath?: string | null;
    /** 目前安裝的 integration 版本號，null 表示未安裝或無法讀取 */
    installedVersion?: number | null;
    lastEventAt?: string | null;
    lastError?: string | null;
};

export type SessionStats = {
    outputTokens: number;
    inputTokens: number;
    interactionCount: number;
    toolCallCount: number;
    durationMinutes: number;
    modelsUsed: string[];
    reasoningCount: number;
    toolBreakdown: Record<string, number>;
    modelMetrics: Record<string, ModelMetricsEntry>;
    isLive: boolean;
};

export type ModelMetricsEntry = {
    requestsCount: number;
    requestsCost: number;
    inputTokens: number;
    outputTokens: number;
};

export type SettingsSection = "general" | "language" | "icon-style";

export type ProjectGroup = {
    key: string;
    title: string;
    pathLabel: string;
    branchLabel?: string | null;
    sessions: SessionInfo[];
    updatedAtLabel: string;
};

export type SortKey = "updatedAt" | "createdAt" | "summary";

export type SessionSearchTarget = {
    id: string;
    provider: string;
    sessionDir: string;
};

export type RealtimeStatus = "connecting" | "active" | "error";

export type ConfirmDialogState = {
    title: string;
    message: string;
    actionLabel: string;
    tone: "danger" | "primary";
    onConfirm: () => void;
};

export type EditDialogState = {
    key?: string;
    title: string;
    message: string;
    actionLabel: string;
    secondaryActionLabel?: string;
    secondaryActionTone?: "danger" | "neutral";
    initialValue: string;
    multiline?: boolean;
    onConfirm: (value: string) => void;
    onSecondaryAction?: (value: string) => void;
};

// Sisyphus (.sisyphus) 相關型別

export type SisyphusBoulder = {
    activePlan: string | null;
    planName: string | null;
    agent: string | null;
    sessionIds: string[];
    startedAt: string | null;
};

export type SisyphusPlan = {
    name: string;
    path: string;
    title: string | null;
    tldr: string | null;
    isActive: boolean;
};

export type SisyphusNotepad = {
    name: string;
    hasIssues: boolean;
    hasLearnings: boolean;
    issuesPath: string | null;
    learningsPath: string | null;
};

export type SisyphusData = {
    activePlan: SisyphusBoulder | null;
    plans: SisyphusPlan[];
    notepads: SisyphusNotepad[];
    evidenceFiles: string[];
    draftFiles: string[];
};

// OpenSpec 相關型別

export type OpenSpecChange = {
    name: string;
    hasProposal: boolean;
    hasDesign: boolean;
    hasTasks: boolean;
    taskProgress?: OpenSpecTaskProgress | null;
    specsCount: number;
    specs: OpenSpecSpec[];
    createdAt?: string | null;
};

export type OpenSpecTaskProgressStatus = "not_started" | "in_progress" | "done" | (string & {});

export type OpenSpecTaskProgress = {
    done: number;
    total: number;
    status: OpenSpecTaskProgressStatus;
};

export type OpenSpecSpec = {
    name: string;
    path: string;
};

export type OpenSpecData = {
    schema: string | null;
    activeChanges: OpenSpecChange[];
    archivedChanges: OpenSpecChange[];
    specs: OpenSpecSpec[];
};

export type TreeNode = {
    id: string;
    label: string;
    badge?: string;
    icon?: "proposal" | "design" | "tasks" | "spec" | "change" | "section" | "folder" | "plan" | "note" | "evidence" | "draft" | "agents";
    tone?: "neutral" | "muted" | "not_started" | "in_progress" | "done";
    progress?: OpenSpecTaskProgress | null;
    children?: TreeNode[];
    defaultOpen?: boolean;
    filePath?: string;
    filePathType?: "absolute" | "openspec";
    trailingMeta?: string;
    trailingMetaTitle?: string;
    sourceKind?: "sisyphus" | "openspec";
};

export type SessionTargetedPayload = {
    sessionId: string;
    cwd: string;
    eventType: string;
};

/** 後端 InterventionRegistry 的單筆等待授權項目，欄位最小化不含指令/路徑 */
export type InterventionItem = {
    sessionId: string;
    projectName: string;
    toolLabel?: string | null;
    since: string;
};

export type ActivityHintPayload = {
    cwd: string;
    eventType: string;
    title: string | null;
    error: string | null;
    /** 後端計算好的狀態，可直接更新 activityStatusMap */
    sessionId?: string | null;
    status?: "active" | "waiting" | "idle" | null;
    detail?: string | null;
    lastActivityAt?: string | null;
};

export type BridgeEventLogEntry = {
    id: string;
    provider: string;
    eventType: string;
    timestamp: string;
    cwd: string | null;
    sessionId: string | null;
    title: string | null;
    error: string | null;
    /** "targeted" | "fallback" | "full_refresh" | "skipped_dedup" | "skipped_rate_limit" */
    status: string;
};

export type ProviderQuota = {
    provider: string;
    billingPeriod: string;
    inputTokens: number;
    outputTokens: number;
    cacheCreationTokens: number;
    cacheReadTokens: number;
    costUsd: number;
    monthlyLimitTokens: number | null;
    monthlyLimitUsd: number | null;
    resetDay: number;
    nextResetDate: string;
};

// ── Provider Quota Snapshot 相關型別 ──────────────────────────────────────────

export type QuotaWindow = {
    windowKey: string;
    label: string;
    utilization: number;
    resetsAt?: string | null;
    /** 模型群組名稱（如 "Gemini Models"），僅 Antigravity 使用 */
    group?: string | null;
};

export type ExtraCredits = {
    isEnabled: boolean;
    monthlyLimit?: number | null;
    usedCredits: number;
    utilization?: number | null;
};

export type ResetCreditEntry = {
    grantedAt?: string | null;
    expiresAt?: string | null;
    status: string;
};

export type ResetCredits = {
    availableCount: number;
    credits: ResetCreditEntry[];
};

export type QuotaSnapshot = {
    provider: string;
    /** "ok" | "error" | "unsupported" | "no_auth" | "rate_limited" */
    status: "ok" | "error" | "unsupported" | "no_auth" | "rate_limited";
    /** "remote_api" | "local_scan" */
    source: "remote_api" | "local_scan";
    fetchedAt: string;
    errorMessage?: string | null;
    windows?: QuotaWindow[] | null;
    extraCredits?: ExtraCredits | null;
    resetCredits?: ResetCredits | null;
    plan?: string | null;
};

export type ClaudeUsageBlock = {
    startTime: string;
    endTime: string;
    isActive: boolean;
    inputTokens: number;
    outputTokens: number;
    cacheCreationTokens: number;
    cacheReadTokens: number;
    costUsd: number;
    usageLimitResetTime: string | null;
};

