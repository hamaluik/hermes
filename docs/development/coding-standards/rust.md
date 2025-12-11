# Rust Coding Standards

## General Guidelines

- Follow Rust standard style guidelines
- Use `rustfmt` for automatic formatting
- Run `cargo clippy` for linting (strict lints are enabled)
- Add doc comments for public items

## Comments

Comments earn their place by adding value the code cannot convey on its own.
Restating what the code does wastes reader attention; explaining why something
exists or why it's done a particular way helps future maintainers.

### Inline comments

Inline comments start with lowercase and explain intent, constraints, or
non-obvious decisions. They should never describe what the code is doing—the
code shows that.

```rust
// wrong: restates the code
let timeout = Duration::from_secs(30); // set timeout to 30 seconds

// wrong: obvious from context
for segment in segments { // loop through segments

// correct: explains why
let timeout = Duration::from_secs(30); // HL7 spec recommends 30s for ACKs

// correct: documents a constraint
let max_retries = 3; // upstream server drops connections after 4 attempts
```

### Doc comments

Doc comments (`///`) appear in LSP hover information and generated
documentation. They're valuable even for internal functions with a single call
site because they help developers understand intent without reading the
implementation.

Write doc comments when the function name and parameters don't fully convey:

- **Intent**: What problem does this solve? Why does it exist?
- **Constraints**: What assumptions or limitations apply?
- **Behaviour**: Any non-obvious side effects or edge cases?

```rust
/// Extracts field repetitions from an HL7 field value.
///
/// HL7 allows fields to repeat using the `~` delimiter. This function
/// splits on that delimiter while preserving empty repetitions, which
/// have semantic meaning in HL7 (they indicate explicitly empty values).
fn split_repetitions(field: &str) -> Vec<&str> {
    field.split('~').collect()
}
```

Skip doc comments when the name is self-explanatory:

```rust
// no doc comment needed
fn is_empty(&self) -> bool {
    self.segments.is_empty()
}
```

## Clippy Lints

The project uses defensive clippy lints to catch potential runtime errors at
compile time. These are configured in `Cargo.toml`:

```toml
[lints.clippy]
indexing_slicing = "deny"
fallible_impl_from = "deny"
wildcard_enum_match_arm = "deny"
unneeded_field_pattern = "deny"
fn_params_excessive_bools = "deny"
must_use_candidate = "deny"
unwrap_used = "deny"
```

### `indexing_slicing`

Direct indexing (`arr[0]`) panics on out-of-bounds access. Use `.get()` instead
and handle the `Option` explicitly.

### `fallible_impl_from`

`From` implementations must not panic. If the conversion can fail, implement
`TryFrom` instead and return a `Result`.

### `wildcard_enum_match_arm`

Wildcard match arms (`_ =>`) silently ignore new enum variants when they're
added. List all variants explicitly so the compiler warns when new ones appear.

### `unneeded_field_pattern`

Avoid matching struct fields you don't use. Use `..` to ignore remaining fields
rather than binding them to unused variables.

### `fn_params_excessive_bools`

Functions with multiple boolean parameters are error-prone—callers can easily
swap `true` and `false` arguments. Use an enum or a builder pattern instead.

### `must_use_candidate`

Functions that return values which should be used (like results of computations
or builders) should be marked `#[must_use]`. This lint identifies candidates.

### `unwrap_used`

`.unwrap()` calls panic on `None` or `Err`. Handle errors explicitly or use
`.expect("reason")` with a message explaining why the value is guaranteed to
exist. Even then, prefer proper error handling over expects, although panicking
is perfectly acceptable if doing so indicates a programming error and not just a
runtime failure.

## Defensive Patterns

This is lifted wholesale from
[https://corrode.dev/blog/defensive-programming/](https://corrode.dev/blog/defensive-programming/).

### Use `.get()` instead of direct indexing

```rust
// wrong: panics if index out of bounds
let value = items[index];

// correct: returns Option, handles missing gracefully
let Some(value) = items.get(index) else {
    return None;
};
```

### Use slice pattern matching instead of indexed access

```rust
// wrong: multiple panic points
let first = parts[0];
let second = parts[1];

// correct: single pattern match, explicit handling
let [first, second] = parts.as_slice() else {
    return Err("expected two parts".into());
};

// for variable-length: use rest patterns
let [first, rest @ ..] = parts.as_slice() else {
    return Err("expected at least one part".into());
};
```

### List all enum variants explicitly in match arms

```rust
// wrong: silently ignores new variants
match error {
    Error::Io(e) => Some(e),
    _ => None,
}

// correct: compiler warns when new variants are added
match error {
    Error::Io(e) => Some(e),
    Error::Parse(_) | Error::Timeout | Error::NotFound => None,
}
```

### Use `is_some_and()` instead of `map_or(false, ...)`

```rust
// verbose
let is_valid = value.get(i).map_or(false, |v| v.is_empty());

// cleaner
let is_valid = value.get(i).is_some_and(|v| v.is_empty());
```

## Documentation Example

```rust
/// Highlights syntax in an HL7 message for display
///
/// # Arguments
/// * `message` - The raw HL7 message text
/// * `schema_cache` - Reference to the schema cache
///
/// # Returns
/// A vector of styled text segments with colour information
#[tauri::command]
pub fn syntax_highlight(
    message: String,
    state: State<'_, AppData>,
) -> Result<Vec<TextSegment>, String> {
    let schema_cache = &state.schema_cache;

    let parsed = hl7_parser::parse(&message)
        .map_err(|e| format!("Parse error: {}", e))?;

    let segments = apply_syntax_highlighting(&parsed, schema_cache);

    Ok(segments)
}
```

## Naming Conventions

| Item        | Convention         | Example             |
|-------------|--------------------|---------------------|
| Variables   | `snake_case`       | `message_text`      |
| Functions   | `snake_case`       | `send_message`      |
| Types       | `PascalCase`       | `MessageSchema`     |
| Traits      | `PascalCase`       | `Parseable`         |
| Constants   | `UPPER_SNAKE_CASE` | `MAX_TIMEOUT`       |
| Modules     | `snake_case`       | `send_receive`      |

## Error Handling

- Use `Result<T, E>` for fallible operations
- Convert internal errors to `String` for Tauri commands
- Use `color-eyre` for detailed error context in complex functions
- Propagate errors with `?` operator
- Use `.wrap_err_with()` to add context when propagating errors

## Related Documentation

- [Backend Architecture](../../architecture/backend.md) — Command structure
- [Adding Features](../adding-features.md) — Creating new Tauri commands
