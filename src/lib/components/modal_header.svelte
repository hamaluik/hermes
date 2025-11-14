<script lang="ts">
  import IconClose from "../icons/IconClose.svelte";
  import type { Snippet } from "svelte";

  let {
    onclose,
    children,
  }: {
    onclose?: () => void;
    children: Snippet<[]>;
  } = $props();
</script>

<header>
  <h1>
    {@render children()}
  </h1>
  <button class="close" onclick={onclose}>
    <IconClose />
  </button>
</header>

<style>
  header {
    width: 100%;
    border-radius: 8px 8px 0 0;
    display: flex;
    flex-direction: row;
    align-items: stretch;
    justify-content: space-between;
    margin: 0;
    padding: 0;
    background: none;

    h1 {
      font-size: medium;
      font-weight: 700;
      padding: 0.5em 1ch;

      display: inline-flex;
      flex-direction: row;
      align-items: center;
      gap: 1ch;
    }

    button.close {
      background: transparent;
      border: none;
      cursor: pointer;
      color: var(--col-text);
      padding: 0.25em 1ch;

      &:hover {
        color: var(--col-love);
      }
    }
  }
</style>
