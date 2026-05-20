use std::path::Path;

use super::{OCRResult, OcrProvider};
use crate::config::OcrSettings;

pub struct ApiOcr {
    config: OcrSettings,
}

impl ApiOcr {
    pub fn new(config: OcrSettings) -> Self {
        Self { config }
    }
}

impl OcrProvider for ApiOcr {
    fn recognize(&self, image_path: &Path) -> Result<OCRResult, String> {
        let endpoint = self.config.api_endpoint.as_ref()
            .ok_or("API endpoint not configured")?;
        let api_key = self.config.api_key.as_ref()
            .ok_or("API key not configured")?;
        
        // Read image file
        let image_data = std::fs::read(image_path)
            .map_err(|e| format!("Failed to read image: {}", e))?;
        
        // Encode to base64
        use base64::Engine;
        let base64_image = base64::engine::general_purpose::STANDARD.encode(&image_data);
        
        // Build request body (generic OCR API format)
        let body = serde_json::json!({
            "image": base64_image,
            "language": self.config.languages.first().unwrap_or(&"zh".to_string()),
        });
        
        // Make HTTP request
        let client = reqwest::blocking::Client::new();
        let response = client.post(endpoint)
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&body)
            .send()
            .map_err(|e| format!("OCR API request failed: {}", e))?;
        
        if !response.status().is_success() {
            return Err(format!("OCR API returned status: {}", response.status()));
        }
        
        let result: serde_json::Value = response.json()
            .map_err(|e| format!("Failed to parse OCR API response: {}", e))?;
        
        // Parse response (adapt to your specific API response format)
        let regions = result.get("regions")
            .and_then(|r| serde_json::from_value(r.clone()).ok())
            .unwrap_or_default();
        
        let raw_text = result.get("text")
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .to_string();
        
        Ok(OCRResult { regions, raw_text })
    }
    
    fn is_available(&self) -> bool {
        self.config.api_endpoint.is_some() && self.config.api_key.is_some()
    }
    
    fn name(&self) -> &str {
        "api"
    }
}