<!--
  Reusable Button Component

  A consistent button component with four variants for different use cases:
  - primary: Main actions like Save, Add (pine background, gold hover)
  - secondary: Secondary actions like Reload, Apply (iris background, gold hover)
  - ghost: Dismissive actions like Cancel, Close (transparent, subtle hover)
  - danger: Destructive actions like Delete (transparent, red hover)

  Usage:
    <Button variant="primary" onclick={handleSave}>
      <IconSave />
      <span>Save</span>
    </Button>

    <Button variant="danger" iconOnly onclick={handleDelete} title="Delete">
      <IconDelete />
    </Button>
-->
<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    variant = "ghost",
    disabled = false,
    type = "button",
    onclick,
    title,
    iconOnly = false,
    children,
  }: {
    variant?: "primary" | "secondary" | "ghost" | "danger";
    disabled?: boolean;
    type?: "button" | "submit" | "reset";
    onclick?: () => void;
    title?: string;
    iconOnly?: boolean;
    children: Snippet;
  } = $props();
</script>

<button
  class="btn {variant}"
  class:icon-only={iconOnly}
  {type}
  {disabled}
  {onclick}
  {title}
>
  {@render children()}
</button>

<style>
  .btn {
    display: inline-flex;
    flex-direction: row;
    align-items: center;
    justify-content: center;
    gap: 0.5ch;
    padding: 0.5rem 1rem;
    border-radius: 4px;
    font-size: 0.9rem;
    cursor: pointer;
    transition: all 0.2s;
    border: 1px solid var(--col-highlightHigh);

    &:disabled {
      opacity: 0.5;
      cursor: not-allowed;
    }

    &.icon-only {
      padding: 0.5rem;
    }
  }

  /* primary: pine background, gold hover */
  .primary {
    background: var(--col-pine);
    color: var(--col-base);
    border-color: var(--col-pine);

    &:hover:not(:disabled) {
      background: var(--col-gold);
      border-color: var(--col-gold);
    }
  }

  /* secondary: iris background, gold hover */
  .secondary {
    background: var(--col-iris);
    color: var(--col-base);
    border-color: var(--col-iris);

    &:hover:not(:disabled) {
      background: var(--col-gold);
      border-color: var(--col-gold);
    }
  }

  /* ghost: transparent, subtle hover */
  .ghost {
    background: transparent;
    color: var(--col-text);

    &:hover:not(:disabled) {
      background: var(--col-highlightLow);
    }
  }

  /* danger: transparent, red hover */
  .danger {
    background: transparent;
    color: var(--col-subtle);
    border-color: var(--col-highlightMed);

    &:hover:not(:disabled) {
      background: var(--col-love);
      border-color: var(--col-love);
      color: var(--col-base);
    }
  }

  /* dark theme: primary needs lighter text on dark pine background */
  :global(html[data-theme="dark"]) .primary {
    color: var(--col-text);
  }

  @media (prefers-color-scheme: dark) {
    :global(html[data-theme="auto"]) .primary {
      color: var(--col-text);
    }
  }
</style>
