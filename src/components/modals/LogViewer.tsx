import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useStore } from "../../store";
import { FileText, X, Trash2, Download, RefreshCw } from "lucide-react";
import styles from "./LogViewer.module.css";

interface LogEntry {
  timestamp: string;
  level: "INFO" | "WARN" | "ERROR" | "DEBUG";
  message: string;
}

const LOG_RE = /^\[(\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2})\]\s+(\w+)\s+(.*)/;

function parseLogs(content: string): LogEntry[] {
  if (!content) return [];
  return content
    .split("\n")
    .filter((l) => l.trim())
    .map((line) => {
      const m = line.match(LOG_RE);
      return m
        ? { timestamp: m[1], level: m[2] as LogEntry["level"], message: m[3] }
        : { timestamp: "", level: "INFO" as const, message: line };
    });
}

export function LogViewer() {
  const { showLogViewer, setShowLogViewer } = useStore();
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [filter, setFilter] = useState("all");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const endRef = useRef<HTMLDivElement>(null);

  const loadLogs = async () => {
    setLoading(true);
    setError(null);
    try {
      const content = await invoke<string>("get_log_content");
      setLogs(parseLogs(content));
    } catch (err) {
      setError(String(err));
      setLogs([]);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    if (showLogViewer) loadLogs();
  }, [showLogViewer]);

  useEffect(() => {
    if (showLogViewer) endRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [logs, filter, showLogViewer]);

  const filtered =
    filter === "all" ? logs : logs.filter((l) => l.level.toLowerCase() === filter);

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
            <button className={styles.actionBtn} onClick={loadLogs} title="刷新">
              <RefreshCw size={13} />
            </button>
            <button className={styles.actionBtn} onClick={handleExport} title="导出">
              <Download size={13} />
            </button>
            <button
              className={styles.actionBtn}
              onClick={() => setLogs([])}
              title="清空"
            >
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
          {["all", "warn", "error"].map((level) => (
            <button
              key={level}
              className={`${styles.filterBtn} ${filter === level ? styles.filterActive : ""}`}
              onClick={() => setFilter(level)}
            >
              {level === "all" ? "全部" : level.toUpperCase()}
            </button>
          ))}
          <span className={styles.count}>
            {loading ? "加载中..." : `${filtered.length} 条`}
          </span>
        </div>
        <div className={styles.content}>
          {error ? (
            <div className={styles.empty}>加载失败: {error}</div>
          ) : filtered.length === 0 ? (
            <div className={styles.empty}>
              {loading ? "加载中..." : "暂无日志"}
            </div>
          ) : (
            filtered.map((log, i) => (
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
          <div ref={endRef} />
        </div>
      </div>
    </div>
  );
}
