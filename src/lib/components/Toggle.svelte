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
    width: 2.35rem;
    height: 1.3rem;
    padding: 0;
    border-radius: 999px;
    background: rgba(0, 0, 0, 0.35);
    border: 1px solid var(--border-default);
    box-shadow: inset 0 1px 2px rgba(0, 0, 0, 0.3);
  }

  .toggle.on {
    background: var(--rust);
    border-color: var(--rust-dark);
  }

  .toggle:disabled {
    opacity: 0.35;
  }

  .toggle:focus-visible {
    outline: 2px solid var(--gold-dim);
    outline-offset: 2px;
  }

  .thumb {
    position: absolute;
    top: 2px;
    left: 2px;
    width: calc(1.3rem - 6px);
    height: calc(1.3rem - 6px);
    border-radius: 50%;
    background: var(--parchment);
  }

  .toggle.on .thumb {
    transform: translateX(1.05rem);
  }
</style>
