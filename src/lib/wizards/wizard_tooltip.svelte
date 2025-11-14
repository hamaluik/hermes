<script lang="ts">
  let {
    title,
  }: {
    title: string;
  } = $props();
</script>

<span class="tooltip-icon" {title}>ⓘ</span>

<style>
  .tooltip-icon {
    display: inline-block;
    color: var(--col-subtle);
    font-size: 0.9em;
    cursor: help;
    transition: color 0.2s ease-in-out;

    &:hover {
      color: var(--col-iris);
    }
  }
</style>
