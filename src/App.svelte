<script>
    import Header from "./components/Header.svelte";
    import UrlInput from "./components/UrlInput.svelte";
    import Queue from "./components/Queue.svelte";
    import Settings from "./components/Settings.svelte";
    import Notification from "./components/Notification.svelte";
    import InstallModal from "./components/InstallModal.svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { listen } from "@tauri-apps/api/event";
    import { onMount } from "svelte";

    let settings = $state(null);
    let ytdlpStatus = $state({ installed: false, update_available: false });
    let ffmpegStatus = $state({ installed: false });
    let aria2Status = $state({ installed: false });
    let loginStatus = $state({
        logged_in: false,
        cookies_valid: false,
        auth_method: "browser",
        avatar_url: null,
    });
    let tasks = $state([]);
    let showSettings = $state(false);
    let notifications = $state([]);
    let notificationActionLoading = $state(false);
    let showInstallModal = $state(false);
    let missingToolsList = $state([]);
    let isInstallingTools = $state(false);

    let defaultResolution = $derived(settings?.default_resolution || "1080p");

    onMount(async () => {
        // Load settings
        try {
            settings = await invoke("get_settings");
        } catch (e) {
            console.error("Failed to load settings:", e);
        }

        // Check tool status
        try {
            ytdlpStatus = await invoke("get_ytdlp_status");
            ffmpegStatus = await invoke("get_ffmpeg_status");
            aria2Status = await invoke("get_aria2_status");

            // Check if tools need install/update and notify user
            checkToolsAndNotify();
        } catch (e) {
            console.error("Failed to check tools:", e);
        }

        // Load login status
        try {
            loginStatus = await invoke("get_login_status");
        } catch (e) {
            console.error("Failed to get login status:", e);
        }

        // Load existing tasks
        try {
            const existingTasks = await invoke("get_all_tasks");
            tasks = existingTasks || [];
        } catch (e) {
            console.error("Failed to load tasks:", e);
        }

        // Listen for progress updates
        await listen("download-progress", (event) => {
            const { task_id, progress, speed, eta, status } = event.payload;
            tasks = tasks.map((t) =>
                t.id === task_id ? { ...t, progress, speed, eta, status } : t,
            );
        });

        // Listen for task info updates
        await listen("task-info-updated", async (event) => {
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
        });

        // Listen for login status updates (avatar saved, etc.)
        await listen("login_status_updated", (event) => {
            loginStatus = event.payload;
        });

        // Validate cookies on startup
        validateCookiesOnStartup();
    });

    async function validateCookiesOnStartup() {
        try {
            const result = await invoke("check_cookies_expiry");
            if (!result.valid && result.message) {
                showNotification(result.message, "warning");
            }
        } catch (e) {
            console.error("Failed to validate cookies:", e);
        }
    }

    function checkToolsAndNotify() {
        const missing = [];

        if (!ytdlpStatus.installed) missing.push("yt-dlp");
        if (!ffmpegStatus.installed) missing.push("FFmpeg");

        if (missing.length > 0) {
            missingToolsList = missing;
            showInstallModal = true;
        } else if (ytdlpStatus.update_available) {
            // Only show update notification if tools are installed
            showNotification(
                "yt-dlp update available",
                "info",
                "Update Now",
                handleUpdateYtdlp,
            );
        }
    }

    async function handleUpdateYtdlp() {
        notificationActionLoading = true;
        try {
            await invoke("download_ytdlp");
            ytdlpStatus = await invoke("get_ytdlp_status");
            closeNotification();
            showNotification("yt-dlp updated successfully!", "success");
        } catch (e) {
            showNotification(`Update failed: ${e}`, "error");
        }
        notificationActionLoading = false;
    }

    async function handleInstallComplete(success) {
        showInstallModal = false;
        isInstallingTools = false;
        if (success) {
            // Refresh tool status
            ytdlpStatus = await invoke("get_ytdlp_status");
            ffmpegStatus = await invoke("get_ffmpeg_status");
        }
    }

    async function handleInstallTools() {
        notificationActionLoading = true;
        let success = true;
        let errorMsg = "";

        try {
            if (!ytdlpStatus.installed || ytdlpStatus.update_available) {
                await invoke("download_ytdlp");
                ytdlpStatus = await invoke("check_ytdlp_status");
            }
            if (!ffmpegStatus.installed) {
                await invoke("download_ffmpeg");
                ffmpegStatus = await invoke("check_ffmpeg_status");
            }
        } catch (e) {
            success = false;
            errorMsg = String(e);
        }

        notificationActionLoading = false;
        closeNotification();

        if (success) {
            showNotification("Tools installed successfully!", "success");
        } else {
            showNotification(`Installation failed: ${errorMsg}`, "error");
        }
    }

    async function handleDownload(urls, resolution) {
        // Validate cookies and cleanup if invalid
        try {
            const result = await invoke("validate_and_cleanup_cookies");

            if (result.status === "valid") {
                showNotification(
                    "🍪 Using logged-in cookies for download",
                    "success",
                );
            } else if (result.deleted) {
                loginStatus = {
                    ...loginStatus,
                    logged_in: false,
                    cookies_valid: false,
                };
                showNotification(`⚠️ ${result.message}`, "warning");
            } else if (result.status === "missing") {
                showNotification("ℹ️ Downloading in anonymous mode", "info");
            }
        } catch (e) {
            console.log("Cookies validation failed:", e);
        }

        let addedCount = 0;

        for (const url of urls) {
            const trimmedUrl = url.trim();
            if (!trimmedUrl) continue;

            try {
                // Check if this is a playlist-only URL
                if (
                    trimmedUrl.includes("/playlist?list=") &&
                    !trimmedUrl.includes("watch?v=")
                ) {
                    // Expand playlist
                    showNotification("🔍 Expanding playlist...", "info");

                    try {
                        const videoUrls = await invoke("expand_playlist", {
                            url: trimmedUrl,
                        });
                        showNotification(
                            `📋 Found ${videoUrls.length} videos in playlist`,
                            "success",
                        );

                        for (const videoUrl of videoUrls) {
                            const task = await invoke("create_download_task", {
                                url: videoUrl,
                                resolution,
                            });
                            tasks = [...tasks, task];
                            invoke("start_download", { taskId: task.id }).catch(
                                console.error,
                            );
                            addedCount++;
                        }
                    } catch (e) {
                        console.error("Failed to expand playlist:", e);
                        showNotification(
                            `❌ Playlist expansion failed: ${e}`,
                            "error",
                        );
                    }
                } else {
                    // Single video
                    const task = await invoke("create_download_task", {
                        url: trimmedUrl,
                        resolution,
                    });
                    tasks = [...tasks, task];
                    invoke("start_download", { taskId: task.id }).catch(
                        console.error,
                    );
                    addedCount++;
                }
            } catch (e) {
                console.error("Failed to start download:", e);
                showNotification(`Download failed: ${e}`, "error");
            }
        }

        // Show download statistics
        if (addedCount > 1) {
            showNotification(
                `✅ Added ${addedCount} download tasks`,
                "success",
            );
        }
    }

    async function handlePause(taskId) {
        try {
            await invoke("pause_download", { taskId });
        } catch (e) {
            console.error("Failed to pause:", e);
        }
    }

    async function handleResume(taskId) {
        try {
            await invoke("resume_download", { taskId });
        } catch (e) {
            console.error("Failed to resume:", e);
        }
    }

    async function handleCancel(taskId) {
        try {
            await invoke("cancel_download", { taskId });
            tasks = tasks.filter((t) => t.id !== taskId);
        } catch (e) {
            console.error("Failed to cancel:", e);
        }
    }

    async function handleRetry(taskId) {
        try {
            await invoke("retry_download", { taskId });
        } catch (e) {
            console.error("Failed to retry:", e);
        }
    }

    async function handleRemove(taskId) {
        tasks = tasks.filter((t) => t.id !== taskId);
        try {
            await invoke("remove_task", { taskId });
        } catch (e) {
            console.error("Failed to remove task:", e);
        }
    }

    async function handleClearCompleted() {
        const completedIds = tasks
            .filter((t) => t.status === "completed")
            .map((t) => t.id);
        tasks = tasks.filter((t) => t.status !== "completed");
        for (const id of completedIds) {
            try {
                await invoke("remove_task", { taskId: id });
            } catch (e) {
                console.error("Failed to remove task:", e);
            }
        }
    }

    async function handleLogin() {
        try {
            // Always open login window - user controls login/logout from there
            await invoke("open_login_window");
        } catch (e) {
            console.error("Failed to open login:", e);
        }
    }

    async function handleLogout() {
        try {
            await invoke("logout");
            loginStatus = {
                logged_in: false,
                cookies_valid: false,
                auth_method: "browser",
                avatar_url: null,
            };
        } catch (e) {
            console.error("Failed to logout:", e);
        }
    }

    function toggleSettings() {
        showSettings = !showSettings;
    }

    let notificationId = 0;

    function showNotification(
        message,
        type = "info",
        actionLabel = null,
        onAction = null,
    ) {
        const id = ++notificationId;
        notifications = [
            ...notifications,
            { id, message, type, actionLabel, onAction },
        ];
    }

    function closeNotification(id) {
        notifications = notifications.filter((n) => n.id !== id);
    }

    async function handleSettingsChange(newSettings) {
        settings = newSettings;
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
        onResolutionChange={async (res) => {
            settings = { ...settings, default_resolution: res };
            await invoke("save_settings", { settings });
        }}
        ytdlpInstalled={ytdlpStatus.installed}
        ffmpegInstalled={ffmpegStatus.installed}
    />

    <Queue
        {tasks}
        onPause={handlePause}
        onResume={handleResume}
        onCancel={handleCancel}
        onRetry={handleRetry}
        onRemove={handleRemove}
        onClearCompleted={handleClearCompleted}
    />
</main>

{#if showSettings}
    <Settings
        {settings}
        {ytdlpStatus}
        {ffmpegStatus}
        {aria2Status}
        {loginStatus}
        {isInstallingTools}
        onSettingsChange={handleSettingsChange}
        onToolStatusChange={(tool, status) => {
            if (tool === "ytdlp") ytdlpStatus = status;
            else if (tool === "ffmpeg") ffmpegStatus = status;
            else if (tool === "aria2") aria2Status = status;
        }}
        onClose={() => (showSettings = false)}
    />
{/if}

<!-- Notification stack container -->
<div class="notification-stack">
    {#each notifications as notif (notif.id)}
        <Notification
            message={notif.message}
            type={notif.type}
            onClose={() => closeNotification(notif.id)}
            actionLabel={notif.actionLabel}
            onAction={notif.onAction}
            actionLoading={notificationActionLoading}
        />
    {/each}
</div>

{#if showInstallModal}
    <InstallModal
        missingTools={missingToolsList}
        onComplete={handleInstallComplete}
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

    .notification-stack {
        position: fixed;
        bottom: 20px;
        right: 20px;
        display: flex;
        flex-direction: column-reverse;
        gap: 10px;
        z-index: 1000;
        pointer-events: none;
    }

    .notification-stack :global(.notification) {
        pointer-events: auto;
    }
</style>
