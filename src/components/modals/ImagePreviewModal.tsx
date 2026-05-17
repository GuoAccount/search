import { useStore } from "../../store";
import { Image, X } from "lucide-react";
import styles from "./ImagePreviewModal.module.css";

export function ImagePreviewModal() {
  const { previewImage, setPreviewImage } = useStore();

  if (!previewImage) return null;

  return (
    <div className={styles.overlay} onClick={() => setPreviewImage(null)}>
      <div className={styles.modal} onClick={(e) => e.stopPropagation()}>
        <div className={styles.header}>
          <div className={styles.title}>
            <Image size={15} />
            <span>图片预览</span>
          </div>
          <button
            className={styles.close}
            onClick={() => setPreviewImage(null)}
          >
            <X size={14} />
          </button>
        </div>
        <div className={styles.body}>
          <img src={previewImage} alt="Preview" className={styles.image} />
        </div>
      </div>
    </div>
  );
}
