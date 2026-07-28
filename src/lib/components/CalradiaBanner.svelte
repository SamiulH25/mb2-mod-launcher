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
    onLaunch,
  }: {
    subtitle?: string;
    enabledCount?: number;
    totalCount?: number;
    loading?: boolean;
    hasGame?: boolean;
    onDetect?: () => void;
    onRefresh?: () => void;
    onLaunch?: () => void;
  } = $props();

  const enlistRatio = $derived(
    totalCount > 0 ? Math.round((enabledCount / totalCount) * 100) : 0,
  );
</script>

<header class="banner panel-inset panel-heraldic">
  <div class="banner-inner">
    <div class="banner-left">
      <div class="crest-wrap">
        <CoatOfArms size={46} />
      </div>
      <div class="titles">
        <p class="eyebrow">Mount &amp; Blade II · Bannerlord</p>
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
        <div class="stat-div" aria-hidden="true"></div>
        <div class="stat">
          <span class="stat-num">{totalCount}</span>
          <span class="stat-lbl">Total</span>
        </div>
        <div class="stat-ring" style="--ratio: {enlistRatio}">
          <svg viewBox="0 0 36 36" aria-hidden="true">
            <circle class="ring-bg" cx="18" cy="18" r="15.5" />
            <circle class="ring-fill" cx="18" cy="18" r="15.5" />
          </svg>
          <span class="ring-label">{enlistRatio}%</span>
        </div>
      </div>
    {/if}

    <div class="banner-actions">
      {#if hasGame}
        <button class="seal seal-primary launch-btn" onclick={onLaunch} disabled={loading}>
          Launch Game
        </button>
        <button class="ghost" onclick={onRefresh} disabled={loading}>Refresh</button>
      {/if}
      <button class="ghost" onclick={onDetect} disabled={loading}>
        {loading && !hasGame ? "Scouting…" : "Detect Game"}
      </button>
    </div>
  </div>

  {#if loading}
    <div class="loading-track" aria-hidden="true">
      <div class="loading-fill"></div>
    </div>
  {/if}
</header>

<style>
  .banner {
    padding: 0;
    border-bottom: 2px solid var(--rust-dark);
    overflow: hidden;
  }

  .banner-inner {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 0.7rem 1rem;
    flex-wrap: wrap;
  }

  .banner-left {
    display: flex;
    align-items: center;
    gap: 0.85rem;
    flex: 1;
    min-width: 0;
  }

  .crest-wrap {
    padding: 0.35rem;
    border-radius: var(--radius-sm);
    background: rgba(0, 0, 0, 0.22);
    border: 1px solid var(--border-subtle);
    box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.04);
  }

  .eyebrow {
    margin: 0 0 0.1rem;
    font-size: 0.62rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.12em;
    color: var(--rust);
  }

  .titles h1 {
    margin: 0;
    font-family: var(--font-display);
    font-size: 1.2rem;
    font-weight: 700;
    color: var(--gold-light);
    letter-spacing: 0.06em;
    text-transform: uppercase;
    text-shadow: 0 1px 0 rgba(0, 0, 0, 0.4);
  }

  .subtitle {
    margin: 0.12rem 0 0;
    font-size: 0.78rem;
    color: var(--text-muted);
    letter-spacing: 0.02em;
  }

  .banner-stats {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.4rem 0.9rem;
    background: rgba(0, 0, 0, 0.28);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.03);
  }

  .stat {
    display: flex;
    flex-direction: column;
    align-items: center;
    min-width: 2.5rem;
  }

  .stat-num {
    font-family: var(--font-display);
    font-size: 1.15rem;
    font-weight: 700;
    color: var(--parchment);
    line-height: 1;
  }

  .stat-lbl {
    font-size: 0.6rem;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--text-muted);
    margin-top: 0.18rem;
  }

  .stat-div {
    width: 1px;
    height: 1.75rem;
    background: linear-gradient(180deg, transparent, var(--border-strong), transparent);
  }

  .stat-ring {
    position: relative;
    width: 2.5rem;
    height: 2.5rem;
    margin-left: 0.15rem;
  }

  .stat-ring svg {
    width: 100%;
    height: 100%;
    transform: rotate(-90deg);
  }

  .ring-bg {
    fill: none;
    stroke: var(--border-subtle);
    stroke-width: 2.5;
  }

  .ring-fill {
    fill: none;
    stroke: var(--gold);
    stroke-width: 2.5;
    stroke-linecap: round;
    stroke-dasharray: 97.4;
    stroke-dashoffset: calc(97.4 - (97.4 * var(--ratio)) / 100);
  }

  .ring-label {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    font-family: var(--font-display);
    font-size: 0.52rem;
    font-weight: 700;
    color: var(--gold-light);
  }

  .banner-actions {
    display: flex;
    gap: 0.4rem;
    flex-shrink: 0;
    align-items: center;
  }

  .launch-btn {
    padding: 0.5rem 1rem;
    font-size: 0.7rem;
  }

  .loading-track {
    height: 2px;
    background: rgba(0, 0, 0, 0.35);
    overflow: hidden;
  }

  .loading-fill {
    height: 100%;
    width: 100%;
    background: linear-gradient(90deg, var(--rust-dark), var(--gold));
    opacity: 0.85;
  }
</style>
