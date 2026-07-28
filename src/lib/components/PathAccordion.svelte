<script lang="ts">
  let {
    gamePath,
    configPath,
  }: {
    gamePath: string;
    configPath: string;
  } = $props();

  let open = $state(false);
</script>

<div class="accordion">
  <button class="accordion-trigger" type="button" onclick={() => (open = !open)} aria-expanded={open}>
    <svg
      width="14"
      height="14"
      viewBox="0 0 14 14"
      fill="none"
      aria-hidden="true"
      class:rotated={open}
    >
      <path
        d="M2 4.5L7 9.5L12 4.5"
        stroke="currentColor"
        stroke-width="1.5"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
    </svg>
    <span class="trigger-label">Camp paths</span>
    {#if !open}
      <span class="path-preview">{gamePath.split("/").pop() ?? gamePath}</span>
    {/if}
  </button>

  {#if open}
    <div class="accordion-body">
      <div class="path-row">
        <span class="path-key">Campaign</span>
        <code class="path-val" title={gamePath}>{gamePath}</code>
      </div>
      <div class="path-row">
        <span class="path-key">Orders</span>
        <code class="path-val" title={configPath}>{configPath}</code>
      </div>
    </div>
  {/if}
</div>

<style>
  .accordion {
    overflow: hidden;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    background: rgba(0, 0, 0, 0.18);
  }

  .accordion-trigger {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.55rem 0.7rem;
    background: transparent;
    border: none;
    color: var(--text-secondary);
    font-size: 0.78rem;
    font-weight: 500;
    text-align: left;
    box-shadow: none;
  }

  .trigger-label {
    font-family: var(--font-display);
    font-size: 0.68rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--text-muted);
  }

  .accordion-trigger:hover {
    background: rgba(255, 255, 255, 0.03);
  }

  .accordion-trigger svg {
    transition: transform 0.2s ease;
    flex-shrink: 0;
    color: var(--gold-dim);
  }

  .accordion-trigger svg.rotated {
    transform: rotate(180deg);
  }

  .path-preview {
    margin-left: auto;
    font-family: var(--font-mono);
    font-size: 0.68rem;
    color: var(--text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 50%;
  }

  .accordion-body {
    padding: 0 0.7rem 0.65rem;
    display: flex;
    flex-direction: column;
    gap: 0.45rem;
    border-top: 1px solid var(--border-subtle);
    animation: fade-in 0.15s ease;
  }

  @keyframes fade-in {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }

  .path-row {
    display: flex;
    flex-direction: column;
    gap: 0.22rem;
    padding-top: 0.45rem;
  }

  .path-key {
    font-family: var(--font-display);
    font-size: 0.58rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--gold);
  }

  .path-val {
    font-size: 0.68rem;
    word-break: break-all;
    line-height: 1.4;
    padding: 0.38rem 0.48rem;
    background: rgba(0, 0, 0, 0.28);
    border-radius: 3px;
    border: 1px solid var(--border-subtle);
    display: block;
  }
</style>
