<script lang="ts">
  import OrnamentDivider from "./OrnamentDivider.svelte";

  const MAX_WAYPOINTS = 24;

  let {
    waypoints,
  }: {
    waypoints: { id: string; name: string }[];
  } = $props();

  const visible = $derived(waypoints.slice(0, MAX_WAYPOINTS));
  const overflow = $derived(Math.max(0, waypoints.length - MAX_WAYPOINTS));
</script>

<div class="campaign panel-inset panel-heraldic">
  <div class="campaign-head">
    <div class="head-left">
      <span class="campaign-title">Campaign Route</span>
      <span class="campaign-count">{waypoints.length} waypoints</span>
    </div>
    <OrnamentDivider />
  </div>

  <div class="route scroll-region" role="list" aria-label="Module load order">
    {#if waypoints.length === 0}
      <div class="empty">
        <span class="empty-icon" aria-hidden="true">⚑</span>
        <span>No modules enlisted — enable mods to chart your route</span>
      </div>
    {:else}
      <div class="route-track">
        {#each visible as waypoint, i (waypoint.id)}
          <div
            class="waypoint"
            class:start={i === 0}
            class:end={i === visible.length - 1 && overflow === 0}
            role="listitem"
            title={waypoint.id}
          >
            <span class="marker">{i + 1}</span>
            <span class="label" title={waypoint.id}>{waypoint.name}</span>
          </div>
          {#if i < visible.length - 1 || overflow > 0}
            <span class="trail" aria-hidden="true"></span>
          {/if}
        {/each}
        {#if overflow > 0}
          <div class="waypoint overflow" role="listitem">
            <span class="marker">+</span>
            <span class="label">{overflow} more</span>
          </div>
        {/if}
      </div>
    {/if}
  </div>
</div>

<style>
  .campaign {
    padding: 0.65rem 0.9rem 0.75rem;
    flex-shrink: 0;
    border-top: 2px solid var(--rust-dark);
    contain: layout style;
  }

  .campaign-head {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    margin-bottom: 0.5rem;
  }

  .head-left {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
  }

  .campaign-title {
    font-family: var(--font-display);
    font-size: 0.72rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: var(--gold);
  }

  .campaign-count {
    font-size: 0.68rem;
    color: var(--text-muted);
    font-style: italic;
  }

  .route {
    overflow-x: auto;
    padding-bottom: 0.2rem;
  }

  .empty {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.78rem;
    font-style: italic;
    color: var(--text-muted);
    padding: 0.35rem 0;
  }

  .empty-icon {
    font-size: 1rem;
    opacity: 0.6;
  }

  .route-track {
    display: flex;
    align-items: center;
    gap: 0;
    min-width: min-content;
    padding: 0.25rem 0;
  }

  .waypoint {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    padding: 0.28rem 0.55rem 0.28rem 0.28rem;
    background: #252019;
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    flex-shrink: 0;
  }

  @media (hover: hover) {
    .waypoint:hover {
      border-color: var(--border-strong);
    }
  }

  .waypoint.start .marker {
    background: var(--gold);
    color: #2a231c;
  }

  .waypoint.end .marker {
    background: var(--rust-dark);
  }

  .waypoint.overflow .marker {
    background: var(--bg-elevated);
    color: var(--gold-light);
    font-size: 0.75rem;
  }

  .marker {
    width: 1.35rem;
    height: 1.35rem;
    display: flex;
    align-items: center;
    justify-content: center;
    font-family: var(--font-display);
    font-size: 0.62rem;
    font-weight: 700;
    color: var(--parchment);
    background: var(--rust);
    border-radius: 50%;
    flex-shrink: 0;
    border: 1px solid rgba(0, 0, 0, 0.25);
  }

  .label {
    font-size: 0.72rem;
    color: var(--parchment-dim);
    max-width: 8rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .trail {
    width: 1.25rem;
    height: 2px;
    flex-shrink: 0;
    background: repeating-linear-gradient(
      90deg,
      var(--gold) 0,
      var(--gold) 3px,
      transparent 3px,
      transparent 6px
    );
    opacity: 0.55;
    margin: 0 0.15rem;
  }
</style>
