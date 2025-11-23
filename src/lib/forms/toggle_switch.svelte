<!--
  Toggle Switch Component

  Styled checkbox input that visually represents on/off state with an animated
  sliding switch. Provides better visual feedback than standard checkboxes for
  boolean settings.

  The component wraps a native checkbox input (hidden via opacity: 0) with a
  custom styled slider. This approach maintains accessibility (screen readers,
  keyboard navigation) while providing enhanced visuals.

  Visual states:
  - Unchecked: Grey background, circle on left
  - Checked: Iris (purple) background, circle slides right
  - Focused: Iris outline for keyboard navigation
  - Hover: Slightly lighter background
-->
<script lang="ts">
  let {
    checked = $bindable(false),
    id,
  }: {
    checked: boolean;
    id: string;
  } = $props();
</script>

<label class="toggle-switch">
  <input type="checkbox" {id} bind:checked />
  <span class="toggle-slider"></span>
</label>

<style>
  .toggle-switch {
    position: relative;
    display: inline-block;
    width: 3em;
    height: 1.75em;

    input[type="checkbox"] {
      opacity: 0;
      width: 0;
      height: 0;

      &:checked + .toggle-slider {
        background-color: var(--col-iris);
      }

      &:checked + .toggle-slider::before {
        transform: translateX(1.25em);
      }

      &:focus + .toggle-slider {
        box-shadow: 0 0 0 2px var(--col-iris);
      }
    }

    .toggle-slider {
      position: absolute;
      cursor: pointer;
      top: 0;
      left: 0;
      right: 0;
      bottom: 0;
      background-color: var(--col-highlightMed);
      transition: 0.2s;
      border-radius: 1.75em;
      border: 1px solid var(--col-highlightHigh);

      &::before {
        position: absolute;
        content: "";
        height: 1.25em;
        width: 1.25em;
        left: 0.25em;
        bottom: 0.125em;
        background-color: var(--col-base);
        transition: 0.2s;
        border-radius: 50%;
      }

      &:hover {
        background-color: var(--col-highlightHigh);
      }
    }
  }
</style>
