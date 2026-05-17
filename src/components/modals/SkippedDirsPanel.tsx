import { useStore } from "../../store";
import { FolderOpen, X } from "lucide-react";
import styles from "./SkippedDirsPanel.module.css";

export function SkippedDirsPanel() {
  const { scanProgress, showSkippedPanel, setShowSkippedPanel } = useStore();

  if (!showSkippedPanel || !scanProgress) return null;

  return (
    <div className={styles.overlay} onClick={() => setShowSkippedPanel(false)}>
      <div className={styles.modal} onClick={(e) => e.stopPropagation()}>
        <div className={styles.header}>
          <div className={styles.title}>
            <FolderOpen size={15} />
            <span>已跳过的目录</span>
          </div>
          <button
            className={styles.close}
            onClick={() => setShowSkippedPanel(false)}
          >
            <X size={14} />
          </button>
        </div>
        <div className={styles.body}>
          {scanProgress.skipped_dirs.length === 0 ? (
            <div className={styles.empty}>没有跳过的目录</div>
          ) : (
            <div className={styles.list}>
              {scanProgress.skipped_dirs.map((skipped, index) => (
                <div key={index} className={styles.item}>
                  <div className={styles.itemPath}>
                    <FolderOpen size={14} />
                    <span>{skipped.path}</span>
                  </div>
                  <div className={styles.itemReason}>
                    {skipped.reason === "rule" && "匹配跳过规则"}
                    {skipped.reason === "large_dir" && "子项过多"}
                    {skipped.reason === "user_skip" && "用户跳过"}
                    {skipped.reason === "user_skip_remembered" && "用户跳过（已记住）"}
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
