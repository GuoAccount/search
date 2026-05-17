import { useStore } from "../../store";
import { FolderOpen, X } from "lucide-react";
import styles from "./ConfirmPanel.module.css";

export function ConfirmPanel() {
  const {
    scanProgress,
    showConfirmPanel,
    setShowConfirmPanel,
    respondConfirmation,
    allowAllConfirmations,
  } = useStore();

  if (!showConfirmPanel || !scanProgress) return null;

  return (
    <div className={styles.overlay} onClick={() => setShowConfirmPanel(false)}>
      <div className={styles.modal} onClick={(e) => e.stopPropagation()}>
        <div className={styles.header}>
          <div className={styles.title}>
            <FolderOpen size={15} />
            <span>待确认事项</span>
          </div>
          <div className={styles.actions}>
            <button className={styles.allowAll} onClick={allowAllConfirmations}>
              全部允许
            </button>
            <button
              className={styles.close}
              onClick={() => setShowConfirmPanel(false)}
            >
              <X size={14} />
            </button>
          </div>
        </div>
        <div className={styles.body}>
          {scanProgress.pending_confirmations.length === 0 ? (
            <div className={styles.empty}>没有待确认事项</div>
          ) : (
            <div className={styles.list}>
              {scanProgress.pending_confirmations.map((confirmation) => (
                <div key={confirmation.id} className={styles.item}>
                  <div className={styles.itemInfo}>
                    <div className={styles.itemPath}>
                      <FolderOpen size={14} />
                      <span>{confirmation.path}</span>
                    </div>
                    <div className={styles.itemCount}>
                      包含 {confirmation.entry_count.toLocaleString()} 个子项
                    </div>
                  </div>
                  <div className={styles.itemActions}>
                    <button
                      className={styles.allow}
                      onClick={() =>
                        respondConfirmation(confirmation.id, true, false)
                      }
                    >
                      允许
                    </button>
                    <button
                      className={styles.deny}
                      onClick={() =>
                        respondConfirmation(confirmation.id, false, false)
                      }
                    >
                      拒绝
                    </button>
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
