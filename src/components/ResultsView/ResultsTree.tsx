import { useEffect, useRef, useMemo, useCallback, ReactNode } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import { useStore } from "../../store";
import { buildTree, getFileDir } from "../../utils/file";
import { getFileCategory } from "../../utils/file";
import {
  FolderOpen,
  Folder,
  FileText,
  FileCode2,
  Image,
  Settings,
  File,
  Eye,
  Search,
} from "lucide-react";
import styles from "./ResultsTree.module.css";

const ITEM_HEIGHT = 36;

function getParentDirs(filePath: string): string[] {
  const dirs: string[] = [];
  const parts = filePath.split("/").filter(Boolean);
  for (let i = 1; i < parts.length; i++) {
    dirs.push("/" + parts.slice(0, i).join("/"));
  }
  return dirs;
}

function highlightText(text: string, keyword: string): ReactNode {
  if (!keyword || !text) return text;
  const lower = text.toLowerCase();
  const kwLower = keyword.toLowerCase();
  const parts: { text: string; isMatch: boolean }[] = [];
  let last = 0;
  let idx = lower.indexOf(kwLower);
  while (idx !== -1) {
    if (idx > last) parts.push({ text: text.slice(last, idx), isMatch: false });
    parts.push({ text: text.slice(idx, idx + keyword.length), isMatch: true });
    last = idx + keyword.length;
    idx = lower.indexOf(kwLower, last);
  }
  if (last < text.length) parts.push({ text: text.slice(last), isMatch: false });
  if (parts.length === 1 && !parts[0].isMatch) return text;
  return parts.map((p, i) =>
    p.isMatch ? <span key={i} className={styles.keyword}>{p.text}</span> : <span key={i}>{p.text}</span>
  );
}

interface FlatNode {
  node: any;
  depth: number;
  isFlat: boolean;
}

function flattenTree(nodes: any[], depth: number, expandedFolders: Set<string>): FlatNode[] {
  const result: FlatNode[] = [];
  for (const node of nodes) {
    result.push({ node, depth, isFlat: false });
    if (node.isDir && expandedFolders.has(node.path) && node.children) {
      result.push(...flattenTree(node.children, depth + 1, expandedFolders));
    }
  }
  return result;
}

