# Svelte Coding Standards

## Component Structure

```svelte
<script lang="ts">
  // 1. Imports
  import { invoke } from '@tauri-apps/api/core';

  // 2. Props (if any)
  interface Props {
    title: string;
    onClose?: () => void;
  }
  let { title, onClose }: Props = $props();

  // 3. State
  let isOpen = $state(false);

  // 4. Derived state
  let displayTitle = $derived(title.toUpperCase());

  // 5. Functions
  function handleClick() {
    isOpen = true;
  }
</script>

<!-- 6. Template -->
<div>
  <h1>{displayTitle}</h1>
  <button onclick={handleClick}>Open</button>
</div>

<!-- 7. Styles -->
<style>
  div {
    padding: 1rem;
  }
</style>
```

## Guidelines

- Use Svelte 5 runes (`$state`, `$derived`, `$effect`)
- Keep components focused and single-purpose
- Extract reusable logic into separate modules
- Use TypeScript for props interfaces

## Comments

Comments should add value that the code cannot convey on its own. Explaining
what the code does is redundant; explaining why it exists or why it's done a
particular way helps future maintainers.

### Inline comments

Inline comments start with lowercase and explain intent, constraints, or
non-obvious decisions. Never describe what the code is doing—the code shows
that.

```svelte
<script lang="ts">
  // wrong: restates the code
  let isOpen = $state(false); // initialise isOpen to false

  // correct: explains why
  let isOpen = $state(false); // closed by default, user must explicitly open

  // correct: documents a workaround
  const offset = position - 1; // Svelte bindings are 0-indexed, API is 1-indexed
</script>
```

### Component documentation

For complex components, add a comment block at the top of the script section
explaining the component's purpose and any non-obvious behaviour. This helps
developers understand the component without reading the entire implementation.

```svelte
<script lang="ts">
  // Wizard step that handles field mapping configuration.
  //
  // This step dynamically reconfigures the wizard based on the selected
  // source format—some formats require additional mapping steps while
  // others can skip directly to validation.

  import { getWizardContext } from './wizard-context.svelte.ts';
  // ...
</script>
```

Skip documentation when the component name and props are self-explanatory:

```svelte
<!-- no comment needed for a simple, well-named component -->
<script lang="ts">
  interface Props {
    label: string;
    onclick: () => void;
  }
  let { label, onclick }: Props = $props();
</script>

<button {onclick}>{label}</button>
```

## File Naming

Use `kebab-case.svelte` for component files:

```
ToolbarButton.svelte    # wrong
toolbar-button.svelte   # correct
toolbar_button.svelte   # correct (snake_case also acceptable)
```

## Related Documentation

- [TypeScript Standards](typescript.md) — General TS conventions
- [Frontend Architecture](../../architecture/frontend.md) — Component hierarchy
