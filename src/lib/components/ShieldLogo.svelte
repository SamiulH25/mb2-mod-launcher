<script lang="ts">
  let { size = 40 }: { size?: number } = $props();
</script>

<svg
  class="shield-logo"
  width={size}
  height={size}
  viewBox="0 0 40 40"
  fill="none"
  xmlns="http://www.w3.org/2000/svg"
  aria-hidden="true"
>
  <defs>
    <linearGradient id="shield-fill" x1="20" y1="2" x2="20" y2="38" gradientUnits="userSpaceOnUse">
      <stop stop-color="#c4783a" />
      <stop offset="1" stop-color="#8b4513" />
    </linearGradient>
    <linearGradient id="shield-shine" x1="12" y1="6" x2="28" y2="22" gradientUnits="userSpaceOnUse">
      <stop stop-color="#e8d48a" stop-opacity="0.5" />
      <stop offset="1" stop-color="#e8d48a" stop-opacity="0" />
    </linearGradient>
  </defs>
  <path
    d="M20 2L34 8.5V19C34 28.5 27.5 35.5 20 38C12.5 35.5 6 28.5 6 19V8.5L20 2Z"
    fill="url(#shield-fill)"
    stroke="rgba(201,162,39,0.5)"
    stroke-width="1"
  />
  <path
    d="M20 2L34 8.5V19C34 28.5 27.5 35.5 20 38C12.5 35.5 6 28.5 6 19V8.5L20 2Z"
    fill="url(#shield-shine)"
  />
  <path
    d="M14 18L18.5 22.5L26.5 14.5"
    stroke="#fff"
    stroke-width="2.2"
    stroke-linecap="round"
    stroke-linejoin="round"
  />
</svg>

<style>
  .shield-logo {
    flex-shrink: 0;
    filter: drop-shadow(0 2px 6px rgba(160, 82, 45, 0.4));
  }
</style>
