<script lang="ts">
  import type { ModuleState } from "../types";

  let { modules }: { modules: ModuleState[] } = $props();

  const enabledModules = $derived(modules.filter((m) => m.enabled));
</script>

<div class="campaign panel-inset">
  <div class="campaign-head">
    <span class="campaign-title">Campaign Route</span>
    <span class="campaign-count">{enabledModules.length} waypoints</span>
  </div>

  <div class="route" role="list" aria-label="Module load order">
    {#if enabledModules.length === 0}
      <span class="empty">No modules enlisted — enable mods to chart your route</span>
    {:else}
      {#each enabledModules as mod, i (mod.module.info.id)}
        <div class="waypoint" role="listitem" title={mod.module.info.id}>
          <span class="marker">{i + 1}</span>
          <span class="label">{mod.module.info.name}</span>
        </div>
        {#if i < enabledModules.length - 1}
          <span class="trail" aria-hidden="true">━━▸</span>
        {/if}
      {/each}
    {/if}
  </div>
</div>

<style>
  .campaign {
    padding: 0.6rem 0.85rem;
    flex-shrink: 0;
    border-top: 2px solid var(--rust-dark);
  }

  .campaign-head {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    margin-bottom: 0.45rem;
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
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.2rem 0.15rem;
    overflow-x: auto;
    padding-bottom: 0.15rem;
  }

  .empty {
    font-size: 0.78rem;
    font-style: italic;
    color: var(--text-muted);
  }

  .waypoint {
    display: flex;
    align-items: center;
    gap: 0.35rem;
    padding: 0.2rem 0.45rem 0.2rem 0.2rem;
    background: var(--bg-base);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    flex-shrink: 0;
  }

  .marker {
    width: 1.25rem;
    height: 1.25rem;
    display: flex;
    align-items: center;
    justify-content: center;
    font-family: var(--font-display);
    font-size: 0.65rem;
    font-weight: 700;
    color: var(--parchment);
    background: var(--rust);
    border-radius: 50%;
    flex-shrink: 0;
  }

  .label {
    font-size: 0.72rem;
    color: var(--parchment-dim);
    max-width: 7rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .trail {
    color: var(--gold);
    font-size: 0.65rem;
    opacity: 0.7;
    flex-shrink: 0;
    margin: 0 0.1rem;
  }
</style>
