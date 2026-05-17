import { useStore } from "../../store";
import { DEFAULT_PRESETS } from "../../constants/presets";
import { Check, ChevronDown, ChevronRight, Plus, X, ScanEye } from "lucide-react";
import styles from "./PresetSelector.module.css";

export function PresetSelector() {
  const {
    settings,
    updateSettings,
    expandedPresets,
    setExpandedPresets,
  } = useStore();

  const enabledPresets = new Set(settings.enabledPresets);
  const customExtensions = settings.customExtensions;

  const handleTogglePreset = (preset: string) => {
    const next = new Set(enabledPresets);
    next.has(preset) ? next.delete(preset) : next.add(preset);
    updateSettings({ enabledPresets: Array.from(next) });
  };

  const handleAddExtension = (preset: string, ext: string) => {
    const cleanExt = ext.trim().toLowerCase().replace(/^\./, "");
    if (!cleanExt) return;
    const current = customExtensions[preset] || [];
    if (current.includes(cleanExt)) return;
    updateSettings({
      customExtensions: {
        ...customExtensions,
        [preset]: [...current, cleanExt],
      },
    });
  };

  const handleRemoveExtension = (preset: string, ext: string) => {
    updateSettings({
      customExtensions: {
        ...customExtensions,
        [preset]: (customExtensions[preset] || []).filter((e) => e !== ext),
      },
    });
  };

  return (
    <div className={styles.section}>
      <div className={styles.title}>
        <span>文件类型</span>
        <span className={styles.badge}>{enabledPresets.size}</span>
      </div>
      {Object.entries(DEFAULT_PRESETS).map(([key, preset]) => {
        const Icon = preset.icon;
        const isEnabled = enabledPresets.has(key);
        const isExpanded = expandedPresets.has(key);
        const custom = customExtensions[key] || [];
        return (
          <div
            key={key}
            className={`${styles.item} ${isEnabled ? styles.active : ""}`}
          >
            <div
              className={styles.header}
              onClick={() => handleTogglePreset(key)}
            >
              <div className={styles.checkbox}>
                {isEnabled && <Check size={10} />}
              </div>
              <Icon size={15} className={styles.icon} />
              <span className={styles.label}>{preset.label}</span>
              <span className={styles.count}>
                {preset.extensions.length + custom.length}
              </span>
              {isEnabled && (
                <button
                  className={styles.expandBtn}
                  onClick={(e) => {
                    e.stopPropagation();
                    setExpandedPresets((prev) => {
                      const next = new Set(prev);
                      next.has(key) ? next.delete(key) : next.add(key);
                      return next;
                    });
                  }}
                >
                  {isExpanded ? (
                    <ChevronDown size={14} />
                  ) : (
                    <ChevronRight size={14} />
                  )}
                </button>
              )}
            </div>
            {isEnabled && isExpanded && (
              <div className={styles.extensions}>
                <div className={styles.tags}>
                  {preset.extensions.map((ext) => (
                    <span key={ext} className={styles.tag}>
                      {ext}
                    </span>
                  ))}
                  {custom.map((ext) => (
                    <span key={ext} className={`${styles.tag} ${styles.custom}`}>
                      {ext}
                      <button
                        className={styles.remove}
                        onClick={(e) => {
                          e.stopPropagation();
                          handleRemoveExtension(key, ext);
                        }}
                      >
                        <X size={8} />
                      </button>
                    </span>
                  ))}
                </div>
                <div className={styles.add}>
                  <input
                    type="text"
                    className={styles.input}
                    placeholder="添加扩展名"
                    onKeyDown={(e) => {
                      if (e.key === "Enter") {
                        handleAddExtension(key, e.currentTarget.value);
                        e.currentTarget.value = "";
                      }
                    }}
                  />
                  <button
                    className={styles.addBtn}
                    onClick={(e) => {
                      const input = e.currentTarget
                        .previousElementSibling as HTMLInputElement;
                      handleAddExtension(key, input.value);
                      input.value = "";
                    }}
                  >
                    <Plus size={12} />
                  </button>
                </div>
                {key === "image" && (
                  <div className={styles.ocr}>
                    <label className={styles.ocrToggle}>
                      <input
                        type="checkbox"
                        checked={settings.ocrEnabled}
                        onChange={(e) => {
                          if (e.target.checked) {
                            const p = navigator.platform.toLowerCase();
                            if (p.includes("linux")) {
                              alert("Linux 平台暂不支持 OCR 功能");
                              return;
                            }
                          }
                          updateSettings({ ocrEnabled: e.target.checked });
                        }}
                      />
                      <span className={styles.ocrSlider}></span>
                    </label>
                    <div className={styles.ocrInfo}>
                      <ScanEye size={13} />
                      <span className={styles.ocrLabel}>OCR 文字识别</span>
                    </div>
                  </div>
                )}
              </div>
            )}
          </div>
        );
      })}
    </div>
  );
}
