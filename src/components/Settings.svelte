<script>
    import { invoke } from "@tauri-apps/api/core";
    import { open, message, ask } from "@tauri-apps/plugin-dialog";

    let {
        settings,
        ytdlpStatus,
        ffmpegStatus,
        aria2Status,
        loginStatus,
        isInstallingTools = false,
        onSettingsChange,
        onToolStatusChange,
        onClose,
    } = $props();

    // Individual loading states for each tool
    let ytdlpLoading = $state(false);
    let ffmpegLoading = $state(false);
    let aria2Loading = $state(false);

    async function selectDownloadDir() {
        try {
            const selected = await open({
                directory: true,
                multiple: false,
                title: "Select Download Directory",
            });
            if (selected) {
                settings = { ...settings, download_dir: selected };
                await saveSettings();
            }
        } catch (e) {
            console.error("Failed to select directory:", e);
        }
    }

    async function saveSettings() {
        try {
            await invoke("save_settings", { settings });
            onSettingsChange(settings);
        } catch (e) {
            console.error("Failed to save settings:", e);
        }
    }

    async function downloadYtdlp() {
        ytdlpLoading = true;
        try {
            await invoke("download_ytdlp");
            const newStatus = await invoke("get_ytdlp_status");
            if (onToolStatusChange) onToolStatusChange("ytdlp", newStatus);
            await message("yt-dlp updated successfully!", {
                title: "Success",
                kind: "info",
            });
        } catch (e) {
            console.error("Failed to download yt-dlp:", e);
            await message(String(e), { title: "Update Failed", kind: "error" });
        }
        ytdlpLoading = false;
    }

    async function downloadFfmpeg() {
        ffmpegLoading = true;
        try {
            await invoke("download_ffmpeg");
            const newStatus = await invoke("get_ffmpeg_status");
            if (onToolStatusChange) onToolStatusChange("ffmpeg", newStatus);
            await message("FFmpeg updated successfully!", {
                title: "Success",
                kind: "info",
            });
        } catch (e) {
            console.error("Failed to download FFmpeg:", e);
            await message(String(e), { title: "Update Failed", kind: "error" });
        }
        ffmpegLoading = false;
    }

    async function downloadAria2() {
        aria2Loading = true;
        try {
            await invoke("download_aria2");
            const newStatus = await invoke("get_aria2_status");
            if (onToolStatusChange) onToolStatusChange("aria2", newStatus);
            await message("aria2 updated successfully!", {
                title: "Success",
                kind: "info",
            });
        } catch (e) {
            console.error("Failed to download aria2:", e);
            await message(String(e), { title: "Update Failed", kind: "error" });
        }
        aria2Loading = false;
    }

    async function importCookies() {
        try {
            const selected = await open({
                multiple: false,
                filters: [{ name: "Text Files", extensions: ["txt"] }],
                title: "Select Cookies File",
            });
            if (selected) {
                await invoke("import_cookies_file", { sourcePath: selected });
                await message("Cookies imported successfully!", {
                    title: "Success",
                    kind: "info",
                });
            }
        } catch (e) {
            console.error("Failed to import cookies:", e);
            await message(e, { title: "Import Failed", kind: "error" });
        }
    }

    async function resetAllData() {
        const confirmed = await ask(
            "This will delete ALL settings, login data, cached files, and downloaded tools. The app will close after reset.\n\nAre you sure? This action cannot be undone!",
            {
                title: "Reset All Data",
                kind: "warning",
                okLabel: "Yes, Reset Everything",
                cancelLabel: "Cancel",
            },
        );

        if (!confirmed) return;

        try {
            await invoke("clear_all_data");
            await message(
                "All data has been cleared. The app will now close.",
                {
                    title: "Reset Complete",
                    kind: "info",
                },
            );
            // Close the app
            const { getCurrentWindow } = await import("@tauri-apps/api/window");
            await getCurrentWindow().close();
        } catch (e) {
            await message(String(e), { title: "Reset Failed", kind: "error" });
        }
    }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="overlay" onclick={onClose}></div>

