<!--
  Theme Toggle - 3-Way Switch Component

  A segmented toggle control for selecting between Light, Auto (system), and Dark themes.
  Designed to match the visual style of the existing ToggleSwitch component.

  ## Why Radio Buttons?

  The component uses hidden radio buttons as its foundation for several reasons:
  - Native form semantics: Works correctly in forms, supports keyboard navigation
  - Accessibility: Screen readers announce it as a radio group with clear options
  - Mutual exclusivity: Radio buttons naturally enforce single selection

  ## CSS-Only Animation

  The sliding thumb animation is achieved purely through CSS using sibling selectors.
  When a radio button is checked, CSS rules select the thumb element and apply
  a transform to position it over the active option. This avoids JavaScript for
  the visual feedback, keeping the component simple and performant.

  ## Icon Highlighting

  The active option's icon changes color to contrast against the thumb background.
  This is done by targeting labels based on their position (nth-child) when the
  corresponding radio is checked. The thumb is child 1, so labels are children 2-4.
-->
<script lang="ts">
  import IconModeLight from "$lib/icons/IconModeLight.svelte";
  import IconModeDark from "$lib/icons/IconModeDark.svelte";
  import IconModeAuto from "$lib/icons/IconModeAuto.svelte";

  let {
    value = $bindable<"light" | "dark" | "auto">("auto"),
    id,
  }: {
    value: "light" | "dark" | "auto";
    id: string;
  } = $props();
</script>

<div class="theme-toggle" role="radiogroup" aria-label="Theme selection">
  <input
    type="radio"
    {id}
    name={id}
    value="light"
    bind:group={value}
    aria-label="Light theme"
  />
  <input
    type="radio"
    id="{id}-auto"
    name={id}
    value="auto"
    bind:group={value}
    aria-label="Auto theme (follow system)"
  />
  <input
    type="radio"
    id="{id}-dark"
    name={id}
    value="dark"
    bind:group={value}
    aria-label="Dark theme"
  />
  <div class="toggle-track">
    <span class="toggle-thumb"></span>
    <label for={id} class="toggle-option" title="Light">
      <IconModeLight />
    </label>
    <label for="{id}-auto" class="toggle-option" title="Auto (System)">
      <IconModeAuto />
    </label>
    <label for="{id}-dark" class="toggle-option" title="Dark">
      <IconModeDark />
    </label>
  </div>
</div>

<style>
  .theme-toggle {
    position: relative;
    display: inline-flex;

    input[type="radio"] {
      position: absolute;
      opacity: 0;
      width: 0;
      height: 0;

      &:focus-visible ~ .toggle-track {
        box-shadow: 0 0 0 2px var(--col-iris);
      }
    }

    /* Position thumb based on which radio is checked */
    input[value="light"]:checked ~ .toggle-track .toggle-thumb {
      transform: translateX(0);
    }

    input[value="auto"]:checked ~ .toggle-track .toggle-thumb {
      transform: translateX(100%);
    }

    input[value="dark"]:checked ~ .toggle-track .toggle-thumb {
      transform: translateX(200%);
    }

    /* Highlight active icon */
    input[value="light"]:checked ~ .toggle-track label:nth-child(2) {
      color: var(--col-base);
    }

    input[value="auto"]:checked ~ .toggle-track label:nth-child(3) {
      color: var(--col-base);
    }

    input[value="dark"]:checked ~ .toggle-track label:nth-child(4) {
      color: var(--col-base);
    }
  }

  .toggle-track {
    display: flex;
    align-items: center;
    background-color: var(--col-highlightMed);
    border-radius: 1.75em;
    border: 1px solid var(--col-highlightHigh);
    padding: 0.2em;
    position: relative;
    cursor: pointer;
  }

  .toggle-thumb {
    position: absolute;
    width: calc((100% - 0.4em) / 3);
    height: calc(100% - 0.4em);
    background-color: var(--col-iris);
    border-radius: 1.5em;
    transition: transform 0.2s ease-in-out;
    left: 0.2em;
    z-index: 0;
  }

  .toggle-option {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 2em;
    height: 1.5em;
    cursor: pointer;
    color: var(--col-subtle);
    transition: color 0.2s ease-in-out;
    z-index: 1;

    &:hover {
      color: var(--col-text);
    }

    :global(svg) {
      width: 1em;
      height: 1em;
    }
  }
</style>
