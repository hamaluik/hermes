# Provide Schema Overrides

This guide shows how to customise field definitions, validation rules, and
allowed values through schema overrides.

## Add a Custom Field Note

```python
def handle_initialize(request_id, params):
    return {
        "jsonrpc": "2.0",
        "id": request_id,
        "result": {
            "name": "My Extension",
            "version": "1.0.0",
            "capabilities": {
                "schemaProvider": True
            },
            "schema": {
                "segments": {
                    "PID": {
                        "fields": [
                            {
                                "field": 3,
                                "component": 1,
                                "note": "8-digit MRN from hospital information system"
                            }
                        ]
                    }
                }
            }
        }
    }
```

The note appears in the Hermes UI when users hover over or select the field.

## Add an Allowed Values Dropdown

```python
"schema": {
    "segments": {
        "PID": {
            "fields": [
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
        }
    }
}
```

When `values` is defined, the UI shows a dropdown instead of free text input,
and validation ensures only listed codes are used.

## Override Validation Rules

```python
"schema": {
    "segments": {
        "PID": {
            "fields": [
                {
                    "field": 3,
                    "component": 1,
                    "required": True,
                    "minlength": 8,
                    "maxlength": 8,
                    "pattern": "^[0-9]{8}$",
                    "note": "MRN must be exactly 8 digits"
                }
            ]
        }
    }
}
```

This enforces that PID.3.1 (patient ID) must be present, exactly 8 characters,
and contain only digits.

## Set Default Template Values

```python
"schema": {
    "segments": {
        "PID": {
            "fields": [
                {
                    "field": 3,
                    "component": 4,
                    "template": "MRN"
                },
                {
                    "field": 18,
                    "component": 4,
                    "template": "ACCT"
                }
            ]
        }
    }
}
```

Template values populate fields when users create a new message from a template
(File > New from Template).

## Mark Date/Time Fields

```python
"schema": {
    "segments": {
        "PID": {
            "fields": [
                {
                    "field": 7,
                    "datatype": "date",
                    "note": "Date of birth in YYYYMMDD format"
                }
            ]
        },
        "PV1": {
            "fields": [
                {
                    "field": 44,
                    "datatype": "datetime",
                    "note": "Admit date/time in YYYYMMDDHHmmss format"
                }
            ]
        }
    }
}
```

Date fields get special UI handling (date picker) and validation.

## Add Placeholder Text

```python
"schema": {
    "segments": {
        "PID": {
            "fields": [
                {
                    "field": 5,
                    "component": 1,
                    "placeholder": "SURNAME"
                },
                {
                    "field": 5,
                    "component": 2,
                    "placeholder": "FIRSTNAME"
                }
            ]
        }
    }
}
```

Placeholders appear in empty input fields as hints.

## Group Related Fields

```python
"schema": {
    "segments": {
        "PID": {
            "fields": [
                {
                    "field": 5,
                    "component": 1,
                    "name": "Family Name",
                    "group": "Patient Name"
                },
                {
                    "field": 5,
                    "component": 2,
                    "name": "Given Name",
                    "group": "Patient Name"
                },
                {
                    "field": 5,
                    "component": 3,
                    "name": "Middle Name",
                    "group": "Patient Name"
                }
            ]
        }
    }
}
```

Grouped fields appear together in the UI.

## Complete Example: Patient Identification

```python
def handle_initialize(request_id, params):
    return {
        "jsonrpc": "2.0",
        "id": request_id,
        "result": {
            "name": "Hospital Schema Extension",
            "version": "1.0.0",
            "capabilities": {
                "schemaProvider": True
            },
            "schema": {
                "segments": {
                    "PID": {
                        "fields": [
                            # patient ID with strict validation
                            {
                                "field": 3,
                                "component": 1,
                                "name": "Patient ID",
                                "note": "8-digit MRN from patient master index",
                                "required": True,
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

                            # patient name
                            {
                                "field": 5,
                                "component": 1,
                                "name": "Family Name",
                                "group": "Patient Name",
                                "required": True,
                                "maxlength": 50,
                                "placeholder": "SURNAME"
                            },
                            {
                                "field": 5,
                                "component": 2,
                                "name": "Given Name",
                                "group": "Patient Name",
                                "required": True,
                                "maxlength": 50,
                                "placeholder": "FIRSTNAME"
                            },

                            # date of birth
                            {
                                "field": 7,
                                "name": "Date of Birth",
                                "datatype": "date",
                                "required": True,
                                "note": "Format: YYYYMMDD"
                            },

                            # administrative sex
                            {
                                "field": 8,
                                "name": "Sex",
                                "required": True,
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
            }
        }
    }
```

## Complete Example: Observation Results

```python
"schema": {
    "segments": {
        "OBX": {
            "fields": [
                {
                    "field": 2,
                    "name": "Value Type",
                    "required": True,
                    "values": {
                        "NM": "Numeric",
                        "ST": "String",
                        "TX": "Text",
                        "CE": "Coded Element",
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
                    "required": True,
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
}
```

## Override Only What You Need

Schema overrides are merged with the built-in schema. Only specify properties
you want to change:

```python
# Good: just add a custom note
{
    "field": 3,
    "note": "Use 8-digit MRN from hospital system"
}

# Unnecessary: repeating built-in values
{
    "field": 3,
    "name": "Patient ID",  # already in built-in schema
    "note": "Use 8-digit MRN from hospital system"
}
```

## Common Validation Patterns

| Pattern               | Matches               |
| --------------------- | --------------------- |
| `^[0-9]+$`            | Digits only           |
| `^[A-Z]{2}[0-9]{6}$`  | 2 letters + 6 digits  |
| `^[A-Za-z ]+$`        | Letters and spaces    |
| `^\d{4}-\d{2}-\d{2}$` | ISO date (YYYY-MM-DD) |
| `^[0-9]{8}$`          | Exactly 8 digits      |

## Merging Behaviour

When you provide schema overrides:

- Properties in your override replace properties in the built-in schema
- Properties you don't specify are inherited from the built-in schema
- Set a property to `null` to explicitly remove an inherited value
- Multiple extensions are merged in load order (later wins)

### Example: Removing an Inherited Constraint

```python
# if built-in schema has:
# {"field": 8, "values": {"M": "Male", "F": "Female"}}

# you can remove the values constraint:
{
    "field": 8,
    "values": None  # removes the inherited constraint
}
```

## Field and Component Numbers

Always use 1-based numbering:

```python
{
    "field": 5,              # PID-5 (Patient Name)
    "component": 1           # PID-5.1 (Family Name)
}
```

To override the entire field (all components), omit `component`:

```python
{
    "field": 3,              # applies to entire PID-3 field
    "note": "MRN format: 8 digits"
}
```

## Use Cases

| Use Case                   | Properties to Use                               |
| -------------------------- | ----------------------------------------------- |
| Add help text              | `note`                                          |
| Restrict to allowed values | `values`                                        |
| Set up templates           | `template`                                      |
| Add validation             | `required`, `minlength`, `maxlength`, `pattern` |
| Handle dates               | `datatype`                                      |

## Linking to External Documentation

For large code tables, reference external documentation rather than embedding
all values:

```python
{
    "field": 3,
    "component": 4,
    "note": "See hospital code table HT-101 for valid assigning authorities: http://intranet/codes/HT-101"
}
```

## Related Documentation

- [Reference: Schema Properties](../reference/schema-properties.md)
- [Explanation: Schema Merging](../explanation/schema-merging.md)
- [Tutorial: Building Your First Extension](../tutorials/first-extension.md)
