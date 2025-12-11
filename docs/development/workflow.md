# Development Workflow

## Starting Development

```bash
# start the application in dev mode (hot-reload enabled)
pnpm tauri dev
```

This command:
1. Starts Vite dev server (frontend)
2. Launches Tauri application (desktop wrapper)
3. Enables hot-reload for frontend changes
4. Automatically rebuilds Rust on backend changes

## Type Checking

Run TypeScript checks without building:

```bash
pnpm check          # one-time check
pnpm check:watch    # continuous checking
```

## Building for Production

```bash
pnpm tauri build
```

Outputs to `src-tauri/target/release/bundle/`

## Related Documentation

- [Setup](setup.md) — Initial project setup
- [Common Tasks](common-tasks.md) — Dependency management and debugging
- [Troubleshooting](troubleshooting.md) — Hot reload and build issues
