import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import {
  AppSettings,
  ScanProgress,
  ScanConfig,
  FilePreview,
  AppConfig,
  ImagePreview,
} from "../types";
import { loadSettings, saveSettings } from "../utils/storage";
import { getAllExtensions } from "../utils/file";

interface AppState {
  // Settings
  settings: AppSettings;
  updateSettings: (updates: Partial<AppSettings>) => void;

  // Scan state
  isScanning: boolean;
  scanProgress: ScanProgress | null;
  scanInterval: number | null;

  // UI state
  selectedResults: Set<string>;
  previewFile: FilePreview | null;
  previewImage: ImagePreview | null;
  activeTab: string;
  expandedFolders: Set<string>;
  expandedPresets: Set<string>;
  showConfirmPanel: boolean;
  showSkippedPanel: boolean;
  showSettings: boolean;
  isFullscreen: boolean;

  // Config
  appConfig: AppConfig | null;
  localConfig: AppConfig | null;
  configDirty: boolean;

  // Scan actions
  startScan: () => Promise<void>;
  cancelScan: () => Promise<void>;
  pauseScan: () => Promise<void>;
  resumeScan: () => Promise<void>;
  respondConfirmation: (confirmationId: string, allow: boolean, remember: boolean) => Promise<void>;
  allowAllConfirmations: () => Promise<void>;

  // UI actions
  setIsScanning: (isScanning: boolean) => void;
  setIsFullscreen: (isFullscreen: boolean) => void;
  setScanProgress: (progress: ScanProgress | null | ((prev: ScanProgress | null) => ScanProgress | null)) => void;
  setSelectedResults: (results: Set<string> | ((prev: Set<string>) => Set<string>)) => void;
  setPreviewFile: (file: FilePreview | null) => void;
  setPreviewImage: (image: ImagePreview | null) => void;
  setActiveTab: (tab: string) => void;
  setExpandedFolders: (folders: Set<string> | ((prev: Set<string>) => Set<string>)) => void;
  setExpandedPresets: (presets: Set<string> | ((prev: Set<string>) => Set<string>)) => void;
  setShowConfirmPanel: (show: boolean) => void;
  setShowSkippedPanel: (show: boolean) => void;
  setShowSettings: (show: boolean) => void;
  setAppConfig: (config: AppConfig | null) => void;
  setLocalConfig: (config: AppConfig | null | ((prev: AppConfig | null) => AppConfig | null)) => void;
  setConfigDirty: (dirty: boolean) => void;
  setScanInterval: (interval: number | null) => void;
}

