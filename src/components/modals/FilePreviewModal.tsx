import { useEffect } from "react";
import { useStore } from "../../store";
import { Eye, X, Loader2 } from "lucide-react";
import styles from "./FilePreviewModal.module.css";

function highlightKeyword(text: string, keyword: string) {
  if (!keyword) return text;
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
  return parts;
}

export function FilePreviewModal() {
  const { settings, previewFile, setPreviewFile } = useStore();
  const keyword = settings.keyword;

  useEffect(() => {
    if (!previewFile) return;
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") setPreviewFile(null);
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [previewFile, setPreviewFile]);

  if (!previewFile) return null;

  const renderContent = (text: string, isMatchLine: boolean) => {
    if (!isMatchLine || !keyword) return text;
    const parts = highlightKeyword(text, keyword);
    if (typeof parts === "string") return parts;
    return parts.map((p, i) =>
      p.isMatch ? (
        <span key={i} className={styles.keyword}>{p.text}</span>
      ) : (
        <span key={i}>{p.text}</span>
      )
    );
  };

  return (
    <div className={styles.overlay} onClick={() => setPreviewFile(null)}>
      <div className={styles.modal} onClick={(e) => e.stopPropagation()}>
        <div className={styles.header}>
          <div className={styles.title}>
            <Eye size={15} />
            <span>{previewFile.file_name}</span>
          </div>
          <button
            className={styles.close}
            onClick={() => setPreviewFile(null)}
          >
            <X size={14} />
          </button>
        </div>
        <div className={styles.body}>
          {previewFile.loading ? (
            <div className={styles.loading}>
              <Loader2 size={24} className={styles.spinner} />
              <span>加载中...</span>
            </div>
          ) : previewFile.context_lines.length > 0 ? (
            <div className={styles.content}>
              {previewFile.context_lines.map((line, index) => (
                <div
                  key={index}
                  className={`${styles.line} ${
                    line.is_match ? styles.match : ""
                  }`}
                >
                  <span className={styles.lineNum}>{line.line_number}</span>
                  <span className={styles.lineContent}>
                    {renderContent(line.content, line.is_match)}
                  </span>
                </div>
              ))}
            </div>
          ) : (
            <div className={styles.empty}>此文件类型不支持预览</div>
          )}
        </div>
      </div>
    </div>
  );
}
