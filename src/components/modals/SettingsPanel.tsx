import { useState, useEffect, useRef, useCallback } from "react";
import { useStore } from "../../store";
import {
  Settings,
  Search,
  Monitor,
  Shield,
  FileCode,
  X,
  Sun,
  Moon,
  Plus,
} from "lucide-react";
import styles from "./SettingsPanel.module.css";

type SettingsTab = "general" | "scan" | "display" | "rules";

const TABS: { id: SettingsTab; label: string; icon: typeof Settings }[] = [
  { id: "general", label: "通用", icon: Settings },
  { id: "scan", label: "扫描", icon: Search },
  { id: "display", label: "显示", icon: Monitor },
  { id: "rules", label: "规则", icon: Shield },
];

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
    settings,
    updateSettings,
  } = useStore();

  const [activeTab, setActiveTab] = useState<SettingsTab>("general");
  const [newSkipRule, setNewSkipRule] = useState("");
  const [newScanRule, setNewScanRule] = useState("");
  const contentRef = useRef<HTMLDivElement>(null);
  const isScrollingRef = useRef(false);
  const scrollTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const handleScroll = useCallback(() => {
    if (isScrollingRef.current) return;
    const container = contentRef.current;
    if (!container) return;

    const sections = TABS.map((tab) => ({
      id: tab.id,
      el: container.querySelector(`[data-section="${tab.id}"]`),
    }));

    const containerRect = container.getBoundingClientRect();
    const containerTop = containerRect.top + 80;
    const isAtBottom = container.scrollTop + container.clientHeight >= container.scrollHeight - 20;

    if (isAtBottom) {
      setActiveTab(TABS[TABS.length - 1].id);
      return;
    }

    for (let i = sections.length - 1; i >= 0; i--) {
      const { id, el } = sections[i];
      if (!el) continue;
      const rect = el.getBoundingClientRect();
      if (rect.top <= containerTop) {
        setActiveTab(id);
        break;
      }
    }
  }, []);

  useEffect(() => {
    const container = contentRef.current;
    if (!container) return;
    container.addEventListener("scroll", handleScroll, { passive: true });
    return () => container.removeEventListener("scroll", handleScroll);
  }, [handleScroll]);

  const scrollToSection = (tabId: SettingsTab) => {
    const container = contentRef.current;
    if (!container) return;
    const section = container.querySelector(`[data-section="${tabId}"]`);
    if (!section) return;

    isScrollingRef.current = true;
    setActiveTab(tabId);

    section.scrollIntoView({ behavior: "smooth", block: "start" });

    if (scrollTimeoutRef.current) clearTimeout(scrollTimeoutRef.current);
    scrollTimeoutRef.current = setTimeout(() => {
      isScrollingRef.current = false;
    }, 500);
  };

  if (!showSettings || !appConfig) return null;

  const cfg = localConfig ?? appConfig;

  const setCfg = (updater: (prev: typeof cfg) => typeof cfg) => {
    setLocalConfig((prev) => updater(prev ?? appConfig));
    setConfigDirty(true);
  };

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

  const handleOpenConfigFile = async () => {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke("open_config_file");
    } catch (err) {
      console.error("Failed to open config file:", err);
    }
  };

  const cycleTheme = () => {
    const themes = ["light", "dark", "system"] as const;
    const next = themes[(themes.indexOf(settings.theme) + 1) % themes.length];
    updateSettings({ theme: next });
  };

  const themeIcon =
    settings.theme === "light" ? (
      <Sun size={14} />
    ) : settings.theme === "dark" ? (
      <Moon size={14} />
    ) : (
      <Monitor size={14} />
    );

  const themeLabel =
    settings.theme === "light"
      ? "浅色"
      : settings.theme === "dark"
      ? "深色"
      : "跟随系统";

  return (
    <div className={styles.overlay} onClick={() => setShowSettings(false)}>
      <div className={styles.panel} onClick={(e) => e.stopPropagation()}>
        <div className={styles.header}>
          <div className={styles.title}>
            <Settings size={16} />
            <span>设置</span>
          </div>
          <div className={styles.headerActions}>
            <button className={styles.openConfigBtn} onClick={handleOpenConfigFile}>
              <FileCode size={13} />
              <span>打开配置文件</span>
            </button>
            <button className={styles.closeBtn} onClick={() => setShowSettings(false)}>
              <X size={14} />
            </button>
          </div>
        </div>
        <div className={styles.body}>
          <div className={styles.sidebar}>
            {TABS.map((tab) => (
              <button
                key={tab.id}
                className={`${styles.navItem} ${activeTab === tab.id ? styles.navItemActive : ""}`}
                onClick={() => scrollToSection(tab.id)}
              >
                <tab.icon size={14} />
                <span>{tab.label}</span>
              </button>
            ))}
          </div>
          <div className={styles.content} ref={contentRef}>
            <div data-section="general" className={styles.section}>
              <h3 className={styles.sectionTitle}>通用</h3>
              <div className={styles.field}>
                <label className={styles.fieldLabel}>主题</label>
                <div className={styles.fieldControl}>
                  <button className={styles.themeBtn} onClick={cycleTheme}>
                    {themeIcon}
                    <span>{themeLabel}</span>
                  </button>
                </div>
              </div>
            </div>

            <div data-section="scan" className={styles.section}>
              <h3 className={styles.sectionTitle}>扫描</h3>
              <div className={styles.field}>
                <label className={styles.fieldLabel}>大目录阈值</label>
                <div className={styles.fieldControl}>
                  <input
                    type="number"
                    className={styles.input}
                    value={cfg.scan.large_dir_threshold}
                    onChange={(e) => {
                      setCfg((prev) => ({
                        ...prev,
                        scan: {
                          ...prev.scan,
                          large_dir_threshold: parseInt(e.target.value) || 1000,
                        },
                      }));
                    }}
                  />
                  <span className={styles.inputSuffix}>个子项</span>
                </div>
              </div>
              <div className={styles.field}>
                <label className={styles.fieldLabel}>遇到大目录时</label>
                <div className={styles.fieldControl}>
                  <select
                    className={styles.select}
                    value={cfg.scan.ask_on_large_dir ? "ask" : "skip"}
                    onChange={(e) => {
                      setCfg((prev) => ({
                        ...prev,
                        scan: {
                          ...prev.scan,
                          ask_on_large_dir: e.target.value === "ask",
                        },
                      }));
                    }}
                  >
                    <option value="ask">询问</option>
                    <option value="skip">始终跳过</option>
                  </select>
                </div>
              </div>
              <h3 className={styles.sectionTitle}>文档内容提取</h3>
              <p className={styles.sectionDescription}>
                启用后将在文档文件中提取文本内容进行搜索（默认开启）
              </p>
              {(["docx", "xlsx", "pdf", "pptx"] as const).map((format) => (
                <div className={styles.field} key={format}>
                  <label className={styles.fieldLabel}>{format.toUpperCase()}</label>
                  <div className={styles.fieldControl}>
                    <label className={styles.toggle}>
                      <input
                        type="checkbox"
                        checked={cfg.content_extraction[format]}
                        onChange={(e) => {
                          setCfg((prev) => ({
                            ...prev,
                            content_extraction: {
                              ...prev.content_extraction,
                              [format]: e.target.checked,
                            },
                          }));
                        }}
                      />
                      <span className={styles.toggleTrack}>
                        <span className={styles.toggleThumb} />
                      </span>
                      <span className={styles.toggleLabel}>
                        {cfg.content_extraction[format] ? "已启用" : "已禁用"}
                      </span>
                    </label>
                  </div>
                </div>
              ))}

              <h3 className={styles.sectionTitle}>OCR 图片文字识别</h3>
              <p className={styles.sectionDescription}>
                启用后可在图片中搜索文字内容（默认关闭）
              </p>
              <div className={styles.field}>
                <label className={styles.fieldLabel}>启用 OCR</label>
                <div className={styles.fieldControl}>
                  <label className={styles.toggle}>
                    <input
                      type="checkbox"
                      checked={cfg.ocr.enabled}
                      onChange={(e) => {
                        setCfg((prev) => ({
                          ...prev,
                          ocr: { ...prev.ocr, enabled: e.target.checked },
                        }));
                      }}
                    />
                    <span className={styles.toggleTrack}>
                      <span className={styles.toggleThumb} />
                    </span>
                    <span className={styles.toggleLabel}>
                      {cfg.ocr.enabled ? "已启用" : "已禁用"}
                    </span>
                  </label>
                </div>
              </div>

              {cfg.ocr.enabled && (
                <>
                  <div className={styles.field}>
                    <label className={styles.fieldLabel}>OCR 提供者</label>
                    <div className={styles.fieldControl}>
                      <select
                        className={styles.select}
                        value={cfg.ocr.provider}
                        onChange={(e) => {
                          setCfg((prev) => ({
                            ...prev,
                            ocr: {
                              ...prev.ocr,
                              provider: e.target.value as "macos_native" | "api",
                            },
                          }));
                        }}
                      >
                        <option value="macos_native">macOS 原生 (仅 macOS)</option>
                        <option value="api">第三方 API</option>
                      </select>
                    </div>
                  </div>

                  {cfg.ocr.provider === "api" && (
                    <>
                      <div className={styles.field}>
                        <label className={styles.fieldLabel}>API 端点</label>
                        <div className={styles.fieldControl}>
                          <input
                            type="text"
                            className={styles.input}
                            placeholder="https://api.example.com/ocr"
                            value={cfg.ocr.api_endpoint || ""}
                            onChange={(e) => {
                              setCfg((prev) => ({
                                ...prev,
                                ocr: { ...prev.ocr, api_endpoint: e.target.value || null },
                              }));
                            }}
                          />
                        </div>
                      </div>
                      <div className={styles.field}>
                        <label className={styles.fieldLabel}>API Key</label>
                        <div className={styles.fieldControl}>
                          <input
                            type="password"
                            className={styles.input}
                            placeholder="输入 API Key"
                            value={cfg.ocr.api_key || ""}
                            onChange={(e) => {
                              setCfg((prev) => ({
                                ...prev,
                                ocr: { ...prev.ocr, api_key: e.target.value || null },
                              }));
                            }}
                          />
                        </div>
                      </div>
                      <div className={styles.field}>
                        <label className={styles.fieldLabel}>API Secret</label>
                        <div className={styles.fieldControl}>
                          <input
                            type="password"
                            className={styles.input}
                            placeholder="输入 API Secret（可选）"
                            value={cfg.ocr.api_secret || ""}
                            onChange={(e) => {
                              setCfg((prev) => ({
                                ...prev,
                                ocr: { ...prev.ocr, api_secret: e.target.value || null },
                              }));
                            }}
                          />
                        </div>
                      </div>
                    </>
                  )}
                </>
              )}
            </div>

            <div data-section="display" className={styles.section}>
              <h3 className={styles.sectionTitle}>显示</h3>
              <div className={styles.field}>
                <label className={styles.fieldLabel}>默认展开目录数</label>
                <div className={styles.fieldControl}>
                  <input
                    type="number"
                    min={0}
                    className={styles.input}
                    value={cfg.display.default_expand_count}
                    onChange={(e) => {
                      setCfg((prev) => ({
                        ...prev,
                        display: {
                          ...prev.display,
                          default_expand_count: Math.max(0, parseInt(e.target.value) || 0),
                        },
                      }));
                    }}
                  />
                  <span className={styles.inputSuffix}>个结果目录</span>
                </div>
                <p className={styles.fieldHint}>
                  设为 0 则全部折叠。搜索结果刷新时自动展开前 N 个结果所在目录
                </p>
              </div>
              <div className={styles.field}>
                <label className={styles.fieldLabel}>匹配上下文长度</label>
                <div className={styles.fieldControl}>
                  <input
                    type="number"
                    min={50}
                    max={500}
                    className={styles.input}
                    value={cfg.display.match_context_length}
                    onChange={(e) => {
                      setCfg((prev) => ({
                        ...prev,
                        display: {
                          ...prev.display,
                          match_context_length: Math.max(50, Math.min(500, parseInt(e.target.value) || 100)),
                        },
                      }));
                    }}
                  />
                  <span className={styles.inputSuffix}>字符</span>
                </div>
                <p className={styles.fieldHint}>
                  匹配关键字前后各显示的字符数（50-500）
                </p>
              </div>
              <div className={styles.field}>
                <label className={styles.fieldLabel}>OCR 匹配描边</label>
                <div className={styles.fieldControl}>
                  <label className={styles.toggle}>
                    <input
                      type="checkbox"
                      checked={cfg.display.ocr_highlight_enabled}
                      onChange={(e) => {
                        setCfg((prev) => ({
                          ...prev,
                          display: {
                            ...prev.display,
                            ocr_highlight_enabled: e.target.checked,
                          },
                        }));
                      }}
                    />
                    <span className={styles.toggleTrack}>
                      <span className={styles.toggleThumb} />
                    </span>
                    <span className={styles.toggleLabel}>
                      {cfg.display.ocr_highlight_enabled ? "已启用" : "已禁用"}
                    </span>
                  </label>
                </div>
                <p className={styles.fieldHint}>
                  在图片预览中用红色边框标识 OCR 匹配的文字区域
                </p>
              </div>
            </div>

            <div data-section="rules" className={styles.section}>
              <h3 className={styles.sectionTitle}>跳过规则</h3>
              <p className={styles.sectionDescription}>
                匹配这些规则的目录将被跳过扫描
              </p>
              <div className={styles.rules}>
                {cfg.skip_rules.map((rule, index) => (
                  <span key={index} className={styles.rule}>
                    {rule}
                    <button
                      onClick={() => {
                        setCfg((prev) => ({
                          ...prev,
                          skip_rules: prev.skip_rules.filter((r) => r !== rule),
                        }));
                      }}
                    >
                      <X size={10} />
                    </button>
                  </span>
                ))}
              </div>
              <div className={styles.addRuleRow}>
                <input
                  type="text"
                  className={styles.addRuleInput}
                  placeholder="输入路径模式，如 /Users/*/Library"
                  value={newSkipRule}
                  onChange={(e) => setNewSkipRule(e.target.value)}
                  onKeyDown={(e) => {
                    if (e.key === "Enter" && newSkipRule.trim()) {
                      setCfg((prev) => ({
                        ...prev,
                        skip_rules: [...prev.skip_rules, newSkipRule.trim()],
                      }));
                      setNewSkipRule("");
                    }
                  }}
                />
                <button
                  className={styles.addRuleBtn}
                  onClick={() => {
                    if (newSkipRule.trim()) {
                      setCfg((prev) => ({
                        ...prev,
                        skip_rules: [...prev.skip_rules, newSkipRule.trim()],
                      }));
                      setNewSkipRule("");
                    }
                  }}
                >
                  <Plus size={12} />
                  添加
                </button>
              </div>

              <h3 className={styles.sectionTitle}>
                扫描规则
              </h3>
              <p className={styles.sectionDescription}>
                匹配这些路径的目录将被自动允许扫描（在"记住我的选择"后自动添加）
              </p>
              <div className={styles.rules}>
                {cfg.scan_rules.map((rule, index) => (
                  <span key={index} className={styles.rule}>
                    {rule}
                    <button
                      onClick={() => {
                        setCfg((prev) => ({
                          ...prev,
                          scan_rules: prev.scan_rules.filter((r) => r !== rule),
                        }));
                      }}
                    >
                      <X size={10} />
                    </button>
                  </span>
                ))}
              </div>
              <div className={styles.addRuleRow}>
                <input
                  type="text"
                  className={styles.addRuleInput}
                  placeholder="输入路径模式，如 /Users/*/Projects"
                  value={newScanRule}
                  onChange={(e) => setNewScanRule(e.target.value)}
                  onKeyDown={(e) => {
                    if (e.key === "Enter" && newScanRule.trim()) {
                      setCfg((prev) => ({
                        ...prev,
                        scan_rules: [...prev.scan_rules, newScanRule.trim()],
                      }));
                      setNewScanRule("");
                    }
                  }}
                />
                <button
                  className={styles.addRuleBtn}
                  onClick={() => {
                    if (newScanRule.trim()) {
                      setCfg((prev) => ({
                        ...prev,
                        scan_rules: [...prev.scan_rules, newScanRule.trim()],
                      }));
                      setNewScanRule("");
                    }
                  }}
                >
                  <Plus size={12} />
                  添加
                </button>
              </div>
            </div>
          </div>
        </div>
        <div className={styles.footer}>
          <button className={styles.resetBtn} onClick={handleReset}>
            重置默认
          </button>
          <button
            className={styles.saveBtn}
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
