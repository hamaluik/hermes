# Troubleshooting Development Issues

## Hot Reload Not Working

1. Restart dev server
2. Clear browser cache
3. Check for syntax errors in console

## Rust Compilation Errors

```bash
cd src-tauri
cargo clean
cargo build
```

## TypeScript Errors

```bash
pnpm check
```

Fix reported errors before committing.

## Schema Not Loading

- Verify `messages.toml` syntax
- Check logs for parse errors
- Ensure file is in correct location

## Related Documentation

- [Workflow](workflow.md) — Dev mode and building
- [Common Tasks](common-tasks.md) — Viewing logs
