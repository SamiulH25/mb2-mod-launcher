<script lang="ts">
  import type { ModuleState } from "../types";
  import { OFFICIAL_MODULE_IDS } from "../constants";
  import Toggle from "./Toggle.svelte";
  import SourceBadge from "./SourceBadge.svelte";

  let {
    module,
    index,
    expanded = false,
    dragging = false,
    dragOver = false,
    ontoggle,
    onexpand,
    ondragstart,
    ondragover,
    ondragleave,
    ondrop,
  }: {
    module: ModuleState;
    index: number;
    expanded?: boolean;
    dragging?: boolean;
    dragOver?: boolean;
    ontoggle?: () => void;
    onexpand?: () => void;
    ondragstart?: () => void;
    ondragover?: (e: DragEvent) => void;
    ondragleave?: () => void;
    ondrop?: () => void;
  } = $props();

  const isOfficial = $derived(OFFICIAL_MODULE_IDS.has(module.module.info.id));
  const hasDeps = $derived(module.module.info.depended_modules.length > 0);
</script>

<article
  class="roster-row"
  class:disabled={!module.enabled}
  class:expanded
  class:dragging
  class:drag-over={dragOver}
  class:official={isOfficial}
  draggable="true"
  role="listitem"
  ondragstart={ondragstart}
  ondragover={ondragover}
  ondragleave={ondragleave}
  ondrop={ondrop}
>
  <span class="rank">{index + 1}</span>

  <button class="roster-main" type="button" onclick={onexpand} aria-expanded={expanded}>
    <span class="name">{module.module.info.name}</span>
    <span class="id">{module.module.info.id}</span>
    {#if hasDeps && !expanded}
      <span class="dep-line">
        Requires: {module.module.info.depended_modules.map((d) => d.id).join(", ")}
      </span>
    {/if}
  </button>

  {#if isOfficial}
    <span class="crest-tag">Crown</span>
  {:else}
    <span class="crest-tag empty"></span>
  {/if}

  <span class="version">{module.module.info.version ?? "—"}</span>
  <SourceBadge source={module.module.source} />

  <span class="drag-handle" aria-hidden="true" title="Drag to reorder">☰</span>
  <Toggle checked={module.enabled} onchange={ontoggle} />

  {#if expanded && hasDeps}
    <div class="dep-detail">
      <span class="dep-label">Allied modules required</span>
      <div class="dep-chips">
        {#each module.module.info.depended_modules as dep}
          <span class="dep-chip">{dep.id}</span>
        {/each}
      </div>
    </div>
  {/if}
</article>

<style>
  .roster-row {
    display: grid;
    grid-template-columns: 2.25rem 1fr 3.5rem auto auto 1.25rem 2.5rem;
    grid-template-rows: auto auto;
    gap: 0 0.6rem;
    align-items: center;
    padding: 0.5rem 0.75rem;
    border-bottom: 1px solid var(--border-subtle);
    background: var(--bg-panel);
    border-left: 3px solid transparent;
    contain: layout style;
  }

  .roster-row:last-child {
    border-bottom: none;
  }

  .roster-row:nth-child(even) {
    background: #211c17;
  }

  .roster-row:hover {
    background: var(--bg-hover);
  }

  .roster-row:not(.disabled) {
    border-left-color: var(--rust);
  }

  .roster-row.official:not(.disabled) {
    border-left-color: var(--gold);
  }

  .roster-row.disabled {
    border-left-color: var(--border-subtle);
    background: #1c1814;
  }

  .roster-row.disabled .name {
    color: var(--text-muted);
    font-weight: 500;
  }

  .roster-row.disabled .id,
  .roster-row.disabled .dep-line {
    color: #6e6458;
  }

  .roster-row.disabled:hover {
    background: #201b16;
  }

  .roster-row.dragging {
    opacity: 0.55;
  }

  .roster-row.drag-over {
    outline: 2px solid var(--gold);
    outline-offset: -2px;
  }

  .rank {
    font-family: var(--font-display);
    font-size: 0.85rem;
    font-weight: 700;
    color: var(--gold);
    text-align: center;
  }

  .roster-row.disabled .rank {
    color: var(--text-muted);
  }

  .roster-main {
    min-width: 0;
    padding: 0;
    background: transparent;
    border: none;
    text-align: left;
    color: inherit;
    cursor: pointer;
    display: flex;
    flex-direction: column;
    gap: 0.08rem;
  }

  .roster-main:hover {
    background: transparent;
  }

  .name {
    font-family: var(--font-display);
    font-size: 0.9rem;
    font-weight: 600;
    color: var(--parchment);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .id {
    font-size: 0.7rem;
    color: var(--text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .dep-line {
    font-size: 0.68rem;
    color: var(--text-muted);
    font-style: italic;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    margin-top: 0.1rem;
  }

  .crest-tag {
    font-family: var(--font-display);
    font-size: 0.58rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    padding: 0.12rem 0.3rem;
    text-align: center;
    border-radius: 2px;
    background: #3a3020;
    color: var(--gold-light);
    border: 1px solid var(--border-strong);
  }

  .crest-tag.empty {
    visibility: hidden;
  }

  .version {
    font-size: 0.68rem;
    color: var(--text-muted);
    white-space: nowrap;
  }

  .drag-handle {
    color: var(--text-muted);
    font-size: 0.8rem;
    cursor: grab;
    user-select: none;
    text-align: center;
  }

  .dep-detail {
    grid-column: 1 / -1;
    padding: 0.35rem 0 0.1rem 2.85rem;
    border-top: 1px dashed var(--border-subtle);
    margin-top: 0.25rem;
    padding-top: 0.45rem;
  }

  .dep-label {
    display: block;
    font-family: var(--font-display);
    font-size: 0.62rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--gold);
    margin-bottom: 0.35rem;
  }

  .dep-chips {
    display: flex;
    flex-wrap: wrap;
    gap: 0.3rem;
  }

  .dep-chip {
    font-size: 0.68rem;
    padding: 0.12rem 0.4rem;
    border-radius: 2px;
    background: var(--bg-base);
    border: 1px solid var(--border-default);
    color: var(--parchment-dim);
  }
</style>
