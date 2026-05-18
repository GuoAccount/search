use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::Manager;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub version: u32,
    pub scan: ScanSettings,
    #[serde(default)]
    pub display: DisplaySettings,
    pub skip_rules: Vec<String>,
    pub scan_rules: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScanSettings {
    pub large_dir_threshold: u64,
    pub ask_on_large_dir: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DisplaySettings {
    pub default_expand_count: u32,
    pub ocr_highlight_enabled: bool,
    #[serde(default = "default_match_context_length")]
    pub match_context_length: u32,
}

fn default_match_context_length() -> u32 {
    100
}

impl Default for DisplaySettings {
    fn default() -> Self {
        DisplaySettings {
            default_expand_count: 1,
            ocr_highlight_enabled: true,
            match_context_length: 100,
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            version: 1,
            scan: ScanSettings {
                large_dir_threshold: 1000,
                ask_on_large_dir: true,
            },
            display: DisplaySettings {
                default_expand_count: 1,
                ocr_highlight_enabled: true,
                match_context_length: 100,
            },
            skip_rules: vec![
                "node_modules".into(),
                ".git".into(),
                "target".into(),
                "__pycache__".into(),
                "vendor".into(),
                ".venv".into(),
                "venv".into(),
                ".mypy_cache".into(),
                ".pytest_cache".into(),
                "dist".into(),
                "build".into(),
                "out".into(),
                ".next".into(),
                ".nuxt".into(),
                ".idea".into(),
                ".DS_Store".into(),
                "Thumbs.db".into(),
            ],
            scan_rules: vec![],
        }
    }
}

impl AppConfig {
    pub fn config_path(app_handle: &tauri::AppHandle) -> PathBuf {
        let config_dir = app_handle
            .path()
            .app_config_dir()
            .expect("Failed to get app config dir");
        config_dir.join("config.json")
    }

    pub fn load(app_handle: &tauri::AppHandle) -> Self {
        let path = Self::config_path(app_handle);
        if path.exists() {
            let content = fs::read_to_string(&path).unwrap_or_default();
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            let config = Self::default();
            let _ = config.save(app_handle);
            config
        }
    }

    pub fn save(&self, app_handle: &tauri::AppHandle) -> Result<(), String> {
        let path = Self::config_path(app_handle);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let content = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(&path, content).map_err(|e| e.to_string())
    }
}
