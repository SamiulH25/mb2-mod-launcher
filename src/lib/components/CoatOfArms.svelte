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
  <path
    d="M24 3L42 10V22C42 33 33 41 24 45C15 41 6 33 6 22V10L24 3Z"
    fill="#2a231c"
    stroke="#b8954a"
    stroke-width="1.5"
  />
  <path d="M24 8L36 13V22C36 30 30 36 24 39C18 36 12 30 12 22V13L24 8Z" fill="#a63d2f" />
  <path
    d="M24 14L30 22L24 30L18 22L24 14Z"
    fill="#d4b56a"
    stroke="#b8954a"
    stroke-width="0.75"
  />
  <path d="M24 18V26M20 22H28" stroke="#2a231c" stroke-width="1.2" stroke-linecap="round" />
</svg>

<style>
  .crest {
    flex-shrink: 0;
  }
</style>
