mod aria2;
mod auth;
mod cookies;
mod download;
mod ffmpeg;
mod models;
mod settings;
mod ytdlp;

use aria2::{Aria2Manager, Aria2Status};
use auth::AuthManager;
use cookies::convert_cookies_to_netscape;
use download::DownloadManager;
use ffmpeg::{FFmpegManager, FFmpegStatus};
use models::{AppSettings, DownloadTask, LoginStatus, YtDlpStatus};
use settings::SettingsManager;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Manager, State, Url, WebviewWindow};

pub struct AppState {
    pub settings: SettingsManager,
    pub ytdlp: Arc<YtDlpManager>,
    pub ffmpeg: Arc<FFmpegManager>,
    pub aria2: Arc<Aria2Manager>,
    pub download: DownloadManager,
    pub auth: Arc<AuthManager>,
}

// ==================== Settings Commands ====================

#[tauri::command]
fn get_settings(state: State<AppState>) -> AppSettings {
    state.settings.get()
}

#[tauri::command]
fn save_settings(state: State<AppState>, settings: AppSettings) -> Result<(), String> {
    state.settings.save(settings)
}

// ==================== yt-dlp Commands ====================

#[tauri::command]
fn get_ytdlp_status(state: State<AppState>) -> YtDlpStatus {
    state.ytdlp.get_status()
}

#[tauri::command]
async fn check_ytdlp_update(state: State<'_, AppState>) -> Result<YtDlpStatus, String> {
    state.ytdlp.check_for_updates().await
}

#[tauri::command]
async fn download_ytdlp(app_handle: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    state.ytdlp.download_ytdlp(&app_handle).await
}

// ==================== FFmpeg Commands ====================

#[tauri::command]
fn get_ffmpeg_status(state: State<AppState>) -> FFmpegStatus {
    state.ffmpeg.get_status()
}

#[tauri::command]
async fn download_ffmpeg(app_handle: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    state.ffmpeg.download_ffmpeg(&app_handle).await
}

// ==================== Aria2 Commands ====================

#[tauri::command]
fn get_aria2_status(state: State<AppState>) -> Aria2Status {
    state.aria2.get_status()
}

#[tauri::command]
async fn download_aria2(app_handle: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    state.aria2.download_aria2(&app_handle).await
}

// ==================== Download Commands ====================

#[tauri::command]
fn create_download_task(state: State<AppState>, url: String, resolution: String) -> DownloadTask {
    state.download.create_task(url, resolution)
}

#[tauri::command]
fn start_download(
    app_handle: AppHandle,
    state: State<AppState>,
    task_id: String,
) -> Result<(), String> {
    let settings = state.settings.get();
    let cookies_path = cookies::get_cookies_file_path(&state.settings.get_app_data_dir());

    state
        .download
        .start_download(task_id, settings.download_dir, app_handle, cookies_path);
    Ok(())
}

#[tauri::command]
fn get_download_task(state: State<AppState>, task_id: String) -> Option<DownloadTask> {
    state.download.get_task(&task_id)
}

#[tauri::command]
fn get_all_tasks(state: State<AppState>) -> Vec<DownloadTask> {
    state.download.get_all_tasks()
}

#[tauri::command]
fn remove_task(state: State<AppState>, task_id: String) {
    state.download.remove_task(&task_id);
}

#[tauri::command]
fn clear_completed_tasks(state: State<AppState>) {
    state.download.clear_completed();
}

#[tauri::command]
fn expand_playlist(state: State<AppState>, url: String) -> Result<Vec<String>, String> {
    let cookies_path = cookies::get_cookies_file_path(&state.settings.get_app_data_dir());
    state.download.expand_playlist(&url, &cookies_path)
}

#[tauri::command]
fn pause_download(state: State<AppState>, task_id: String) {
    state.download.pause_download(&task_id);
}

#[tauri::command]
fn resume_download(
    app_handle: AppHandle,
    state: State<AppState>,
    task_id: String,
) -> Result<(), String> {
    let settings = state.settings.get();
    let cookies_path = cookies::get_cookies_file_path(&state.settings.get_app_data_dir());

    // Resume by re-starting the download (yt-dlp will continue from .part file)
    state
        .download
        .start_download(task_id, settings.download_dir, app_handle, cookies_path);
    Ok(())
}

#[tauri::command]
fn open_download_folder(state: State<AppState>) -> Result<(), String> {
    let settings = state.settings.get();
    let path = std::path::Path::new(&settings.download_dir);
    if path.exists() {
        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("explorer")
                .arg(path)
                .spawn()
                .map_err(|e| e.to_string())?;
        }
        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("open")
                .arg(path)
                .spawn()
                .map_err(|e| e.to_string())?;
        }
        #[cfg(target_os = "linux")]
        {
            std::process::Command::new("xdg-open")
                .arg(path)
                .spawn()
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    } else {
        Err("Download folder does not exist".to_string())
    }
}

// ==================== Auth Commands ====================

#[tauri::command]
fn get_login_status(state: State<AppState>) -> LoginStatus {
    state.auth.get_status()
}

#[tauri::command]
async fn open_login_window(
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state.auth.open_login_window(&app_handle)
}