export const useStore = create<AppState>((set, get) => ({
  // Settings
  settings: loadSettings(),
  updateSettings: (updates) => {
    set((state) => {
      const newSettings = { ...state.settings, ...updates };
      saveSettings(newSettings);
      return { settings: newSettings };
    });
  },

  // Scan state
  isScanning: false,
  scanProgress: null,
  scanInterval: null,

  // UI state
  selectedResults: new Set(),
  previewFile: null,
  previewImage: null,
  activeTab: "all",
  expandedFolders: new Set(),
  expandedPresets: new Set(),
  showConfirmPanel: false,
  showSkippedPanel: false,
  showSettings: false,
  isFullscreen: false,

  // Config
  appConfig: null,
  localConfig: null,
  configDirty: false,

  // Scan actions
  startScan: async () => {
    const { settings } = get();
    const extensions = getAllExtensions(settings.enabledPresets, settings.customExtensions);
    if (!settings.scanPath || !settings.keyword || extensions.length === 0) return;

    const scanTypes = ["file_name", "text_content"];
    if (settings.enabledPresets.includes("image")) {
      scanTypes.push("exif_data");
      if (settings.ocrEnabled) {
        scanTypes.push("ocr_text");
      }
    }

    const config: ScanConfig = {
      path: settings.scanPath,
      keyword: settings.keyword,
      scan_types: scanTypes,
      file_extensions: extensions,
      exclude_patterns: [],
    };

    try {
      set({
        isScanning: true,
        selectedResults: new Set(),
        activeTab: "all",
        scanProgress: null,
      });

      const scanId = await invoke<string>("start_scan", { config });

      const interval = window.setInterval(async () => {
        try {
          const progress = await invoke<ScanProgress>("get_scan_progress", { scanId });
          set({ scanProgress: progress });
          if (progress.status === "completed" || progress.status === "cancelled") {
            set({ isScanning: false });
            window.clearInterval(interval);
            set({ scanInterval: null });
          }
        } catch (err) {
          console.error("Failed to get progress:", err);
        }
      }, 200);

      set({ scanInterval: interval });
    } catch (err) {
      console.error("Failed to start scan:", err);
      set({ isScanning: false });
    }
  },

  cancelScan: async () => {
    const { scanProgress, scanInterval } = get();
    if (scanProgress?.scan_id) {
      try {
        await invoke("cancel_scan", { scanId: scanProgress.scan_id });
        set({ isScanning: false });
        if (scanInterval) {
          window.clearInterval(scanInterval);
          set({ scanInterval: null });
        }
      } catch (err) {
        console.error("Failed to cancel scan:", err);
      }
    }
  },

  pauseScan: async () => {
    const { scanProgress } = get();
    if (scanProgress?.scan_id) {
      try {
        await invoke("pause_scan", { scanId: scanProgress.scan_id });
        set((state) => ({
          scanProgress: state.scanProgress
            ? { ...state.scanProgress, status: "paused" }
            : null,
        }));
      } catch (err) {
        console.error("Failed to pause scan:", err);
      }
    }
  },

  resumeScan: async () => {
    const { scanProgress } = get();
    if (scanProgress?.scan_id) {
      try {
        await invoke("resume_scan", { scanId: scanProgress.scan_id });
        set((state) => ({
          scanProgress: state.scanProgress
            ? { ...state.scanProgress, status: "scanning" }
            : null,
        }));
      } catch (err) {
        console.error("Failed to resume scan:", err);
      }
    }
  },

  respondConfirmation: async (confirmationId: string, allow: boolean, remember: boolean) => {
    const { scanProgress } = get();
    if (!scanProgress) return;

    const confirmation = scanProgress.pending_confirmations.find(
      (c) => c.id === confirmationId
    );

    try {
      await invoke("respond_confirmation", {
        scanId: scanProgress.scan_id,
        confirmationId,
        allow,
        remember,
      });

      // Update local state
      set((state) => {
        if (!state.scanProgress) return state;
        return {
          scanProgress: {
            ...state.scanProgress,
            pending_confirmations: state.scanProgress.pending_confirmations.filter(
              (c) => c.id !== confirmationId
            ),
            skipped_dirs: allow
              ? state.scanProgress.skipped_dirs
              : [
                  ...state.scanProgress.skipped_dirs,
                  {
                    path: confirmation?.path || "",
                    reason: remember ? "user_skip_remembered" : "user_skip",
                  },
                ],
          },
        };
      });

      // No sub-scan needed! Results flow through the main scan's work channel.
      // The backend's respond_confirmation sends the directory to search workers
      // via work_tx, and results appear in the main scan's progress automatically.

      // Refresh config if remember was checked
      if (remember) {
        const updatedConfig = await invoke<AppConfig>("get_config");
        set({ appConfig: updatedConfig, localConfig: updatedConfig });
      }
    } catch (err) {
      console.error("Failed to respond confirmation:", err);
    }
  },

  allowAllConfirmations: async () => {
    const { scanProgress, respondConfirmation } = get();
    if (!scanProgress) return;

    for (const confirmation of scanProgress.pending_confirmations) {
      await respondConfirmation(confirmation.id, true, false);
    }
  },

  // UI actions
  setIsScanning: (isScanning) => set({ isScanning }),
  setIsFullscreen: (isFullscreen) => set({ isFullscreen }),
  setScanProgress: (progress) =>
    set((state) => ({
      scanProgress:
        typeof progress === "function" ? progress(state.scanProgress) : progress,
    })),
  setSelectedResults: (results) =>
    set((state) => ({
      selectedResults:
        typeof results === "function" ? results(state.selectedResults) : results,
    })),
  setPreviewFile: (file) => set({ previewFile: file }),
  setPreviewImage: (image) => set({ previewImage: image }),
  setActiveTab: (tab) => set({ activeTab: tab }),
  setExpandedFolders: (folders) =>
    set((state) => ({
      expandedFolders:
        typeof folders === "function"
          ? folders(state.expandedFolders)
          : folders,
    })),
  setExpandedPresets: (presets) =>
    set((state) => ({
      expandedPresets:
        typeof presets === "function"
          ? presets(state.expandedPresets)
          : presets,
    })),
  setShowConfirmPanel: (show) => set({ showConfirmPanel: show }),
  setShowSkippedPanel: (show) => set({ showSkippedPanel: show }),
  setShowSettings: (show) => set({ showSettings: show }),
  setAppConfig: (config) => set({ appConfig: config }),
  setLocalConfig: (config) =>
    set((state) => ({
      localConfig:
        typeof config === "function" ? config(state.localConfig) : config,
    })),
  setConfigDirty: (dirty) => set({ configDirty: dirty }),
  setScanInterval: (interval) => set({ scanInterval: interval }),
}));
