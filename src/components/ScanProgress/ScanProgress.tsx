import { useStore } from "../../store";
import { FolderOpen } from "lucide-react";
import styles from "./ScanProgress.module.css";

export function ScanProgress() {
  const { isScanning, scanProgress, setShowSkippedPanel } = useStore();

  if (!isScanning) return null;

  return (
    <div className={styles.container}>
      <div className={styles.bar}>
        <div className={styles.track} />
      </div>
      <div className={styles.info}>
        <div className={styles.stats}>
          <span className={styles.count}>
            已扫描 {scanProgress?.files_scanned || 0} 个文件
          </span>
          <span className={styles.found}>
            找到 {scanProgress?.results_found || 0} 个结果
          </span>
          {scanProgress && scanProgress.skipped_dirs.length > 0 && (
            <span
              className={styles.skipped}
              onClick={() => setShowSkippedPanel(true)}
            >
              跳过 {scanProgress.skipped_dirs.length} 个大目录
            </span>
          )}
        </div>
        {scanProgress?.current_path && (
          <div className={styles.path} title={scanProgress.current_path}>
            <FolderOpen size={12} />
            <span>{scanProgress.current_path}</span>
          </div>
        )}
      </div>
    </div>
  );
}
