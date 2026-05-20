import { useState, useEffect, useRef } from "react";
import { useStore } from "../../store";
import { FileText, X, Trash2, Download, RefreshCw } from "lucide-react";
import styles from "./LogViewer.module.css";

interface LogEntry {
  timestamp: string;
  level: "INFO" | "WARN" | "ERROR" | "DEBUG";
  message: string;
}

export function LogViewer() {
  const { showLogViewer, setShowLogViewer } = useStore();
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [filter, setFilter] = useState<string>("all");
  const logsEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (showLogViewer) {
      loadLogs();
    }
  }, [showLogViewer]);

  const loadLogs = async () => {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      const logContent = await invoke<string>("get_log_content");
      const parsed = parseLogs(logContent);
      setLogs(parsed);
    } catch (err) {
      console.error("Failed to load logs:", err);
    }
  };

  const parseLogs = (content: string): LogEntry[] => {
    if (!content) return [];
    const lines = content.split("\n").filter((l) => l.trim());
    return lines.map((line) => {
      const match = line.match(
        /\[(\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2})\]\s+(\w+)\s+(.*)/
      );
      if (match) {
        return {
          timestamp: match[1],
          level: match[2] as LogEntry["level"],
          message: match[3],
        };
      }
      return { timestamp: "", level: "INFO", message: line };
    });
  };

  const filteredLogs =
    filter === "all"
      ? logs
      : logs.filter((l) => l.level.toLowerCase() === filter);

  const handleClear = () => setLogs([]);

  const handleExport = () => {
    const text = logs
      .map((l) => `[${l.timestamp}] ${l.level} ${l.message}`)
      .join("\n");
    const blob = new Blob([text], { type: "text/plain" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `lumina-logs-${new Date().toISOString().slice(0, 10)}.txt`;
    a.click();
    URL.revokeObjectURL(url);
  };

  if (!showLogViewer) return null;

  return (
    <div className={styles.overlay} onClick={() => setShowLogViewer(false)}>
      <div className={styles.panel} onClick={(e) => e.stopPropagation()}>
        <div className={styles.header}>
          <div className={styles.title}>
            <FileText size={16} />
            <span>日志</span>
          </div>
          <div className={styles.headerActions}>
            <button className={styles.actionBtn} onClick={loadLogs}>
              <RefreshCw size={13} />
            </button>
            <button className={styles.actionBtn} onClick={handleExport}>
              <Download size={13} />
            </button>
            <button className={styles.actionBtn} onClick={handleClear}>
              <Trash2 size={13} />
            </button>
            <button
              className={styles.closeBtn}
              onClick={() => setShowLogViewer(false)}
            >
              <X size={14} />
            </button>
          </div>
        </div>
        <div className={styles.toolbar}>
          {["all", "info", "warn", "error", "debug"].map((level) => (
            <button
              key={level}
              className={`${styles.filterBtn} ${filter === level ? styles.filterActive : ""}`}
              onClick={() => setFilter(level)}
            >
              {level === "all" ? "全部" : level.toUpperCase()}
            </button>
          ))}
          <span className={styles.count}>{filteredLogs.length} 条</span>
        </div>
        <div className={styles.content}>
          {filteredLogs.length === 0 ? (
            <div className={styles.empty}>暂无日志</div>
          ) : (
            filteredLogs.map((log, i) => (
              <div
                key={i}
                className={`${styles.logEntry} ${styles[`level${log.level}`]}`}
              >
                <span className={styles.timestamp}>{log.timestamp}</span>
                <span className={styles.level}>{log.level}</span>
                <span className={styles.message}>{log.message}</span>
              </div>
            ))
          )}
          <div ref={logsEndRef} />
        </div>
      </div>
    </div>
  );
}
