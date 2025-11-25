# Schema Overrides

Extensions can provide schema overrides to customise how Hermes interprets and displays HL7 fields. This is useful for:

- Adding organisation-specific field notes
- Defining allowed values for coded fields
- Providing default template values
- Customising validation rules

## Overview

Schema overrides are provided in the `initialize` response:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "name": "My Extension",
    "version": "1.0.0",
    "capabilities": {
      "schemaProvider": true
    },
    "schema": {
      "segments": {
        "PID": {
          "fields": [
            {
              "field": 3,
              "note": "Patient MRN from hospital information system"
            }
          ]
        }
      }
    }
  }
}
```

## Schema Structure

```typescript
interface SchemaOverride {
  segments?: {
    [segmentName: string]: SegmentOverride;
  };
}

interface SegmentOverride {
  fields?: FieldOverride[];
}
```

### Segment Names

Use standard HL7 segment identifiers:

| Segment | Description           |
|---------|-----------------------|
| `MSH`   | Message Header        |
| `PID`   | Patient Identification|
| `PV1`   | Patient Visit         |
| `OBR`   | Observation Request   |
| `OBX`   | Observation Result    |
| `NK1`   | Next of Kin           |
| `NTE`   | Notes and Comments    |
| `EVN`   | Event Type            |
| `DG1`   | Diagnosis             |

## Field Override Properties

Each field override can include any combination of these properties:

```typescript
interface FieldOverride {
  // identification (required)
  field: number;           // 1-based field number
  component?: number;      // 1-based component number

  // display
  name?: string;           // human-readable name
  group?: string;          // UI grouping
  note?: string;           // help text

  // validation
  required?: boolean;      // is this field required?
  minlength?: number;      // minimum character length
  maxlength?: number;      // maximum character length
  pattern?: string;        // regex pattern

  // special handling
  datatype?: "date" | "datetime";
  placeholder?: string;    // UI hint text

  // coded values
  values?: {
    [code: string]: string;
  };

