import { Shield, FileText, FolderOpen, Trash2 } from "lucide-react";
import styles from "./EmptyState.module.css";

export function EmptyState() {
  return (
    <div className={styles.container}>
      <div className={styles.hero}>
        <Shield size={48} />
      </div>
      <div className={styles.title}>文件搜索与定位</div>
      <div className={styles.subtitle}>选择目录，输入关键字，快速定位文件</div>
      <div className={styles.hints}>
        <div className={styles.hint}>
          <FileText size={16} />
          <span>支持文件名、内容、EXIF 搜索</span>
        </div>
        <div className={styles.hint}>
          <FolderOpen size={16} />
          <span>树形结构展示，清晰定位</span>
        </div>
        <div className={styles.hint}>
          <Trash2 size={16} />
          <span>一键移到废纸篓，安全删除</span>
        </div>
      </div>
    </div>
  );
}
