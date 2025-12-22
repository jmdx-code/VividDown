use crate::models::{DownloadProgressEvent, DownloadStatus, DownloadTask, FormatInfo, VideoInfo};
use crate::ytdlp::YtDlpManager;
use regex::Regex;
use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Arc, RwLock};
use std::thread;
use tauri::{AppHandle, Emitter};
use tokio::sync::Semaphore;
use uuid::Uuid;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

use crate::aria2::Aria2Manager;
use crate::ffmpeg::FFmpegManager;

pub struct DownloadManager {
    tasks: Arc<RwLock<HashMap<String, DownloadTask>>>,
    /// Maps task_id to process ID for cancellation
    process_ids: Arc<RwLock<HashMap<String, u32>>>,
    ytdlp: Arc<YtDlpManager>,
    ffmpeg: Arc<FFmpegManager>,
    aria2: Arc<Aria2Manager>,
    semaphore: Arc<Semaphore>,
    max_concurrent: RwLock<u32>,
    app_data_dir: PathBuf,
}

impl DownloadManager {
    pub fn new(
        ytdlp: Arc<YtDlpManager>,
        ffmpeg: Arc<FFmpegManager>,
        aria2: Arc<Aria2Manager>,
        max_concurrent: u32,
        app_data_dir: PathBuf,
    ) -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            process_ids: Arc::new(RwLock::new(HashMap::new())),
            ytdlp,
            ffmpeg,
            aria2,
            semaphore: Arc::new(Semaphore::new(max_concurrent as usize)),
            max_concurrent: RwLock::new(max_concurrent),
            app_data_dir,
        }
    }

    pub fn set_max_concurrent(&self, max: u32) {
        *self.max_concurrent.write().unwrap() = max;
    }

    fn parse_video_info(&self, json: &serde_json::Value) -> Result<VideoInfo, String> {
        let id = json["id"].as_str().unwrap_or("unknown").to_string();
        let title = json["title"]
            .as_str()
            .unwrap_or("Unknown Title")
            .to_string();

        let url = json["webpage_url"]
            .as_str()
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("https://www.youtube.com/watch?v={}", id));

        let duration = json["duration"].as_u64();
        let duration_string = json["duration_string"].as_str().map(|s| s.to_string());
        let thumbnail = json["thumbnail"].as_str().map(|s| s.to_string());
        let uploader = json["uploader"].as_str().map(|s| s.to_string());
        let view_count = json["view_count"].as_u64();
        let playlist_index = json["playlist_index"].as_u64().map(|v| v as u32);
        let playlist_count = json["playlist_count"].as_u64().map(|v| v as u32);

        let mut formats = Vec::new();
        if let Some(format_array) = json["formats"].as_array() {
            for f in format_array {
                let format_info = FormatInfo {
                    format_id: f["format_id"].as_str().unwrap_or("").to_string(),
                    format_note: f["format_note"].as_str().map(|s| s.to_string()),
                    ext: f["ext"].as_str().unwrap_or("mp4").to_string(),
                    resolution: f["resolution"].as_str().map(|s| s.to_string()),
                    filesize: f["filesize"].as_u64(),
                    filesize_approx: f["filesize_approx"].as_u64(),
                    vcodec: f["vcodec"].as_str().map(|s| s.to_string()),
                    acodec: f["acodec"].as_str().map(|s| s.to_string()),
                };
                formats.push(format_info);
            }
        }

        Ok(VideoInfo {
            id,
            url,
            title,
            duration,
            duration_string,
            thumbnail,
            uploader,
            view_count,
            formats,
            playlist_index,
            playlist_count,
        })
    }

    pub fn create_task(&self, url: String, resolution: String) -> DownloadTask {
        let task = DownloadTask {
            id: Uuid::new_v4().to_string(),
            url,
            video_info: None,
            status: DownloadStatus::Pending,
            progress: 0.0,
            speed: None,
            eta: None,
            error: None,
            resolution,
            output_path: None,
        };

        self.tasks
            .write()
            .unwrap()
            .insert(task.id.clone(), task.clone());
        task
    }

    pub fn get_task(&self, task_id: &str) -> Option<DownloadTask> {
        self.tasks.read().unwrap().get(task_id).cloned()
    }

    pub fn get_all_tasks(&self) -> Vec<DownloadTask> {
        self.tasks.read().unwrap().values().cloned().collect()
    }

    pub fn update_task_status(&self, task_id: &str, status: DownloadStatus) {
        if let Some(task) = self.tasks.write().unwrap().get_mut(task_id) {
            task.status = status;
        }
    }

    pub fn update_task_error(&self, task_id: &str, error: String) {
        if let Some(task) = self.tasks.write().unwrap().get_mut(task_id) {
            task.status = DownloadStatus::Failed;
            task.error = Some(error);
        }
    }

    pub fn update_task_video_info(&self, task_id: &str, info: VideoInfo) {
        if let Some(task) = self.tasks.write().unwrap().get_mut(task_id) {
            task.video_info = Some(info);
        }
    }

    /// Cancel a running download by killing the yt-dlp process
    pub fn cancel_download(&self, task_id: &str) {
        // Kill the process if it exists
        if let Some(pid) = self.process_ids.write().unwrap().remove(task_id) {
            #[cfg(target_os = "windows")]
            {
                // Use taskkill on Windows to kill the process tree
                let _ = Command::new("taskkill")
                    .args(&["/F", "/T", "/PID", &pid.to_string()])
                    .creation_flags(CREATE_NO_WINDOW)
                    .output();
            }
            #[cfg(not(target_os = "windows"))]
            {
                // Use kill on Unix
                let _ = Command::new("kill")
                    .args(&["-9", &pid.to_string()])
                    .output();
            }
        }
        // Update task status
        if let Some(task) = self.tasks.write().unwrap().get_mut(task_id) {
            task.status = DownloadStatus::Cancelled;
        }
    }

    /// Pause a running download by killing the yt-dlp process
    /// The .part file is preserved so download can be resumed
    pub fn pause_download(&self, task_id: &str) {
        // Kill the process if it exists
        if let Some(pid) = self.process_ids.write().unwrap().remove(task_id) {
            #[cfg(target_os = "windows")]
            {
                let _ = Command::new("taskkill")
                    .args(&["/F", "/T", "/PID", &pid.to_string()])
                    .creation_flags(CREATE_NO_WINDOW)
                    .output();
            }
            #[cfg(not(target_os = "windows"))]
            {
                let _ = Command::new("kill")
                    .args(&["-9", &pid.to_string()])
                    .output();
            }
        }
        // Update task status to Paused (not Cancelled)
        if let Some(task) = self.tasks.write().unwrap().get_mut(task_id) {
            task.status = DownloadStatus::Paused;
        }
    }

    pub fn remove_task(&self, task_id: &str) {
        // First cancel any running download
        self.cancel_download(task_id);
        // Then remove from tasks
        self.tasks.write().unwrap().remove(task_id);
    }

    pub fn clear_completed(&self) {
        self.tasks.write().unwrap().retain(|_, task| {
            task.status != DownloadStatus::Completed && task.status != DownloadStatus::Failed
        });
    }

    /// Check if URL should use --no-playlist flag
    /// Returns true if URL contains a video ID (should download single video)
    pub fn should_use_no_playlist(url: &str) -> bool {
        // If URL contains v= or is a youtu.be short link, it's a single video
        url.contains("v=") || url.contains("youtu.be/")
    }

    /// Expand a playlist URL into individual video URLs
    pub fn expand_playlist(
        &self,
        url: &str,
        cookies_path: &PathBuf,
    ) -> Result<Vec<String>, String> {
        let exe_path = self.ytdlp.get_exe_path();
        if !exe_path.exists() {
            return Err("yt-dlp not installed".to_string());
        }

        let mut cmd = Command::new(&exe_path);
        cmd.args(&["--flat-playlist", "--dump-json", "--no-warnings"]);

        if cookies_path.exists() {
            cmd.args(&["--cookies", &cookies_path.to_string_lossy()]);
        }

        cmd.arg(url);
        cmd.stdout(Stdio::piped()).stderr(Stdio::null());

        #[cfg(target_os = "windows")]
        cmd.creation_flags(CREATE_NO_WINDOW);

        let output = cmd
            .output()
            .map_err(|e| format!("Failed to run yt-dlp: {}", e))?;

        if !output.status.success() {
            return Err("Failed to fetch playlist".to_string());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut video_urls = Vec::new();

        // Each line is a JSON object for one video in the playlist
        for line in stdout.lines() {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                if let Some(video_id) = json["id"].as_str() {
                    video_urls.push(format!("https://www.youtube.com/watch?v={}", video_id));
                } else if let Some(video_url) = json["url"].as_str() {
                    video_urls.push(video_url.to_string());
                }
            }
        }

        Ok(video_urls)
    }

    /// Start download using saved cookies file
    pub fn start_download(
        &self,
        task_id: String,
        download_dir: PathBuf,
        app_handle: AppHandle,
        cookies_path: PathBuf,
    ) {
        let exe_path = self.ytdlp.get_exe_path();
        let task = match self.get_task(&task_id) {
            Some(t) => t,
            None => return,
        };

        let tasks = self.tasks.clone();
        let process_ids = self.process_ids.clone();
        let semaphore = self.semaphore.clone();
        let ffmpeg = self.ffmpeg.clone();
        let aria2 = self.aria2.clone();
        let use_cookies = cookies_path.exists();
        let cookies_path_str = cookies_path.to_string_lossy().to_string();

        thread::spawn(move || {
            // Wait for permit (concurrency control)
            let _permit = tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(semaphore.acquire())
                .unwrap();

            // Update status to Downloading (no separate Fetching phase now)
            if let Some(t) = tasks.write().unwrap().get_mut(&task_id) {
                t.status = DownloadStatus::Downloading;
            }

            let _ = app_handle.emit(
                "download-status-changed",
                DownloadProgressEvent {
                    task_id: task_id.clone(),
                    progress: 0.0,
                    speed: None,
                    eta: None,
                    status: DownloadStatus::Downloading,
                    downloaded_bytes: None,
                    total_bytes: None,
                },
            );

            // Build output filename template
            let output_template = download_dir.join("%(title)s.%(ext)s");

            // Info JSON path with unique task_id to avoid conflicts in concurrent downloads
            let info_json_path = download_dir.join(format!(".{}.info.json", task_id));
            let info_json_template = download_dir.join(format!(".{}", task_id));

            // Build resolution argument
            let format_arg = match task.resolution.as_str() {
                "best" => "bestvideo[ext=mp4]+bestaudio[ext=m4a]/best[ext=mp4]/best".to_string(),
                "2160p" | "4K" => {
                    "bestvideo[height<=2160][ext=mp4]+bestaudio[ext=m4a]/best[height<=2160]"
                        .to_string()
                }
                "1440p" | "2K" => {
                    "bestvideo[height<=1440][ext=mp4]+bestaudio[ext=m4a]/best[height<=1440]"
                        .to_string()
                }
                "1080p" => "bestvideo[height<=1080][ext=mp4]+bestaudio[ext=m4a]/best[height<=1080]"
                    .to_string(),
                "720p" => "bestvideo[height<=720][ext=mp4]+bestaudio[ext=m4a]/best[height<=720]"
                    .to_string(),
                "480p" => "bestvideo[height<=480][ext=mp4]+bestaudio[ext=m4a]/best[height<=480]"
                    .to_string(),
                "360p" => "bestvideo[height<=360][ext=mp4]+bestaudio[ext=m4a]/best[height<=360]"
                    .to_string(),
                "audio" => "bestaudio[ext=m4a]/bestaudio".to_string(),
                _ => "bestvideo[height<=1080][ext=mp4]+bestaudio[ext=m4a]/best[height<=1080]"
                    .to_string(),
            };

            let output_str = output_template.to_string_lossy().to_string();
            let info_json_output = info_json_template.to_string_lossy().to_string();
            let mut args = vec![
                "-f".to_string(),
                format_arg,
                "--newline".to_string(),
                "--no-warnings".to_string(),
                "--progress".to_string(),
                "-o".to_string(),
                output_str,
                // Write info.json with unique task_id filename
                "--write-info-json".to_string(),
                "--output".to_string(),
                format!("infojson:{}", info_json_output),
            ];

            // If ffmpeg is installed locally, specify path
            let ffmpeg_path_buf = ffmpeg.get_exe_path();
            if ffmpeg_path_buf.exists() {
                if let Some(path_str) = ffmpeg_path_buf.to_str() {
                    args.push("--ffmpeg-location".to_string());
                    args.push(path_str.to_string());
                }
            }

            // Add cookies if available
            if use_cookies {
                args.push("--cookies".to_string());
                args.push(cookies_path_str);
            }

            // Add --no-playlist if URL contains a video ID
            if DownloadManager::should_use_no_playlist(&task.url) {
                args.push("--no-playlist".to_string());
            }

            // Use aria2 as external downloader if available (faster multi-connection download)
            // Only for http/https downloads, not HLS fragments (which have their own progress format)
            let aria2_path = aria2.get_exe_path();
            if aria2_path.exists() {
                // Use aria2 only for http/https protocols, not for m3u8/HLS
                args.push("--downloader".to_string());
                args.push("http,https:aria2c".to_string());
                args.push("--downloader-args".to_string());
                // -x: max connections per server, -s: split file into segments
                args.push(
                    "aria2c:-x 16 -s 16 --file-allocation=none --summary-interval=1".to_string(),
                );
                // Add aria2c directory to PATH so yt-dlp can find it
                if let Some(parent) = aria2_path.parent() {
                    let current_path = std::env::var("PATH").unwrap_or_default();
                    std::env::set_var(
                        "PATH",
                        format!("{};{}", parent.to_string_lossy(), current_path),
                    );
                }
            }

            args.push(task.url.clone());

            let mut cmd = Command::new(&exe_path);
            cmd.args(&args)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());

            #[cfg(target_os = "windows")]
            cmd.creation_flags(CREATE_NO_WINDOW);

            let mut child = match cmd.spawn() {
                Ok(c) => c,
                Err(e) => {
                    if let Some(t) = tasks.write().unwrap().get_mut(&task_id) {
                        t.status = DownloadStatus::Failed;
                        t.error = Some(format!("Failed to start download: {}", e));
                    }
                    return;
                }
            };

            // Store process ID for cancellation
            process_ids
                .write()
                .unwrap()
                .insert(task_id.clone(), child.id());

            // Parse progress output
            let stdout = child.stdout.take().unwrap();
            let reader = BufReader::new(stdout);

            // yt-dlp output formats:
            // Progress: [download]  45.2% of 100MiB at 5.23MiB/s ETA 00:10
            // Complete: [download] 100% of   31.76MiB in 00:00:01 at 19.91MiB/s
            let progress_regex = Regex::new(r"\[download\]\s*([\d.]+)%").unwrap();
            let speed_regex = Regex::new(r"at\s+([\d.]+\s*\w+/s)").unwrap();
            let eta_regex = Regex::new(r"ETA\s+(\S+)").unwrap();
            // Check for "already downloaded" or "has already been downloaded"
            let already_regex =
                Regex::new(r"(?i)(already\s+(been\s+)?downloaded|has already been recorded)")
                    .unwrap();

            // Flag to track if we've loaded the info.json
            let mut info_loaded = false;
            let info_json_path_clone = info_json_path.clone();
            let tasks_clone = tasks.clone();
            let app_handle_clone = app_handle.clone();
            let task_id_clone = task_id.clone();

            for line in reader.lines() {
                // Try to load video info from .info.json if not loaded yet
                if !info_loaded && info_json_path_clone.exists() {
                    if let Ok(content) = std::fs::read_to_string(&info_json_path_clone) {
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                            let video_info = VideoInfo {
                                id: json["id"].as_str().unwrap_or("unknown").to_string(),
                                url: json["webpage_url"]
                                    .as_str()
                                    .map(|s| s.to_string())
                                    .unwrap_or_default(),
                                title: json["title"]
                                    .as_str()
                                    .unwrap_or("Unknown Title")
                                    .to_string(),
                                duration: json["duration"].as_u64(),
                                duration_string: json["duration_string"]
                                    .as_str()
                                    .map(|s| s.to_string()),
                                thumbnail: json["thumbnail"].as_str().map(|s| s.to_string()),
                                uploader: json["uploader"].as_str().map(|s| s.to_string()),
                                view_count: json["view_count"].as_u64(),
                                formats: vec![],
                                playlist_index: None,
                                playlist_count: None,
                            };

                            // Update task with video info
                            if let Some(t) = tasks_clone.write().unwrap().get_mut(&task_id_clone) {
                                t.video_info = Some(video_info);
                            }

                            // Emit update to frontend
                            let _ = app_handle_clone.emit("task-info-updated", &task_id_clone);
                            info_loaded = true;
                        }
                    }
                }

                if let Ok(line) = line {
                    // DEBUG: Print all output lines to console
                    println!("[yt-dlp] {}", line);

                    // Check if file already exists
                    if already_regex.is_match(&line) {
                        if let Some(t) = tasks.write().unwrap().get_mut(&task_id) {
                            t.progress = 100.0;
                            t.status = DownloadStatus::Completed;
                        }
                        let _ = app_handle.emit(
                            "download-progress",
                            DownloadProgressEvent {
                                task_id: task_id.clone(),
                                progress: 100.0,
                                speed: None,
                                eta: Some("Already downloaded".to_string()),
                                status: DownloadStatus::Completed,
                                downloaded_bytes: None,
                                total_bytes: None,
                            },
                        );
                        continue;
                    }

                    // Parse download progress
                    if let Some(caps) = progress_regex.captures(&line) {
                        if let Some(progress_str) = caps.get(1) {
                            if let Ok(progress) = progress_str.as_str().parse::<f64>() {
                                let speed = speed_regex
                                    .captures(&line)
                                    .and_then(|c| c.get(1))
                                    .map(|m| m.as_str().to_string());

                                let eta = eta_regex
                                    .captures(&line)
                                    .and_then(|c| c.get(1))
                                    .map(|m| m.as_str().to_string());

                                // Update task progress
                                if let Some(t) = tasks.write().unwrap().get_mut(&task_id) {
                                    t.progress = progress;
                                    t.speed = speed.clone();
                                    t.eta = eta.clone();
                                }

                                // Emit progress event
                                let _ = app_handle.emit(
                                    "download-progress",
                                    DownloadProgressEvent {
                                        task_id: task_id.clone(),
                                        progress,
                                        speed,
                                        eta,
                                        status: DownloadStatus::Downloading,
                                        downloaded_bytes: None,
                                        total_bytes: None,
                                    },
                                );
                            }
                        }
                    }
                }
            }

            // Wait for process to finish
            match child.wait() {
                Ok(status) => {
                    // Remove from process_ids since process has ended
                    process_ids.write().unwrap().remove(&task_id);

                    // Check if the task was already paused or cancelled (by user action)
                    let current_status = tasks
                        .read()
                        .unwrap()
                        .get(&task_id)
                        .map(|t| t.status.clone());

                    // If already paused or cancelled, don't override the status
                    if current_status == Some(DownloadStatus::Paused)
                        || current_status == Some(DownloadStatus::Cancelled)
                    {
                        // Task was intentionally stopped, emit the current status
                        let _ = app_handle.emit(
                            "download-progress",
                            DownloadProgressEvent {
                                task_id: task_id.clone(),
                                progress: tasks
                                    .read()
                                    .unwrap()
                                    .get(&task_id)
                                    .map(|t| t.progress)
                                    .unwrap_or(0.0),
                                speed: None,
                                eta: None,
                                status: current_status.unwrap_or(DownloadStatus::Paused),
                                downloaded_bytes: None,
                                total_bytes: None,
                            },
                        );
                        return;
                    }

                    if status.success() {
                        if let Some(t) = tasks.write().unwrap().get_mut(&task_id) {
                            t.status = DownloadStatus::Completed;
                            t.progress = 100.0;
                        }
                        let _ = app_handle.emit(
                            "download-progress",
                            DownloadProgressEvent {
                                task_id: task_id.clone(),
                                progress: 100.0,
                                speed: None,
                                eta: None,
                                status: DownloadStatus::Completed,
                                downloaded_bytes: None,
                                total_bytes: None,
                            },
                        );
                    } else {
                        if let Some(t) = tasks.write().unwrap().get_mut(&task_id) {
                            t.status = DownloadStatus::Failed;
                            t.error = Some("Download failed".to_string());
                        }
                        let _ = app_handle.emit(
                            "download-progress",
                            DownloadProgressEvent {
                                task_id: task_id.clone(),
                                progress: 0.0,
                                speed: None,
                                eta: None,
                                status: DownloadStatus::Failed,
                                downloaded_bytes: None,
                                total_bytes: None,
                            },
                        );
                    }
                }
                Err(e) => {
                    // Remove from process_ids since process has ended
                    process_ids.write().unwrap().remove(&task_id);

                    // Check if the task was already paused or cancelled
                    let current_status = tasks
                        .read()
                        .unwrap()
                        .get(&task_id)
                        .map(|t| t.status.clone());

                    if current_status == Some(DownloadStatus::Paused)
                        || current_status == Some(DownloadStatus::Cancelled)
                    {
                        // Clean up info.json on pause/cancel
                        let _ = std::fs::remove_file(&info_json_path);
                        return;
                    }

                    if let Some(t) = tasks.write().unwrap().get_mut(&task_id) {
                        t.status = DownloadStatus::Failed;
                        t.error = Some(format!("Process error: {}", e));
                    }
                }
            }

            // Clean up the .info.json file after download completes
            let _ = std::fs::remove_file(&info_json_path);
        });
    }
}
