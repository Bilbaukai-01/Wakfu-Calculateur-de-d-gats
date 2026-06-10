use crate::model::{AppConfig};

// ==================================================================================
// CONSTANTS
// ==================================================================================

pub const LOGO_BYTES: &[u8] = include_bytes!("../logo.png");

// ==================================================================================
// CONFIG & PERSISTENCE
// ==================================================================================

pub fn load_config() -> AppConfig {
    if let Ok(content) = std::fs::read_to_string("config_v2.json") {
        if let Ok(config) = serde_json::from_str(&content) {
            return config;
        }
    }
    AppConfig::default()
}

pub fn save_config(config: &AppConfig) {
    if let Ok(json) = serde_json::to_string_pretty(config) {
        let _ = std::fs::write("config_v2.json", json);
    }
}

pub fn get_embedded_window_icon() -> eframe::egui::IconData {
    if let Ok(image) = image::load_from_memory(LOGO_BYTES) {
        let image = image.to_rgba8();
        let (width, height) = image.dimensions();
        return eframe::egui::IconData { rgba: image.into_raw(), width, height };
    }
    eframe::egui::IconData::default()
}
