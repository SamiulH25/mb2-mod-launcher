<script lang="ts">
  let {
    checked = false,
    disabled = false,
    onchange,
  }: {
    checked?: boolean;
    disabled?: boolean;
    onchange?: () => void;
  } = $props();
</script>

<button
  type="button"
  class="toggle"
  class:on={checked}
  {disabled}
  onclick={onchange}
  role="switch"
  aria-checked={checked}
  aria-label={checked ? "Disable module" : "Enable module"}
>
  <span class="thumb"></span>
</button>

<style>
  .toggle {
    position: relative;
    width: 2.25rem;
    height: 1.25rem;
    padding: 0;
    border-radius: 999px;
    background: var(--bg-base);
    border: 1px solid var(--border-default);
  }

  .toggle.on {
    background: var(--rust);
    border-color: var(--rust-dark);
  }

  .toggle:disabled {
    opacity: 0.35;
  }

  .thumb {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 0.9rem;
    height: 0.9rem;
    border-radius: 50%;
    background: var(--parchment);
  }

  .toggle.on .thumb {
    transform: translateX(1rem);
  }
</style>
