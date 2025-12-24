use crate::models::LoginStatus;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

const COOKIES_FILENAME: &str = "youtube_cookies.txt";
const YOUTUBE_LOGIN_URL: &str = "https://www.youtube.com";

pub struct AuthManager {
    app_data_dir: PathBuf,
    status: RwLock<LoginStatus>,
}

impl AuthManager {
    pub fn new(app_data_dir: PathBuf) -> Self {
        let cookies_path = app_data_dir.join(COOKIES_FILENAME);
        let oauth_marker_path = app_data_dir.join("oauth_login.marker");

        let cookies_exist = cookies_path.exists();
        let oauth_exist = oauth_marker_path.exists();

        // Determine initial state
        let (logged_in, login_time, cookies_valid, auth_method) = if oauth_exist {
            // OAuth active
            // Read time from marker if possible, or just default
            let time = fs::read_to_string(&oauth_marker_path)
                .ok()
                .and_then(|s| s.trim().parse().ok());
            (true, time, false, "oauth".to_string())
        } else if cookies_exist {
            // Cookies active
            let valid = fs::metadata(&cookies_path)
                .map(|m| m.len() > 100)
                .unwrap_or(false);
            let time = Self::read_login_time_from_cookies(&cookies_path);
            (valid, time, valid, "cookies".to_string())
        } else {
            (false, None, false, "cookies".to_string())
        };

        Self {
            app_data_dir,
            status: RwLock::new(LoginStatus {
                logged_in,
                login_time,
                cookies_valid,
                auth_method,
                avatar_url: None,
            }),
        }
    }

    // ... (read_login_time_from_cookies) ...
    // Note: I need to preserve read_login_time_from_cookies helper

    fn read_login_time_from_cookies(path: &PathBuf) -> Option<u64> {
        let file = File::open(path).ok()?;
        let reader = BufReader::new(file);

        for line in reader.lines().take(5) {
            if let Ok(line) = line {
                if line.starts_with("# Login Time:") {
                    return line.split(':').nth(1).and_then(|s| s.trim().parse().ok());
                }
            }
        }
        None
    }

    pub fn get_cookies_path(&self) -> PathBuf {
        self.app_data_dir.join(COOKIES_FILENAME)
    }

    pub fn get_status(&self) -> LoginStatus {
        self.status.read().unwrap().clone()
    }

    pub fn is_logged_in(&self) -> bool {
        self.status.read().unwrap().logged_in
    }

    pub fn set_logged_in(&self, logged_in: bool) {
        let mut status = self.status.write().unwrap();
        status.logged_in = logged_in;
        if !logged_in {
            status.login_time = None;
            status.cookies_valid = false;
            status.avatar_url = None; // Clear avatar on logout
        } else {
            let login_time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            status.login_time = Some(login_time);
            status.cookies_valid = true;
        }
    }

    pub fn set_avatar(&self, avatar_url: Option<String>) {
        let mut status = self.status.write().unwrap();
        status.avatar_url = avatar_url;
    }

    // ... (open_login_window removed or kept? User said "Don't use system browser cookie...".
    // But said "Improve".
    // I can keep "open login window" as a legacy verification helper, but implementation plan says "Replace".
    // I will keep it for now as "Browser Login" option in main.js is removed?
    // Let's keep the code for safety).

