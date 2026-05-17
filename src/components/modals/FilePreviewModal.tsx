import { useStore } from "../../store";
import { Eye, X } from "lucide-react";
import styles from "./FilePreviewModal.module.css";

export function FilePreviewModal() {
  const { previewFile, setPreviewFile } = useStore();

  if (!previewFile) return null;

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
          {previewFile.context_lines.length > 0 ? (
            <div className={styles.content}>
              {previewFile.context_lines.map((line, index) => (
                <div
                  key={index}
                  className={`${styles.line} ${
                    line.is_match ? styles.match : ""
                  }`}
                >
                  <span className={styles.lineNum}>{line.line_number}</span>
                  <span className={styles.lineContent}>{line.content}</span>
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
