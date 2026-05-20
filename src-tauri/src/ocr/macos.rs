use std::path::Path;
use std::process::Command;
use std::fs;

use super::{OCRRegion, OCRResult, OcrProvider};

pub struct MacOSNativeOcr {
    languages: Vec<String>,
}

impl MacOSNativeOcr {
    pub fn new(languages: Vec<String>) -> Self {
        Self { languages }
    }
}

impl OcrProvider for MacOSNativeOcr {
    fn recognize(&self, image_path: &Path) -> Result<OCRResult, String> {
        let script = include_str!("../../resources/ocr.swift");
        let temp_dir = std::env::temp_dir();
        let temp_script = temp_dir.join("lumina_ocr.swift");
        fs::write(&temp_script, script).map_err(|e| e.to_string())?;
        
        let output = Command::new("swift")
            .arg(&temp_script)
            .arg(image_path.to_string_lossy().to_string())
            .output()
            .map_err(|e| e.to_string())?;
        
        let _ = fs::remove_file(&temp_script);
        
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let trimmed = stdout.trim().to_string();
            
            if trimmed.starts_with('[') {
                let regions: Vec<OCRRegion> = serde_json::from_str(&trimmed)
                    .map_err(|e| format!("Failed to parse OCR output: {}", e))?;
                
                let raw_text = regions.iter()
                    .map(|r| r.text.as_str())
                    .collect::<Vec<_>>()
                    .join("\n");
                
                Ok(OCRResult { regions, raw_text })
            } else if trimmed.starts_with("ERROR") {
                Err(trimmed)
            } else {
                Err("Unexpected OCR output".to_string())
            }
        } else {
            let error = String::from_utf8_lossy(&output.stderr).to_string();
            Err(format!("OCR process failed: {}", error))
        }
    }
    
    fn is_available(&self) -> bool {
        #[cfg(target_os = "macos")]
        {
            Command::new("swift")
                .arg("--version")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        }
        #[cfg(not(target_os = "macos"))]
        {
            false
        }
    }
    
    fn name(&self) -> &str {
        "macos_native"
    }
}