export function ResultsTree() {
  const {
    settings,
    scanProgress,
    activeTab,
    expandedFolders,
    setExpandedFolders,
    selectedResults,
    setSelectedResults,
    setPreviewFile,
    setPreviewImage,
    appConfig,
  } = useStore();

  const parentRef = useRef<HTMLDivElement>(null);
  const lastAutoExpandLen = useRef(0);

  useEffect(() => {
    if (!scanProgress || activeTab !== "all") return;
    if (scanProgress.results.length <= lastAutoExpandLen.current) return;
    lastAutoExpandLen.current = scanProgress.results.length;

    const expandCount = appConfig?.display?.default_expand_count ?? 1;
    if (expandCount === 0) return;

    const toExpand = new Set(expandedFolders);
    const seen = new Set<string>();
    let added = 0;

    for (const result of scanProgress.results) {
      if (added >= expandCount) break;
      if (seen.has(result.file_path)) continue;
      seen.add(result.file_path);
      const dirs = getParentDirs(result.file_path);
      for (const dir of dirs) {
        toExpand.add(dir);
      }
      added++;
    }

    setExpandedFolders(toExpand);
  }, [scanProgress?.results.length, activeTab, appConfig?.display?.default_expand_count]);

  const isFlatMode = activeTab !== "all";

  const flatItems = useMemo<FlatNode[]>(() => {
    if (!scanProgress) return [];

    const filteredResults = scanProgress.results.filter((result) => {
      if (isFlatMode) return getFileCategory(result.file_extension) === activeTab;
      return true;
    });

    if (isFlatMode) {
      return filteredResults.map((r) => ({ node: r, depth: 0, isFlat: true }));
    }

    const tree = buildTree(filteredResults);
    return flattenTree(tree.children, 0, expandedFolders);
  }, [scanProgress, isFlatMode, activeTab, expandedFolders]);

  const virtualizer = useVirtualizer({
    count: flatItems.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => ITEM_HEIGHT,
    overscan: 10,
  });

  const handleToggleFolder = useCallback((path: string) => {
    setExpandedFolders((prev) => {
      const next = new Set(prev);
      next.has(path) ? next.delete(path) : next.add(path);
      return next;
    });
  }, [setExpandedFolders]);

  const handleToggleSelect = useCallback((filePath: string) => {
    setSelectedResults((prev) => {
      const next = new Set(prev);
      next.has(filePath) ? next.delete(filePath) : next.add(filePath);
      return next;
    });
  }, [setSelectedResults]);

  const handlePreviewFile = useCallback(async (result: any) => {
    if (result.is_dir) {
      handleToggleFolder(result.file_path);
      return;
    }
    const imageExts = ["jpg", "jpeg", "png", "gif", "webp", "bmp", "tiff", "tif", "heic", "heif", "svg", "ico"];
    if (imageExts.includes(result.file_extension.toLowerCase())) {
      try {
        const { invoke } = await import("@tauri-apps/api/core");
        const base64 = await invoke<string>("read_image_as_base64", {
          filePath: result.file_path,
        });
        const bboxes = result.match_bboxes ? JSON.parse(result.match_bboxes) : [];
        setPreviewImage({ base64, bboxes });
      } catch (err) {
        console.error("Failed to load image:", err);
      }
      return;
    }
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      const preview = await invoke<any>("read_file_preview", {
        filePath: result.file_path,
        matchLine: result.match_line,
        matchType: result.match_type,
        contextLines: 5,
        keyword: settings.keyword,
        contextLength: appConfig?.display?.match_context_length || 100,
      });
      setPreviewFile(preview);
    } catch (err) {
      console.error("Failed to read file:", err);
    }
  }, [handleToggleFolder, settings.keyword, appConfig, setPreviewFile, setPreviewImage]);

  const handleRevealInFinder = useCallback(async (filePath: string) => {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke("reveal_in_finder", { filePath });
    } catch (err) {
      console.error("Failed to reveal in finder:", err);
    }
  }, []);

  const getFileIcon = useCallback((extension: string, isDir: boolean) => {
    if (isDir) {
      return expandedFolders.has(extension) ? (
        <FolderOpen size={14} />
      ) : (
        <Folder size={14} />
      );
    }
    const category = getFileCategory(extension);
    switch (category) {
      case "document":
        return <FileText size={14} />;
      case "code":
        return <FileCode2 size={14} />;
      case "image":
        return <Image size={14} />;
      case "config":
        return <Settings size={14} />;
      default:
        return <File size={14} />;
    }
  }, [expandedFolders]);

  const renderItem = useCallback((flatNode: FlatNode) => {
    const { node, depth, isFlat } = flatNode;

    if (isFlat) {
      const result = node;
      const isSelected = selectedResults.has(result.file_path);
      return (
        <div
          className={`${styles.item} ${isSelected ? styles.selected : ""}`}
          style={{ paddingLeft: "8px" }}
          onClick={() => handleToggleSelect(result.file_path)}
        >
          <div className={styles.icon}>
            {getFileIcon(result.file_extension, false)}
          </div>
          <div className={styles.fileInfo}>
            <span className={styles.name}>{result.file_name}</span>
            <div className={styles.pathRow}>
              <span className={styles.path}>{getFileDir(result.file_path)}</span>
              <button
                className={styles.revealBtn}
                onClick={(e) => {
                  e.stopPropagation();
                  handleRevealInFinder(result.file_path);
                }}
                title="在 Finder 中显示"
              >
                <Search size={10} />
              </button>
            </div>
          </div>
          <span className={styles.matchType}>
            {result.match_type === "filename"
              ? "文件名"
              : result.match_type === "content"
              ? "内容"
              : result.match_type === "exif"
              ? "EXIF"
              : "OCR"}
          </span>
          {result.match_context && (
            <span className={styles.context}>
              {highlightText(result.match_context.substring(0, 80), settings.keyword)}
            </span>
          )}
          {result.match_type !== "filename" && (
            <button
              className={styles.preview}
              onClick={(e) => {
                e.stopPropagation();
                handlePreviewFile(result);
              }}
            >
              <Eye size={12} />
            </button>
          )}
        </div>
      );
    }

    // Tree node
    const isSelected = node.result
      ? selectedResults.has(node.result.file_path)
      : false;

    return (
      <div
        className={`${styles.item} ${isSelected ? styles.selected : ""}`}
        style={{ paddingLeft: `${depth * 16 + 8}px` }}
        onClick={() => {
          if (node.isDir) {
            handleToggleFolder(node.path);
          } else if (node.result) {
            handleToggleSelect(node.result.file_path);
          }
        }}
      >
        <div className={styles.icon}>
          {getFileIcon(node.isDir ? node.path : node.name, node.isDir)}
        </div>
        <div className={styles.fileInfo}>
          <span className={styles.name}>{node.name}</span>
          {node.result && (
            <div className={styles.pathRow}>
              <span className={styles.path}>{getFileDir(node.result.file_path)}</span>
              <button
                className={styles.revealBtn}
                onClick={(e) => {
                  e.stopPropagation();
                  handleRevealInFinder(node.result.file_path);
                }}
                title="在 Finder 中显示"
              >
                <Search size={10} />
              </button>
            </div>
          )}
        </div>
        {node.result && (
          <>
            <span className={styles.matchType}>
              {node.result.match_type === "filename"
                ? "文件名"
                : node.result.match_type === "content"
                ? "内容"
                : node.result.match_type === "exif"
                ? "EXIF"
                : "OCR"}
            </span>
            {node.result.match_context && (
              <span className={styles.context}>
                {highlightText(node.result.match_context.substring(0, 80), settings.keyword)}
              </span>
            )}
          </>
        )}
        {node.result && node.result.match_type !== "filename" && (
          <button
            className={styles.preview}
            onClick={(e) => {
              e.stopPropagation();
              handlePreviewFile(node.result);
            }}
          >
            <Eye size={12} />
          </button>
        )}
      </div>
    );
  }, [selectedResults, handleToggleSelect, handleToggleFolder, handlePreviewFile, handleRevealInFinder, getFileIcon, settings.keyword]);

  if (!scanProgress) return null;

  if (flatItems.length === 0) {
    return (
      <div className={styles.empty}>
        <div className={styles.emptyTitle}>未找到匹配文件</div>
        <div className={styles.emptySubtitle}>
          尝试调整关键字或文件类型筛选
        </div>
      </div>
    );
  }

  return (
    <div ref={parentRef} className={styles.container}>
      <div
        className={styles.virtualInner}
        style={{ height: `${virtualizer.getTotalSize()}px` }}
      >
        {virtualizer.getVirtualItems().map((virtualItem) => {
          const flatNode = flatItems[virtualItem.index];
          return (
            <div
              key={virtualItem.key}
              data-index={virtualItem.index}
              ref={virtualizer.measureElement}
              className={styles.virtualItem}
              style={{
                transform: `translateY(${virtualItem.start}px)`,
              }}
            >
              {renderItem(flatNode)}
            </div>
          );
        })}
      </div>
    </div>
  );
}
