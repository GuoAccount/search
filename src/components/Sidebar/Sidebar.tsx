import { useStore } from "../../store";
import { invoke } from "@tauri-apps/api/core";
import { PresetSelector } from "../PresetSelector/PresetSelector";
import { DEFAULT_PRESETS } from "../../constants/presets";
import { Sun, Moon, Monitor, Settings, FileText } from "lucide-react";
import styles from "./Sidebar.module.css";

export function Sidebar() {
  const {
    settings,
    updateSettings,
    setShowSettings,
    setLocalConfig,
    setConfigDirty,
    appConfig,
  } = useStore();

  const isOpen = settings.sidebarOpen;
  const selectedCount = settings.enabledPresets.length;
  const extCount = settings.enabledPresets.reduce((acc, preset) => {
    const defaultExts = DEFAULT_PRESETS[preset]?.extensions.length || 0;
    const custom = settings.customExtensions[preset] || [];
    return acc + defaultExts + custom.length;
  }, 0);

  const themeIcon =
    settings.theme === "light" ? (
      <Sun size={14} />
    ) : settings.theme === "dark" ? (
      <Moon size={14} />
    ) : (
      <Monitor size={14} />
    );

  const themeTitle =
    settings.theme === "light"
      ? "浅色模式"
      : settings.theme === "dark"
      ? "深色模式"
      : "跟随系统";

  const cycleTheme = () => {
    const themes = ["light", "dark", "system"] as const;
    const next = themes[(themes.indexOf(settings.theme) + 1) % themes.length];
    updateSettings({ theme: next });
  };

  return (
    <div className={`${styles.sidebar} ${isOpen ? styles.open : ""}`}>
      <div className={styles.header} data-tauri-drag-region="deep" />
      <div className={styles.scroll}>
        <PresetSelector />
      </div>
      <div className={styles.footer}>
        <div className={styles.footerRow}>
          <button
            className={styles.settingsBtn}
            onClick={() => {
              setLocalConfig(appConfig);
              setConfigDirty(false);
              setShowSettings(true);
            }}
            title="设置"
          >
            <Settings size={14} />
          </button>
          <button
            className={styles.settingsBtn}
            onClick={() => invoke('open_log_dir')}
            title="打开日志目录"
          >
            <FileText size={14} />
          </button>
          <button
            className={styles.themeBtn}
            onClick={cycleTheme}
            title={themeTitle}
          >
            {themeIcon}
          </button>
        </div>
        <div className={styles.info}>
          <span>
            {selectedCount} 类 · {extCount} 种格式
          </span>
        </div>
      </div>
    </div>
  );
}
