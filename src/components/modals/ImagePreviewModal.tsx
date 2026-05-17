import { useEffect, useRef, useState } from "react";
import { useStore } from "../../store";
import { Image, X } from "lucide-react";
import styles from "./ImagePreviewModal.module.css";

export function ImagePreviewModal() {
  const { previewImage, setPreviewImage } = useStore();
  const imgRef = useRef<HTMLImageElement>(null);
  const [naturalSize, setNaturalSize] = useState({ w: 1, h: 1 });

  useEffect(() => {
    if (!previewImage) return;
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") setPreviewImage(null);
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [previewImage, setPreviewImage]);

  if (!previewImage) return null;

  const { base64, bboxes } = previewImage;

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
          <div className={styles.imageWrapper}>
            <img
              ref={imgRef}
              src={base64}
              alt="Preview"
              className={styles.image}
              onLoad={(e) => {
                const img = e.target as HTMLImageElement;
                setNaturalSize({ w: img.naturalWidth, h: img.naturalHeight });
              }}
            />
            {bboxes.length > 0 && (
              <svg
                className={styles.bboxOverlay}
                viewBox={`0 0 ${naturalSize.w} ${naturalSize.h}`}
                preserveAspectRatio="xMidYMid meet"
              >
                {bboxes.map((bbox, i) => (
                  <rect
                    key={i}
                    x={bbox.x * naturalSize.w}
                    y={(1 - bbox.y - bbox.h) * naturalSize.h}
                    width={bbox.w * naturalSize.w}
                    height={bbox.h * naturalSize.h}
                    fill="none"
                    stroke="#ff3b30"
                    strokeWidth={3}
                    rx={4}
                  />
                ))}
              </svg>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
