# Tutorials

This section contains learning-oriented, step-by-step guides for building
Hermes extensions. Tutorials teach you the fundamentals by walking through
complete, working examples.

## What Are Tutorials?

Tutorials are designed for learning. Unlike [how-to guides](../how-to/) (which
assume you know what you want to do) or [reference docs](../reference/) (which
assume you know what you're looking for), tutorials guide you through building
something from start to finish.

Each tutorial:

- Builds incrementally on the previous one
- Focuses on new concepts without repeating boilerplate
- Includes complete working code at the end
- Is safe to follow—you can't break anything

The first tutorial teaches the foundation (protocol, lifecycle, basic structure).
Subsequent tutorials reference the first and show only the new additions.

## Available Tutorials

| Tutorial                                              | Difficulty   | Description                                       |
|-------------------------------------------------------|--------------|---------------------------------------------------|
| [Your First Extension](first-extension.md)            | Beginner     | Build a minimal extension with one toolbar button |
| [Adding Toolbar Buttons](toolbar-buttons.md)          | Beginner     | Multiple buttons with custom SVG icons            |
| [Using Dialogs](dialogs.md)                           | Beginner     | Show messages, confirmations, and file dialogs    |
| [Working with Messages](working-with-messages.md)     | Intermediate | Read and modify HL7 messages in the editor        |
| [Building a Wizard with UI](wizard-with-ui.md)        | Advanced     | Create a multi-step extension with a web UI       |
| [Providing Schema Overrides](schema-overrides.md)     | Advanced     | Customise field validation for your organisation  |

## Suggested Learning Path

If you're completely new to Hermes extensions:

1. **Start with Your First Extension** — Learn the basic protocol, lifecycle,
   and how to add a toolbar button that modifies a message. This is the
   foundation that all other tutorials build upon.

2. **Choose your path** — After the first tutorial, pick based on what you want
   to build:
   - **Adding Toolbar Buttons** — Multiple buttons with custom SVG icons
   - **Using Dialogs** — Messages, confirmations, and file dialogs
   - **Working with Messages** — Read and transform HL7 messages programmatically
   - **Building a Wizard with UI** — Rich web interfaces with HTTP servers
   - **Providing Schema Overrides** — Field validation for your organisation

Each tutorial after the first starts with "Prerequisites: Start with the code
from Your First Extension" and shows only the new additions needed.

## Test Extensions

The [test-extensions](../test-extensions/) directory contains focused scripts
that exercise specific API features:

| Extension             | Purpose                                      |
|-----------------------|----------------------------------------------|
| `echo.py`             | Minimal extension for protocol testing       |
| `dialogs.py`          | Tests all dialog API methods                 |
| `editor-ops.py`       | Tests message read/write operations          |
| `schema-override.py`  | Tests schema merging behaviour               |
| `windows.py`          | Tests window management API                  |

These are useful for understanding individual APIs but lack the step-by-step
explanations that tutorials provide.

## After the Tutorials

Once you've completed the tutorials and understand the basics:

- Consult the [How-To Guides](../how-to/) for specific tasks
- Read the [Explanation](../explanation/) docs to deepen understanding
- Use the [Reference](../reference/) for API lookup and specifications
