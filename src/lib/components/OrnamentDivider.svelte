<script lang="ts">
  let { label = "" }: { label?: string } = $props();
</script>

<div class="ornament" role="presentation">
  <span class="line"></span>
  {#if label}
    <span class="label">{label}</span>
  {/if}
  <span class="line"></span>
</div>

<style>
  .ornament {
    display: flex;
    align-items: center;
    gap: 0.65rem;
    padding: 0.15rem 0;
  }

  .line {
    flex: 1;
    height: 1px;
    background: linear-gradient(
      90deg,
      transparent,
      var(--border-strong) 20%,
      var(--gold) 50%,
      var(--border-strong) 80%,
      transparent
    );
    opacity: 0.65;
  }

  .label {
    font-family: var(--font-display);
    font-size: 0.58rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.14em;
    color: var(--text-muted);
    white-space: nowrap;
  }
</style>
