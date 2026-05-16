import { useState, useMemo, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import {
  Search,
  FolderOpen,
  FileText,
  FileCode2,
  Image,
  Settings,
  Trash2,
  ChevronRight,
  ChevronDown,
  Plus,
  X,
  Check,
  Loader2,
  Shield,
  Eye,
  Maximize2,
  Minimize2,
  Scan,
  PanelLeftClose,
  PanelLeftOpen,
  SlidersHorizontal,
  ExternalLink,
  ScanEye,
} from "lucide-react";
import "./index.css";

interface ScanConfig {
  path: string;
  keyword: string;
  scan_types: string[];
  file_extensions: string[];
  exclude_patterns: string[];
}

interface ScanResult {
  file_path: string;
  file_name: string;
  match_type: string;
  match_line: number | null;
  match_context: string | null;
  file_size: number;
  file_extension: string;
  is_dir: boolean;
}

interface ScanProgress {
  scan_id: string;
  status: string;
  files_scanned: number;
  results_found: number;
  current_path: string;
  results: ScanResult[];
}

interface ContextLine {
  line_number: number;
  content: string;
  is_match: boolean;
}

interface FilePreview {
  file_path: string;
  file_name: string;
  file_size: number;
  file_type: string;
  match_line: number | null;
  context_lines: ContextLine[];
  match_type: string;
}

interface TreeNode {
  name: string;
  path: string;
  isDir: boolean;
  children: TreeNode[];
  files: ScanResult[];
}

type ResultTab = "all" | "document" | "code" | "image" | "config";

interface FileTypePreset {
  label: string;
  icon: typeof FileText;
  extensions: string[];
}

const DEFAULT_PRESETS: Record<string, FileTypePreset> = {
  document: {
    label: "文档",
    icon: FileText,
    extensions: ["txt", "md", "csv", "rtf", "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx", "pages", "numbers", "key"],
  },
  code: {
    label: "代码",
    icon: FileCode2,
    extensions: ["rs", "py", "js", "ts", "tsx", "jsx", "go", "java", "c", "cpp", "h", "hpp", "swift", "rb", "php", "html", "css", "scss", "less", "sql", "sh", "bash", "zsh"],
  },
  image: {
    label: "图片",
    icon: Image,
    extensions: ["jpg", "jpeg", "png", "gif", "webp", "bmp", "tiff", "tif", "heic", "heif", "svg", "ico"],
  },
  config: {
    label: "配置",
    icon: Settings,
    extensions: ["env", "gitignore", "dockerignore", "makefile", "cmake", "ini", "cfg", "conf", "config", "json", "yaml", "yml", "toml", "xml"],
  },
};

// Persistence helpers
const STORAGE_KEY = "filescope_settings";

interface AppSettings {
  scanPath: string;
  keyword: string;
  enabledPresets: string[];
  customExtensions: Record<string, string[]>;
  ocrEnabled: boolean;
  sidebarOpen: boolean;
}

function loadSettings(): AppSettings {
  try {
    const saved = localStorage.getItem(STORAGE_KEY);
    if (saved) {
      return JSON.parse(saved);
    }
  } catch (e) {
    console.error("Failed to load settings:", e);
  }
  return {
    scanPath: "",
    keyword: "",
    enabledPresets: ["document", "code", "config"],
    customExtensions: {},
    ocrEnabled: false,
    sidebarOpen: true,
  };
}

function saveSettings(settings: AppSettings) {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(settings));
  } catch (e) {
    console.error("Failed to save settings:", e);
  }
}