  // templates
  template?: string;       // default value for templates
}
```

### Field and Component Numbers

Fields and components use **1-based** numbering:

```json
{
  "field": 5,              // PID-5 (Patient Name)
  "component": 1           // PID-5.1 (Family Name)
}
```

To override the entire field (all components), omit `component`:

```json
{
  "field": 3,              // PID-3 (entire Patient ID field)
  "note": "MRN format: 8 digits"
}
```

## Property Details

### name

Human-readable name for the field:

```json
{
  "field": 18,
  "name": "Patient Account Number"
}
```

### group

Groups related fields in the UI:

```json
{
  "fields": [
    { "field": 5, "component": 1, "name": "Last Name", "group": "Patient Name" },
    { "field": 5, "component": 2, "name": "First Name", "group": "Patient Name" },
    { "field": 5, "component": 3, "name": "Middle Name", "group": "Patient Name" }
  ]
}
```

### note

Contextual help text shown to users:

```json
{
  "field": 3,
  "component": 1,
  "note": "Enter the 8-digit MRN from the patient's wristband"
}
```

Notes can include:
- Format requirements
- Where to find the value
- Organisation-specific instructions
- References to external documentation

### required

Marks a field as required for validation:

```json
{
  "field": 3,
  "required": true
}
```

Required fields trigger validation errors when empty.

### minlength / maxlength

Character length constraints:

```json
{
  "field": 3,
  "component": 1,
  "minlength": 8,
  "maxlength": 8
}
```

### pattern

Regular expression for validation:

```json
{
  "field": 3,
  "component": 1,
  "pattern": "^[0-9]{8}$"
}
```

Patterns use standard regex syntax. Common patterns:

| Pattern               | Matches                      |
|-----------------------|------------------------------|
| `^[0-9]+$`            | Digits only                  |
| `^[A-Z]{2}[0-9]{6}$`  | 2 letters + 6 digits         |
| `^[A-Za-z ]+$`        | Letters and spaces           |
| `^\d{4}-\d{2}-\d{2}$` | ISO date (YYYY-MM-DD)        |

### datatype

Special handling for date/time fields:

```json
{
  "field": 7,
  "datatype": "date"
}
```

| Value      | Format Expected  | Example          |
|------------|------------------|------------------|
| `date`     | YYYYMMDD         | 19800101         |
| `datetime` | YYYYMMDDHHmmss   | 20231215120000   |

Date fields get special UI handling (date picker) and validation.

### placeholder

Hint text shown in empty input fields:

```json
{
  "field": 5,
  "component": 1,
  "placeholder": "SURNAME"
}
```

### values

Allowed values for coded fields:

```json
{
  "field": 8,
  "values": {
    "M": "Male",
    "F": "Female",
    "O": "Other",
    "U": "Unknown"
  }
}
```

The key is the code stored in the message; the value is the display text.

When `values` is defined:
- UI shows a dropdown instead of free text
- Validation ensures only listed values are used
- Display shows the description alongside the code

### template

Default value used when creating new messages from templates:

```json
{
  "field": 3,
  "component": 4,
  "template": "MRN"
}
```

Template values populate the message when user creates "New from Template".

## Complete Examples

### Patient Identification (PID)

```json
{
  "PID": {
    "fields": [
      {
        "field": 3,
        "component": 1,
        "name": "Patient ID",
        "note": "8-digit MRN from HIS",
        "required": true,
        "minlength": 8,
        "maxlength": 8,
        "pattern": "^[0-9]{8}$",
        "placeholder": "00000000"
      },
      {
        "field": 3,
        "component": 4,
        "name": "Assigning Authority",
        "template": "MRN"
      },
      {
        "field": 5,
        "component": 1,
        "name": "Family Name",
        "group": "Patient Name",
        "required": true,
        "maxlength": 50,
        "placeholder": "SURNAME"
      },
      {
        "field": 5,
        "component": 2,
        "name": "Given Name",
        "group": "Patient Name",
        "required": true,
        "maxlength": 50,
        "placeholder": "FIRSTNAME"
      },
      {
        "field": 7,
        "name": "Date of Birth",
        "datatype": "date",
        "required": true,
        "note": "Format: YYYYMMDD"
      },
      {
        "field": 8,
        "name": "Sex",
        "required": true,
        "values": {
          "M": "Male",
          "F": "Female",
          "O": "Other",
          "U": "Unknown"
        }
      }
    ]
  }
}
```

### Patient Visit (PV1)

```json
{
  "PV1": {
    "fields": [
      {
        "field": 2,
        "name": "Patient Class",
        "required": true,
        "values": {
          "I": "Inpatient",
          "O": "Outpatient",
          "E": "Emergency",
          "P": "Preadmit",
          "R": "Recurring Patient",
          "B": "Obstetrics"
        },
        "note": "Determines billing category"
      },
      {
        "field": 3,
        "component": 1,
        "name": "Point of Care",
        "note": "Ward/unit code from ADT system",
        "placeholder": "WARD01"
      },
      {
        "field": 3,
        "component": 2,
        "name": "Room",
        "placeholder": "101"
      },
      {
        "field": 3,
        "component": 3,
        "name": "Bed",
        "placeholder": "A"
      },
      {
        "field": 44,
        "name": "Admit Date/Time",
        "datatype": "datetime",
        "required": true
      }
    ]
  }
}
```

### Observation Result (OBX)

```json
{
  "OBX": {
    "fields": [
      {
        "field": 2,
        "name": "Value Type",
        "required": true,
        "values": {
          "NM": "Numeric",
          "ST": "String",
          "TX": "Text",
          "CE": "Coded Element",
          "CWE": "Coded with Exceptions",
          "DT": "Date",
          "TM": "Time",
          "TS": "Timestamp"
        }
      },
      {
        "field": 3,
        "component": 1,
        "name": "Observation ID",
        "note": "LOINC code preferred",
        "placeholder": "12345-6"
      },
      {
        "field": 3,
        "component": 2,
        "name": "Observation Name",
        "placeholder": "Test Name"
      },
      {
        "field": 11,
        "name": "Observation Status",
        "required": true,
        "values": {
          "F": "Final",
          "P": "Preliminary",
          "C": "Corrected",
          "X": "Cancelled",
          "I": "In Progress"
        }
      }
    ]
  }
}
```

## Merging Behaviour

Schema overrides are **merged** with the Hermes built-in schema:

| Scenario                        | Behaviour                         |
|---------------------------------|-----------------------------------|
| Field exists in both            | Extension values override built-in|
| Field only in extension         | Added to schema                   |
| Field only in built-in          | Preserved unchanged               |
| Property exists in both         | Extension value wins              |
| Property only in extension      | Added to field                    |
| Property set to `null`          | Inherited value is removed        |
| Property absent in override     | Inherited value is preserved      |

### Field Matching

Hermes uses a two-pass matching algorithm when merging field overrides:

1. **Exact match** (first pass): Both field number AND component must match exactly
2. **Flexible match** (second pass): If the base schema has a field-level entry (no
   component) and the override specifies a component, the override can match and
   modify the field-level entry

This allows component-specific overrides to match against field-level schema entries
when no component-level entry exists.

**Example: Flexible matching**

If the built-in schema has:
```json
{ "field": 3, "name": "Patient ID" }  // field-level (no component)
```

An override can target it with a component:
```json
{ "field": 3, "component": 1, "note": "8-digit MRN" }
```

The note will be added to the field-level entry. If the base schema had a separate
component-level entry for PID.3.1, the override would match that instead (exact
match takes priority).

### Merge Example

Built-in schema:
```json
{
  "PID": {
    "fields": [
      { "field": 8, "name": "Sex", "values": { "M": "Male", "F": "Female" } }
    ]
  }
}
```

Extension override:
```json
{
  "PID": {
    "fields": [
      { "field": 8, "note": "Required for paediatrics", "values": { "M": "Male", "F": "Female", "U": "Unknown" } }
    ]
  }
}
```

Result:
```json
{
  "PID": {
    "fields": [
      {
        "field": 8,
        "name": "Sex",
        "note": "Required for paediatrics",
        "values": { "M": "Male", "F": "Female", "U": "Unknown" }
      }
    ]
  }
}
```

## Best Practices

### Use Cases

| Use Case                          | Properties to Use                    |
|-----------------------------------|--------------------------------------|
| Add help text                     | `note`                               |
| Restrict to allowed values        | `values`                             |
| Set up templates                  | `template`                           |
| Add validation                    | `required`, `minlength`, `maxlength`, `pattern` |
| Handle dates                      | `datatype`                           |

### Keep It Minimal

Only override what you need. Don't duplicate the built-in schema unnecessarily.

```json
// Good: just the custom note
{ "field": 3, "note": "Use 8-digit MRN" }

