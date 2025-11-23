<!--
  Toolbar Separator

  Visual separator (vertical line) for grouping related toolbar buttons.
  Used to create distinct sections in the toolbar (e.g., File operations | Settings).
-->
<div class="separator"></div>

<style>
  .separator {
    margin: 0 -0.5rem;
  }
  .separator::after {
    content: "";
    height: 120%;
    background-color: transparent;
    margin: 0 0.5rem;
    border-left: 1px solid var(--col-muted);
  }
</style>