<!-- svelte-ignore a11y_label_has_associated_control -->
<div class="drawer">
    <div class="drawer-header">
        <span>Settings</span>
        <button class="close-btn" onclick={onClose} title="Close">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
                <path
                    d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"
                />
            </svg>
        </button>
    </div>

    <div class="drawer-content">
        <div class="section">
            <label>Download Directory</label>
            <div class="dir-row">
                <span class="dir-path"
                    >{settings?.download_dir || "Not set"}</span
                >
                <button onclick={selectDownloadDir}>Browse</button>
            </div>
        </div>

        <div class="section">
            <label>Concurrent Downloads</label>
            <select
                value={settings?.default_concurrent || 3}
                onchange={(e) => {
                    settings = {
                        ...settings,
                        default_concurrent: parseInt(e.target.value),
                    };
                    saveSettings();
                }}
            >
                {#each [1, 2, 3, 4, 5, 6, 7, 8, 9, 10] as num}
                    <option value={num}>{num}</option>
                {/each}
            </select>
        </div>

        <div class="section">
            <label>Tools</label>
            <div class="tool-row">
                <span>yt-dlp</span>
                <span
                    class="tool-status"
                    class:installed={ytdlpStatus?.installed}
                >
                    {ytdlpStatus?.installed
                        ? ytdlpStatus?.version || "Installed"
                        : "Not Installed"}
                </span>
                <button
                    onclick={downloadYtdlp}
                    disabled={ytdlpLoading || isInstallingTools}
                    title={ytdlpLoading ? "Updating..." : ""}
                >
                    {ytdlpLoading
                        ? "Updating..."
                        : ytdlpStatus?.installed
                          ? "Update"
                          : "Install"}
                </button>
            </div>
            <div class="tool-row">
                <span>FFmpeg</span>
                <span
                    class="tool-status"
                    class:installed={ffmpegStatus?.installed}
                >
                    {ffmpegStatus?.installed ? "Installed" : "Not Installed"}
                </span>
                <button
                    onclick={downloadFfmpeg}
                    disabled={ffmpegLoading || isInstallingTools}
                    title={ffmpegLoading ? "Updating..." : ""}
                >
                    {ffmpegLoading
                        ? "Updating..."
                        : ffmpegStatus?.installed
                          ? "Update"
                          : "Install"}
                </button>
            </div>
            <div class="tool-row">
                <span>aria2 <span class="tool-hint">(optional)</span></span>
                <span
                    class="tool-status"
                    class:installed={aria2Status?.installed}
                    class:warning={!aria2Status?.installed}
                >
                    {aria2Status?.installed ? "Installed" : "Not Installed"}
                </span>
                <button
                    onclick={downloadAria2}
                    disabled={aria2Loading || isInstallingTools}
                    title={aria2Loading ? "Updating..." : ""}
                >
                    {aria2Loading
                        ? "Updating..."
                        : aria2Status?.installed
                          ? "Update"
                          : "Install"}
                </button>
            </div>
        </div>

        <div class="section">
            <label>Authentication</label>
            <p class="hint">Import cookies file for age-restricted content</p>
            <button class="full-btn" onclick={importCookies}>
                Import Cookies File...
            </button>
        </div>

        <div class="section danger">
            <label>Danger Zone</label>
            <p class="hint">Reset all settings and delete all user data</p>
            <button class="full-btn danger-btn" onclick={resetAllData}>
                Reset All Data
            </button>
        </div>
    </div>
</div>

<style>
    .overlay {
        position: fixed;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        background: rgba(0, 0, 0, 0.5);
        z-index: 100;
    }

    .drawer {
        position: fixed;
        top: 0;
        right: 0;
        bottom: 0;
        width: 320px;
        background: var(--bg-secondary);
        border-left: 1px solid var(--border);
        z-index: 101;
        display: flex;
        flex-direction: column;
        animation: slideIn 0.2s ease;
    }

    @keyframes slideIn {
        from {
            transform: translateX(100%);
        }
        to {
            transform: translateX(0);
        }
    }

    .drawer-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 16px;
        border-bottom: 1px solid var(--border);
        font-weight: 600;
    }

    .close-btn {
        color: var(--text-secondary);
    }

    .close-btn:hover {
        color: var(--text-primary);
    }

    .drawer-content {
        flex: 1;
        overflow-y: auto;
        padding: 16px;
    }

    .section {
        margin-bottom: 24px;
    }

    .section label {
        display: block;
        font-weight: 500;
        margin-bottom: 8px;
        font-size: 13px;
    }

    .section select {
        width: 100%;
        padding: 8px;
        background: var(--bg-tertiary);
        color: var(--text-primary);
        border: 1px solid var(--border);
        border-radius: 4px;
    }

    .dir-row {
        display: flex;
        gap: 8px;
        align-items: center;
    }

    .dir-path {
        flex: 1;
        font-size: 12px;
        color: var(--text-secondary);
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }

    .dir-row button {
        padding: 6px 12px;
        background: var(--bg-tertiary);
        border-radius: 4px;
    }

    .dir-row button:hover {
        background: var(--accent);
        color: white;
    }

    .tool-row {
        display: flex;
        align-items: center;
        gap: 8px;
        padding: 8px 0;
        border-bottom: 1px solid var(--border);
    }

    .tool-row span:first-child {
        flex: 1;
    }

    .tool-status {
        font-size: 11px;
        padding: 2px 6px;
        border-radius: 3px;
        background: rgba(231, 76, 60, 0.15);
        color: var(--error);
    }

    .tool-status.installed {
        background: rgba(46, 204, 113, 0.15);
        color: var(--success);
    }

    .tool-status.warning {
        background: rgba(243, 156, 18, 0.15);
        color: var(--warning);
    }

    .tool-row button {
        padding: 4px 8px;
        font-size: 12px;
        background: var(--bg-tertiary);
        border-radius: 4px;
    }

    .tool-row button:hover {
        background: var(--accent);
        color: white;
    }

    .tool-hint {
        font-size: 11px;
        color: var(--text-secondary);
    }

    .hint {
        font-size: 11px;
        color: var(--text-secondary);
        margin-bottom: 8px;
    }

    .full-btn {
        width: 100%;
        padding: 10px;
        background: var(--bg-tertiary);
        border-radius: 4px;
        text-align: center;
    }

    .full-btn:hover {
        background: var(--accent);
        color: white;
    }

    .section.danger {
        border-top: 1px solid var(--error);
        padding-top: 16px;
        margin-top: 24px;
    }

    .section.danger label {
        color: var(--error);
    }

    .danger-btn {
        background: transparent;
        border: 1px solid var(--error);
        color: var(--error);
    }

    .danger-btn:hover {
        background: var(--error);
        color: white;
    }
</style>
