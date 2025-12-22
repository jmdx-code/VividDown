<script>
    let { message, type = "info", onClose } = $props();

    // Auto-close after 3 seconds
    setTimeout(() => {
        onClose?.();
    }, 3000);
</script>

<div class="notification {type}">
    <div class="icon">
        {#if type === "success"}
            <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
                <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z" />
            </svg>
        {:else if type === "warning"}
            <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
                <path d="M1 21h22L12 2 1 21zm12-3h-2v-2h2v2zm0-4h-2v-4h2v4z" />
            </svg>
        {:else}
            <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
                <path
                    d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z"
                />
            </svg>
        {/if}
    </div>
    <div class="content">{message}</div>
    <button class="close-btn" onclick={onClose} aria-label="¹Ø±ÕÍ¨Öª">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path
                d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"
            />
        </svg>
    </button>
</div>

<style>
    .notification {
        position: fixed;
        bottom: 20px;
        right: 20px;
        display: flex;
        align-items: center;
        gap: 12px;
        padding: 12px 16px;
        background: var(--bg-secondary);
        border: 1px solid var(--border);
        border-radius: 8px;
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
        min-width: 300px;
        max-width: 400px;
        z-index: 1000;
        animation: slideIn 0.3s ease-out;
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

    .icon {
        flex-shrink: 0;
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .notification.success .icon {
        color: var(--success);
    }

    .notification.warning .icon {
        color: #ff9800;
    }

    .notification.info .icon {
        color: #2196f3;
    }

    .content {
        flex: 1;
        color: var(--text-primary);
        font-size: 14px;
        line-height: 1.4;
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
    }

    .close-btn:hover {
        background: var(--bg-tertiary);
        color: var(--text-primary);
    }
</style>
