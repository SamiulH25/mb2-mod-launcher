<script lang="ts">
  let { size = 40 }: { size?: number } = $props();
</script>

<svg
  class="crest"
  width={size}
  height={size}
  viewBox="0 0 48 48"
  fill="none"
  xmlns="http://www.w3.org/2000/svg"
  aria-hidden="true"
>
  <defs>
    <linearGradient id="crest-shield" x1="24" y1="3" x2="24" y2="45" gradientUnits="userSpaceOnUse">
      <stop stop-color="#3a3028" />
      <stop offset="1" stop-color="#1f1a15" />
    </linearGradient>
    <linearGradient id="crest-inner" x1="24" y1="8" x2="24" y2="39" gradientUnits="userSpaceOnUse">
      <stop stop-color="#c45444" />
      <stop offset="1" stop-color="#7a2e24" />
    </linearGradient>
    <linearGradient id="crest-gem" x1="24" y1="14" x2="24" y2="30" gradientUnits="userSpaceOnUse">
      <stop stop-color="#e8d080" />
      <stop offset="1" stop-color="#b8954a" />
    </linearGradient>
  </defs>
  <path
    d="M24 3L42 10V22C42 33 33 41 24 45C15 41 6 33 6 22V10L24 3Z"
    fill="url(#crest-shield)"
    stroke="#b8954a"
    stroke-width="1.5"
  />
  <path d="M24 8L36 13V22C36 30 30 36 24 39C18 36 12 30 12 22V13L24 8Z" fill="url(#crest-inner)" />
  <path
    d="M24 14L30 22L24 30L18 22L24 14Z"
    fill="url(#crest-gem)"
    stroke="#d4b56a"
    stroke-width="0.75"
  />
  <path d="M24 18V26M20 22H28" stroke="#2a231c" stroke-width="1.2" stroke-linecap="round" />
</svg>

<style>
  .crest {
    flex-shrink: 0;
    filter: drop-shadow(0 2px 4px rgba(0, 0, 0, 0.45));
  }
</style>
