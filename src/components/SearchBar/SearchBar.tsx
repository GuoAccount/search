import { useStore } from "../../store";
import { Search, Scan, Loader2 } from "lucide-react";
import styles from "./SearchBar.module.css";

export function SearchBar() {
  const {
    settings,
    updateSettings,
    isScanning,
    scanProgress,
    startScan,
    cancelScan,
    setShowConfirmPanel,
  } = useStore();

  return (
    <div className={styles.container}>
      <div className={styles.row}>
        <div className={styles.inputWrapper}>
          <Search size={14} className={styles.icon} />
          <input
            type="text"
            className={styles.input}
            placeholder="输入关键字搜索文件..."
            value={settings.keyword}
            onChange={(e) => updateSettings({ keyword: e.target.value })}
            onKeyDown={(e) => e.key === "Enter" && !isScanning && startScan()}
          />
        </div>
        {isScanning ? (
          <>
            <button className={styles.btn} onClick={cancelScan}>
              <Loader2 size={14} className={styles.spin} />
              <span>停止</span>
            </button>
          </>
        ) : (
          <button
            className={styles.btn}
            onClick={startScan}
            disabled={
              !settings.scanPath || !settings.keyword || settings.enabledPresets.length === 0
            }
          >
            <Scan size={14} />
            <span>搜索</span>
          </button>
        )}
        {scanProgress && scanProgress.pending_confirmations.length > 0 && (
          <button
            className={`${styles.btn} ${styles.confirm}`}
            onClick={() => setShowConfirmPanel(true)}
          >
            <span>待确认({scanProgress.pending_confirmations.length})</span>
          </button>
        )}
      </div>
    </div>
  );
}
