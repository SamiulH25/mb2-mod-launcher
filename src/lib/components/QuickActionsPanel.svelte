<script lang="ts">
  import PathAccordion from "./PathAccordion.svelte";
  import OrnamentDivider from "./OrnamentDivider.svelte";

  let {
    searchQuery = $bindable(""),
    gamePath = "",
    configPath = "",
    loading = false,
    warnings = [] as string[],
    onAutoSort,
    onLaunch,
    onUnblockDlls,
    onSave,
    onRefresh,
  }: {
    searchQuery?: string;
    gamePath?: string;
    configPath?: string;
    loading?: boolean;
    warnings?: string[];
    onAutoSort?: () => void;
    onLaunch?: () => void;
    onUnblockDlls?: () => void;
    onSave?: () => void;
    onRefresh?: () => void;
  } = $props();
</script>

<aside class="orders panel-inset panel-heraldic">
  <header class="orders-header">
    <h2 class="orders-title">Battle Orders</h2>
    <p class="orders-desc">Issue commands to your mod roster</p>
  </header>

  <OrnamentDivider label="Scout" />

  <div class="search-wrap">
    <label class="field-label" for="mod-search">Search the roster</label>
    <div class="search-field">
      <span class="search-icon" aria-hidden="true">⌕</span>
      <input
        id="mod-search"
        type="search"
        placeholder="Name or module ID…"
        bind:value={searchQuery}
      />
    </div>
  </div>

  <OrnamentDivider label="Deploy" />

  <div class="seal-stack">
    <button class="seal seal-primary" onclick={onLaunch} disabled={loading}>
      <span class="btn-icon" aria-hidden="true">▶</span>
      Launch via Steam
    </button>
    <button class="seal" onclick={onAutoSort} disabled={loading}>
      <span class="btn-icon" aria-hidden="true">⟳</span>
      Auto Sort
    </button>
    <button class="seal" onclick={onUnblockDlls} disabled={loading}>
      <span class="btn-icon" aria-hidden="true">⚿</span>
      Unblock DLLs
    </button>
    <button class="seal seal-primary" onclick={onSave} disabled={loading}>
      <span class="btn-icon" aria-hidden="true">⚔</span>
      Save Load Order
    </button>
  </div>

  <button class="link-btn" onclick={onRefresh} disabled={loading}>↻ Recon modules</button>

  {#if gamePath}
    <PathAccordion {gamePath} configPath={configPath} />
  {/if}

  {#if warnings.length > 0}
    <div class="dispatches">
      <OrnamentDivider label="Reports" />
      <ul>
        {#each warnings as warning}
          <li>{warning}</li>
        {/each}
      </ul>
    </div>
  {/if}
</aside>

<style>
  .orders {
    width: 15.5rem;
    flex-shrink: 0;
    padding: 0.9rem;
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
    align-self: stretch;
    box-shadow:
      var(--shadow-panel),
      inset 0 1px 0 rgba(184, 149, 74, 0.12);
  }

  .orders-header {
    margin-bottom: -0.15rem;
  }

  .orders-title {
    margin: 0;
    font-family: var(--font-display);
    font-size: 0.95rem;
    font-weight: 700;
    color: var(--gold-light);
    letter-spacing: 0.05em;
    text-transform: uppercase;
  }

  .orders-desc {
    margin: 0.2rem 0 0;
    font-size: 0.72rem;
    color: var(--text-muted);
    font-style: italic;
  }

  .field-label {
    display: block;
    font-family: var(--font-display);
    font-size: 0.62rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--text-muted);
    margin-bottom: 0.28rem;
  }

  .search-field {
    position: relative;
  }

  .search-icon {
    position: absolute;
    left: 0.55rem;
    top: 50%;
    transform: translateY(-50%);
    color: var(--text-muted);
    font-size: 0.9rem;
    pointer-events: none;
  }

  .search-field input {
    width: 100%;
    padding-left: 1.65rem;
  }

  .seal-stack {
    display: flex;
    flex-direction: column;
    gap: 0.42rem;
  }

  .seal-stack button {
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.45rem;
    padding: 0.58rem 0.65rem;
  }

  .btn-icon {
    font-size: 0.85rem;
    opacity: 0.85;
  }

  .link-btn {
    padding: 0;
    background: transparent;
    border: none;
    color: var(--text-muted);
    font-size: 0.72rem;
    font-style: italic;
    text-align: left;
    box-shadow: none;
  }

  .link-btn:hover:not(:disabled) {
    color: var(--gold);
    background: transparent;
  }

  .dispatches ul {
    margin: 0;
    padding: 0;
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
  }

  .dispatches li {
    font-size: 0.72rem;
    line-height: 1.4;
    padding: 0.42rem 0.55rem;
    background: var(--warning-bg);
    border: 1px solid var(--warning-border);
    border-radius: var(--radius-sm);
    color: var(--parchment-dim);
    box-shadow: inset 3px 0 0 var(--warning-text);
  }
</style>