// Unnecessary: repeating built-in values
{ "field": 3, "name": "Patient ID", "note": "Use 8-digit MRN" }
```

### Organisation-Specific Notes

Notes are ideal for organisation-specific guidance:

```json
{
  "field": 18,
  "note": "Account number from Finance system. Contact ext. 1234 for new accounts."
}
```

### Coded Value Tables

For large value tables, consider linking to external documentation rather than embedding all values:

```json
{
  "field": 3,
  "component": 4,
  "note": "See hospital code table HT-101 for valid assigning authorities"
}
```

## Multiple Extensions

When multiple extensions provide schema overrides, they are merged in the order extensions are defined in the Hermes configuration.

### Load Order

Extensions are processed in the order they appear in `settings.json`:

```json
{
  "extensions": [
    { "path": "/path/to/extension-a" },  // loaded first
    { "path": "/path/to/extension-b" },  // loaded second, overrides A
    { "path": "/path/to/extension-c" }   // loaded last, overrides A and B
  ]
}
```

Later extensions override earlier ones when they define the same field.

### Conflict Resolution

| Scenario                              | Result                                   |
|---------------------------------------|------------------------------------------|
| Same field, different properties      | Properties are merged                    |
| Same field, same property             | Later extension wins                     |
| Different fields                      | Both are applied                         |

### Unsetting Inherited Values

To explicitly remove an inherited property, set it to `null`:

```json
{
  "field": 8,
  "values": null  // removes any inherited values constraint
}
```

This allows an extension to "undo" a constraint from an earlier extension or the built-in schema.

### Avoiding Conflicts

- Use schema overrides sparingly, only for organisation-specific customisations
- Coordinate with other extension authors when deploying multiple extensions
- Document which fields your extension modifies

## Related Documentation

- [Initialize](messages/initialize.md) - Where schema is provided
- [Types](types.md) - Type definitions
