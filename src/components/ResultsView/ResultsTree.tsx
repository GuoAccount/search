import { useStore } from "../../store";
import { buildTree } from "../../utils/file";
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
} from "lucide-react";
import styles from "./ResultsTree.module.css";

export function ResultsTree() {
  const {
    scanProgress,
    activeTab,
    expandedFolders,
    setExpandedFolders,
    selectedResults,
    setSelectedResults,
    setPreviewFile,
    setPreviewImage,
  } = useStore();

  if (!scanProgress) return null;

  const filteredResults = scanProgress.results.filter((result) => {
    if (activeTab === "all") return true;
    return getFileCategory(result.file_extension) === activeTab;
  });

  const tree = buildTree(filteredResults);

  const handleToggleFolder = (path: string) => {
    setExpandedFolders((prev) => {
      const next = new Set(prev);
      next.has(path) ? next.delete(path) : next.add(path);
      return next;
    });
  };

  const handleToggleSelect = (filePath: string) => {
    setSelectedResults((prev) => {
      const next = new Set(prev);
      next.has(filePath) ? next.delete(filePath) : next.add(filePath);
      return next;
    });
  };

  const handlePreviewFile = async (result: any) => {
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
        setPreviewImage(base64);
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
        contextLines: 5,
      });
      setPreviewFile(preview);
    } catch (err) {
      console.error("Failed to read file:", err);
    }
  };

  const getFileIcon = (extension: string, isDir: boolean) => {
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
  };

  const renderNode = (node: any, depth: number = 0) => {
    const isExpanded = expandedFolders.has(node.path);
    const isSelected = node.result
      ? selectedResults.has(node.result.file_path)
      : false;

    return (
      <div key={node.path}>
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
          onDoubleClick={() => node.result && handlePreviewFile(node.result)}
        >
          <div className={styles.icon}>
            {getFileIcon(node.isDir ? node.path : node.name, node.isDir)}
          </div>
          <span className={styles.name}>{node.name}</span>
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
                  {node.result.match_context.substring(0, 50)}
                </span>
              )}
            </>
          )}
          {node.result && (
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
        {node.isDir && isExpanded && (
          <div>
            {node.children.map((child: any) => renderNode(child, depth + 1))}
          </div>
        )}
      </div>
    );
  };

  if (tree.children.length === 0) {
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
    <div className={styles.container}>
      {tree.children.map((child) => renderNode(child))}
    </div>
  );
}