#[tauri::command]
async fn export_cookies(
    window: WebviewWindow,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let youtube_url =
        Url::parse("https://www.youtube.com").map_err(|e| format!("Failed to parse URL: {}", e))?;

    let cookies = window
        .cookies_for_url(youtube_url)
        .map_err(|e| format!("Failed to get cookies: {}", e))?;

    if cookies.is_empty() {
        return Err("No cookies found. Please login first.".to_string());
    }

    let netscape_content = convert_cookies_to_netscape(&cookies);
    let cookies_path = cookies::get_cookies_file_path(&state.settings.get_app_data_dir());

    cookies::save_cookies_to_file(&netscape_content, &cookies_path)
        .map_err(|e| format!("Failed to save cookies: {}", e))?;

    // Update auth status
    state.auth.set_logged_in(true);

    Ok(format!("Exported {} cookies", cookies.len()))
}

#[tauri::command]
fn import_cookies_file(state: State<AppState>, file_path: String) -> Result<(), String> {
    let source = PathBuf::from(&file_path);
    let dest = cookies::get_cookies_file_path(&state.settings.get_app_data_dir());

    cookies::import_cookies_file(&source, &dest)?;
    state.auth.set_logged_in(true);

    Ok(())
}

#[tauri::command]
fn logout(state: State<AppState>) -> Result<(), String> {
    let cookies_path = cookies::get_cookies_file_path(&state.settings.get_app_data_dir());
    if cookies_path.exists() {
        std::fs::remove_file(&cookies_path)
            .map_err(|e| format!("Failed to remove cookies: {}", e))?;
    }
    state.auth.set_logged_in(false);
    Ok(())
}

#[tauri::command]
fn save_avatar(state: State<AppState>, avatar_url: Option<String>) {
    state.auth.set_avatar(avatar_url);
}

#[tauri::command]
fn check_cookies_exist(state: State<AppState>) -> bool {
    let cookies_path = cookies::get_cookies_file_path(&state.settings.get_app_data_dir());
    cookies_path.exists()
}

/// Check if cookies are valid by testing with yt-dlp
/// This makes an actual network request to verify cookies still work
#[tauri::command]
async fn validate_cookies_async(state: State<'_, AppState>) -> Result<bool, String> {
    let cookies_path = cookies::get_cookies_file_path(&state.settings.get_app_data_dir());

    if !cookies_path.exists() {
        return Ok(false);
    }

    let ytdlp_path = state.ytdlp.get_exe_path();
    if !ytdlp_path.exists() {
        return Err("yt-dlp not installed".to_string());
    }

    // Test with a simple YouTube URL (just get metadata, don't download)
    let test_url = "https://www.youtube.com/watch?v=jNQXAC9IVRw"; // "Me at the zoo" - first YouTube video

    use std::process::Command;
    use std::process::Stdio;

    #[cfg(target_os = "windows")]
    use std::os::windows::process::CommandExt;
    #[cfg(target_os = "windows")]
    const CREATE_NO_WINDOW: u32 = 0x08000000;

    let mut cmd = Command::new(&ytdlp_path);
    cmd.args(&[
        "--cookies",
        &cookies_path.to_string_lossy(),
        "--skip-download",
        "--no-warnings",
        "--quiet",
        test_url,
    ])
    .stdout(Stdio::null())
    .stderr(Stdio::null());

    #[cfg(target_os = "windows")]
    cmd.creation_flags(CREATE_NO_WINDOW);

    // Run in a separate thread to not block
    let result = tokio::task::spawn_blocking(move || cmd.status()).await;

    match result {
        Ok(Ok(status)) => Ok(status.success()),
        _ => Ok(false),
    }
}

/// Check if cookies file exists and is not expired
/// Returns: "valid", "expired", or "not_found"
#[tauri::command]
fn check_cookies_valid(state: State<AppState>) -> String {
    let cookies_path = cookies::get_cookies_file_path(&state.settings.get_app_data_dir());

    if !cookies_path.exists() {
        return "not_found".to_string();
    }

    match cookies::check_cookies_expiry(&cookies_path) {
        Ok(true) => "valid".to_string(),
        Ok(false) => "expired".to_string(),
        Err(_) => "error".to_string(),
    }
}

// ==================== Entry Point ====================

use ytdlp::YtDlpManager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to get app data dir");
            std::fs::create_dir_all(&app_data_dir).expect("Failed to create app data dir");

            let settings = SettingsManager::new(app_data_dir.clone());
            let ytdlp = Arc::new(YtDlpManager::new(app_data_dir.clone()));
            let ffmpeg = Arc::new(FFmpegManager::new(app_data_dir.clone()));
            let aria2 = Arc::new(Aria2Manager::new(app_data_dir.clone()));
            let auth = Arc::new(AuthManager::new(app_data_dir.clone()));
            let default_concurrent = settings.get().default_concurrent;
            let download = DownloadManager::new(
                ytdlp.clone(),
                ffmpeg.clone(),
                aria2.clone(),
                default_concurrent,
                app_data_dir,
            );

            app.manage(AppState {
                settings,
                ytdlp,
                ffmpeg,
                aria2,
                download,
                auth,
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Settings
            get_settings,
            save_settings,
            // yt-dlp
            get_ytdlp_status,
            check_ytdlp_update,
            download_ytdlp,
            // FFmpeg
            get_ffmpeg_status,
            download_ffmpeg,
            // Aria2
            get_aria2_status,
            download_aria2,
            // Downloads
            create_download_task,
            start_download,
            get_download_task,
            get_all_tasks,
            remove_task,
            clear_completed_tasks,
            expand_playlist,
            pause_download,
            resume_download,
            open_download_folder,
            // Auth
            get_login_status,
            open_login_window,
            export_cookies,
            import_cookies_file,
            logout,
            save_avatar,
            check_cookies_exist,
            validate_cookies_async,
            check_cookies_valid,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
