# Schema Merging

This document explains how Hermes combines schema definitions from multiple
sources, the precedence rules, the role of `Nullable<T>`, and how extensions
coordinate (or conflict) when providing schema overrides.

## Schema Sources

Hermes builds its schema from three sources, in order:

1. **Built-in schema:** Defined in `src-tauri/data/*.toml`, these files
   contain standard HL7 v2.5.1 segment definitions with field metadata
   (names, data types, lengths, allowed values, templates).

2. **Extension schemas:** Each extension can provide schema overrides in its
   `initialize` response. These overrides add organisation-specific field
   notes, validation rules, allowed values, and templates.

3. **Runtime overrides:** Future versions may support user-level overrides
   (not yet implemented).

The schema cache combines these sources into a unified view that drives the
editor, validation, and UI.

## Merging Process

When extensions start, Hermes performs schema merging in these steps:

1. **Load built-in schema:** Parse TOML files and populate the schema cache
   with standard definitions.

2. **Collect extension schemas:** After each extension initialises, extract
   its `schema` object from the initialize response.

3. **Merge extension schemas:** Combine all extension schemas in the order
   extensions were loaded (based on settings file order). Later extensions
   override earlier ones.

4. **Apply merged overrides:** Call `schema_cache.set_extension_overrides()`
   to merge the combined extension schema with the built-in schema.

5. **Serve merged schema:** When code calls `schema_cache.get_segment()`, the
   cache returns fields with overrides applied.

This process ensures a consistent view of the schema throughout Hermes, while
allowing extensions to customise fields for their organisation's needs.

## Precedence Rules

When multiple sources define the same field, precedence determines which
values are used:

### Source Precedence

```
Extension schemas (last loaded)
    ↓ overrides
Extension schemas (first loaded)
    ↓ overrides
Built-in schema
```

**Example:** If the built-in schema defines PID.8 (Sex) with values `{M, F}`,
and an extension adds `{M, F, U}`, the extension's values win. If a second
extension later specifies `{M, F, O}`, that extension's values win.

### Property Precedence

Within a single field, properties merge at the property level:

| Scenario                                                | Result                                 |
| ------------------------------------------------------- | -------------------------------------- |
| Base: `name: "Sex"`<br>Override: `note: "Required"`     | Both `name` and `note` are present     |
| Base: `maxlength: 1`<br>Override: `maxlength: 10`       | `maxlength: 10` wins                   |
| Base: `values: {M, F}`<br>Override: `values: {M, F, U}` | `values: {M, F, U}` wins               |
| Base: `note: "Standard"`<br>Override: `note: null`      | `note` is removed (see Nullable below) |

Properties that exist in the base but aren't mentioned in the override are
preserved. Properties that exist in the override replace base values.

## Field Matching Algorithm

How does Hermes determine which override entry matches which base schema
entry? The algorithm uses a two-pass approach.

### Pass 1: Exact Match

Match overrides to base fields where **both** field number and component match
exactly.

**Example:**

Base:
```json
{ "field": 5, "component": 1, "name": "Family Name" }
```

Override:
```json
{ "field": 5, "component": 1, "note": "Last name required" }
```

**Result:** Exact match. Override merges with base, adding `note`.

### Pass 2: Flexible Match

If an override specifies a component but no exact match was found in pass 1,
try matching a field-level base entry (no component specified).

**Example:**

Base:
```json
{ "field": 3, "name": "Patient ID" }  // field-level, no component
```

Override:
```json
{ "field": 3, "component": 1, "note": "8-digit MRN" }
```

**Result:** Flexible match. The override applies to the field-level entry,
adding `note`. The override's `component` property is ignored for matching
purposes but preserved in the result.

**Why flexible matching?** The built-in schema often defines fields at the
field level (e.g., PID.3) because most metadata applies to the whole field.
Extensions might want to provide component-specific notes (e.g., PID.3.1 for
the ID number, PID.3.4 for the assigning authority). Flexible matching allows
this without requiring the built-in schema to enumerate every component.

