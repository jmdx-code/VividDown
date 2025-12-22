use crate::models::YtDlpStatus;
use futures_util::StreamExt;
use reqwest::Client;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::sync::RwLock;
use tauri::{AppHandle, Emitter};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

const YTDLP_RELEASE_API: &str = "https://api.github.com/repos/yt-dlp/yt-dlp/releases/latest";
const YTDLP_DOWNLOAD_URL: &str = "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe";

pub struct YtDlpManager {
    tools_dir: PathBuf,
    status: RwLock<YtDlpStatus>,
}

impl YtDlpManager {
    pub fn new(app_data_dir: PathBuf) -> Self {
        let tools_dir = app_data_dir.join("tools");
        let _ = fs::create_dir_all(&tools_dir);
        
        let manager = Self {
            tools_dir,
            status: RwLock::new(YtDlpStatus {
                installed: false,
                version: None,
                path: None,
                latest_version: None,
                update_available: false,
            }),
        };
        
        manager.refresh_status();
        manager
    }
    
    pub fn get_exe_path(&self) -> PathBuf {
        self.tools_dir.join("yt-dlp.exe")
    }
    
    pub fn refresh_status(&self) {
        let exe_path = self.get_exe_path();
        let installed = exe_path.exists();
        
        let version = if installed {
            Self::get_version(&exe_path)
        } else {
            None
        };
        
        let mut status = self.status.write().unwrap();
        status.installed = installed;
        status.version = version;
        status.path = if installed { Some(exe_path) } else { None };
    }
    
    fn get_version(exe_path: &PathBuf) -> Option<String> {
        let mut cmd = Command::new(exe_path);
        cmd.arg("--version");
        
        #[cfg(target_os = "windows")]
        cmd.creation_flags(CREATE_NO_WINDOW);

        let output = cmd.output().ok()?;
        
        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout)
                .trim()
                .to_string();
            Some(version)
        } else {
            None
        }
    }
    
    pub fn get_status(&self) -> YtDlpStatus {
        self.status.read().unwrap().clone()
    }
    
    pub async fn check_for_updates(&self) -> Result<YtDlpStatus, String> {
        let client = Client::builder()
            .user_agent("YouTube-Downloader/0.1.0")
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
        
        let response: serde_json::Value = client
            .get(YTDLP_RELEASE_API)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch release info: {}", e))?
            .json()
            .await
            .map_err(|e| format!("Failed to parse release info: {}", e))?;
        
        let latest_version = response["tag_name"]
            .as_str()
            .map(|s| s.to_string());
        
        let mut status = self.status.write().unwrap();
        status.latest_version = latest_version.clone();
        
        if let (Some(current), Some(latest)) = (&status.version, &latest_version) {
            status.update_available = current != latest;
        } else {
            status.update_available = !status.installed;
        }
        
        Ok(status.clone())
    }
    
    pub async fn download_ytdlp(&self, app_handle: &AppHandle) -> Result<(), String> {
        let exe_path = self.get_exe_path();
        let temp_path = self.tools_dir.join("yt-dlp.exe.tmp");
        
        // Backup old version
        let backup_path = self.tools_dir.join("yt-dlp.exe.bak");
        if exe_path.exists() {
            let _ = fs::copy(&exe_path, &backup_path);
        }
        
        let client = Client::builder()
            .user_agent("YouTube-Downloader/0.1.0")
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
        
        let response = client
            .get(YTDLP_DOWNLOAD_URL)
            .send()
            .await
            .map_err(|e| format!("Failed to download yt-dlp: {}", e))?;
        
        let total_size = response.content_length().unwrap_or(0);
        let mut downloaded: u64 = 0;
        
        let mut file = File::create(&temp_path)
            .map_err(|e| format!("Failed to create temp file: {}", e))?;
        
        let mut stream = response.bytes_stream();
        
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| format!("Download error: {}", e))?;
            file.write_all(&chunk)
                .map_err(|e| format!("Failed to write chunk: {}", e))?;
            
            downloaded += chunk.len() as u64;
            
            if total_size > 0 {
                let progress = (downloaded as f64 / total_size as f64) * 100.0;
                let _ = app_handle.emit("ytdlp-download-progress", progress);
            }
        }
        
        drop(file);
        
        // Move temp file to target location
        if exe_path.exists() {
            fs::remove_file(&exe_path)
                .map_err(|e| format!("Failed to remove old yt-dlp: {}", e))?;
        }
        
        fs::rename(&temp_path, &exe_path)
            .map_err(|e| format!("Failed to move new yt-dlp: {}", e))?;
        
        // Clean up backup
        let _ = fs::remove_file(&backup_path);
        
        // Refresh status
        self.refresh_status();
        
        let _ = app_handle.emit("ytdlp-download-complete", ());
        
        Ok(())
    }
    
    pub fn is_installed(&self) -> bool {
        self.status.read().unwrap().installed
    }
}