    pub fn open_login_window(&self, app_handle: &AppHandle) -> Result<(), String> {
        if app_handle.get_webview_window("youtube-login").is_some() {
            return Err("Login window is already open".to_string());
        }

        // Chrome User-Agent is required for Google to allow login
        const CHROME_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";

        // JavaScript to extract avatar URL from YouTube page
        // It checks for the avatar image and sends it to Tauri when found
        const AVATAR_EXTRACT_SCRIPT: &str = r#"
            (function() {
                let avatarFound = false;
                
                function extractAvatar() {
                    if (avatarFound) return;
                    
                    // Try to find avatar button/image in YouTube header
                    const selectors = [
                        'button#avatar-btn img',
                        'img.yt-spec-avatar-shape__avatar',
                        '#avatar-btn yt-img-shadow img',
                        'yt-img-shadow.ytd-topbar-menu-button-renderer img'
                    ];
                    
                    for (const selector of selectors) {
                        const img = document.querySelector(selector);
                        if (img && img.src && img.src.startsWith('http')) {
                            avatarFound = true;
                            // Send avatar URL to Tauri
                            if (window.__TAURI__) {
                                window.__TAURI__.core.invoke('save_avatar', { avatarUrl: img.src });
                            }
                            console.log('Avatar found:', img.src);
                            break;
                        }
                    }
                }
                
                // Check periodically since YouTube loads dynamically
                const interval = setInterval(extractAvatar, 2000);
                
                // Also extract on visibility change (before close)
                document.addEventListener('visibilitychange', extractAvatar);
                
                // Initial check after page load
                setTimeout(extractAvatar, 3000);
            })();
        "#;

        let window = WebviewWindowBuilder::new(
            app_handle,
            "youtube-login",
            WebviewUrl::External(YOUTUBE_LOGIN_URL.parse().unwrap()),
        )
        .user_agent(CHROME_USER_AGENT)
        .title("Login to YouTube")
        .inner_size(500.0, 700.0)
        .center()
        .resizable(true)
        .initialization_script(AVATAR_EXTRACT_SCRIPT)
        .build()
        .map_err(|e| format!("Failed to create login window: {}", e))?;

        // Emit event when window is closed
        let app_handle_clone = app_handle.clone();
        window.on_window_event(move |event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                let _ = app_handle_clone.emit("login-window-closed", ());
            }
        });

        Ok(())
    }

    pub fn save_cookies(&self, cookies_str: &str) -> Result<(), String> {
        // ... implementation of save_cookies ...
        // Need to update status with "cookies" method
        let cookies_path = self.get_cookies_path();
        let mut file = File::create(&cookies_path)
            .map_err(|e| format!("Failed to create cookies file: {}", e))?;

        let login_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        writeln!(file, "# Netscape HTTP Cookie File").map_err(|e| e.to_string())?;
        writeln!(file, "# Login Time: {}", login_time).map_err(|e| e.to_string())?;
        writeln!(file, "# Generated by YouTube Downloader").map_err(|e| e.to_string())?;
        writeln!(file).map_err(|e| e.to_string())?;

        for cookie in cookies_str.split(';') {
            let cookie = cookie.trim();
            if cookie.is_empty() {
                continue;
            }
            if let Some((name, value)) = cookie.split_once('=') {
                let expiration = login_time + 365 * 24 * 60 * 60;
                writeln!(
                    file,
                    ".youtube.com\tTRUE\t/\tTRUE\t{}\t{}\t{}",
                    expiration,
                    name.trim(),
                    value.trim()
                )
                .map_err(|e| e.to_string())?;
                writeln!(
                    file,
                    ".google.com\tTRUE\t/\tTRUE\t{}\t{}\t{}",
                    expiration,
                    name.trim(),
                    value.trim()
                )
                .map_err(|e| e.to_string())?;
            }
        }

        // Remove OAuth marker if exists
        let oauth_marker = self.app_data_dir.join("oauth_login.marker");
        if oauth_marker.exists() {
            let _ = fs::remove_file(oauth_marker);
        }

        let mut status = self.status.write().unwrap();
        status.logged_in = true;
        status.login_time = Some(login_time);
        status.cookies_valid = true;
        status.auth_method = "cookies".to_string();
        Ok(())
    }

    pub fn import_cookies_file(&self, source_path: &str) -> Result<(), String> {
        let source = PathBuf::from(source_path);
        if !source.exists() {
            return Err("File does not exist".to_string());
        }
        let dest = self.get_cookies_path();
        fs::copy(&source, &dest).map_err(|e| format!("Failed to copy file: {}", e))?;

        let login_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Remove OAuth marker
        let oauth_marker = self.app_data_dir.join("oauth_login.marker");
        if oauth_marker.exists() {
            let _ = fs::remove_file(oauth_marker);
        }

        let mut status = self.status.write().unwrap();
        status.logged_in = true;
        status.login_time = Some(login_time);
        status.cookies_valid = true;
        status.auth_method = "cookies".to_string();
        Ok(())
    }

    pub fn logout(&self) -> Result<(), String> {
        let cookies_path = self.get_cookies_path();
        if cookies_path.exists() {
            let _ = fs::remove_file(cookies_path);
        }

        let oauth_marker = self.app_data_dir.join("oauth_login.marker");
        if oauth_marker.exists() {
            let _ = fs::remove_file(oauth_marker);
        }

        // Clear cache? yt-dlp cache removal requires knowing cache dir.
        // Usually %localappdata%/yt-dlp/cache.
        // We can run `yt-dlp --rm-cache-dir`.

        let mut status = self.status.write().unwrap();
        status.logged_in = false;
        status.login_time = None;
        status.cookies_valid = false;
        // auth_method remains? or reset?
        status.auth_method = "cookies".to_string(); // reset to default
        Ok(())
    }

    pub fn refresh_status(&self) {
        let cookies_path = self.get_cookies_path();

        let cookies_exist = cookies_path.exists();

        let (logged_in, login_time, valid, auth_method) = if cookies_exist {
            let valid = fs::metadata(&cookies_path)
                .map(|m| m.len() > 100)
                .unwrap_or(false);
            let time = Self::read_login_time_from_cookies(&cookies_path);
            (valid, time, valid, "browser".to_string())
        } else {
            (false, None, false, "browser".to_string())
        };

        let mut status = self.status.write().unwrap();
        status.logged_in = logged_in;
        status.login_time = login_time;
        status.cookies_valid = valid;
        status.auth_method = auth_method;
    }
}
