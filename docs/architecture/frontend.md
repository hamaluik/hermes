# Frontend Architecture

The frontend is built using Svelte 5 with SvelteKit, leveraging modern reactive
programming patterns.

## Component Hierarchy

```
+page.svelte (Main Application)
├── Toolbar
├── FindReplaceBar
├── Tabs (Segment Navigation)
├── SegmentTab (Active Segment Form)
├── MessageEditor (Raw HL7 Text)
├── ValidationPanel
├── CursorDescription (Field Info)
├── CommunicationDrawer
└── Modals (Settings, Diff, JumpToField, etc.)
```

The main page orchestrates all major components. The toolbar provides file and
edit operations. Below it, segment tabs let users navigate between segments in
form view, while the message editor shows raw HL7 text. These two views stay
synchronised through the parse-render cycle described below.

## Parse-Render Editing Cycle

Hermes provides two ways to edit HL7 messages: form-based editing through
segment tabs, and direct text editing in the raw editor. These views must stay
synchronised, and the parse-render cycle makes this possible.

When a user opens a segment tab, the frontend sends the raw message text to the
backend for parsing. The backend extracts that segment's fields into a flat
structure suitable for form rendering. The frontend displays each field in an
input, flattening nested components into dot-notation paths like `PID.5.1` for
the first component of the fifth PID field.

When the user edits a form field, the frontend collects all field values and
sends them back to the backend for rendering. The backend reconstructs the
segment with proper HL7 delimiters and returns the updated message text. The
frontend replaces the raw text, completing the cycle.

This approach ensures HL7 delimiter handling stays in Rust. The frontend never
manipulates HL7 syntax directly, avoiding bugs from incorrect escaping or
delimiter placement.

## Cursor Synchronisation

The segment tabs follow the cursor position in the raw editor. As users navigate
through the HL7 text, the application determines which segment contains the
cursor and activates the corresponding tab. This lets users click anywhere in
the raw message and immediately see that segment's form fields.

The synchronisation works in both directions. Clicking a segment tab scrolls the
raw editor to that segment's location and positions the cursor there. This
bidirectional linking means users can work in whichever view suits them while
keeping context in the other.

## Co-located TypeScript Bridges

Each feature folder contains its TypeScript bridges alongside components. The
`communication/` folder holds both the send/listen UI components and the
TypeScript functions that invoke the corresponding Rust commands.

This co-location keeps related code together. When adding a new feature, the
command invocation lives next to the component that uses it rather than in a
central `api/` directory. Imports stay local, and the feature folder becomes
self-contained.

## Lazy Schema Loading

Segment schemas load on demand when users open tabs, not all at startup. The
application caches loaded schemas in the backend so subsequent tab switches are
fast, but the initial load only fetches what's needed for the current message
type. This improves startup time and memory usage for the common case where
users work with a subset of HL7 segments.

## Related Documentation

- [Backend Architecture](backend.md) — Rust commands that handle parsing
- [Communication Patterns](communication.md) — IPC patterns for the parse-render
  cycle
- [Svelte Coding Standards](../development/coding-standards/svelte.md) —
  Component patterns and conventions
