<script>
    import { invoke } from "@tauri-apps/api/core";
    import { open, message } from "@tauri-apps/plugin-dialog";

    let {
        settings = $bindable(),
        ytdlpStatus,
        ffmpegStatus,
        aria2Status,
        onClose,
    } = $props();

    let ytdlpDownloading = $state(false);
    let ffmpegDownloading = $state(false);
    let aria2Downloading = $state(false);

    async function selectDownloadDir() {
        const selected = await open({
            directory: true,
            multiple: false,
            title: "Select Download Directory",
        });
        if (selected) {
            settings = { ...settings, download_dir: selected };
            await saveSettings();
        }
    }

    async function saveSettings() {
        try {
            await invoke("save_settings", { settings });
        } catch (e) {
            console.error("Failed to save settings:", e);
        }
    }

    async function downloadYtdlp() {
        ytdlpDownloading = true;
        try {
            await invoke("download_ytdlp");
            ytdlpStatus = await invoke("get_ytdlp_status");
        } catch (e) {
            console.error("Failed to download yt-dlp:", e);
        }
        ytdlpDownloading = false;
    }

    async function downloadFfmpeg() {
        ffmpegDownloading = true;
        try {
            await invoke("download_ffmpeg");
            ffmpegStatus = await invoke("get_ffmpeg_status");
        } catch (e) {
            console.error("Failed to download FFmpeg:", e);
        }
        ffmpegDownloading = false;
    }

    async function downloadAria2() {
        aria2Downloading = true;
        try {
            await invoke("download_aria2");
            aria2Status = await invoke("get_aria2_status");
        } catch (e) {
            console.error("Failed to download aria2:", e);
        }
        aria2Downloading = false;
    }

    async function importCookies() {
        const selected = await open({
            multiple: false,
            title: "Select Cookies File",
            filters: [{ name: "Cookies", extensions: ["txt"] }],
        });
        if (selected) {
            try {
                await invoke("import_cookies_file", { filePath: selected });
                await message("Cookies imported successfully!", {
                    title: "Success",
                    kind: "info",
                });
            } catch (e) {
                console.error("Failed to import cookies:", e);
                await message(e, { title: "Import Failed", kind: "error" });
            }
        }
    }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="overlay" onclick={onClose}></div>

<div class="drawer">
    <div class="drawer-header">
        <span>Settings</span>
        <button class="close-btn" onclick={onClose}>
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
            <div class="input-row">
                <input
                    type="text"
                    value={settings?.download_dir || ""}
                    readonly
                />
                <button onclick={selectDownloadDir}>Browse</button>
            </div>
        </div>

        <div class="section">
            <label>Default Quality</label>
            <select
                value={settings?.default_resolution || "1080p"}
                onchange={(e) => {
                    settings = {
                        ...settings,
                        default_resolution: e.target.value,
                    };
                    saveSettings();
                }}
            >
                <option value="best">Best</option>
                <option value="2160p">4K (2160p)</option>
                <option value="1440p">2K (1440p)</option>
                <option value="1080p">1080p</option>
                <option value="720p">720p</option>
                <option value="480p">480p</option>
                <option value="audio">Audio Only</option>
            </select>
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

        <hr />

        <div class="section">
            <label>Tools Status</label>

            <div class="tool-row">
                <span class="tool-name">yt-dlp</span>
                {#if ytdlpStatus?.installed}
                    <span class="tool-status success"
                        >? {ytdlpStatus.version || "Installed"}</span
                    >
                {:else}
                    <span class="tool-status error">Not installed</span>
                {/if}
                <button onclick={downloadYtdlp} disabled={ytdlpDownloading}>
                    {ytdlpDownloading
                        ? "Downloading..."
                        : ytdlpStatus?.installed
                          ? "Update"
                          : "Install"}
                </button>
            </div>

            <div class="tool-row">
                <span class="tool-name">FFmpeg</span>
                {#if ffmpegStatus?.installed}
                    <span class="tool-status success">? Installed</span>
                {:else}
                    <span class="tool-status error">Not installed</span>
                {/if}
                <button onclick={downloadFfmpeg} disabled={ffmpegDownloading}>
                    {ffmpegDownloading
                        ? "Downloading..."
                        : ffmpegStatus?.installed
                          ? "Reinstall"
                          : "Install"}
                </button>
            </div>

            <div class="tool-row">
                <span class="tool-name">aria2</span>
                {#if aria2Status?.installed}
                    <span class="tool-status success"
                        >? {aria2Status.version || "Installed"}</span
                    >
                {:else}
                    <span class="tool-status warning">Optional</span>
                {/if}
                <button onclick={downloadAria2} disabled={aria2Downloading}>
                    {aria2Downloading
                        ? "Downloading..."
                        : aria2Status?.installed
                          ? "Reinstall"
                          : "Install"}
                </button>
            </div>
            <p class="hint tool-hint">
                aria2 enables faster multi-threaded downloads (16 connections).
            </p>
        </div>

        <hr />

        <div class="section">
            <label>Import Cookies File</label>
            <p class="hint">
                Import a Netscape format cookies.txt file for authentication.
            </p>
            <button class="full-btn" onclick={importCookies}
                >Select File...</button
            >
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
        padding: 12px 16px;
        border-bottom: 1px solid var(--border);
        font-weight: 600;
    }

    .close-btn {
        width: 28px;
        height: 28px;
        display: flex;
        align-items: center;
        justify-content: center;
        border-radius: 4px;
        color: var(--text-secondary);
    }

    .close-btn:hover {
        background: var(--bg-tertiary);
        color: var(--text-primary);
    }

    .drawer-content {
        flex: 1;
        overflow-y: auto;
        padding: 16px;
    }

    .section {
        margin-bottom: 16px;
    }

    label {
        display: block;
        font-size: 12px;
        font-weight: 500;
        color: var(--text-secondary);
        margin-bottom: 6px;
    }

    .input-row {
        display: flex;
        gap: 8px;
    }

    input[type="text"] {
        flex: 1;
        background: var(--bg-tertiary);
        color: var(--text-primary);
        border: none;
        border-radius: 4px;
        padding: 8px;
        font-size: 12px;
    }

    select {
        width: 100%;
        background: var(--bg-tertiary);
        color: var(--text-primary);
        border: none;
        border-radius: 4px;
        padding: 8px;
        font-size: 12px;
        cursor: pointer;
    }

    button {
        background: var(--bg-tertiary);
        color: var(--text-primary);
        padding: 8px 12px;
        border-radius: 4px;
        font-size: 12px;
    }

    button:hover:not(:disabled) {
        background: #3a3a3a;
    }

    hr {
        border: none;
        border-top: 1px solid var(--border);
        margin: 16px 0;
    }

    .tool-row {
        display: flex;
        align-items: center;
        gap: 8px;
        margin-bottom: 8px;
    }

    .tool-name {
        flex: 1;
        font-size: 13px;
    }

    .tool-status {
        font-size: 11px;
        padding: 2px 6px;
        border-radius: 3px;
    }

    .tool-status.success {
        color: var(--success);
        background: rgba(46, 204, 113, 0.15);
    }

    .tool-status.error {
        color: var(--error);
        background: rgba(231, 76, 60, 0.15);
    }

    .tool-status.warning {
        color: #f39c12;
        background: rgba(243, 156, 18, 0.15);
    }

    .tool-hint {
        margin-top: 0;
        margin-bottom: 0;
    }

    .hint {
        font-size: 11px;
        color: var(--text-secondary);
        margin-bottom: 8px;
    }

    .full-btn {
        width: 100%;
    }
</style>
