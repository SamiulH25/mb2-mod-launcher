<script lang="ts">
  import PathAccordion from "./PathAccordion.svelte";

  let {
    searchQuery = $bindable(""),
    gamePath = "",
    configPath = "",
    loading = false,
    warnings = [] as string[],
    onAutoSort,
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
    onUnblockDlls?: () => void;
    onSave?: () => void;
    onRefresh?: () => void;
  } = $props();
</script>

<aside class="orders panel-inset">
  <h2 class="orders-title">Battle Orders</h2>
  <p class="orders-desc">Issue commands to your mod roster</p>

  <div class="search-wrap">
    <label class="field-label" for="mod-search">Scout mods</label>
    <input id="mod-search" type="search" placeholder="Search by name or ID…" bind:value={searchQuery} />
  </div>

  <div class="seal-stack">
    <button class="seal" onclick={onAutoSort} disabled={loading}>⟳ Auto Sort</button>
    <button class="seal" onclick={onUnblockDlls} disabled={loading}>⚿ Unblock DLLs</button>
    <button class="seal seal-primary" onclick={onSave} disabled={loading}>⚔ Save Load Order</button>
  </div>

  <button class="link-btn" onclick={onRefresh} disabled={loading}>Recon modules</button>

  {#if gamePath}
    <PathAccordion {gamePath} configPath={configPath} />
  {/if}

  {#if warnings.length > 0}
    <div class="dispatches">
      <h3>Field Reports</h3>
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
    width: 15rem;
    flex-shrink: 0;
    padding: 0.85rem;
    display: flex;
    flex-direction: column;
    gap: 0.55rem;
    align-self: stretch;
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
    margin: -0.25rem 0 0.25rem;
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
    margin-bottom: 0.25rem;
  }

  .search-wrap input {
    width: 100%;
  }

  .seal-stack {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }

  .seal-stack button {
    width: 100%;
    text-align: center;
    padding: 0.55rem 0.65rem;
  }

  .link-btn {
    padding: 0;
    background: transparent;
    border: none;
    color: var(--text-muted);
    font-size: 0.72rem;
    font-style: italic;
    text-align: left;
  }

  .link-btn:hover:not(:disabled) {
    color: var(--gold);
    background: transparent;
  }

  .dispatches h3 {
    margin: 0.5rem 0 0.35rem;
    font-family: var(--font-display);
    font-size: 0.65rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--warning-text);
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
    padding: 0.4rem 0.5rem;
    background: var(--warning-bg);
    border: 1px solid var(--warning-border);
    border-radius: var(--radius-sm);
    color: var(--parchment-dim);
  }
</style>
