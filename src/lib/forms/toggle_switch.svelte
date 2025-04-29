<script lang="ts">
  let {
    checked = $bindable(false),
    id,
    onchange,
    disabled,
  }: {
    checked: boolean;
    id: string;
    onchange?: (checked: boolean) => void;
    disabled?: boolean;
  } = $props();

  const toggle = (event: Event) => {
    const input = event.target as HTMLInputElement;
    checked = input.checked;
    onchange?.(checked);
  };
</script>

<label class="toggle-switch">
  <input type="checkbox" {id} {checked} onchange={toggle} {disabled} />
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