function App() {
  const [settings, setSettings] = useState<AppSettings>(loadSettings);
  const [isScanning, setIsScanning] = useState(false);
  const [scanProgress, setScanProgress] = useState<ScanProgress | null>(null);
  const [selectedResults, setSelectedResults] = useState<Set<string>>(new Set());
  const [previewFile, setPreviewFile] = useState<FilePreview | null>(null);
  const [previewImage, setPreviewImage] = useState<string | null>(null);
  const [scanInterval, setScanInterval] = useState<number | null>(null);
  const [activeTab, setActiveTab] = useState<ResultTab>("all");
  const [expandedFolders, setExpandedFolders] = useState<Set<string>>(new Set());

  const enabledPresets = new Set(settings.enabledPresets);
  const customExtensions = settings.customExtensions;

  // Save settings on change
  useEffect(() => {
    saveSettings(settings);
  }, [settings]);

  // Keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Cmd+B or Ctrl+B to toggle sidebar
      if ((e.metaKey || e.ctrlKey) && e.key === "b") {
        e.preventDefault();
        updateSettings({ sidebarOpen: !settings.sidebarOpen });
      }
      // Cmd+Enter to start scan
      if ((e.metaKey || e.ctrlKey) && e.key === "Enter" && !isScanning) {
        e.preventDefault();
        handleStartScan();
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [isScanning, settings]);

  const updateSettings = (updates: Partial<AppSettings>) => {
    setSettings((prev) => ({ ...prev, ...updates }));
  };

  const getAllExtensions = (): string[] => {
    const exts: string[] = [];
    for (const preset of settings.enabledPresets) {
      const defaultExts = DEFAULT_PRESETS[preset]?.extensions || [];
      const custom = settings.customExtensions[preset] || [];
      exts.push(...defaultExts, ...custom);
    }
    return [...new Set(exts)];
  };

  const handleSelectDirectory = async () => {
    try {
      const selected = await open({ directory: true, multiple: false, title: "选择扫描目录" });
      if (selected) updateSettings({ scanPath: selected });
    } catch (err) {
      console.error("Failed to select directory:", err);
    }
  };

  const handleTogglePreset = (preset: string) => {
    const next = new Set(settings.enabledPresets);
    next.has(preset) ? next.delete(preset) : next.add(preset);
    updateSettings({ enabledPresets: Array.from(next) });
  };

  const handleAddExtension = (preset: string, ext: string) => {
    const cleanExt = ext.trim().toLowerCase().replace(/^\./, "");
    if (!cleanExt) return;
    const current = settings.customExtensions[preset] || [];
    if (current.includes(cleanExt)) return;
    updateSettings({
      customExtensions: {
        ...settings.customExtensions,
        [preset]: [...current, cleanExt],
      },
    });
  };

  const handleRemoveExtension = (preset: string, ext: string) => {
    updateSettings({
      customExtensions: {
        ...settings.customExtensions,
        [preset]: (settings.customExtensions[preset] || []).filter((e) => e !== ext),
      },
    });
  };

  const handleStartScan = async () => {
    const extensions = getAllExtensions();
    if (!settings.scanPath || !settings.keyword || extensions.length === 0) return;

    const scanTypes = ["file_name", "text_content"];
    if (settings.enabledPresets.includes("image")) {
      scanTypes.push("exif_data");
      if (settings.ocrEnabled) {
        scanTypes.push("ocr_text");
      }
    }

    const config: ScanConfig = {
      path: settings.scanPath,
      keyword: settings.keyword,
      scan_types: scanTypes,
      file_extensions: extensions,
      exclude_patterns: ["node_modules", ".git", "target", "dist", "build"],
    };
    try {
      setIsScanning(true);
      setSelectedResults(new Set());
      setActiveTab("all");
      const scanId = await invoke<string>("start_scan", { config });
      const interval = window.setInterval(async () => {
        try {
          const progress = await invoke<ScanProgress>("get_scan_progress", { scanId });
          setScanProgress(progress);
          if (progress.status === "completed" || progress.status === "cancelled") {
            setIsScanning(false);
            window.clearInterval(interval);
            setScanInterval(null);
          }
        } catch (err) {
          console.error("Failed to get progress:", err);
        }
      }, 200);
      setScanInterval(interval);
    } catch (err) {
      console.error("Failed to start scan:", err);
      setIsScanning(false);
    }
  };

  const handleCancelScan = async () => {
    if (scanProgress?.scan_id) {
      try {
        await invoke("cancel_scan", { scanId: scanProgress.scan_id });
        setIsScanning(false);
        if (scanInterval) { window.clearInterval(scanInterval); setScanInterval(null); }
      } catch (err) { console.error("Failed to cancel scan:", err); }
    }
  };

  const handleSelectAll = () => {
    if (scanProgress?.results) setSelectedResults(new Set(scanProgress.results.map((r) => r.file_path)));
  };

  const handleDeselectAll = () => setSelectedResults(new Set());

  const handleToggleSelect = (filePath: string) => {
    setSelectedResults((prev) => {
      const next = new Set(prev);
      next.has(filePath) ? next.delete(filePath) : next.add(filePath);
      return next;
    });
  };

  const handleDeleteSelected = async () => {
    if (selectedResults.size === 0) return;
    try {
      await invoke<number>("move_to_trash", { filePaths: Array.from(selectedResults) });
      if (scanProgress) {
        const remaining = scanProgress.results.filter((r) => !selectedResults.has(r.file_path));
        setScanProgress({ ...scanProgress, results: remaining, results_found: remaining.length });
      }
      setSelectedResults(new Set());
    } catch (err) { console.error("Failed to delete files:", err); }
  };

  const handlePreviewFile = async (result: ScanResult) => {
    if (result.is_dir) {
      handleToggleFolder(result.file_path);
      return;
    }
    // Check if it's an image
    const imageExts = ["jpg", "jpeg", "png", "gif", "webp", "bmp", "tiff", "tif", "heic", "heif", "svg", "ico"];
    if (imageExts.includes(result.file_extension.toLowerCase())) {
      try {
        const base64 = await invoke<string>("read_image_as_base64", { filePath: result.file_path });
        setPreviewImage(base64);
      } catch (err) { console.error("Failed to load image:", err); }
      return;
    }
    try {
      const preview = await invoke<FilePreview>("read_file_preview", { filePath: result.file_path, matchLine: result.match_line, contextLines: 5 });
      setPreviewFile(preview);
    } catch (err) { console.error("Failed to read file:", err); }
  };

  const handleRevealInFinder = async (e: React.MouseEvent, filePath: string) => {
    e.stopPropagation();
    try {
      await invoke("reveal_in_finder", { filePath });
    } catch (err) { console.error("Failed to reveal in finder:", err); }
  };

  const getFileCategory = (ext: string): ResultTab => {
    for (const [key, preset] of Object.entries(DEFAULT_PRESETS)) {
      if (preset.extensions.includes(ext) || (customExtensions[key] || []).includes(ext)) {
        return key as ResultTab;
      }
    }
    return "document";
  };

  const getFileIcon = (ext: string) => {
    const cat = getFileCategory(ext);
    const IconMap: Record<string, typeof FileText> = {
      document: FileText,
      code: FileCode2,
      image: Image,
      config: Settings,
    };
    return IconMap[cat] || FileText;
  };

  const getMatchTypeLabel = (type: string): string => ({ filename: "文件名", content: "内容", exif: "EXIF", ocr: "OCR" }[type] || type);

  const filteredResults = useMemo(() => {
    if (!scanProgress?.results) return [];
    if (activeTab === "all") return scanProgress.results;
    return scanProgress.results.filter((r) => getFileCategory(r.file_extension) === activeTab);
  }, [scanProgress, activeTab, customExtensions]);

  const tabCounts = useMemo(() => {
    if (!scanProgress?.results) return { all: 0, document: 0, code: 0, image: 0, config: 0 };
    const counts = { all: scanProgress.results.length, document: 0, code: 0, image: 0, config: 0 };
    scanProgress.results.forEach((r) => {
      const cat = getFileCategory(r.file_extension);
      counts[cat]++;
    });
    return counts;
  }, [scanProgress, customExtensions]);

  const buildTree = useMemo(() => {
    if (!settings.scanPath || filteredResults.length === 0) return null;
    const root: TreeNode = { name: settings.scanPath.split("/").pop() || settings.scanPath, path: settings.scanPath, isDir: true, children: [], files: [] };
    const dirMap = new Map<string, TreeNode>();
    dirMap.set(settings.scanPath, root);
    const sorted = [...filteredResults].sort((a, b) => a.file_path.localeCompare(b.file_path));
    for (const result of sorted) {
      const parts = result.file_path.replace(settings.scanPath, "").split("/").filter(Boolean);
      let currentPath = settings.scanPath;
      let currentNode = root;
      for (let i = 0; i < parts.length - 1; i++) {
        currentPath += "/" + parts[i];
        if (!dirMap.has(currentPath)) {
          const newNode: TreeNode = { name: parts[i], path: currentPath, isDir: true, children: [], files: [] };
          currentNode.children.push(newNode);
          dirMap.set(currentPath, newNode);
        }
        currentNode = dirMap.get(currentPath)!;
      }
      currentNode.files.push(result);
    }
    return root;
  }, [settings.scanPath, filteredResults]);

  const handleToggleFolder = (path: string) => {
    setExpandedFolders((prev) => {
      const next = new Set(prev);
      next.has(path) ? next.delete(path) : next.add(path);
      return next;
    });
  };

  const handleExpandAll = () => {
    if (!buildTree) return;
    const paths: string[] = [];
    const collect = (node: TreeNode) => { paths.push(node.path); node.children.forEach(collect); };
    collect(buildTree);
    setExpandedFolders(new Set(paths));
  };

  const handleCollapseAll = () => setExpandedFolders(new Set());

  const selectedCount = settings.enabledPresets.length;
  const extCount = getAllExtensions().length;

  const renderTreeNode = (node: TreeNode, depth: number = 0) => {
    return (
      <div key={node.path}>
        {node.children.map((child) => {
          const isChildExpanded = expandedFolders.has(child.path);
          const childCount = child.files.length + child.children.reduce((acc, c) => acc + c.files.length, 0);
          return (
            <div key={child.path}>
              <div className="tree-folder" style={{ paddingLeft: `${depth * 20 + 16}px` }}>
                <span className="tree-arrow" onClick={() => handleToggleFolder(child.path)}>
                  {isChildExpanded ? <ChevronDown size={14} /> : <ChevronRight size={14} />}
                </span>
                <FolderOpen size={16} className="tree-folder-icon" onClick={() => handleToggleFolder(child.path)} />
                <span className="tree-folder-name" onClick={() => handleToggleFolder(child.path)}>{child.name}</span>
                <span className="tree-folder-count">{childCount}</span>
                <button className="tree-action-btn" onClick={(e) => handleRevealInFinder(e, child.path)} title="在访达中打开">
                  <ExternalLink size={12} />
                </button>
              </div>
              {isChildExpanded && renderTreeNode(child, depth + 1)}
            </div>
          );
        })}
        {node.files.map((file, idx) => {
          const Icon = file.is_dir ? FolderOpen : getFileIcon(file.file_extension);
          return (
            <div key={`${file.file_path}-${idx}`} className={`tree-file ${file.is_dir ? "tree-file--dir" : ""}`} style={{ paddingLeft: `${(depth + 1) * 20 + 16}px` }} onClick={() => handlePreviewFile(file)}>
              <input type="checkbox" className="result-item-checkbox" checked={selectedResults.has(file.file_path)} onClick={(e) => e.stopPropagation()} onChange={() => handleToggleSelect(file.file_path)} />
              <Icon size={16} className={`tree-file-icon ${file.is_dir ? "tree-file-icon--dir" : ""}`} />
              <span className="tree-file-name">{file.file_name}</span>
              {file.is_dir && <span className="result-item-badge result-item-badge--dir">文件夹</span>}
              <span className={`result-item-badge result-item-badge--${file.match_type}`}>{getMatchTypeLabel(file.match_type)}</span>
              {file.match_line && <span className="result-item-badge">行 {file.match_line}</span>}
              {file.match_context && !file.is_dir && <span className="tree-file-preview">{file.match_context}</span>}
              <button className="tree-action-btn" onClick={(e) => handleRevealInFinder(e, file.file_path)} title="在访达中打开">
                <ExternalLink size={12} />
              </button>
            </div>
          );
        })}
      </div>
    );
  };

  return (
    <div className="app-container">
      <div className="title-bar">
        <div className="title-bar-left">
          <button className="sidebar-toggle" onClick={() => updateSettings({ sidebarOpen: !settings.sidebarOpen })}>
            {settings.sidebarOpen ? <PanelLeftClose size={16} /> : <PanelLeftOpen size={16} />}
          </button>
          <div className="title-bar-title">
            <Shield size={16} />
            <span>FileScope</span>
          </div>
        </div>
        <div className="title-bar-center">
          {settings.scanPath && (
            <div className="title-path">
              <FolderOpen size={12} />
              <span>{settings.scanPath}</span>
            </div>
          )}
        </div>
        <div className="title-bar-right">
          <button className="title-bar-btn" onClick={handleSelectDirectory}>
            <FolderOpen size={14} />
            <span>选择目录</span>
          </button>
        </div>
      </div>

      <div className="main-content">
        <div className={`sidebar ${settings.sidebarOpen ? "open" : ""}`}>
          <div className="sidebar-content">
            <div className="sidebar-section">
              <div className="sidebar-section-title">
                <SlidersHorizontal size={14} />
                <span>文件类型</span>
                <span className="sidebar-badge">{selectedCount}</span>
              </div>
              {Object.entries(DEFAULT_PRESETS).map(([key, preset]) => {
                const Icon = preset.icon;
                const isEnabled = enabledPresets.has(key);
                const custom = customExtensions[key] || [];
                return (
                  <div key={key} className={`preset-item ${isEnabled ? "active" : ""}`}>
                    <div className="preset-header" onClick={() => handleTogglePreset(key)}>
                      <div className="preset-checkbox">
                        {isEnabled && <Check size={10} />}
                      </div>
                      <Icon size={15} className="preset-icon" />
                      <span className="preset-label">{preset.label}</span>
                      <span className="preset-count">{preset.extensions.length + custom.length}</span>
                    </div>
                    {isEnabled && (
                      <div className="preset-extensions">
                        <div className="ext-tags">
                          {preset.extensions.map((ext) => (
                            <span key={ext} className="ext-tag">{ext}</span>
                          ))}
                          {custom.map((ext) => (
                            <span key={ext} className="ext-tag ext-tag--custom">
                              {ext}
                              <button className="ext-remove" onClick={(e) => { e.stopPropagation(); handleRemoveExtension(key, ext); }}>
                                <X size={8} />
                              </button>
                            </span>
                          ))}
                        </div>
                        <div className="ext-add">
                          <input
                            type="text"
                            className="ext-input"
                            placeholder="添加扩展名"
                            onKeyDown={(e) => {
                              if (e.key === "Enter") {
                                handleAddExtension(key, e.currentTarget.value);
                                e.currentTarget.value = "";
                              }
                            }}
                          />
                          <button className="ext-add-btn" onClick={(e) => {
                            const input = (e.currentTarget.previousElementSibling as HTMLInputElement);
                            handleAddExtension(key, input.value);
                            input.value = "";
                          }}>
                            <Plus size={12} />
                          </button>
                        </div>
                      </div>
                    )}
                  </div>
                );
              })}
            </div>

            {settings.enabledPresets.includes("image") && (
              <div className="sidebar-section">
                <div className="sidebar-section-title">
                  <ScanEye size={14} />
                  <span>图片 OCR</span>
                </div>
                <div className="ocr-toggle">
                  <label className="toggle-switch">
                    <input
                      type="checkbox"
                      checked={settings.ocrEnabled}
                      onChange={(e) => {
                        if (e.target.checked) {
                          // Check if platform is Linux
                          const platform = navigator.platform.toLowerCase();
                          if (platform.includes("linux")) {
                            alert("Linux 平台暂不支持 OCR 功能");
                            return;
                          }
                        }
                        updateSettings({ ocrEnabled: e.target.checked });
                      }}
                    />
                    <span className="toggle-slider"></span>
                  </label>
                  <div className="toggle-info">
                    <span className="toggle-label">启用文字识别</span>
                    <span className="toggle-desc">识别图片中的文字内容</span>
                  </div>
                </div>
              </div>
            )}

            <div className="sidebar-footer">
              <div className="sidebar-info">
                <span>{selectedCount} 类 · {extCount} 种格式</span>
                {settings.ocrEnabled && <span> · OCR 已启用</span>}
              </div>
            </div>
          </div>
        </div>

        <div className="main-panel">
          <div className="search-bar">
            <div className="search-row">
              <div className="search-input-wrapper">
                <Search size={15} className="search-icon" />
                <input
                  type="text"
                  className="search-input"
                  placeholder="输入关键字搜索文件..."
                  value={settings.keyword}
                  onChange={(e) => updateSettings({ keyword: e.target.value })}
                  onKeyDown={(e) => e.key === "Enter" && !isScanning && handleStartScan()}
                />
              </div>
              {isScanning ? (
                <button className="search-btn" onClick={handleCancelScan}>
                  <Loader2 size={14} className="spin" />
                  <span>停止</span>
                </button>
              ) : (
                <button className="search-btn" onClick={handleStartScan} disabled={!settings.scanPath || !settings.keyword || extCount === 0}>
                  <Scan size={14} />
                  <span>搜索</span>
                </button>
              )}
            </div>
          </div>

          {isScanning && (
            <div className="scan-progress">
              <div className="progress-bar">
                <div className="progress-track" />
              </div>
              <div className="progress-info">
                <div className="progress-stats">
                  <span className="progress-count">已扫描 {scanProgress?.files_scanned || 0} 个文件</span>
                  <span className="progress-found">找到 {scanProgress?.results_found || 0} 个结果</span>
                </div>
                {scanProgress?.current_path && (
                  <div className="progress-path" title={scanProgress.current_path}>
                    <FolderOpen size={12} />
                    <span>{scanProgress.current_path}</span>
                  </div>
                )}
              </div>
            </div>
          )}

          {(scanProgress || isScanning) && (
            <>
              <div className="results-toolbar">
                <div className="results-tabs">
                  <button className={`tab ${activeTab === "all" ? "active" : ""}`} onClick={() => setActiveTab("all")}>
                    全部 <span className="tab-badge">{tabCounts.all}</span>
                  </button>
                  {tabCounts.document > 0 && (
                    <button className={`tab ${activeTab === "document" ? "active" : ""}`} onClick={() => setActiveTab("document")}>
                      <FileText size={12} /> 文档 <span className="tab-badge">{tabCounts.document}</span>
                    </button>
                  )}
                  {tabCounts.code > 0 && (
                    <button className={`tab ${activeTab === "code" ? "active" : ""}`} onClick={() => setActiveTab("code")}>
                      <FileCode2 size={12} /> 代码 <span className="tab-badge">{tabCounts.code}</span>
                    </button>
                  )}
                  {tabCounts.image > 0 && (
                    <button className={`tab ${activeTab === "image" ? "active" : ""}`} onClick={() => setActiveTab("image")}>
                      <Image size={12} /> 图片 <span className="tab-badge">{tabCounts.image}</span>
                    </button>
                  )}
                  {tabCounts.config > 0 && (
                    <button className={`tab ${activeTab === "config" ? "active" : ""}`} onClick={() => setActiveTab("config")}>
                      <Settings size={12} /> 配置 <span className="tab-badge">{tabCounts.config}</span>
                    </button>
                  )}
                </div>
                <div className="results-info">
                  <span className="results-count">{filteredResults.length} 项</span>
                  {selectedResults.size > 0 && <span className="results-selected">已选 {selectedResults.size}</span>}
                </div>
              </div>

              <div className="results-actions-bar">
                <div className="results-actions-left">
                  <button className="action-btn" onClick={handleExpandAll} title="展开"><Maximize2 size={13} /></button>
                  <button className="action-btn" onClick={handleCollapseAll} title="折叠"><Minimize2 size={13} /></button>
                  <div className="action-divider" />
                  <button className="action-btn" onClick={handleSelectAll}>全选</button>
                  <button className="action-btn" onClick={handleDeselectAll}>取消</button>
                </div>
                <button className="action-btn action-btn--danger" onClick={handleDeleteSelected} disabled={selectedResults.size === 0}>
                  <Trash2 size={13} /> 移到废纸篓
                </button>
              </div>

              <div className="results-tree">
                {buildTree ? renderTreeNode(buildTree) : (
                  <div className="empty-state">
                    <Shield size={40} className="empty-icon" />
                    <div className="empty-title">未找到匹配文件</div>
                    <div className="empty-subtitle">尝试调整关键字或文件类型筛选</div>
                  </div>
                )}
              </div>
            </>
          )}

          {!scanProgress && (
            <div className="empty-state">
              <div className="empty-hero">
                <Shield size={48} />
              </div>
              <div className="empty-title">文件搜索与定位</div>
              <div className="empty-subtitle">选择目录，输入关键字，快速定位文件</div>
              <div className="empty-hints">
                <div className="hint">
                  <FileText size={16} />
                  <span>支持文件名、内容、EXIF 搜索</span>
                </div>
                <div className="hint">
                  <FolderOpen size={16} />
                  <span>树形结构展示，清晰定位</span>
                </div>
                <div className="hint">
                  <Trash2 size={16} />
                  <span>一键移到废纸篓，安全删除</span>
                </div>
              </div>
            </div>
          )}
        </div>
      </div>

      {/* Text Preview Modal */}
      {previewFile && (
        <div className="modal-overlay" onClick={() => setPreviewFile(null)}>
          <div className="modal" onClick={(e) => e.stopPropagation()}>
            <div className="modal-header">
              <div className="modal-title">
                <Eye size={15} />
                <span>{previewFile.file_name}</span>
              </div>
              <button className="modal-close" onClick={() => setPreviewFile(null)}>
                <X size={14} />
              </button>
            </div>
            <div className="modal-body">
              {previewFile.context_lines.length > 0 ? (
                <div className="preview-content">
                  {previewFile.context_lines.map((line, index) => (
                    <div key={index} className={`preview-line ${line.is_match ? "preview-line--match" : ""}`}>
                      <span className="line-num">{line.line_number}</span>
                      <span className="line-content">{line.content}</span>
                    </div>
                  ))}
                </div>
              ) : (
                <div className="empty-state" style={{ padding: "40px" }}>
                  <div className="empty-subtitle">此文件类型不支持预览</div>
                </div>
              )}
            </div>
          </div>
        </div>
      )}

      {/* Image Preview Modal */}
      {previewImage && (
        <div className="modal-overlay" onClick={() => setPreviewImage(null)}>
          <div className="modal modal--image" onClick={(e) => e.stopPropagation()}>
            <div className="modal-header">
              <div className="modal-title">
                <Image size={15} />
                <span>图片预览</span>
              </div>
              <button className="modal-close" onClick={() => setPreviewImage(null)}>
                <X size={14} />
              </button>
            </div>
            <div className="modal-body modal-body--image">
              <img src={previewImage} alt="Preview" className="preview-image" />
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

export default App;
