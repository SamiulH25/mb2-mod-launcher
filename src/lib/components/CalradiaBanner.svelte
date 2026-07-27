<script lang="ts">
  import CoatOfArms from "./CoatOfArms.svelte";

  let {
    subtitle = "",
    enabledCount = 0,
    totalCount = 0,
    loading = false,
    hasGame = false,
    onDetect,
    onRefresh,
  }: {
    subtitle?: string;
    enabledCount?: number;
    totalCount?: number;
    loading?: boolean;
    hasGame?: boolean;
    onDetect?: () => void;
    onRefresh?: () => void;
  } = $props();
</script>

<header class="banner panel-inset">
  <div class="banner-left">
    <CoatOfArms size={44} />
    <div class="titles">
      <h1>Calradia Command</h1>
      <p class="subtitle">{subtitle || "MB2 Mod Launcher"}</p>
    </div>
  </div>

  {#if hasGame}
    <div class="banner-stats">
      <div class="stat">
        <span class="stat-num">{enabledCount}</span>
        <span class="stat-lbl">Active</span>
      </div>
      <div class="stat-div"></div>
      <div class="stat">
        <span class="stat-num">{totalCount}</span>
        <span class="stat-lbl">Total</span>
      </div>
    </div>
  {/if}

  <div class="banner-actions">
    <button class="ghost" onclick={onDetect} disabled={loading}>
      {loading && !hasGame ? "Scouting…" : "Detect Game"}
    </button>
    {#if hasGame}
      <button class="ghost" onclick={onRefresh} disabled={loading}>Refresh</button>
    {/if}
  </div>
</header>

<style>
  .banner {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 0.65rem 1rem;
    flex-wrap: wrap;
    border-bottom: 2px solid var(--rust-dark);
  }

  .banner-left {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    flex: 1;
    min-width: 0;
  }

  .titles h1 {
    margin: 0;
    font-family: var(--font-display);
    font-size: 1.15rem;
    font-weight: 700;
    color: var(--gold-light);
    letter-spacing: 0.06em;
    text-transform: uppercase;
  }

  .subtitle {
    margin: 0.1rem 0 0;
    font-size: 0.78rem;
    color: var(--text-muted);
    letter-spacing: 0.02em;
  }

  .banner-stats {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.35rem 0.85rem;
    background: var(--bg-base);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
  }

  .stat {
    display: flex;
    flex-direction: column;
    align-items: center;
    min-width: 2.5rem;
  }

  .stat-num {
    font-family: var(--font-display);
    font-size: 1.1rem;
    font-weight: 700;
    color: var(--parchment);
    line-height: 1;
  }

  .stat-lbl {
    font-size: 0.62rem;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--text-muted);
    margin-top: 0.15rem;
  }

  .stat-div {
    width: 1px;
    height: 1.5rem;
    background: var(--border-subtle);
  }

  .banner-actions {
    display: flex;
    gap: 0.4rem;
    flex-shrink: 0;
  }
</style>
