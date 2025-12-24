<script>
    import { onMount } from "svelte";

    let {
        message,
        type = "info",
        onClose,
        actionLabel = null,
        onAction = null,
        actionLoading = false,
    } = $props();

    // Auto-dismiss notifications based on type
    onMount(() => {
        let delay;
        if (type === "success" || type === "info") {
            delay = 5000; // 5 seconds
        } else {
            delay = 10000; // 10 seconds for warning/error
        }

        const timer = setTimeout(() => {
            onClose?.();
        }, delay);
        return () => clearTimeout(timer);
    });
</script>

<div class="notification {type}">
    <!-- Color indicator bar on left side -->
    <div class="indicator"></div>

    <div class="content">{message}</div>

    {#if actionLabel && onAction}
        <button class="action-btn" onclick={onAction} disabled={actionLoading}>
            {actionLoading ? "Installing..." : actionLabel}
        </button>
    {/if}

    <button class="close-btn" onclick={onClose} title="Close">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
            <path
                d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"
            />
        </svg>
    </button>
</div>

<style>
    .notification {
        display: flex;
        align-items: center;
        gap: 12px;
        padding: 14px 16px;
        padding-left: 0;
        background: var(--bg-secondary);
        border: 1px solid var(--border);
        border-radius: 8px;
        box-shadow: 0 4px 16px rgba(0, 0, 0, 0.25);
        min-width: 280px;
        max-width: 420px;
        animation: slideIn 0.3s ease-out;
        overflow: hidden;
    }

    @keyframes slideIn {
        from {
            transform: translateX(120%);
            opacity: 0;
        }
        to {
            transform: translateX(0);
            opacity: 1;
        }
    }

    /* Left-side color indicator bar */
    .indicator {
        width: 4px;
        align-self: stretch;
        border-radius: 4px 0 0 4px;
        margin-left: -1px;
    }

    .notification.success .indicator {
        background: #22c55e;
    }

    .notification.warning .indicator {
        background: #f59e0b;
    }

    .notification.error .indicator {
        background: #ef4444;
    }

    .notification.info .indicator {
        background: #3b82f6;
    }

    .content {
        flex: 1;
        color: var(--text-primary);
        font-size: 13px;
        line-height: 1.5;
        padding-left: 12px;
    }

    .action-btn {
        flex-shrink: 0;
        padding: 6px 12px;
        background: var(--accent);
        color: white;
        border-radius: 4px;
        font-size: 12px;
        font-weight: 500;
        transition: background 0.2s;
    }

    .action-btn:hover:not(:disabled) {
        background: var(--accent-hover);
    }

    .action-btn:disabled {
        opacity: 0.7;
        cursor: wait;
    }

    .close-btn {
        flex-shrink: 0;
        width: 24px;
        height: 24px;
        display: flex;
        align-items: center;
        justify-content: center;
        border-radius: 4px;
        color: var(--text-secondary);
        opacity: 0.6;
        transition:
            opacity 0.2s,
            background 0.2s;
    }

    .close-btn:hover {
        background: var(--bg-tertiary);
        color: var(--text-primary);
        opacity: 1;
    }
</style>
