# TypeScript Coding Standards

## General Guidelines

- Use TypeScript for all new code
- Prefer `const` over `let`, avoid `var`
- Use arrow functions for callbacks
- Use async/await over raw Promises
- Add JSDoc comments for exported functions

## Comments

Comments should add value that the code itself cannot provide. Explaining what
the code does is redundant; explaining why it does something, or why it's done
in a particular way, helps future maintainers understand the reasoning.

### Inline comments

Inline comments start with lowercase and focus on intent, constraints, or
justifications—never on describing what the code is doing.

```typescript
// wrong: restates the code
const timeout = 30000; // timeout is 30 seconds

// wrong: obvious from context
for (const segment of segments) { // iterate over segments

// correct: explains why this value
const timeout = 30000; // HL7 spec recommends 30s for ACK responses

// correct: documents a workaround
const offset = position - 1; // Tauri text positions are 1-indexed
```

### JSDoc comments

JSDoc comments appear in LSP hover information and help developers understand
functions without reading their implementation. They're valuable even for
internal functions with a single call site.

Write JSDoc comments when the function name and parameters don't fully convey:

- **Intent**: What problem does this solve? Why does it exist?
- **Constraints**: What assumptions or limitations apply?
- **Behaviour**: Any non-obvious side effects or edge cases?

```typescript
/**
 * Splits an HL7 field into its component parts.
 *
 * HL7 uses the `^` delimiter for components within a field. This function
 * preserves empty components, which have semantic meaning in HL7 (they
 * indicate explicitly empty values rather than missing data).
 */
function splitComponents(field: string): string[] {
  return field.split('^');
}
```

Skip JSDoc comments when the name is self-explanatory:

```typescript
// no doc comment needed
function isEmpty(): boolean {
  return segments.length === 0;
}
```

## Example

```typescript
/**
 * Sends an HL7 message via MLLP protocol
 * @param host - Target host address
 * @param port - Target port number
 * @param message - HL7 message content
 * @param timeout - Connection timeout in milliseconds
 */
export async function sendMessage(
  host: string,
  port: number,
  message: string,
  timeout: number
): Promise<void> {
  await invoke('send_message', { host, port, message, timeout });
}
```

## Naming Conventions

| Item              | Convention         | Example              |
|-------------------|--------------------|----------------------|
| Variables         | `camelCase`        | `messageText`        |
| Functions         | `camelCase`        | `sendMessage`        |
| Types/Interfaces  | `PascalCase`       | `MessageSchema`      |
| Constants         | `UPPER_SNAKE_CASE` | `MAX_TIMEOUT`        |
| Files             | `snake_case.ts`    | `send_receive.ts`    |

## Related Documentation

- [Svelte Standards](svelte.md) — Component patterns
- [Project Structure](../project-structure.md) — File organisation
