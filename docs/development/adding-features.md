# Adding Features

## Adding a New Tauri Command

### 1. Create Rust command

```rust
// src-tauri/src/commands/my_feature/my_command.rs
#[tauri::command]
pub async fn my_feature_command(
    param: String,
    state: State<'_, AppData>,
) -> Result<String, String> {
    // implementation
    Ok(format!("Result: {}", param))
}
```

### 2. Export and register

```rust
// src-tauri/src/commands/my_feature/mod.rs
mod my_command;
pub use my_command::my_feature_command;

// src-tauri/src/commands/mod.rs
pub mod my_feature;
pub use my_feature::my_feature_command;

// src-tauri/src/lib.rs
.invoke_handler(tauri::generate_handler![
    // ... existing
    my_feature_command,
])
```

### 3. Create TypeScript bridge (co-located with feature)

```typescript
// src/lib/my_feature/my_feature.ts
import { invoke } from '@tauri-apps/api/core';

export async function myFeatureCommand(param: string): Promise<string> {
  return await invoke('my_feature_command', { param });
}
```

### 4. Use in component

```svelte
<script lang="ts">
  // relative import within same feature directory
  import { myFeatureCommand } from './my_feature';
  // or use $lib for cross-directory imports
  // import { myFeatureCommand } from '$lib/my_feature/my_feature';

  async function handleClick() {
    const result = await myFeatureCommand('test');
    console.log(result);
  }
</script>
```

## Adding a New UI Component

1. Create component file in appropriate feature directory under `src/lib/`
2. Define props interface using TypeScript
3. Use Svelte 5 runes for state
4. Add styles scoped to component
5. Use relative imports within feature, `$lib/` for cross-directory

## Adding New HL7 Message Types

1. Update `messages.toml` with message structure
2. Schema cache will automatically load on next run
3. Add any custom field descriptions to `src-tauri/src/spec/`

## Adding Support for New HL7 Versions

1. Update `hl7-definitions` dependency to include new version
2. Add new message definitions to `messages.toml`
3. Update schema cache to handle version-specific fields
4. Add version-specific specs to `spec` module if needed

## Related Documentation

- [Project Structure](project-structure.md) — Where to put new files
- [Rust Standards](coding-standards/rust.md) — Command conventions
- [Backend Architecture](../architecture/backend.md) — Command structure
- [HL7 & Schema](../architecture/hl7-schema.md) — Schema file format
