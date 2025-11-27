# Schema Field Properties

Field property reference for schema overrides.

## Property Reference

### field

- **Type:** `number`
- **Required:** Yes
- **Description:** 1-based field number
- **Example:** `3` for PID-3

### component

- **Type:** `number`
- **Required:** No
- **Description:** 1-based component number
- **Example:** `1` for PID-5.1
- **Notes:** Omit to target entire field

### name

- **Type:** `string | null`
- **Required:** No
- **Description:** Human-readable field name
- **Example:** `"Patient ID"`

### group

- **Type:** `string | null`
- **Required:** No
- **Description:** UI grouping for related fields
- **Example:** `"Patient Name"`

### note

- **Type:** `string | null`
- **Required:** No
- **Description:** Help text or instructions
- **Example:** `"Enter 8-digit MRN from patient wristband"`

### minlength

- **Type:** `number | null`
- **Required:** No
- **Description:** Minimum character length
- **Example:** `8`

### maxlength

- **Type:** `number | null`
- **Required:** No
- **Description:** Maximum character length
- **Example:** `50`

### pattern

- **Type:** `string | null`
- **Required:** No
- **Description:** Regular expression for validation
- **Example:** `"^[0-9]{8}$"` for 8 digits
- **Syntax:** Standard regex

Common patterns:

| Pattern               | Matches              |
|-----------------------|----------------------|
| `^[0-9]+$`            | Digits only          |
| `^[A-Z]{2}[0-9]{6}$`  | 2 letters + 6 digits |
| `^[A-Za-z ]+$`        | Letters and spaces   |
| `^\d{4}-\d{2}-\d{2}$` | ISO date YYYY-MM-DD  |

### required

- **Type:** `boolean | null`
- **Required:** No
- **Description:** Whether field is required
- **Values:** `true` or `false`
- **Notes:** Triggers validation error when empty

### datatype

- **Type:** `"date" | "datetime" | null`
- **Required:** No
- **Description:** Special handling for date/time fields
- **Values:**
  - `"date"` - YYYYMMDD format
  - `"datetime"` - YYYYMMDDHHmmss format

### placeholder

- **Type:** `string | null`
- **Required:** No
- **Description:** Hint text for empty input fields
- **Example:** `"SURNAME"`

### values

- **Type:** `{ [code: string]: string } | null`
- **Required:** No
- **Description:** Allowed values for coded fields (code → description)
- **Example:**
  ```json
  {
    "M": "Male",
    "F": "Female",
    "O": "Other",
    "U": "Unknown"
  }
  ```
- **Notes:** When defined, UI shows dropdown; validation enforces values

### template

- **Type:** `string | null`
- **Required:** No
- **Description:** Default value for new messages from templates
- **Example:** `"MRN"`

## Three-State Semantics

All optional properties support three states:

| JSON Value | Meaning                          |
|------------|----------------------------------|
| Absent     | Inherit from base schema         |
| `null`     | Explicitly unset inherited value |
| Value      | Override with this value         |

Example:

```json
{
  "field": 3,
  "name": "Custom MRN",
  "note": null
}
```

This overrides `name`, removes any inherited `note`, and preserves all other
inherited properties.

## Field Identification

Fields identified by `field` number alone or `field` + `component`:

```json
{"field": 5}
```

Targets entire PID-5 field.

```json
{"field": 5, "component": 1}
```

Targets PID-5.1 component only.

## Merging Behaviour

### Field Matching

Two-pass matching algorithm:

1. **Exact match:** Both field number and component must match exactly
2. **Flexible match:** If base schema has field-level entry (no component)
   and override specifies component, override can match field-level entry

### Property Merging

| Scenario                    | Result                           |
|-----------------------------|----------------------------------|
| Property in both            | Extension value overrides base   |
| Property only in extension  | Added to field                   |
| Property only in base       | Preserved unchanged              |
| Property set to `null`      | Inherited value removed          |
| Property absent in override | Inherited value preserved        |

## Validation Properties

Properties affecting validation:

- `required` - field must have value
- `minlength` - minimum character count
- `maxlength` - maximum character count
- `pattern` - regex validation
- `values` - allowed value set
- `datatype` - date/datetime format validation

## Display Properties

Properties affecting UI:

- `name` - field label
- `group` - visual grouping
- `note` - help text
- `placeholder` - input hint
- `values` - dropdown options (when present)

## Template Properties

Properties affecting template generation:

- `template` - default value for new messages
