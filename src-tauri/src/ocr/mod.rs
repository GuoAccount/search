use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OCRRegion {
    pub text: String,
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OCRResult {
    pub regions: Vec<OCRRegion>,
    pub raw_text: String,
}

pub trait OcrProvider: Send + Sync {
    fn recognize(&self, image_path: &Path) -> Result<OCRResult, String>;
    fn is_available(&self) -> bool;
    fn name(&self) -> &str;
}

#[cfg(target_os = "macos")]
pub mod macos;

pub mod api;

pub fn create_ocr_provider(config: &crate::config::OcrSettings) -> Box<dyn OcrProvider> {
    match config.provider {
        #[cfg(target_os = "macos")]
        crate::config::OcrProviderType::MacOSNative => {
            Box::new(macos::MacOSNativeOcr::new(config.languages.clone()))
        }
        #[cfg(not(target_os = "macos"))]
        crate::config::OcrProviderType::MacOSNative => {
            // Fallback to API on non-macOS platforms
            log::warn!("macOS native OCR not available on this platform, falling back to API");
            Box::new(api::ApiOcr::new(config.clone()))
        }
        crate::config::OcrProviderType::Api => {
            Box::new(api::ApiOcr::new(config.clone()))
        }
    }
}