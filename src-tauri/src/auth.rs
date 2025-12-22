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
            status.avatar_url = None;  // Clear avatar on logout
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
        
        let login_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        
        writeln!(file, "# Netscape HTTP Cookie File").map_err(|e| e.to_string())?;
        writeln!(file, "# Login Time: {}", login_time).map_err(|e| e.to_string())?;
        writeln!(file, "# Generated by YouTube Downloader").map_err(|e| e.to_string())?;
        writeln!(file).map_err(|e| e.to_string())?;
        
        for cookie in cookies_str.split(';') {
            let cookie = cookie.trim();
            if cookie.is_empty() { continue; }
            if let Some((name, value)) = cookie.split_once('=') {
                let expiration = login_time + 365 * 24 * 60 * 60;
                writeln!(file, ".youtube.com\tTRUE\t/\tTRUE\t{}\t{}\t{}", expiration, name.trim(), value.trim()).map_err(|e| e.to_string())?;
                writeln!(file, ".google.com\tTRUE\t/\tTRUE\t{}\t{}\t{}", expiration, name.trim(), value.trim()).map_err(|e| e.to_string())?;
            }
        }
        
        // Remove OAuth marker if exists
        let oauth_marker = self.app_data_dir.join("oauth_login.marker");
        if oauth_marker.exists() { let _ = fs::remove_file(oauth_marker); }

        let mut status = self.status.write().unwrap();
        status.logged_in = true;
        status.login_time = Some(login_time);
        status.cookies_valid = true;
        status.auth_method = "cookies".to_string();
        Ok(())
    }

    pub fn import_cookies_file(&self, source_path: &str) -> Result<(), String> {
        let source = PathBuf::from(source_path);
        if !source.exists() { return Err("File does not exist".to_string()); }
        let dest = self.get_cookies_path();
        fs::copy(&source, &dest).map_err(|e| format!("Failed to copy file: {}", e))?;
        
        let login_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();

        // Remove OAuth marker
        let oauth_marker = self.app_data_dir.join("oauth_login.marker");
        if oauth_marker.exists() { let _ = fs::remove_file(oauth_marker); }
        
        let mut status = self.status.write().unwrap();
        status.logged_in = true;
        status.login_time = Some(login_time);
        status.cookies_valid = true;
        status.auth_method = "cookies".to_string();
        Ok(())
    }

    pub fn logout(&self) -> Result<(), String> {
        let cookies_path = self.get_cookies_path();
        if cookies_path.exists() { let _ = fs::remove_file(cookies_path); }
        
        let oauth_marker = self.app_data_dir.join("oauth_login.marker");
        if oauth_marker.exists() { let _ = fs::remove_file(oauth_marker); }
        
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

use crate::models::BrowserProfile;

/// Get browser profiles for cookies extraction
pub fn get_browser_profiles(browser: &str) -> Vec<BrowserProfile> {
    let user_data_dir = match browser {
        "chrome" => get_chrome_user_data_dir(),
        "edge" => get_edge_user_data_dir(),
        "firefox" => return get_firefox_profiles(),
        "brave" => get_brave_user_data_dir(),
        _ => return vec![],
    };
    
    let Some(user_data_dir) = user_data_dir else {
        return vec![];
    };
    
    // Read Local State file for profile info
    let local_state_path = user_data_dir.join("Local State");
    let profiles = read_chromium_profiles(&local_state_path, &user_data_dir);
    
    profiles
}

fn get_chrome_user_data_dir() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        std::env::var("LOCALAPPDATA").ok()
            .map(|p| PathBuf::from(p).join("Google").join("Chrome").join("User Data"))
            .filter(|p| p.exists())
    }
    #[cfg(not(target_os = "windows"))]
    { None }
}

fn get_edge_user_data_dir() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        std::env::var("LOCALAPPDATA").ok()
            .map(|p| PathBuf::from(p).join("Microsoft").join("Edge").join("User Data"))
            .filter(|p| p.exists())
    }
    #[cfg(not(target_os = "windows"))]
    { None }
}

fn get_brave_user_data_dir() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        std::env::var("LOCALAPPDATA").ok()
            .map(|p| PathBuf::from(p).join("BraveSoftware").join("Brave-Browser").join("User Data"))
            .filter(|p| p.exists())
    }
    #[cfg(not(target_os = "windows"))]
    { None }
}

fn read_chromium_profiles(local_state_path: &PathBuf, user_data_dir: &PathBuf) -> Vec<BrowserProfile> {
    let mut profiles = Vec::new();
    
    // Read and parse Local State JSON
    let content = match fs::read_to_string(local_state_path) {
        Ok(c) => c,
        Err(_) => return profiles,
    };
    
    let json: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return profiles,
    };
    
    // Navigate to profile.info_cache
    if let Some(info_cache) = json.pointer("/profile/info_cache").and_then(|v| v.as_object()) {
        for (folder_name, profile_info) in info_cache {
            // Check if profile folder exists
            let profile_path = user_data_dir.join(folder_name);
            if !profile_path.exists() {
                continue;
            }
            
            let display_name = profile_info.get("name")
                .and_then(|v| v.as_str())
                .unwrap_or(folder_name)
                .to_string();
            
            let email = profile_info.get("gaia_name")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .or_else(|| {
                    profile_info.get("user_name")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                });
            
            profiles.push(BrowserProfile {
                folder_name: folder_name.clone(),
                display_name,
                email,
            });
        }
    }
    
    // If no profiles found, add Default if it exists
    if profiles.is_empty() {
        let default_path = user_data_dir.join("Default");
        if default_path.exists() {
            profiles.push(BrowserProfile {
                folder_name: "Default".to_string(),
                display_name: "Default".to_string(),
                email: None,
            });
        }
    }
    
    profiles
}

