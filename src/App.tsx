import { useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { sendNotification } from "@tauri-apps/plugin-notification";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useStore } from "./store";
import { Sidebar } from "./components/Sidebar/Sidebar";
import { SearchBar } from "./components/SearchBar/SearchBar";
import { ResultsView } from "./components/ResultsView/ResultsView";
import { ConfirmPanel } from "./components/modals/ConfirmPanel";
import { SkippedDirsPanel } from "./components/modals/SkippedDirsPanel";
import { SettingsPanel } from "./components/modals/SettingsPanel";
import { FilePreviewModal } from "./components/modals/FilePreviewModal";
import { ImagePreviewModal } from "./components/modals/ImagePreviewModal";
import {
  PanelLeftClose,
  PanelLeftOpen,
  FolderOpen,
} from "lucide-react";
import "./index.css";
import "./App.css";

function App() {
  const {
    settings,
    updateSettings,
    setAppConfig,
    scanProgress,
    setShowConfirmPanel,
    showConfirmPanel,
    isFullscreen,
    setIsFullscreen,
  } = useStore();

  const isOpen = settings.sidebarOpen;
  const lastPendingCount = useRef(0);
  const lastNotifyTime = useRef(0);
  const pendingNotifyCount = useRef(0);
  const notifyTimer = useRef<number | null>(null);

  // Load app config
  useEffect(() => {
    invoke<any>("get_config").then(setAppConfig).catch(console.error);
  }, []);

  // Apply theme
  useEffect(() => {
    document.documentElement.setAttribute("data-theme", settings.theme);
  }, [settings.theme]);

  // Fullscreen detection
  useEffect(() => {
    const appWindow = getCurrentWindow();
    
    const checkFullscreen = async () => {
      try {
        const fullscreen = await appWindow.isFullscreen();
        setIsFullscreen(fullscreen);
      } catch (err) {
        console.error("Failed to check fullscreen:", err);
      }
    };

    checkFullscreen();

    const unlisten = appWindow.onResized(() => {
      checkFullscreen();
    });

    return () => {
      unlisten.then(fn => fn());
    };
  }, []);

  // Keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === "b") {
        e.preventDefault();
        updateSettings({ sidebarOpen: !settings.sidebarOpen });
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [settings.sidebarOpen]);

  // Notification detection from polling + auto-open confirm panel
  useEffect(() => {
    const currentCount = scanProgress?.pending_confirmations.length || 0;
    if (currentCount > lastPendingCount.current) {
      const newCount = currentCount - lastPendingCount.current;

      // Send system notification
      const doNotify = () => {
        sendNotification({ title: "FileScope - 待确认", body: `收到 ${newCount} 个待确认事项` });
        invoke("play_system_sound").catch(() => {});
      };

      const now = Date.now();
      const THROTTLE_MS = 5000;
      const timeSinceLastNotify = now - lastNotifyTime.current;

      if (timeSinceLastNotify < THROTTLE_MS) {
        pendingNotifyCount.current += 1;
        if (!notifyTimer.current) {
          notifyTimer.current = window.setTimeout(() => {
            const count = pendingNotifyCount.current;
            if (count > 0) {
              sendNotification({ title: "FileScope - 待确认", body: `收到 ${count + 1} 个待确认事项` });
              invoke("play_system_sound").catch(() => {});
            }
            pendingNotifyCount.current = 0;
            notifyTimer.current = null;
            lastNotifyTime.current = Date.now();
          }, THROTTLE_MS - timeSinceLastNotify);
        }
      } else {
        doNotify();
        lastNotifyTime.current = now;
        pendingNotifyCount.current = 0;
      }

      // Auto-open confirm panel when new confirmations arrive
      if (!showConfirmPanel) {
        setShowConfirmPanel(true);
      }
    }
    lastPendingCount.current = currentCount;
  }, [scanProgress?.pending_confirmations.length]);

  // Auto-open confirm panel when scan completes with pending confirmations
  useEffect(() => {
    if (
      scanProgress &&
      scanProgress.status === "completed" &&
      scanProgress.pending_confirmations.length > 0 &&
      scanProgress.results.length === 0 &&
      !showConfirmPanel
    ) {
      setShowConfirmPanel(true);
    }
  }, [scanProgress?.status]);

  return (
    <div className="app-container">
      <button
        className={`fixed-toggle ${isFullscreen ? "fullscreen" : ""}`}
        onClick={() => updateSettings({ sidebarOpen: !settings.sidebarOpen })}
      >
        {isOpen ? <PanelLeftClose size={16} /> : <PanelLeftOpen size={16} />}
      </button>
      <div className={`main-wrap ${isOpen ? "is-open" : ""}`}>
        <Sidebar />
        <div className="content-col">
          <div className="content-header" data-tauri-drag-region="deep">
            <div className={`content-header-left ${!isOpen ? (isFullscreen ? "sidebar-closed-fullscreen" : "sidebar-closed") : ""}`}>
              {settings.scanPath && (
                <div className="header-path">
                  <FolderOpen size={12} />
                  <span>{settings.scanPath}</span>
                </div>
              )}
            </div>
            <div className="content-header-right">
              <button
                className="content-header-btn"
                onClick={async () => {
                  try {
                    const { open } = await import("@tauri-apps/plugin-dialog");
                    const selected = await open({ directory: true });
                    if (selected) {
                      updateSettings({ scanPath: selected as string });
                    }
                  } catch (err) {
                    console.error("Failed to select directory:", err);
                  }
                }}
              >
                <FolderOpen size={13} />
                <span>选择目录</span>
              </button>
            </div>
          </div>
          <div className="content-body">
            <SearchBar />
            <ResultsView />
          </div>
        </div>
      </div>
      <ConfirmPanel />
      <SkippedDirsPanel />
      <SettingsPanel />
      <FilePreviewModal />
      <ImagePreviewModal />
    </div>
  );
}

export default App;
