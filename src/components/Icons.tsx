import {
  Archive,
  ArchiveRestore,
  Bot,
  ChartNoAxesCombined,
  ChevronLeft,
  ChevronRight,
  CircleDot,
  Clipboard,
  Eye,
  ExternalLink,
  FilePenLine,
  FileText,
  Folder,
  FolderOpen,
  LayoutDashboard,
  Lock,
  Move,
  Moon,
  PanelLeft,
  Pin,
  PinOff,
  Plug,
  RefreshCw,
  Save,
  Search,
  Settings,
  Sun,
  Repeat2,
  Tags,
  Terminal,
  Trash2,
  type LucideProps,
  X,
} from "lucide-react";

export type IconProps = LucideProps;

export const ICON_SIZE = 16;
export const ICON_STROKE_WIDTH = 1.8;

function withDefaults(Icon: typeof Archive) {
  return function IconWithDefaults({ size = ICON_SIZE, strokeWidth = ICON_STROKE_WIDTH, ...props }: IconProps) {
    return <Icon aria-hidden="true" size={size} strokeWidth={strokeWidth} {...props} />;
  };
}

export const TerminalIcon = withDefaults(Terminal);
export const CopyIcon = withDefaults(Clipboard);
export const ArchiveIcon = withDefaults(Archive);
export const UnarchiveIcon = withDefaults(ArchiveRestore);
export const DeleteIcon = withDefaults(Trash2);
export const PinIcon = withDefaults(Pin);
export const UnpinIcon = withDefaults(PinOff);
export const EditNotesIcon = withDefaults(FilePenLine);
export const EditTagsIcon = withDefaults(Tags);
export const PlanIcon = withDefaults(FileText);
export const SunIcon = withDefaults(Sun);
export const MoonIcon = withDefaults(Moon);
export const StatsIcon = withDefaults(ChartNoAxesCombined);
export const AgentsIcon = withDefaults(Bot);
export const RefreshIcon = withDefaults(RefreshCw);
export const ChevronLeftIcon = withDefaults(ChevronLeft);
export const SaveIcon = withDefaults(Save);
export const PlugIcon = withDefaults(Plug);
export const ExternalLinkIcon = withDefaults(ExternalLink);
export const FolderIcon = withDefaults(Folder);
export const FolderOpenIcon = withDefaults(FolderOpen);
export const FocusIcon = withDefaults(CircleDot);
export const SearchIcon = withDefaults(Search);
export const EyeIcon = withDefaults(Eye);
export const SyncIcon = withDefaults(Repeat2);
export const DashboardIcon = withDefaults(LayoutDashboard);
export const SettingsIcon = withDefaults(Settings);
export const PanelLeftIcon = withDefaults(PanelLeft);
export const CloseIcon = withDefaults(X);
export const ChevronRightIcon = withDefaults(ChevronRight);
export const LockIcon = withDefaults(Lock);
export const MoveIcon = withDefaults(Move);