fn get_firefox_profiles() -> Vec<BrowserProfile> {
    let mut profiles = Vec::new();
    
    #[cfg(target_os = "windows")]
    {
        let profiles_dir = std::env::var("APPDATA").ok()
            .map(|p| PathBuf::from(p).join("Mozilla").join("Firefox").join("Profiles"));
        
        if let Some(profiles_dir) = profiles_dir {
            if profiles_dir.exists() {
                if let Ok(entries) = fs::read_dir(&profiles_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_dir() {
                            let folder_name = path.file_name()
                                .and_then(|s| s.to_str())
                                .unwrap_or("")
                                .to_string();
                            
                            // Firefox profiles are named like "xxxxxxxx.profile-name"
                            let display_name = folder_name.split('.').last()
                                .unwrap_or(&folder_name)
                                .to_string();
                            
                            profiles.push(BrowserProfile {
                                folder_name: folder_name.clone(),
                                display_name,
                                email: None,
                            });
                        }
                    }
                }
            }
        }
    }
    
    profiles
}

/// Get list of available browsers
pub fn get_available_browsers() -> Vec<String> {
    let mut browsers = Vec::new();
    
    if get_chrome_user_data_dir().is_some() {
        browsers.push("chrome".to_string());
    }
    if get_edge_user_data_dir().is_some() {
        browsers.push("edge".to_string());
    }
    if get_brave_user_data_dir().is_some() {
        browsers.push("brave".to_string());
    }
    
    // Firefox check
    #[cfg(target_os = "windows")]
    {
        let ff_profiles = std::env::var("APPDATA").ok()
            .map(|p| PathBuf::from(p).join("Mozilla").join("Firefox").join("Profiles"));
        if ff_profiles.map(|p| p.exists()).unwrap_or(false) {
            browsers.push("firefox".to_string());
        }
    }
    
    browsers
}

/// Prepare browser cookies by copying relevant files to a temp location
/// Returns the browser argument to use with yt-dlp (e.g., "chrome:/path/to/temp/profile")
/// This avoids the DPAPI lock issue when the browser is running
pub fn prepare_browser_cookies_path(app_data_dir: &PathBuf, browser: &str, profile: &str) -> Result<String, String> {
    let user_data_dir = match browser {
        "chrome" => get_chrome_user_data_dir(),
        "edge" => get_edge_user_data_dir(),
        "brave" => get_brave_user_data_dir(),
        "firefox" => {
            // Firefox doesn't have the DPAPI issue, use directly
            let profile_arg = if profile.is_empty() { "firefox".to_string() } else { format!("firefox:{}", profile) };
            return Ok(profile_arg);
        },
        _ => return Err(format!("Unsupported browser: {}", browser)),
    };
    
    let user_data_dir = user_data_dir.ok_or_else(|| format!("{} user data directory not found", browser))?;
    
    // Determine profile folder
    let profile_folder = if profile.is_empty() || profile == "Default" {
        "Default".to_string()
    } else {
        profile.to_string()
    };
    
    let source_profile_dir = user_data_dir.join(&profile_folder);
    if !source_profile_dir.exists() {
        return Err(format!("Profile directory not found: {:?}", source_profile_dir));
    }
    
    // Create temp directory for copied files
    let temp_dir = app_data_dir.join("temp_cookies").join(browser).join(&profile_folder);
    fs::create_dir_all(&temp_dir).map_err(|e| format!("Failed to create temp dir: {}", e))?;
    
    // Files we need to copy:
    // 1. Cookies database (in profile folder)
    // 2. Local State file (in user data dir, contains encryption key)
    
    // Copy Cookies file
    let cookies_src = source_profile_dir.join("Network").join("Cookies");
    let cookies_alt_src = source_profile_dir.join("Cookies"); // Older Chrome versions
    
    let cookies_dst_dir = temp_dir.join("Network");
    fs::create_dir_all(&cookies_dst_dir).ok();
    let cookies_dst = cookies_dst_dir.join("Cookies");
    
    if cookies_src.exists() {
        fs::copy(&cookies_src, &cookies_dst).map_err(|e| format!("Failed to copy Cookies: {}", e))?;
    } else if cookies_alt_src.exists() {
        // For older structure
        let alt_dst = temp_dir.join("Cookies");
        fs::copy(&cookies_alt_src, &alt_dst).map_err(|e| format!("Failed to copy Cookies: {}", e))?;
    } else {
        return Err("Cookies database not found in profile".to_string());
    }
    
    // Copy Local State file to temp user data root
    let temp_user_data = app_data_dir.join("temp_cookies").join(browser);
    let local_state_src = user_data_dir.join("Local State");
    let local_state_dst = temp_user_data.join("Local State");
    
    if local_state_src.exists() {
        fs::copy(&local_state_src, &local_state_dst).map_err(|e| format!("Failed to copy Local State: {}", e))?;
    }
    
    // Get the cookies database path
    let cookies_db_path = if cookies_dst.exists() {
        cookies_dst.to_string_lossy().to_string()
    } else {
        temp_dir.join("Cookies").to_string_lossy().to_string()
    };
    
    // Note: This function is now largely unused since we have cookies.rs for extraction
    // But we keep it for potential future use with yt-dlp's database path feature
    // Format: browser:profile:keyring:container:database
    Ok(format!("{}:{}:auto::{}", browser, profile_folder, cookies_db_path))
}

