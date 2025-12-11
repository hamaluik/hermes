# Best Practices

## Code Quality

Take advantage of the type systems in both TypeScript and Rust—they're there to
help catch bugs early. When errors happen (and they will), handle them
gracefully rather than letting things crash unexpectedly.

Sprinkle in log statements where they'd help with debugging later, and document
any complex logic or business rules that aren't immediately obvious from reading
the code. Try to keep functions focused on doing one thing well.

## Performance

For any I/O operations, stick with async/await to avoid blocking. Be thoughtful
about Svelte's reactivity—unnecessary re-renders add up. The schema cache in
this project is a good example of caching done right, and you should apply
similar thinking elsewhere: don't load resources until you actually need them.

## Security

Always validate user inputs before trusting them. On the network side, make sure
to validate host and port values before attempting any connections.

## Maintainability

Stick with the naming conventions already established in the project. When you
notice yourself repeating logic, pull it out into a shared function. Keep the
documentation current as you add features, and take a moment to review your own
code before committing—you'll often catch things you missed.

## Git Workflow

Write commit messages that actually describe what changed and why. Something
like `feat: add MLLP listen server for receiving messages` tells a story, while
`update stuff` leaves everyone guessing.

Commit in logical chunks—each commit should represent a coherent unit of work.
Work out a branching strategy with your team based on what fits your workflow.

Before you commit, run the checks:

```bash
pnpm check                     # TypeScript checks
cd src-tauri && cargo clippy   # Rust linting
```

## Documentation

When adding features, update:
- README.md if it affects setup or usage
- Architecture docs for architectural changes
- Add code comments for complex logic

## Related Documentation

- [Coding Standards](coding-standards/) — Language-specific conventions
- [Testing](testing.md) — Testing checklist