**Limitation:** If the base schema has both field-level and component-level
entries, exact match takes precedence. Overrides cannot match against
field-level entries if a component-level entry exists for that specific field
and component combination.

## The Role of `Nullable<T>`

JSON serialisation has a problem: How do you distinguish between "property not
specified" and "property explicitly set to null"?

**Standard `Option<T>` in Rust:**
- `None` serialises to omission (no field in JSON)
- `Some(value)` serialises to the value

**Problem:** You can't serialise `None` as `null` and distinguish it from
omission.

**Why this matters for merging:** Consider these scenarios:

1. **Inherit:** Override doesn't mention `maxlength`. Base has `maxlength: 10`.
   Result should preserve `maxlength: 10`.

2. **Set:** Override specifies `maxlength: 20`. Result should be
   `maxlength: 20`.

3. **Unset:** Override specifies `maxlength: null`. Result should remove
   `maxlength`, even if base had it.

Without distinguishing null from omission, scenario 3 is impossible.

### How `Nullable<T>` Works

```rust
pub enum Nullable<T> {
    Null,
    Value(T),
}
```

```json
{
  // absent property: inherit from base
  // "maxlength": null → unset (remove inherited value)
  // "maxlength": 20 → override with new value
}
```

Deserialisation:
- Property absent: `None` (inherit from base)
- Property is `null`: `Some(Nullable::Null)` (unset inherited value)
- Property has value: `Some(Nullable::Value(20))` (override with 20)

Merge logic:
```rust
match override_field.maxlength {
    None => {
        // absent in override, preserve base value
        merged.maxlength = base.maxlength;
    }
    Some(Nullable::Null) => {
        // explicitly null, remove base value
        merged.maxlength = None;
    }
    Some(Nullable::Value(len)) => {
        // explicit value, use it
        merged.maxlength = Some(len);
    }
}
```

This allows extensions to:
- Add properties that didn't exist in the base
- Override properties with new values
- Explicitly remove inherited properties

### Example: Unsetting a Constraint

**Built-in schema:**
```json
{
  "field": 8,
  "name": "Sex",
  "values": { "M": "Male", "F": "Female" }
}
```

The built-in schema restricts PID.8 to M or F.

**Extension A override:**
```json
{
  "field": 8,
  "note": "Required for regulatory reporting"
}
```

Extension A adds a note. The `values` constraint is inherited:
```json
{
  "field": 8,
  "name": "Sex",
  "note": "Required for regulatory reporting",
  "values": { "M": "Male", "F": "Female" }
}
```

**Extension B override (loaded after A):**
```json
{
  "field": 8,
  "values": null
}
```

Extension B explicitly removes the values constraint. Result:
```json
{
  "field": 8,
  "name": "Sex",
  "note": "Required for regulatory reporting"
}
```

The `values` property is gone, allowing free text entry.

## Multiple Extensions

When multiple extensions provide schema overrides, they merge in the order
defined in Hermes settings.

### Load Order

Extensions are processed sequentially:

```json
{
  "extensions": [
    { "path": "/path/to/extension-a" },  // loaded first
    { "path": "/path/to/extension-b" },  // loaded second
    { "path": "/path/to/extension-c" }   // loaded last
  ]
}
```

Hermes merges:
1. Built-in schema
2. Extension A schema → intermediate result
3. Extension B schema → intermediate result
4. Extension C schema → final result

Extension C has the final say on any fields it overrides.

### Conflict Resolution

When multiple extensions override the same field property, **last wins**:

**Extension A:**
```json
{ "field": 3, "note": "MRN format: 8 digits" }
```

**Extension B:**
```json
{ "field": 3, "note": "Patient ID from HIS" }
```

**Result:**
```json
{ "field": 3, "note": "Patient ID from HIS" }
```

Extension B's note replaces Extension A's note.

### Complementary Overrides

Extensions can add different properties to the same field without conflict:

