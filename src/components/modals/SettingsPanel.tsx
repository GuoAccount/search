import { useStore } from "../../store";
import { Settings, X } from "lucide-react";
import styles from "./SettingsPanel.module.css";

export function SettingsPanel() {
  const {
    showSettings,
    setShowSettings,
    localConfig,
    setLocalConfig,
    configDirty,
    setConfigDirty,
    appConfig,
    setAppConfig,
  } = useStore();

  if (!showSettings || !appConfig) return null;

  const handleSave = async () => {
    if (!localConfig) return;
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke("save_config", { cfg: localConfig });
      setAppConfig(localConfig);
      setConfigDirty(false);
    } catch (err) {
      console.error("Failed to save config:", err);
    }
  };

  const handleReset = async () => {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      const defaultConfig = await invoke<any>("reset_config");
      setAppConfig(defaultConfig);
      setLocalConfig(defaultConfig);
      setConfigDirty(false);
    } catch (err) {
      console.error("Failed to reset config:", err);
    }
  };

  return (
    <div className={styles.overlay} onClick={() => setShowSettings(false)}>
      <div className={styles.modal} onClick={(e) => e.stopPropagation()}>
        <div className={styles.header}>
          <div className={styles.title}>
            <Settings size={15} />
            <span>设置</span>
          </div>
          <button
            className={styles.close}
            onClick={() => setShowSettings(false)}
          >
            <X size={14} />
          </button>
        </div>
        <div className={styles.body}>
          <div className={styles.section}>
            <h3 className={styles.sectionTitle}>扫描配置</h3>
            <div className={styles.item}>
              <label>大目录阈值:</label>
              <input
                type="number"
                value={
                  localConfig?.scan.large_dir_threshold ??
                  appConfig.scan.large_dir_threshold
                }
                onChange={(e) => {
                  const val = parseInt(e.target.value) || 1000;
                  setLocalConfig((prev) => ({
                    ...(prev || appConfig),
                    scan: {
                      ...(prev?.scan || appConfig.scan),
                      large_dir_threshold: val,
                    },
                  }));
                  setConfigDirty(true);
                }}
              />
              <span>个子项</span>
            </div>
            <div className={styles.item}>
              <label>遇到大目录时:</label>
              <select
                value={
                  (localConfig?.scan.ask_on_large_dir ??
                  appConfig.scan.ask_on_large_dir)
                    ? "ask"
                    : "skip"
                }
                onChange={(e) => {
                  setLocalConfig((prev) => ({
                    ...(prev || appConfig),
                    scan: {
                      ...(prev?.scan || appConfig.scan),
                      ask_on_large_dir: e.target.value === "ask",
                    },
                  }));
                  setConfigDirty(true);
                }}
              >
                <option value="ask">询问</option>
                <option value="skip">始终跳过</option>
              </select>
            </div>
          </div>

          <div className={styles.section}>
            <h3 className={styles.sectionTitle}>跳过规则</h3>
            <div className={styles.rules}>
              {(localConfig?.skip_rules ?? appConfig.skip_rules).map(
                (rule, index) => (
                  <span key={index} className={styles.rule}>
                    {rule}
                    <button
                      onClick={() => {
                        setLocalConfig((prev) => ({
                          ...(prev || appConfig),
                          skip_rules: (
                            prev?.skip_rules || appConfig.skip_rules
                          ).filter((r) => r !== rule),
                        }));
                        setConfigDirty(true);
                      }}
                    >
                      <X size={10} />
                    </button>
                  </span>
                )
              )}
            </div>
          </div>

          <div className={styles.section}>
            <h3 className={styles.sectionTitle}>扫描规则</h3>
            <div className={styles.rules}>
              {(localConfig?.scan_rules ?? appConfig.scan_rules).map(
                (rule, index) => (
                  <span key={index} className={styles.rule}>
                    {rule}
                    <button
                      onClick={() => {
                        setLocalConfig((prev) => ({
                          ...(prev || appConfig),
                          scan_rules: (
                            prev?.scan_rules || appConfig.scan_rules
                          ).filter((r) => r !== rule),
                        }));
                        setConfigDirty(true);
                      }}
                    >
                      <X size={10} />
                    </button>
                  </span>
                )
              )}
            </div>
          </div>
        </div>
        <div className={styles.footer}>
          <button className={styles.reset} onClick={handleReset}>
            重置默认
          </button>
          <button
            className={styles.save}
            onClick={handleSave}
            disabled={!configDirty}
          >
            保存
          </button>
        </div>
      </div>
    </div>
  );
}
