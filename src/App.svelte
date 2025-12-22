<script>
    import Header from "./components/Header.svelte";
    import UrlInput from "./components/UrlInput.svelte";
    import Queue from "./components/Queue.svelte";
    import Settings from "./components/Settings.svelte";
    import Notification from "./components/Notification.svelte";
    import { listen } from "@tauri-apps/api/event";
    import { invoke } from "@tauri-apps/api/core";
    import { onMount } from "svelte";

    let tasks = $state([]);
    let settings = $state(null);
    let loginStatus = $state({ logged_in: false });
    let ytdlpStatus = $state({ installed: false });
    let ffmpegStatus = $state({ installed: false });
    let aria2Status = $state({ installed: false });
    let showSettings = $state(false);
    let defaultResolution = $state("1080p");
    let notification = $state(null);

    onMount(async () => {
        // Load initial data
        try {
            settings = await invoke("get_settings");
            loginStatus = await invoke("get_login_status");
            ytdlpStatus = await invoke("get_ytdlp_status");
            ffmpegStatus = await invoke("get_ffmpeg_status");
            aria2Status = await invoke("get_aria2_status");
            tasks = await invoke("get_all_tasks");
            defaultResolution = settings?.default_resolution || "1080p";
        } catch (e) {
            console.error("Failed to load initial data:", e);
        }

        // Validate cookies in background (async, don't block UI)
        validateCookiesOnStartup();

        // Listen for download progress events
        const unlistenProgress = await listen("download-progress", (event) => {
            const { task_id, progress, speed, eta, status } = event.payload;
            tasks = tasks.map((t) =>
                t.id === task_id ? { ...t, progress, speed, eta, status } : t,
            );
        });

        // Listen for task info updated (when video metadata is fetched)
        const unlistenTaskInfo = await listen(
            "task-info-updated",
            async (event) => {
                const taskId = event.payload;
                try {
                    const updatedTask = await invoke("get_download_task", {
                        taskId,
                    });
                    if (updatedTask) {
                        tasks = tasks.map((t) =>
                            t.id === taskId ? updatedTask : t,
                        );
                    }
                } catch (e) {
                    console.error("Failed to get updated task:", e);
                }
            },
        );

        // Listen for download status changed
        const unlistenStatusChanged = await listen(
            "download-status-changed",
            (event) => {
                const { task_id, status } = event.payload;
                tasks = tasks.map((t) =>
                    t.id === task_id ? { ...t, status } : t,
                );
            },
        );

        // Listen for login window closed - export cookies and extract avatar
        const unlistenLoginClosed = await listen(
            "login-window-closed",
            async () => {
                try {
                    // Export cookies from the login window
                    await invoke("export_cookies");

                    // Get updated login status
                    loginStatus = await invoke("get_login_status");
                } catch (e) {
                    console.error(
                        "Failed to export cookies on login close:",
                        e,
                    );
                    // Still try to get login status even if export fails
                    loginStatus = await invoke("get_login_status");
                }
            },
        );

        return () => {
            unlistenProgress();
            unlistenTaskInfo();
            unlistenStatusChanged();
            unlistenLoginClosed();
        };
    });

    // Validate cookies on startup
    async function validateCookiesOnStartup() {
        try {
            if (!ytdlpStatus.installed) {
                return; // Can't validate without yt-dlp
            }

            const isValid = await invoke("validate_cookies_async");
            if (!isValid) {
                // Cookies are invalid, delete and auto-open login
                await invoke("logout"); // This deletes the cookies file
                loginStatus = { logged_in: false };

                // Auto-open login window
                try {
                    await invoke("open_login_window");
                } catch (e) {
                    console.error("Failed to auto-open login:", e);
                }
            }
        } catch (e) {
            console.error("Failed to validate cookies:", e);
        }
    }

    // Check if URL is a playlist-only URL (no video ID)
    function isPlaylistOnlyUrl(url) {
        const hasVideoId = url.includes("v=") || url.includes("youtu.be/");
        const hasPlaylist = url.includes("list=") || url.includes("/playlist");
        return hasPlaylist && !hasVideoId;
    }

    async function handleDownload(urls, resolution) {
        for (const url of urls) {
            if (!url.trim()) continue;

            try {
                // Check if this is a playlist-only URL
                if (isPlaylistOnlyUrl(url.trim())) {
                    // Expand playlist and create individual tasks
                    try {
                        notification = {
                            message: "🔍 正在获取播放列表...",
                            type: "info",
                        };

                        const videoUrls = await invoke("expand_playlist", {
                            url: url.trim(),
                        });

                        notification = {
                            message: `📋 发现 ${videoUrls.length} 个视频，开始下载...`,
                            type: "success",
                        };

                        // Create tasks for each video
                        for (const videoUrl of videoUrls) {
                            const task = await invoke("create_download_task", {
                                url: videoUrl,
                                resolution,
                            });
                            tasks = [...tasks, task];
                            await invoke("start_download", { taskId: task.id });
                        }
                    } catch (e) {
                        console.error("Failed to expand playlist:", e);
                        notification = {
                            message: "❌ 播放列表获取失败",
                            type: "warning",
                        };
                    }
                } else {
                    // Single video URL
                    const task = await invoke("create_download_task", {
                        url: url.trim(),
                        resolution,
                    });
                    tasks = [...tasks, task];
                    await invoke("start_download", { taskId: task.id });

                    // Check cookies validity (expiry check, fast)
                    const cookiesStatus = await invoke("check_cookies_valid");

                    if (cookiesStatus === "expired") {
                        // Cookies expired, delete file
                        await invoke("logout");
                        loginStatus = { logged_in: false };

                        notification = {
                            message:
                                "⚠️ Cookies 已过期，已自动删除。请重新登录以下载受限视频。",
                            type: "warning",
                        };
                    } else if (cookiesStatus === "valid") {
                        notification = {
                            message: "🍪 使用 Cookies 下载（已登录）",
                            type: "success",
                        };
                    } else {
                        notification = {
                            message:
                                "⚠️ 匿名下载（未登录，部分视频可能无法下载）",
                            type: "warning",
                        };
                    }
                }
            } catch (e) {
                console.error("Failed to create task:", e);
            }
        }
    }

    async function handleRemoveTask(taskId) {
        await invoke("remove_task", { taskId });
        tasks = tasks.filter((t) => t.id !== taskId);
    }

    async function handleClearCompleted() {
        await invoke("clear_completed_tasks");
        tasks = tasks.filter(
            (t) => t.status !== "completed" && t.status !== "failed",
        );
    }

    async function handleLogin() {
        try {
            await invoke("open_login_window");
        } catch (e) {
            console.error("Failed to open login:", e);
        }
    }

    async function handleLogout() {
        try {
            await invoke("logout");
            loginStatus = { logged_in: false };
        } catch (e) {
            console.error("Failed to logout:", e);
        }
    }

    function toggleSettings() {
        showSettings = !showSettings;
    }

    function closeNotification() {
        notification = null;
    }
</script>

<Header
    {loginStatus}
    onLogin={handleLogin}
    onLogout={handleLogout}
    onToggleSettings={toggleSettings}
/>

<main>
    <UrlInput
        {defaultResolution}
        onDownload={handleDownload}
        disabled={!ytdlpStatus.installed}
    />

    <Queue
        {tasks}
        onRemove={handleRemoveTask}
        onClearCompleted={handleClearCompleted}
    />
</main>

{#if showSettings}
    <Settings
        bind:settings
        {ytdlpStatus}
        {ffmpegStatus}
        {aria2Status}
        onClose={() => (showSettings = false)}
    />
{/if}

{#if notification}
    <Notification
        message={notification.message}
        type={notification.type}
        onClose={closeNotification}
    />
{/if}

<style>
    main {
        flex: 1;
        display: flex;
        flex-direction: column;
        overflow: hidden;
        padding: 12px;
        gap: 12px;
    }
</style>
