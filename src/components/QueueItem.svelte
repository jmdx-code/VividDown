<script>
    import { invoke } from "@tauri-apps/api/core";

    let { task, onRemove } = $props();

    const statusConfig = {
        pending: { icon: "Pending", label: "Waiting", class: "pending" },
        fetching: { icon: "Fetching", label: "Fetching", class: "fetching" },
        downloading: {
            icon: "Down",
            label: "Downloading",
            class: "downloading",
        },
        paused: { icon: "Paused", label: "Paused", class: "paused" },
        completed: { icon: "Done", label: "Completed", class: "completed" },
        failed: { icon: "Fail", label: "Failed", class: "failed" },
        cancelled: { icon: "Cancel", label: "Cancelled", class: "cancelled" },
    };

    let status = $derived(statusConfig[task.status] || statusConfig.pending);
    let videoInfo = $derived(task.video_info);
    let thumbnail = $derived(videoInfo?.thumbnail || null);
    let title = $derived(videoInfo?.title || task.url);
    let uploader = $derived(videoInfo?.uploader || null);
    let duration = $derived(videoInfo?.duration_string || null);
    let viewCount = $derived(videoInfo?.view_count || null);
    let canPause = $derived(task.status === "downloading");
    let canResume = $derived(task.status === "paused");

    // Format view count (e.g., 1234567 -> "1.2M")
    function formatViewCount(count) {
        if (!count) return null;
        if (count >= 1000000) return (count / 1000000).toFixed(1) + "M";
        if (count >= 1000) return (count / 1000).toFixed(1) + "K";
        return count.toString();
    }

    // Format file size (e.g., 123456789 -> "117.7 MB")
    function formatFileSize(bytes) {
        if (!bytes) return null;
        if (bytes >= 1073741824) return (bytes / 1073741824).toFixed(1) + " GB";
        if (bytes >= 1048576) return (bytes / 1048576).toFixed(1) + " MB";
        if (bytes >= 1024) return (bytes / 1024).toFixed(1) + " KB";
        return bytes + " B";
    }

    let formattedViews = $derived(formatViewCount(viewCount));
    let formattedSize = $derived(formatFileSize(task.total_bytes));

    let showTooltip = $state(false);
    let tooltipX = $state(0);
    let tooltipY = $state(0);

    async function handlePause() {
        try {
            await invoke("pause_download", { taskId: task.id });
        } catch (e) {
            console.error("Failed to pause:", e);
        }
    }

    async function handleResume() {
        try {
            await invoke("resume_download", { taskId: task.id });
        } catch (e) {
            console.error("Failed to resume:", e);
        }
    }

    async function handleCopyLink() {
        try {
            await navigator.clipboard.writeText(task.url);
        } catch (e) {
            console.error("Failed to copy:", e);
        }
    }

    async function handleOpenFolder() {
        try {
            await invoke("open_download_folder");
        } catch (e) {
            console.error("Failed to open folder:", e);
        }
    }

    function handleMouseEnter(e) {
        showTooltip = true;
        tooltipX = e.clientX;
        tooltipY = e.clientY;
    }

    function handleMouseLeave() {
        showTooltip = false;
    }
</script>

