<script lang="ts">
  let { source }: { source: string } = $props();

  const label = $derived(
    source === "workshop" ? "Workshop" : source === "manual" ? "Manual" : "Game",
  );
</script>

<span
  class="badge"
  class:workshop={source === "workshop"}
  class:game={source === "game"}
  class:manual={source === "manual"}
>
  {label}
</span>

<style>
  .badge {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 4.5rem;
    padding: 0.18rem 0.4rem;
    border-radius: 2px;
    font-family: var(--font-display);
    font-size: 0.58rem;
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    border: 1px solid var(--border-default);
    white-space: nowrap;
  }

  .badge.game {
    background: #2a2820;
    color: var(--gold-light);
  }

  .badge.workshop {
    background: #1e2828;
    color: #8ec8c0;
    border-color: #3a4a48;
  }

  .badge.manual {
    background: #28202a;
    color: #c8a8d8;
    border-color: #4a3a4a;
  }
</style>
