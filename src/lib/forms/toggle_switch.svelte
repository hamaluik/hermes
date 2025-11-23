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
  - Checked: Pine (green) background, circle slides right
  - Disabled: Muted colours, cursor changes to not-allowed
  - Focused: Blue outline for keyboard navigation
-->
<script lang="ts">
  let {
    checked = $bindable(false),
    id,
    onchange,
    disabled,
    onfocus,
    onblur,
  }: {
    checked: boolean;
    id: string;
    onchange?: (checked: boolean) => void;
    disabled?: boolean;
    onfocus?: (event: Event) => void;
    onblur?: (event: Event) => void;
  } = $props();

  const toggle = (event: Event) => {
    const input = event.target as HTMLInputElement;
    checked = input.checked;
    onchange?.(checked);
  };
</script>

<label class="toggle-switch">
  <input
    type="checkbox"
    {id}
    {checked}
    onchange={toggle}
    {disabled}
    {onfocus}
    {onblur}
  />
  <span class="slider"></span>
</label>

<style>
  .toggle-switch {
    position: relative;
    display: inline-block;
    width: 2.125rem;
    height: 1.25rem;
  }

  .toggle-switch input {
    opacity: 0;
    width: 0;
    height: 0;
  }

  .slider {
    position: absolute;
    cursor: pointer;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: var(--col-subtle);
    transition: 0.4s;
    border-radius: 1.25rem;
  }

  input:focus + .slider {
    outline: 2px solid var(--col-iris);
  }

  input:checked + .slider {
    background-color: var(--col-pine);
  }

  input:disabled + .slider {
    background-color: var(--col-subtle);
    cursor: not-allowed;
  }

  /* the circle inside the slider */
  .slider:before {
    position: absolute;
    content: "";
    height: 0.875rem;
    width: 0.875rem;
    left: 0.1875rem;
    bottom: 0.1875rem;
    background-color: var(--col-text);
    transition: 0.4s;
    border-radius: 50%;
  }

  input:checked + .slider:before {
    background-color: var(--col-foam);
  }

  input:disabled + .slider:before {
    background-color: var(--col-muted);
  }

  input:checked + .slider:before {
    transform: translateX(0.875rem);
  }
</style>
