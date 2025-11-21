<!--
  Notification Icon Component

  Wraps an icon (typically a toolbar button icon) with an optional notification
  badge showing a count. Used for features that accumulate items requiring user
  attention (e.g., unread messages in the Listen feature).

  ## Badge Positioning

  The badge is absolutely positioned at the top-right corner of the icon, extending
  slightly beyond the icon's bounds (-5px offset). This standard notification pattern
  makes the badge immediately visible without obscuring the icon itself.

  ## Conditional Display

  The badge only renders when count > 0. This avoids visual clutter when there are
  no notifications, and provides clear visual feedback when notifications arrive
  (the badge suddenly appears).

  ## Circular Badge

  The badge is circular (border-radius: 50%, aspect-ratio: 1/1) which is a common
  notification pattern across platforms. The size (1lh = 1 line-height) ensures
  it scales proportionally with the UI font size.
-->
<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    count,
    children,
  }: {
    count: number;
    children: Snippet;
  } = $props();
</script>

<div class="notification-icon">
  {@render children?.()}
  {#if count > 0}
    <div class="notification-count">{count}</div>
  {/if}
</div>

<style>
  .notification-icon {
    position: relative;
    display: inline-block;
  }

  .notification-count {
    position: absolute;
    top: -5px;
    right: -5px;
    background-color: var(--col-love);
    color: var(--col-text);
    border-radius: 50%;
    aspect-ratio: 1 / 1;
    height: 1lh;
    font-size: x-small;

    display: inline-flex;
    align-items: center;
    justify-content: center;
  }
</style>
