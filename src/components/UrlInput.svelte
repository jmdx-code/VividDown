<script>
  let { defaultResolution, onDownload, disabled } = $props();
  let urlText = $state("");
  let resolution = $state(defaultResolution);

  const resolutions = [
    "best",
    "2160p",
    "1440p",
    "1080p",
    "720p",
    "480p",
    "360p",
    "audio",
  ];

  // Valid YouTube URL patterns (video, shorts, playlist only - no channels/homepage)
  const validPatterns = [
    /^(https?:\/\/)?(www\.)?youtube\.com\/watch\?v=[\w-]+/i,
    /^(https?:\/\/)?(www\.)?youtube\.com\/shorts\/[\w-]+/i,
    /^(https?:\/\/)?(www\.)?youtube\.com\/playlist\?list=[\w-]+/i,
    /^(https?:\/\/)?youtu\.be\/[\w-]+/i,
  ];

  function isValidYouTubeUrl(url) {
    const trimmed = url.trim();
    if (!trimmed) return true; // Empty is OK (not invalid)
    return validPatterns.some((pattern) => pattern.test(trimmed));
  }

  // Validate all URLs in the textarea
  let validationState = $derived(() => {
    if (!urlText.trim()) return "empty";
    const urls = urlText
      .split("\n")
      .map((u) => u.trim())
      .filter((u) => u);
    const allValid = urls.every(isValidYouTubeUrl);
    return allValid ? "valid" : "invalid";
  });

  let isValid = $derived(validationState() === "valid");
  let showError = $derived(validationState() === "invalid");

  function handleSubmit() {
    if (!urlText.trim() || disabled || !isValid) return;
    const urls = urlText
      .split("\n")
      .map((u) => u.trim())
      .filter((u) => u);
    onDownload(urls, resolution);
    urlText = "";
  }

  function handleKeydown(e) {
    // Ctrl+Enter to download, plain Enter for newline
    if (e.key === "Enter" && e.ctrlKey) {
      e.preventDefault();
      handleSubmit();
    }
  }
</script>

<div class="url-input">
  <div class="input-wrapper">
    <textarea
      bind:value={urlText}
      placeholder="Paste YouTube URL here (videos, shorts, playlists)"
      onkeydown={handleKeydown}
      class:valid={isValid && urlText.trim()}
      class:invalid={showError}
      {disabled}
    ></textarea>
    {#if showError}
      <div class="error-hint">请输入有效的 YouTube 视频/播放列表链接</div>
    {/if}
  </div>

  <div class="controls">
    <select bind:value={resolution} {disabled}>
      {#each resolutions as res}
        <option value={res}>{res === "audio" ? "Audio Only" : res}</option>
      {/each}
    </select>

    <button
      class="download-btn"
      onclick={handleSubmit}
      disabled={disabled || !urlText.trim() || !isValid}
    >
      <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
        <path d="M19 9h-4V3H9v6H5l7 7 7-7zM5 18v2h14v-2H5z" />
      </svg>
    </button>
  </div>
</div>

<style>
  .url-input {
    display: flex;
    gap: 8px;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 8px;
  }

  textarea {
    width: 100%;
    min-height: 60px;
    max-height: 200px;
    resize: vertical;
    background: var(--bg-tertiary);
    color: var(--text-primary);
    border: none;
    border-radius: 4px;
    padding: 8px 12px;
    font-size: 13px;
    line-height: 1.5;
  }

  textarea::placeholder {
    color: var(--text-secondary);
  }

  textarea:focus {
    outline: 1px solid var(--accent);
  }

  .controls {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  select {
    background: var(--bg-tertiary);
    color: var(--text-primary);
    border: none;
    border-radius: 4px;
    padding: 6px 8px;
    font-size: 12px;
    cursor: pointer;
  }

  select:focus {
    outline: 1px solid var(--accent);
  }

  .download-btn {
    flex: 1;
    min-width: 48px;
    background: var(--accent);
    color: white;
    border-radius: 4px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .download-btn:hover:not(:disabled) {
    background: var(--accent-hover);
  }

  .input-wrapper {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  textarea.valid {
    outline: 1px solid var(--success);
  }

  textarea.invalid {
    outline: 1px solid var(--error);
  }

  .error-hint {
    font-size: 11px;
    color: var(--error);
    padding-left: 4px;
  }
</style>