<div class="queue-item">
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
        class="thumbnail"
        onmouseenter={handleMouseEnter}
        onmouseleave={handleMouseLeave}
    >
        {#if thumbnail}
            <img src={thumbnail} alt="Thumbnail" />
        {:else}
            <div class="placeholder">
                <svg
                    width="24"
                    height="24"
                    viewBox="0 0 24 24"
                    fill="currentColor"
                >
                    <path
                        d="M21 3H3c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h18c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zm0 16H3V5h18v14zM9.5 13l2.5 3.01L15.5 12l4.5 6H4l5.5-5z"
                    />
                </svg>
            </div>
        {/if}
        {#if duration}
            <span class="duration">{duration}</span>
        {/if}
    </div>

    <div class="info">
        <div class="row1">
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <span
                class="title"
                {title}
                onmouseenter={handleMouseEnter}
                onmouseleave={handleMouseLeave}>{title}</span
            >
        </div>

        <div class="row2">
            {#if uploader}
                <span class="uploader">{uploader}</span>
                <span class="separator">|</span>
            {/if}
            <span class="resolution">{task.resolution}</span>
            <span class="separator">|</span>
            <span class="status {status.class}">{status.label}</span>
            {#if formattedViews}
                <span class="badge views">{formattedViews} views</span>
            {/if}
            {#if formattedSize}
                <span class="badge size">{formattedSize}</span>
            {/if}
        </div>

        <div class="row3">
            {#if task.status === "downloading"}
                <div class="progress-wrapper">
                    <div
                        class="progress-bar"
                        style="width: {task.progress}%"
                    ></div>
                </div>
                <span class="meta">
                    {task.progress?.toFixed(0)}%
                    {#if task.speed}
                        - {task.speed}{/if}
                    {#if task.eta}
                        - ETA: {task.eta}{/if}
                </span>
            {:else if task.status === "paused"}
                <div class="progress-wrapper">
                    <div
                        class="progress-bar paused"
                        style="width: {task.progress}%"
                    ></div>
                </div>
                <span class="meta">{task.progress?.toFixed(0)}% - Paused</span>
            {:else if task.status === "completed"}
                <span class="meta success">Download Complete</span>
            {:else if task.status === "failed"}
                <span class="error">{task.error || "Download failed"}</span>
            {:else if task.status === "fetching"}
                <span class="meta">Fetching video info...</span>
            {:else}
                <span class="meta">Waiting...</span>
            {/if}
        </div>
    </div>

    <div class="actions">
        <button
            class="action-btn copy-btn"
            onclick={handleCopyLink}
            aria-label="Copy link"
            title="Copy link"
        >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                <path
                    d="M16 1H4c-1.1 0-2 .9-2 2v14h2V3h12V1zm3 4H8c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h11c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm0 16H8V7h11v14z"
                />
            </svg>
        </button>
        <button
            class="action-btn folder-btn"
            onclick={handleOpenFolder}
            aria-label="Open folder"
            title="Open download folder"
        >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                <path
                    d="M10 4H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2h-8l-2-2z"
                />
            </svg>
        </button>
        {#if canPause}
            <button
                class="action-btn pause-btn"
                onclick={handlePause}
                aria-label="Pause download"
                title="Pause"
            >
                <svg
                    width="16"
                    height="16"
                    viewBox="0 0 24 24"
                    fill="currentColor"
                >
                    <path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z" />
                </svg>
            </button>
        {/if}
        {#if canResume}
            <button
                class="action-btn resume-btn"
                onclick={handleResume}
                aria-label="Resume download"
                title="Resume"
            >
                <svg
                    width="16"
                    height="16"
                    viewBox="0 0 24 24"
                    fill="currentColor"
                >
                    <path d="M8 5v14l11-7z" />
                </svg>
            </button>
        {/if}
        <button
            class="action-btn remove-btn"
            onclick={onRemove}
            aria-label="Remove task"
            title="Remove"
        >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                <path
                    d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"
                />
            </svg>
        </button>
    </div>
</div>

{#if showTooltip && videoInfo}
    <div class="tooltip" style="left: {tooltipX}px; top: {tooltipY + 20}px">
        <div class="tooltip-title">{title}</div>
        {#if uploader}
            <div class="tooltip-row">? {uploader}</div>
        {/if}
        {#if videoInfo.view_count}
            <div class="tooltip-row">
                ?? {videoInfo.view_count.toLocaleString()} views
            </div>
        {/if}
        {#if duration}
            <div class="tooltip-row">?? {duration}</div>
        {/if}
    </div>
{/if}

<style>
    .queue-item {
        display: flex;
        align-items: center;
        gap: 12px;
        padding: 10px;
        border-radius: 8px;
        background: var(--bg-tertiary);
        margin-bottom: 8px;
    }

    .queue-item:hover {
        background: #2a2a2a;
    }

    .thumbnail {
        width: 120px;
        height: 68px;
        border-radius: 6px;
        overflow: hidden;
        flex-shrink: 0;
        background: var(--bg-primary);
        position: relative;
    }

    .thumbnail img {
        width: 100%;
        height: 100%;
        object-fit: cover;
    }

    .placeholder {
        width: 100%;
        height: 100%;
        display: flex;
        align-items: center;
        justify-content: center;
        color: var(--text-secondary);
    }

    .duration {
        position: absolute;
        bottom: 4px;
        right: 4px;
        background: rgba(0, 0, 0, 0.8);
        color: white;
        font-size: 10px;
        padding: 2px 4px;
        border-radius: 2px;
    }

    .info {
        flex: 1;
        min-width: 0;
        display: flex;
        flex-direction: column;
        gap: 4px;
    }

    .row1,
    .row2,
    .row3 {
        display: flex;
        align-items: center;
        gap: 6px;
        flex-wrap: wrap;
    }

    .title {
        font-size: 13px;
        font-weight: 500;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        color: var(--text-primary);
    }

    .uploader {
        font-size: 11px;
        color: var(--text-secondary);
    }

    .resolution {
        font-size: 10px;
        color: var(--text-secondary);
        background: var(--bg-primary);
        padding: 1px 4px;
        border-radius: 2px;
    }

    .separator {
        color: var(--text-secondary);
        font-size: 10px;
    }

    .status {
        font-size: 10px;
        padding: 2px 6px;
        border-radius: 4px;
    }

    .status.pending {
        color: var(--text-secondary);
        background: var(--bg-primary);
    }
    .status.fetching {
        color: #9b59b6;
        background: rgba(155, 89, 182, 0.15);
    }
    .status.downloading {
        color: #3498db;
        background: rgba(52, 152, 219, 0.15);
    }
    .status.completed {
        color: var(--success);
        background: rgba(46, 204, 113, 0.15);
    }
    .status.failed,
    .status.cancelled {
        color: var(--error);
        background: rgba(231, 76, 60, 0.15);
    }

    .badge {
        font-size: 9px;
        padding: 2px 5px;
        border-radius: 3px;
        margin-left: 4px;
    }

    .badge.views {
        color: #9b59b6;
        background: rgba(155, 89, 182, 0.2);
    }

    .badge.size {
        color: #3498db;
        background: rgba(52, 152, 219, 0.2);
    }

    .status.paused {
        color: #f39c12;
        background: rgba(243, 156, 18, 0.15);
    }

    .progress-wrapper {
        flex: 1;
        height: 4px;
        background: var(--bg-primary);
        border-radius: 2px;
        overflow: hidden;
        min-width: 100px;
    }

    .progress-bar {
        height: 100%;
        background: linear-gradient(90deg, var(--accent), #e74c3c);
        transition: width 0.2s ease;
    }

    .progress-bar.paused {
        background: linear-gradient(90deg, #f39c12, #e67e22);
    }

    .meta {
        font-size: 11px;
        color: var(--text-secondary);
    }
    .meta.success {
        color: var(--success);
    }
    .error {
        font-size: 11px;
        color: var(--error);
    }

    .actions {
        display: flex;
        gap: 4px;
        flex-shrink: 0;
        opacity: 0;
        transition: opacity 0s; /* Instant display, no delay */
    }

    .queue-item:hover .actions {
        opacity: 1;
    }

    .action-btn {
        width: 28px;
        height: 28px;
        display: flex;
        align-items: center;
        justify-content: center;
        border-radius: 6px;
        color: var(--text-secondary);
        background: transparent;
        border: none;
        cursor: pointer;
        transition: all 0.15s ease;
    }

    .action-btn:hover {
        background: var(--bg-primary);
    }

    .copy-btn:hover {
        color: #3498db;
    }

    .folder-btn:hover {
        color: #9b59b6;
    }

    .pause-btn:hover {
        color: #f39c12;
    }

    .resume-btn:hover {
        color: var(--success);
    }

    .remove-btn:hover {
        color: var(--error);
    }

    .tooltip {
        position: fixed;
        background: rgba(0, 0, 0, 0.95);
        color: white;
        padding: 8px 12px;
        border-radius: 6px;
        font-size: 12px;
        z-index: 1000;
        pointer-events: none;
        max-width: 300px;
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
    }

    .tooltip-title {
        font-weight: 500;
        margin-bottom: 6px;
        word-wrap: break-word;
    }

    .tooltip-row {
        font-size: 11px;
        color: rgba(255, 255, 255, 0.8);
        margin-top: 4px;
    }
</style>
