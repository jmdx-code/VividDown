use crate::models::AppSettings;
use std::fs;
use std::path::PathBuf;
use std::sync::RwLock;

pub struct SettingsManager {
    settings: RwLock<AppSettings>,
    config_path: PathBuf,
    app_data_dir: PathBuf,
}

impl SettingsManager {
    pub fn new(app_data_dir: PathBuf) -> Self {
        let config_path = app_data_dir.join("settings.json");
        let settings = Self::load_from_file(&config_path).unwrap_or_default();
        
        Self {
            settings: RwLock::new(settings),
            config_path,
            app_data_dir,
        }
    }
    
    pub fn get(&self) -> AppSettings {
        self.settings.read().unwrap().clone()
    }
    
    pub fn get_app_data_dir(&self) -> PathBuf {
        self.app_data_dir.clone()
    }
    
    pub fn save(&self, settings: AppSettings) -> Result<(), String> {
        // Ensure download directory exists
        if !settings.download_dir.exists() {
            let _ = fs::create_dir_all(&settings.download_dir);
        }
        
        // Ensure config directory exists
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }
        
        // Save settings
        let json = serde_json::to_string_pretty(&settings)
            .map_err(|e| format!("Failed to serialize settings: {}", e))?;
        
        fs::write(&self.config_path, json)
            .map_err(|e| format!("Failed to write settings file: {}", e))?;
        
        *self.settings.write().unwrap() = settings;
        
        Ok(())
    }
    
    fn load_from_file(path: &PathBuf) -> Option<AppSettings> {
        let content = fs::read_to_string(path).ok()?;
        serde_json::from_str(&content).ok()
    }
}
