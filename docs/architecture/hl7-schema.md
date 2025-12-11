# Schema System

The schema system provides metadata about HL7 messages that drives form
generation, validation, and documentation lookup.

## Embedded Schemas

Schemas embed in the binary at compile time rather than loading from external
files at runtime. This produces a single-file distribution with no external
dependencies, simplifying deployment.

The trade-off is that schema updates require recompilation. For an application
targeting a stable HL7 version, this constraint matters little. The extension
override system provides runtime flexibility when needed.

## Schema-Driven Forms

When a user opens a segment tab, the schema determines what appears. Each field
defined in the segment schema becomes an input in the form. The schema provides
field names for labels, data types for input formatting, and constraints like
required flags and length limits.

This data-driven approach means adding support for a new segment type requires
only schema definitions, not new UI code. The form rendering logic reads the
schema and generates appropriate inputs automatically.

## Extension Overrides

Extensions can provide schema overrides that merge with the base schemas at
runtime. This allows organisations to customise field definitions without
forking the application or waiting for upstream changes.

Overrides merge at the field level. An extension can change a single field's
properties while leaving others untouched. When multiple extensions provide
overrides for the same field, later extensions win. This simple precedence rule
keeps merge semantics predictable.

Typical override use cases include adding organisation-specific field
descriptions, changing validation rules for local requirements, and adding
dropdown options that reflect local code tables.

## Template Values

Schemas can include template values that generate default content when creating
new messages. A field definition might specify a template like `{auto}` for
timestamps or control IDs, indicating that the application should generate an
appropriate value rather than leaving the field empty.

When sending messages, the backend processes these placeholders. A timestamp
placeholder becomes the current time in HL7 format. A control ID placeholder
becomes a random identifier. This automation reduces manual work when composing
test messages.

## Field Descriptions

The schema system provides field descriptions from two sources. The standard
HL7 specification supplies definitions for all standard fields. Custom spec
files can override these with organisation-specific descriptions.

Custom specs take precedence over standard definitions, allowing organisations
to document local extensions or provide clearer explanations for fields they use
frequently. The cursor position panel displays these descriptions as users
navigate through messages.

## Related Documentation

- [Backend Architecture](backend.md) — How the schema cache is initialised and
  used
- [Frontend Architecture](frontend.md) — How forms render from schema data
- [Adding Features](../development/adding-features.md) — How to add new HL7
  message types
