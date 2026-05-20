export type Theme = "light" | "dark" | "system";

export interface AppSettings {
  scanPath: string;
  keyword: string;
  enabledPresets: string[];
  customExtensions: Record<string, string[]>;
  ocrEnabled: boolean;
  sidebarOpen: boolean;
  theme: Theme;
}

export interface ScanConfig {
  path: string;
  keyword: string;
  scan_types: string[];
  file_extensions: string[];
  exclude_patterns: string[];
}

export interface ScanResult {
  file_path: string;
  file_name: string;
  match_type: string;
  match_line: number | null;
  match_context: string | null;
  match_bboxes: string | null;
  file_size: number;
  file_extension: string;
  is_dir: boolean;
}

export interface OCRBox {
  x: number;
  y: number;
  w: number;
  h: number;
}

export interface ImagePreview {
  base64: string;
  bboxes: OCRBox[];
}

export interface ScanProgress {
  scan_id: string;
  parent_scan_id?: string;
  status: "scanning" | "completed" | "cancelled";
  files_scanned: number;
  results_found: number;
  current_path: string;
  results: ScanResult[];
  pending_confirmations: PendingConfirmation[];
  skipped_dirs: SkippedDir[];
}

export interface PendingConfirmation {
  id: string;
  path: string;
  entry_count: number;
}

export interface SkippedDir {
  path: string;
  reason: string;
}

export interface ContextLine {
  line_number: number;
  content: string;
  is_match: boolean;
}

export interface FilePreview {
  file_path: string;
  file_name: string;
  file_size: number;
  file_type: string;
  match_line: number | null;
  context_lines: ContextLine[];
  match_type: string;
}

export interface FileTypePreset {
  label: string;
  icon: any;
  extensions: string[];
}

export type ResultTab = "all" | "document" | "code" | "image" | "config";

export interface TreeNode {
  name: string;
  path: string;
  isDir: boolean;
  children: TreeNode[];
  result?: ScanResult;
}

export interface AppConfig {
  version: number;
  scan: {
    large_dir_threshold: number;
    ask_on_large_dir: boolean;
  };
  display: {
    default_expand_count: number;
    ocr_highlight_enabled: boolean;
    match_context_length: number;
  };
  content_extraction: {
    docx: boolean;
    xlsx: boolean;
    pdf: boolean;
    pptx: boolean;
  };
  ocr: {
    enabled: boolean;
    provider: "macos_native" | "api";
    api_endpoint: string | null;
    api_key: string | null;
    api_secret: string | null;
    languages: string[];
  };
  skip_rules: string[];
  scan_rules: string[];
}
