# Initialize

The `initialize` request is sent from Hermes to the extension immediately after the extension process starts. This is the handshake that establishes the extension's identity, capabilities, and contributions.

## Direction

**Hermes → Extension**

## Request

### Method

```
initialize
```

### Parameters

```typescript
interface InitializeParams {
  /** Version of Hermes (e.g., "1.0.0") */
  hermesVersion: string;

  /** Version of the Extension API (e.g., "1.0.0") */
  apiVersion: string;

  /** Path to Hermes data directory for extension storage */
  dataDirectory: string;
}
```

### Example Request

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "initialize",
  "params": {
    "hermesVersion": "1.0.0",
    "apiVersion": "1.0.0",
    "dataDirectory": "/Users/user/.hermes"
  }
}
```

## Response

### Result

```typescript
interface InitializeResult {
  /** Display name of the extension */
  name: string;

  /** Semantic version of the extension */
  version: string;

  /** Brief description of what the extension does (optional) */
  description?: string;

  /** List of authors (optional) */
  authors?: string[];

  /** Homepage or documentation URL (optional) */
  homepage?: string;

  /** Extension capabilities */
  capabilities: Capabilities;

  /** Toolbar buttons to add (optional) */
  toolbarButtons?: ToolbarButton[];

  /** Schema overrides (optional) */
  schema?: SchemaOverride;
}
```

### Capabilities

Capabilities declare what features the extension supports:

```typescript
interface Capabilities {
  /** Extension can handle commands */
  commands?: boolean;

  /** Extension provides schema overrides (included in this response) */
  schemaProvider?: boolean;
}
```

### ToolbarButton

Each toolbar button adds a clickable icon to the Hermes toolbar:

```typescript
interface ToolbarButton {
  /** Unique identifier for this button */
  id: string;

  /** Tooltip text shown on hover */
  label: string;

  /** SVG icon markup (should use viewBox, not fixed dimensions) */
  icon: string;

  /** Command ID to execute when clicked */
  command: string;

  /** Button group for visual separation (optional) */
  group?: string;
}
```

#### Icon Guidelines

- Use SVG format with a `viewBox` attribute
- Avoid fixed `width` and `height` - Hermes will size appropriately
- Use `currentColor` for stroke and fill to inherit theme colours
- Keep icons simple and recognisable at 20×20 pixels

Example icon:
```svg
<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
  <path d="M12 5v14M5 12h14"/>
</svg>
```

### SchemaOverride

Extensions can override or extend the built-in HL7 schema. See [schema.md](../schema.md) for the complete schema format.

```typescript
interface SchemaOverride {
  /** Segment overrides keyed by segment name */
  segments?: {
    [segmentName: string]: SegmentOverride;
  };
}

interface SegmentOverride {
  /** Field overrides for this segment */
  fields?: FieldOverride[];
}

interface FieldOverride {
  /** 1-based field number */
  field: number;

  /** 1-based component number (optional) */
  component?: number;

  /** Custom note for this field (optional) */
  note?: string;

  /** Allowed values map (optional) */
  values?: { [code: string]: string };

  /** Default template value (optional) */
  template?: string;
}
```

### Example Response

#### Minimal Response

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "name": "My Extension",
    "version": "1.0.0",
    "capabilities": {
      "commands": true
    }
  }
}
```

#### Full Response

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "name": "Patient Lookup Wizard",
    "version": "2.1.0",
    "description": "Create patient messages by looking up data from the hospital database",
    "authors": [
      "Jane Developer <jane@hospital.org>",
      "John Coder <john@hospital.org>"
    ],
    "homepage": "https://internal.hospital.org/hermes-extensions/patient-lookup",
    "capabilities": {
      "commands": true,
      "schemaProvider": true
    },
    "toolbarButtons": [
      {
        "id": "patient-lookup",
        "label": "Patient Lookup",
        "icon": "<svg viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\"><circle cx=\"11\" cy=\"11\" r=\"8\"/><path d=\"M21 21l-4.35-4.35\"/></svg>",
        "command": "patientLookup/search",
        "group": "wizards"
      },
      {
        "id": "patient-create",
        "label": "Create Patient",
        "icon": "<svg viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\"><path d=\"M16 21v-2a4 4 0 00-4-4H6a4 4 0 00-4 4v2\"/><circle cx=\"9\" cy=\"7\" r=\"4\"/><path d=\"M22 21v-2a4 4 0 00-3-3.87M16 3.13a4 4 0 010 7.75\"/></svg>",
        "command": "patientLookup/create",
        "group": "wizards"
      }
    ],
    "schema": {
      "segments": {
        "PID": {
          "fields": [
            {
              "field": 3,
              "note": "Patient MRN from HIS system"
            },
            {
              "field": 8,
              "values": {
                "M": "Male",
                "F": "Female",
                "O": "Other",
                "U": "Unknown"
              }
            }
          ]
        },
        "PV1": {
          "fields": [
            {
              "field": 2,
              "note": "Patient class for this facility",
              "values": {
                "I": "Inpatient",
                "O": "Outpatient",
                "E": "Emergency",
                "P": "Preadmit"
              }
            }
          ]
        }
      }
    }
  }
}
```

## Error Response

If initialization fails, respond with an error:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32603,
    "message": "Failed to initialize extension",
    "data": {
      "reason": "Database connection failed",
      "details": "Could not connect to postgres://db.hospital.org:5432"
    }
  }
}
```

### Common Error Scenarios

| Scenario                  | Recommended Action                           |
|---------------------------|----------------------------------------------|
| Missing configuration     | Return error with instructions               |
| Database unavailable      | Return error with connection details         |
| Incompatible API version  | Return error explaining version requirements |
| License validation failed | Return error with license info               |

## Behaviour

### Timeout

Hermes waits **10 seconds** for the initialize response. Extensions that do not respond are terminated.

### Validation

Hermes validates the response:

| Field             | Validation                                    |
|-------------------|-----------------------------------------------|
| `name`            | Required, non-empty string                    |
| `version`         | Required, valid semver string                 |
| `capabilities`    | Required object                               |
| `toolbarButtons`  | Each must have `id`, `label`, `icon`, `command` |
| `schema`          | If present, must follow schema format         |

### After Success

Upon successful initialization:

1. Toolbar buttons are added to the Hermes UI
2. Schema overrides are merged with built-in schema
3. Extension enters the **running** state
4. Commands can now be executed

### After Failure

If initialization fails (error response or timeout):

1. Extension process is terminated
2. No toolbar buttons are added
3. Error is logged
4. Hermes continues without this extension

## Best Practices

### Version Compatibility

Check the `apiVersion` parameter and fail gracefully if incompatible:

```python
def handle_initialize(params):
    api_version = params.get("apiVersion", "0.0.0")
    major = int(api_version.split(".")[0])

    if major < 1:
        return {
            "error": {
                "code": -32600,
                "message": "Incompatible API version",
                "data": f"Extension requires API v1.x, got {api_version}"
            }
        }

    # continue with initialization
```

### Unique Button IDs

Use namespaced IDs to avoid conflicts with other extensions:

```json
{
  "id": "myExtension.search",
  "command": "myExtension/search"
}
```

### Minimal Capabilities

Only declare capabilities you actually support:

```json
{
  "capabilities": {
    "commands": true
  }
}
```

Do not set `schemaProvider: true` unless you are actually providing schema overrides.

## Related Documentation

- [Lifecycle](../lifecycle.md) - Full lifecycle documentation
- [Schema](../schema.md) - Schema override format details
- [Types](../types.md) - Type definitions
