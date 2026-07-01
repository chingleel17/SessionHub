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
    claudeRoot?: string;
    hookScriptsPath?: string;
    claudeQuotaResetDay?: number;
    claudeMonthlyLimitTokens?: number | null;
    claudeMonthlyLimitUsd?: number | null;
    enableQuotaMonitoring?: boolean;
    quotaEnabledProviders?: string[];
};

export type AnalyticsGroupBy = "day" | "week" | "month";

export type AnalyticsDataPoint = {
    label: string;
    outputTokens: number;
    inputTokens: number;
    interactionCount: number;
    costPoints: number;
    sessionCount: number;
    missingCount: number;
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
    | "copilot"
    | "vscode"
    | "gemini"
    | "explorer";

export type ToolAvailability = {
    copilot: boolean;
    opencode: boolean;
    gemini: boolean;
    vscode: boolean;
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

export type SortKey = "updatedAt" | "createdAt" | "summaryCount" | "summary";

export type RealtimeStatus = "connecting" | "active" | "error";

export type ConfirmDialogState = {
    title: string;
    message: string;
    actionLabel: string;
    tone: "danger" | "primary";
    onConfirm: () => void;
};

export type EditDialogState = {
    title: string;
    message: string;
    actionLabel: string;
    initialValue: string;
    multiline?: boolean;
    onConfirm: (value: string) => void;
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
    icon?: "proposal" | "design" | "tasks" | "spec" | "change" | "section" | "folder" | "plan" | "note" | "evidence" | "draft";
    tone?: "neutral" | "muted" | "not_started" | "in_progress" | "done";
    progress?: OpenSpecTaskProgress | null;
    children?: TreeNode[];
    defaultOpen?: boolean;
    filePath?: string;
    filePathType?: "absolute" | "openspec";
};

export type SessionTargetedPayload = {
    sessionId: string;
    cwd: string;
    eventType: string;
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
};

export type LocalTokenUsage = {
    inputTokens: number;
    outputTokens: number;
    periodLabel: string;
};

export type ExtraCredits = {
    isEnabled: boolean;
    monthlyLimit?: number | null;
    usedCredits: number;
    utilization?: number | null;
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
    localTokens?: LocalTokenUsage | null;
    extraCredits?: ExtraCredits | null;
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
