<script>
    import { invoke } from "@tauri-apps/api/core";

    let { missingTools = [], onComplete } = $props();

    let status = $state("prompt"); // "prompt" | "installing" | "success" | "error"
    let errorMessage = $state("");
    let progress = $state("");

    async function handleInstall() {
        status = "installing";
        progress = "Preparing...";

        try {
            // Install yt-dlp if missing
            if (missingTools.includes("yt-dlp")) {
                progress = "Downloading yt-dlp...";
                await invoke("download_ytdlp");
            }

            // Install FFmpeg if missing
            if (missingTools.includes("FFmpeg")) {
                progress = "Downloading FFmpeg...";
                await invoke("download_ffmpeg");
            }

            status = "success";
            progress = "Installation complete!";

            // Wait a moment then close
            setTimeout(() => {
                onComplete(true);
            }, 1500);
        } catch (e) {
            status = "error";
            errorMessage = String(e);
        }
    }

    function handleRetry() {
        status = "prompt";
        errorMessage = "";
    }
</script>

<div class="modal-overlay">
    <div class="modal">
        {#if status === "prompt"}
            <div class="modal-icon warning">
                <svg
                    width="48"
                    height="48"
                    viewBox="0 0 24 24"
                    fill="currentColor"
                >
                    <path
                        d="M1 21h22L12 2 1 21zm12-3h-2v-2h2v2zm0-4h-2v-4h2v4z"
                    />
                </svg>
            </div>
            <h2>Required Tools Missing</h2>
            <p>The following tools are required to use VividDown:</p>
            <ul class="tool-list">
                {#each missingTools as tool}
                    <li>{tool}</li>
                {/each}
            </ul>
            <p class="hint">
                Click "Install" to download and install them automatically.
            </p>
            <button class="primary-btn" onclick={handleInstall}>Install</button>
        {:else if status === "installing"}
            <div class="modal-icon loading">
                <svg
                    width="48"
                    height="48"
                    viewBox="0 0 24 24"
                    fill="currentColor"
                    class="spin"
                >
                    <path
                        d="M12 4V1L8 5l4 4V6c3.31 0 6 2.69 6 6 0 1.01-.25 1.97-.7 2.8l1.46 1.46C19.54 15.03 20 13.57 20 12c0-4.42-3.58-8-8-8zm0 14c-3.31 0-6-2.69-6-6 0-1.01.25-1.97.7-2.8L5.24 7.74C4.46 8.97 4 10.43 4 12c0 4.42 3.58 8 8 8v3l4-4-4-4v3z"
                    />
                </svg>
            </div>
            <h2>Installing Tools</h2>
            <p class="progress-text">{progress}</p>
            <p class="hint">Please wait, this may take a few minutes...</p>
        {:else if status === "success"}
            <div class="modal-icon success">
                <svg
                    width="48"
                    height="48"
                    viewBox="0 0 24 24"
                    fill="currentColor"
                >
                    <path
                        d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z"
                    />
                </svg>
            </div>
            <h2>Installation Complete!</h2>
            <p>All tools have been installed successfully.</p>
        {:else if status === "error"}
            <div class="modal-icon error">
                <svg
                    width="48"
                    height="48"
                    viewBox="0 0 24 24"
                    fill="currentColor"
                >
                    <path
                        d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z"
                    />
                </svg>
            </div>
            <h2>Installation Failed</h2>
            <p class="error-msg">{errorMessage}</p>
            <button class="primary-btn" onclick={handleRetry}>Retry</button>
        {/if}
    </div>
</div>

<style>
    .modal-overlay {
        position: fixed;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        background: rgba(0, 0, 0, 0.8);
        display: flex;
        align-items: center;
        justify-content: center;
        z-index: 1000;
    }

    .modal {
        background: var(--bg-secondary);
        border: 1px solid var(--border);
        border-radius: 12px;
        padding: 32px;
        max-width: 400px;
        text-align: center;
        animation: fadeIn 0.3s ease;
    }

    @keyframes fadeIn {
        from {
            opacity: 0;
            transform: scale(0.95);
        }
        to {
            opacity: 1;
            transform: scale(1);
        }
    }

    .modal-icon {
        margin-bottom: 16px;
    }

    .modal-icon.warning {
        color: var(--warning);
    }
    .modal-icon.success {
        color: var(--success);
    }
    .modal-icon.error {
        color: var(--error);
    }
    .modal-icon.loading {
        color: var(--accent);
    }

    .spin {
        animation: spin 1s linear infinite;
    }

    @keyframes spin {
        from {
            transform: rotate(0deg);
        }
        to {
            transform: rotate(360deg);
        }
    }

    h2 {
        color: var(--text-primary);
        font-size: 20px;
        margin-bottom: 12px;
    }

    p {
        color: var(--text-secondary);
        font-size: 14px;
        margin-bottom: 8px;
    }

    .tool-list {
        list-style: none;
        padding: 0;
        margin: 16px 0;
    }

    .tool-list li {
        background: var(--bg-tertiary);
        padding: 8px 16px;
        border-radius: 4px;
        margin: 4px 0;
        color: var(--text-primary);
        font-weight: 500;
    }

    .hint {
        font-size: 12px;
        color: var(--text-secondary);
        margin: 16px 0;
    }

    .progress-text {
        color: var(--accent);
        font-weight: 500;
    }

    .error-msg {
        color: var(--error);
        font-size: 12px;
        background: rgba(231, 76, 60, 0.1);
        padding: 8px 12px;
        border-radius: 4px;
        margin: 16px 0;
    }

    .primary-btn {
        background: var(--accent);
        color: white;
        padding: 12px 32px;
        border-radius: 6px;
        font-size: 14px;
        font-weight: 600;
        margin-top: 8px;
        transition: background 0.2s;
    }

    .primary-btn:hover {
        background: var(--accent-hover);
    }
</style>
