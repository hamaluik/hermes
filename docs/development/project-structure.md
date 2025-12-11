# Project Structure

## Frontend

```
src/
├── routes/              # SvelteKit pages
├── lib/                 # Feature-based organisation
│   ├── communication/   # MLLP send/receive (components + Tauri bridges)
│   ├── editor/          # Message editor (components + bridges)
│   ├── diff/            # Message comparison
│   ├── find_replace/    # Search functionality
│   ├── validation/      # Validation UI and logic
│   ├── settings/        # Settings UI
│   ├── modals/          # Standalone modals
│   ├── shared/          # Cross-feature utilities
│   ├── tabs/            # Tab navigation
│   ├── toolbar/         # Toolbar components
│   ├── forms/           # Form inputs
│   ├── components/      # Generic UI primitives
│   └── icons/           # SVG icon components
└── settings.ts          # Settings persistence
```

## Backend

```
src-tauri/src/
├── lib.rs               # Entry point
├── menu/                # Native menu system
├── commands/            # Tauri commands (organised by feature)
│   ├── communication/   # send.rs, listen.rs
│   ├── editor/          # cursor.rs, data.rs, syntax_highlight.rs
│   ├── validation/      # validate.rs, diff.rs
│   └── support/         # field_description.rs, schema.rs
├── schema/              # Schema caching system
└── spec/                # HL7 specifications
```

## Module Organisation

TypeScript modules should export related functions together, use named exports
(avoid default exports), and stay focused on a single responsibility.

Rust modules should have one module per major feature, with the public API in
`mod.rs` and internal helpers in separate files.

## Naming Conventions for Features

When adding a new feature, create a `src-tauri/src/commands/feature_name/`
directory on the backend and a corresponding `src/lib/feature_name/` directory
on the frontend containing both components and Tauri bridges.

Use `snake_case` for Tauri commands (e.g., `get_field_description`) and
`camelCase` for the TypeScript wrapper functions (e.g., `getFieldDescription`).
For imports, use `$lib/` paths for cross-directory references and relative paths
within the same feature.

## Related Documentation

- [Adding Features](adding-features.md) — Step-by-step feature implementation
- [TypeScript Standards](coding-standards/typescript.md) — Naming conventions
- [Rust Standards](coding-standards/rust.md) — Module conventions
