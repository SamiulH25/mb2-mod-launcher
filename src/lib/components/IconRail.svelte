<script lang="ts">
  import ShieldLogo from "./ShieldLogo.svelte";

  let {
    onDetect,
    loading = false,
  }: {
    onDetect?: () => void;
    loading?: boolean;
  } = $props();
</script>

<nav class="rail" aria-label="Main navigation">
  <button class="rail-logo" onclick={onDetect} disabled={loading} title="Detect game">
    <ShieldLogo size={28} />
  </button>
</nav>

<style>
  .rail {
    width: 3.25rem;
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 0.65rem 0;
    background: var(--bg-shell);
    border-right: 1px solid var(--border-subtle);
  }

  .rail-logo {
    padding: 0.25rem;
    background: transparent;
    border: none;
    border-radius: var(--radius-sm);
  }

  .rail-logo:hover:not(:disabled) {
    background: var(--bg-hover);
  }
</style>
