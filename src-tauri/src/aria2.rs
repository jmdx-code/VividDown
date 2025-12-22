use futures_util::StreamExt;
use reqwest::Client;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::sync::RwLock;
use tauri::{AppHandle, Emitter};
use zip::ZipArchive;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

// Windows x64 aria2 release
const ARIA2_RELEASE_URL: &str = "https://github.com/aria2/aria2/releases/download/release-1.37.0/aria2-1.37.0-win-64bit-build1.zip";

#[derive(Debug, Clone, serde::Serialize)]
pub struct Aria2Status {
    pub installed: bool,
    pub path: Option<PathBuf>,
    pub version: Option<String>,
    pub is_downloading: bool,
}

pub struct Aria2Manager {
    tools_dir: PathBuf,
    status: RwLock<Aria2Status>,
}

impl Aria2Manager {
    pub fn new(app_data_dir: PathBuf) -> Self {
        let tools_dir = app_data_dir.join("tools");
        let _ = fs::create_dir_all(&tools_dir);

        let manager = Self {
            tools_dir,
            status: RwLock::new(Aria2Status {
                installed: false,
                path: None,
                version: None,
                is_downloading: false,
            }),
        };

        manager.refresh_status();
        manager
    }

    pub fn get_exe_path(&self) -> PathBuf {
        self.tools_dir.join("aria2c.exe")
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
            let stdout = String::from_utf8_lossy(&output.stdout);
            let first_line = stdout.lines().next()?;
            // Format: "aria2 version 1.37.0"
            let parts: Vec<&str> = first_line.split_whitespace().collect();
            if parts.len() >= 3 && parts[1] == "version" {
                return Some(parts[2].to_string());
            }
            Some("Unknown".to_string())
        } else {
            None
        }
    }

    pub fn get_status(&self) -> Aria2Status {
        self.status.read().unwrap().clone()
    }

    pub async fn download_aria2(&self, app_handle: &AppHandle) -> Result<(), String> {
        {
            let mut status = self.status.write().unwrap();
            if status.is_downloading {
                return Err("Already downloading".to_string());
            }
            status.is_downloading = true;
        }

        let result = self.download_process(app_handle).await;

        {
            let mut status = self.status.write().unwrap();
            status.is_downloading = false;
        }
        self.refresh_status();

        result
    }

    async fn download_process(&self, app_handle: &AppHandle) -> Result<(), String> {
        let temp_zip = self.tools_dir.join("aria2.zip.tmp");

        let client = Client::builder()
            .user_agent("YouTube-Downloader/0.1.0")
            .build()
            .map_err(|e| format!("Failed to create client: {}", e))?;

        let response = client
            .get(ARIA2_RELEASE_URL)
            .send()
            .await
            .map_err(|e| format!("Failed to request aria2: {}", e))?;

        let total_size = response.content_length().unwrap_or(0);
        let mut downloaded: u64 = 0;

        let mut file =
            File::create(&temp_zip).map_err(|e| format!("Failed to create temp file: {}", e))?;

        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| format!("Download error: {}", e))?;
            file.write_all(&chunk)
                .map_err(|e| format!("Write error: {}", e))?;

            downloaded += chunk.len() as u64;

            if total_size > 0 {
                let progress = (downloaded as f64 / total_size as f64) * 100.0;
                let _ = app_handle.emit("aria2-download-progress", progress);
            }
        }

        drop(file);

        // Extract
        let _ = app_handle.emit("aria2-download-progress", 100.0);
        self.extract_aria2(&temp_zip)
            .map_err(|e| format!("Extraction failed: {}", e))?;

        // Cleanup
        let _ = fs::remove_file(&temp_zip);

        let _ = app_handle.emit("aria2-download-complete", ());

        Ok(())
    }

    fn extract_aria2(&self, zip_path: &PathBuf) -> Result<(), String> {
        let file = File::open(zip_path).map_err(|e| e.to_string())?;
        let mut archive = ZipArchive::new(file).map_err(|e| e.to_string())?;

        // Find aria2c.exe in the archive
        // Structure: aria2-1.37.0-win-64bit-build1/aria2c.exe

        let mut aria2_file = None;

        for i in 0..archive.len() {
            let file = archive.by_index(i).map_err(|e| e.to_string())?;
            if file.name().ends_with("aria2c.exe") {
                aria2_file = Some(i);
                break;
            }
        }

        if let Some(index) = aria2_file {
            let mut file = archive.by_index(index).map_err(|e| e.to_string())?;
            let target_path = self.tools_dir.join("aria2c.exe");

            let mut outfile = File::create(&target_path).map_err(|e| e.to_string())?;
            std::io::copy(&mut file, &mut outfile).map_err(|e| e.to_string())?;
        } else {
            return Err("aria2c.exe not found in zip archive".to_string());
        }

        Ok(())
    }

    pub fn is_installed(&self) -> bool {
        self.status.read().unwrap().installed
    }
}
