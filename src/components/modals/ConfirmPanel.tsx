import { useState, useRef, useEffect } from "react";
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

  const [rememberMap, setRememberMap] = useState<Record<string, boolean>>({});

  const toggleRemember = (id: string) => {
    setRememberMap((prev) => ({ ...prev, [id]: !prev[id] }));
  };

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
              {scanProgress.pending_confirmations.map((confirmation) => {
                const remembered = rememberMap[confirmation.id] || false;
                return (
                  <div key={confirmation.id} className={styles.item}>
                    <div className={styles.itemInfo}>
                      <ScrollingPath path={confirmation.path} />
                      <div className={styles.itemCount}>
                        包含 {confirmation.entry_count.toLocaleString()} 个子项
                      </div>
                    </div>
                    <div className={styles.itemActions}>
                      <div
                        className={styles.rememberToggle}
                        onClick={() => toggleRemember(confirmation.id)}
                      >
                        <div
                          className={styles.toggleTrack}
                          data-on={remembered}
                        >
                          <span className={styles.toggleThumb} />
                        </div>
                        <span className={styles.toggleLabel}>
                          {remembered ? "记住" : "仅本次"}
                        </span>
                      </div>
                      <div className={styles.divider} />
                      <button
                        className={styles.allow}
                        onClick={() =>
                          respondConfirmation(confirmation.id, true, remembered)
                        }
                      >
                        允许
                      </button>
                      <button
                        className={styles.deny}
                        onClick={() =>
                          respondConfirmation(confirmation.id, false, remembered)
                        }
                      >
                        拒绝
                      </button>
                    </div>
                  </div>
                );
              })}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

function ScrollingPath({ path }: { path: string }) {
  const containerRef = useRef<HTMLDivElement>(null);
  const textRef = useRef<HTMLSpanElement>(null);
  const [offset, setOffset] = useState(0);
  const [isOverflowing, setIsOverflowing] = useState(false);
  const animRef = useRef<number | null>(null);
  const stateRef = useRef({ offset: 0, direction: 1, paused: false });

  useEffect(() => {
    const container = containerRef.current;
    const text = textRef.current;
    if (!container || !text) return;

    const check = () => {
      setIsOverflowing(text.scrollWidth > container.clientWidth);
    };
    check();

    const ro = new ResizeObserver(check);
    ro.observe(container);
    ro.observe(text);
    return () => ro.disconnect();
  }, [path]);

  useEffect(() => {
    if (!isOverflowing) {
      setOffset(0);
      return;
    }

    const container = containerRef.current;
    const text = textRef.current;
    if (!container || !text) return;

    const speed = 30;
    const pauseAtEnd = 1200;
    let lastTime: number | null = null;
    let pauseUntil = 0;

    const st = stateRef.current;
    st.offset = 0;
    st.direction = 1;
    st.paused = false;

    const step = (timestamp: number) => {
      if (st.paused) return;

      if (!lastTime) lastTime = timestamp;
      const delta = (timestamp - lastTime) / 1000;
      lastTime = timestamp;

      if (timestamp < pauseUntil) {
        animRef.current = requestAnimationFrame(step);
        return;
      }

      const containerW = container.clientWidth;
      const textW = text.scrollWidth;
      const iconSpace = 22; // icon(14) + gap(8)
      const maxOffset = textW - containerW + iconSpace;

      st.offset += st.direction * speed * delta;

      if (st.offset >= maxOffset) {
        st.offset = maxOffset;
        st.direction = -1;
        pauseUntil = timestamp + pauseAtEnd;
      } else if (st.offset <= 0) {
        st.offset = 0;
        st.direction = 1;
        pauseUntil = timestamp + pauseAtEnd;
      }

      setOffset(st.offset);
      animRef.current = requestAnimationFrame(step);
    };

    const timeout = setTimeout(() => {
      animRef.current = requestAnimationFrame(step);
    }, 800);

    const handleEnter = () => {
      st.paused = true;
      if (animRef.current) cancelAnimationFrame(animRef.current);
    };

    const handleLeave = () => {
      st.paused = false;
      lastTime = null;
      animRef.current = requestAnimationFrame(step);
    };

    container.addEventListener("mouseenter", handleEnter);
    container.addEventListener("mouseleave", handleLeave);

    return () => {
      clearTimeout(timeout);
      if (animRef.current) cancelAnimationFrame(animRef.current);
      container.removeEventListener("mouseenter", handleEnter);
      container.removeEventListener("mouseleave", handleLeave);
    };
  }, [isOverflowing, path]);

  return (
    <div className={styles.itemPath} ref={containerRef} title={path}>
      <FolderOpen size={14} />
      <span
        ref={textRef}
        style={{ transform: `translateX(-${offset}px)` }}
      >
        {path}
      </span>
    </div>
  );
}
