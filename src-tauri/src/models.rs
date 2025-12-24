use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// Default resolution (e.g. "1080p", "720p", "best")
    pub default_resolution: String,
    /// Default concurrent downloads
    pub default_concurrent: u32,
    /// Download directory
    pub download_dir: PathBuf,
    /// Whether to use in-app login state
    #[serde(default)]
    pub use_app_login: bool,
    /// Last login time (Unix timestamp)
    #[serde(default)]
    pub login_time: Option<u64>,
    /// Auth method: "browser" or "cookies_file"
    #[serde(default)]
    pub auth_method: String,
    /// Browser to read cookies from (chrome, edge, firefox, brave)
    #[serde(default)]
    pub cookies_browser: String,
    /// Browser profile folder name (e.g. "Profile 1", "Default")
    #[serde(default)]
    pub cookies_profile: String,
    /// User avatar URL (persisted across restarts)
    #[serde(default)]
    pub avatar_url: Option<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        let download_dir = dirs::download_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("YouTube Downloads");

        Self {
            default_resolution: "1080p".to_string(),
            default_concurrent: 3,
            download_dir,
            use_app_login: false,
            login_time: None,
            auth_method: "browser".to_string(),
            cookies_browser: "chrome".to_string(),
            cookies_profile: "Default".to_string(),
            avatar_url: None,
        }
    }
}

/// Video Information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoInfo {
    pub id: String,
    pub url: String,
    pub title: String,
    pub duration: Option<u64>,
    pub duration_string: Option<String>,
    pub thumbnail: Option<String>,
    pub uploader: Option<String>,
    pub view_count: Option<u64>,
    pub formats: Vec<FormatInfo>,
    pub playlist_index: Option<u32>,
    pub playlist_count: Option<u32>,
}

/// Video Format Information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatInfo {
    pub format_id: String,
    pub format_note: Option<String>,
    pub ext: String,
    pub resolution: Option<String>,
    pub filesize: Option<u64>,
    pub filesize_approx: Option<u64>,
    pub vcodec: Option<String>,
    pub acodec: Option<String>,
}

/// Download Task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadTask {
    pub id: String,
    pub url: String,
    pub video_info: Option<VideoInfo>,
    pub status: DownloadStatus,
    pub progress: f64,
    pub speed: Option<String>,
    pub eta: Option<String>,
    pub error: Option<String>,
    pub resolution: String,
    pub output_path: Option<PathBuf>,
}

/// Download Status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DownloadStatus {
    Pending,
    Fetching,
    Downloading,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

/// yt-dlp Status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YtDlpStatus {
    pub installed: bool,
    pub version: Option<String>,
    pub path: Option<PathBuf>,
    pub latest_version: Option<String>,
    pub update_available: bool,
}

/// Download Progress Event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgressEvent {
    pub task_id: String,
    pub progress: f64,
    pub speed: Option<String>,
    pub eta: Option<String>,
    pub status: DownloadStatus,
    pub downloaded_bytes: Option<u64>,
    pub total_bytes: Option<u64>,
}

/// Login Status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginStatus {
    pub logged_in: bool,
    pub login_time: Option<u64>,
    pub cookies_valid: bool,
    pub auth_method: String,
    /// User avatar URL extracted from YouTube page
    #[serde(default)]
    pub avatar_url: Option<String>,
}

/// Cookies validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CookiesValidationResult {
    /// Status: "valid", "missing", "expired", "incomplete"
    pub status: String,
    /// Whether the invalid cookies file was deleted
    pub deleted: bool,
    /// User-friendly message
    pub message: String,
}
