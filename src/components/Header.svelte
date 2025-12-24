<script>
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { convertFileSrc } from "@tauri-apps/api/core";

  let { loginStatus, onLogin, onLogout, onToggleSettings } = $props();

  // Convert local file path to WebView-accessible URL
  let avatarSrc = $derived(() => {
    const url = loginStatus.avatar_url;
    if (!url) return null;
    // If it's a local path (not http), convert it
    if (!url.startsWith("http")) {
      return convertFileSrc(url);
    }
    return url;
  });

  async function minimizeWindow() {
    try {
      const win = getCurrentWindow();
      await win.minimize();
    } catch (e) {
      console.error("Failed to minimize:", e);
    }
  }

  async function closeWindow() {
    try {
      const win = getCurrentWindow();
      await win.close();
    } catch (e) {
      console.error("Failed to close:", e);
    }
  }

  function handleAvatarClick() {
    onLogin();
  }
</script>

<header>
  <div class="title drag-region">
    <!-- VividDown SVG Logo -->
    <svg
      width="24"
      height="24"
      viewBox="0 0 24 24"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
    >
      <defs>
        <linearGradient id="redGradient" x1="0%" y1="0%" x2="100%" y2="0%">
          <stop offset="0%" style="stop-color:#ff4b2b" />
          <stop offset="100%" style="stop-color:#ff416c" />
        </linearGradient>
        <linearGradient id="blueGradient" x1="0%" y1="0%" x2="100%" y2="0%">
          <stop offset="0%" style="stop-color:#00c6ff" />
          <stop offset="100%" style="stop-color:#0072ff" />
        </linearGradient>
      </defs>
      <path
        d="M12 2C6.48 2 2 6.48 2 12C2 13.59 2.37 15.09 3.03 16.42"
        stroke="url(#redGradient)"
        stroke-width="2.5"
        stroke-linecap="round"
      />
      <path
        d="M12 22C17.52 22 22 17.52 22 12C22 10.41 21.63 8.91 20.97 7.58"
        stroke="url(#blueGradient)"
        stroke-width="2.5"
        stroke-linecap="round"
      />
      <path
        d="M12 7V17M12 17L8 13M12 17L16 13"
        stroke="currentColor"
        stroke-width="2.5"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
    </svg>
    <span>VividDown</span>
  </div>

  <div class="spacer drag-region"></div>

  <div class="actions">
    {#if loginStatus.logged_in}
      <button class="avatar-btn" onclick={handleAvatarClick} title="Account">
        {#if avatarSrc()}
          <img src={avatarSrc()} alt="Avatar" class="avatar-img" />
        {:else}
          <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
            <path
              d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 3c1.66 0 3 1.34 3 3s-1.34 3-3 3-3-1.34-3-3 1.34-3 3-3zm0 14.2c-2.5 0-4.71-1.28-6-3.22.03-1.99 4-3.08 6-3.08 1.99 0 5.97 1.09 6 3.08-1.29 1.94-3.5 3.22-6 3.22z"
            />
          </svg>
        {/if}
      </button>
    {:else}
      <button class="login-btn" onclick={onLogin}> Login </button>
    {/if}

    <button class="settings-btn" onclick={onToggleSettings} title="Settings">
      <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
        <path
          d="M19.14 12.94c.04-.31.06-.63.06-.94 0-.31-.02-.63-.06-.94l2.03-1.58c.18-.14.23-.41.12-.61l-1.92-3.32c-.12-.22-.37-.29-.59-.22l-2.39.96c-.5-.38-1.03-.7-1.62-.94l-.36-2.54c-.04-.24-.24-.41-.48-.41h-3.84c-.24 0-.43.17-.47.41l-.36 2.54c-.59.24-1.13.57-1.62.94l-2.39-.96c-.22-.08-.47 0-.59.22L2.74 8.87c-.12.21-.08.47.12.61l2.03 1.58c-.04.31-.06.63-.06.94s.02.63.06.94l-2.03 1.58c-.18.14-.23.41-.12.61l1.92 3.32c.12.22.37.29.59.22l2.39-.96c.5.38 1.03.7 1.62.94l.36 2.54c.05.24.24.41.48.41h3.84c.24 0 .44-.17.47-.41l.36-2.54c.59-.24 1.13-.56 1.62-.94l2.39.96c.22.08.47 0 .59-.22l1.92-3.32c.12-.22.07-.47-.12-.61l-2.01-1.58zM12 15.6c-1.98 0-3.6-1.62-3.6-3.6s1.62-3.6 3.6-3.6 3.6 1.62 3.6 3.6-1.62 3.6-3.6 3.6z"
        />
      </svg>
    </button>

    <div class="window-controls">
      <button
        class="window-btn minimize-btn"
        onclick={minimizeWindow}
        title="Minimize"
      >
        <svg width="12" height="12" viewBox="0 0 12 12">
          <rect y="5" width="12" height="2" fill="currentColor" />
        </svg>
      </button>
      <button class="window-btn close-btn" onclick={closeWindow} title="Close">
        <svg width="12" height="12" viewBox="0 0 12 12">
          <path
            d="M1 1L11 11M1 11L11 1"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
          />
        </svg>
      </button>
    </div>
  </div>
</header>

<style>
  header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 16px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
  }

  .drag-region {
    -webkit-app-region: drag;
  }

  .title {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 16px;
    font-weight: 600;
    color: var(--text-primary);
    cursor: default;
  }

  .title svg {
    color: var(--accent);
  }

  .spacer {
    flex: 1;
    height: 100%;
    min-height: 32px;
  }

  .actions {
    display: flex;
    align-items: center;
    gap: 8px;
    -webkit-app-region: no-drag;
  }

  .login-btn {
    padding: 6px 16px;
    background: var(--accent);
    color: white;
    border-radius: 4px;
    font-weight: 500;
  }

  .login-btn:hover {
    background: var(--accent-hover);
  }

  .avatar-btn {
    width: 32px;
    height: 32px;
    border-radius: 50%;
    background: var(--bg-tertiary);
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--success);
    overflow: hidden;
    padding: 0;
  }

  .avatar-btn:hover {
    background: #3a3a3a;
  }

  .avatar-img {
    width: 28px;
    height: 28px;
    border-radius: 50%;
    object-fit: cover;
  }

  .settings-btn {
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 4px;
    color: var(--text-secondary);
  }

  .settings-btn:hover {
    background: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .window-controls {
    display: flex;
    gap: 4px;
    margin-left: 8px;
    padding-left: 8px;
    border-left: 1px solid var(--border);
  }

  .window-btn {
    width: 28px;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 4px;
    color: var(--text-secondary);
    transition: all 0.15s ease;
  }

  .window-btn:hover {
    background: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .close-btn:hover {
    background: var(--accent);
    color: white;
  }
</style>
