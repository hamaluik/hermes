# Development Setup

## Prerequisites

Ensure you have all required tools installed (see
[README.md](../../README.md#prerequisites)):

- Node.js (v18+)
- pnpm (v8+)
- Rust (latest stable)
- Platform-specific build tools

## Initial Setup

```bash
# clone and install
git clone git@github.com:hamaluik/hermes.git
cd hermes
pnpm install

# verify installation
pnpm check                   # TypeScript
cd src-tauri && cargo check  # Rust
```

## Related Documentation

- [Workflow](workflow.md) — Starting the dev server and building
- [Troubleshooting](troubleshooting.md) — Common setup issues