**Extension A:**
```json
{ "field": 3, "component": 1, "note": "8-digit MRN" }
```

**Extension B:**
```json
{ "field": 3, "component": 1, "pattern": "^[0-9]{8}$" }
```

**Result:**
```json
{
  "field": 3,
  "component": 1,
  "note": "8-digit MRN",
  "pattern": "^[0-9]{8}$"
}
```

Both properties coexist because they don't overlap.

### Coordination Strategies

**Problem:** Multiple extensions from different authors might make conflicting
assumptions about fields.

**Solutions:**

1. **Single authoritative extension:** One organisation-wide extension
   provides all schema customisations. Other extensions focus on commands and
   UI, not schema.

2. **Namespaced fields:** Extensions override different segments or fields.
   Extension A handles PID, Extension B handles OBX, etc.

3. **Explicit ordering:** If conflicts are unavoidable, control extension
   order in settings to ensure the desired extension wins.

4. **Documentation:** Extension authors document which fields they override,
   making conflicts visible during deployment planning.

**Best practice:** Keep schema overrides minimal and organisation-specific.
Avoid duplicating built-in schema unnecessarily.

## When Schema Changes Take Effect

Schema changes apply **immediately** when extensions reload:

1. User clicks "Reload Extensions" in settings
2. Hermes shuts down all extensions
3. Hermes restarts extensions
4. Extensions respond to `initialize` with new schemas
5. Hermes merges schemas and updates the cache
6. Next validation, UI update, or editor operation uses the new schema

**No restart required.** The schema cache is mutable and updates in place.

### Active Message Impact

If the user has a message open when extensions reload:

- **Validation:** Re-running validation (Cmd+Shift+V) uses the new schema.
  Fields that were required might no longer be, or vice versa.

- **UI:** Field descriptions, allowed values, and placeholders update the next
  time the UI queries the schema (e.g., opening the jump-to-field dialog).

- **Syntax highlighting:** The next highlight pass uses the new schema to
  determine field validity.

**The message content doesn't change.** Only the interpretation of that
content changes based on the new schema.

## Implementation Details

### Merge Function

```rust
pub fn merge_field(base: &FieldSchema, override_field: &FieldSchema) -> FieldSchema {
    FieldSchema {
        field: override_field.field,
        component: override_field.component.or(base.component),
        name: override_field.name.clone().or_else(|| base.name.clone()),
        note: match &override_field.note {
            None => base.note.clone(),
            Some(Nullable::Null) => None,
            Some(Nullable::Value(s)) => Some(s.clone()),
        },
        // ... similar logic for all properties
    }
}
```

Each property checks three states:
- Absent in override → use base value
- Explicitly null in override → remove value
- Present in override → use override value

### Schema Cache Integration

```rust
impl SchemaCache {
    pub fn set_extension_overrides(&mut self, overrides: SchemaOverride) {
        self.extension_overrides = Some(overrides);
    }

    pub fn get_segment(&self, name: &str) -> Option<SegmentSchema> {
        let base = self.segments.get(name)?;

        if let Some(overrides) = &self.extension_overrides {
            if let Some(segment_override) = overrides.segments.get(name) {
                return Some(merge_segment(base, segment_override));
            }
        }

        Some(base.clone())
    }
}
```

The cache stores overrides separately and applies them on read. This keeps the
built-in schema pristine and allows overrides to be swapped out during
extension reloads.

## Conclusion

Schema merging allows extensions to customise Hermes for organisation-specific
needs without modifying the core application. The merge rules are:

1. **Later sources override earlier sources:** Extensions override built-in,
   later extensions override earlier ones.

2. **Properties merge individually:** Each property (name, note, values, etc.)
   can be inherited, overridden, or unset independently.

3. **Flexible matching:** Component-level overrides can match field-level base
   entries, allowing fine-grained customisation.

4. **Nullable semantics:** Absent means inherit, null means unset, value means
   override.

Understanding these rules helps extension developers provide precise,
non-conflicting schema customisations that enhance Hermes for their users.
