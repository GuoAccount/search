import { useStore } from "../../store";
import { getTabCounts } from "../../utils/file";
import {
  FileText,
  FileCode2,
  Image,
  Settings,
  Maximize2,
  Minimize2,
  Trash2,
} from "lucide-react";
import styles from "./ResultsToolbar.module.css";

export function ResultsToolbar() {
  const {
    scanProgress,
    activeTab,
    setActiveTab,
    selectedResults,
    setSelectedResults,
    setExpandedFolders,
    setScanProgress,
  } = useStore();

  if (!scanProgress) return null;

  const tabCounts = getTabCounts(scanProgress.results);

  const handleSelectAll = () => {
    setSelectedResults(
      new Set(scanProgress.results.map((r) => r.file_path))
    );
  };

  const handleDeselectAll = () => {
    setSelectedResults(new Set());
  };

  const handleExpandAll = () => {
    const allPaths = new Set(scanProgress.results.map((r) => r.file_path));
    setExpandedFolders(allPaths);
  };

  const handleCollapseAll = () => {
    setExpandedFolders(new Set());
  };

  const handleDeleteSelected = async () => {
    if (selectedResults.size === 0) return;
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke<number>("move_to_trash", {
        filePaths: Array.from(selectedResults),
      });
      const remaining = scanProgress.results.filter(
        (r) => !selectedResults.has(r.file_path)
      );
      setScanProgress({
        ...scanProgress,
        results: remaining,
        results_found: remaining.length,
      });
      setSelectedResults(new Set());
    } catch (err) {
      console.error("Failed to delete files:", err);
    }
  };

  return (
    <div className={styles.container}>
      <div className={styles.tabs}>
        <button
          className={`${styles.tab} ${activeTab === "all" ? styles.active : ""}`}
          onClick={() => setActiveTab("all")}
        >
          全部 <span className={styles.badge}>{tabCounts.all}</span>
        </button>
        {tabCounts.document > 0 && (
          <button
            className={`${styles.tab} ${activeTab === "document" ? styles.active : ""}`}
            onClick={() => setActiveTab("document")}
          >
            <FileText size={12} /> 文档{" "}
            <span className={styles.badge}>{tabCounts.document}</span>
          </button>
        )}
        {tabCounts.code > 0 && (
          <button
            className={`${styles.tab} ${activeTab === "code" ? styles.active : ""}`}
            onClick={() => setActiveTab("code")}
          >
            <FileCode2 size={12} /> 代码{" "}
            <span className={styles.badge}>{tabCounts.code}</span>
          </button>
        )}
        {tabCounts.image > 0 && (
          <button
            className={`${styles.tab} ${activeTab === "image" ? styles.active : ""}`}
            onClick={() => setActiveTab("image")}
          >
            <Image size={12} /> 图片{" "}
            <span className={styles.badge}>{tabCounts.image}</span>
          </button>
        )}
        {tabCounts.config > 0 && (
          <button
            className={`${styles.tab} ${activeTab === "config" ? styles.active : ""}`}
            onClick={() => setActiveTab("config")}
          >
            <Settings size={12} /> 配置{" "}
            <span className={styles.badge}>{tabCounts.config}</span>
          </button>
        )}
      </div>
      <div className={styles.info}>
        <span className={styles.count}>{scanProgress.results.length} 项</span>
        {selectedResults.size > 0 && (
          <span className={styles.selected}>已选 {selectedResults.size}</span>
        )}
      </div>
      <div className={styles.actions}>
        <div className={styles.actionsLeft}>
          <button className={styles.action} onClick={handleExpandAll} title="展开">
            <Maximize2 size={13} />
          </button>
          <button className={styles.action} onClick={handleCollapseAll} title="折叠">
            <Minimize2 size={13} />
          </button>
          <div className={styles.divider} />
          <button className={styles.action} onClick={handleSelectAll}>
            全选
          </button>
          <button className={styles.action} onClick={handleDeselectAll}>
            取消
          </button>
        </div>
        <button
          className={`${styles.action} ${styles.danger}`}
          onClick={handleDeleteSelected}
          disabled={selectedResults.size === 0}
        >
          <Trash2 size={13} /> 移到废纸篓
        </button>
      </div>
    </div>
  );
}
