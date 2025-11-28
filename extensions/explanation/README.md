# Explanation

This section provides understanding-oriented documentation for the Hermes
Extension API. These documents explain concepts, design decisions, and the
rationale behind how the system works.

## What is Explanation Documentation?

Explanation documentation helps you build mental models and understand *why*
things work the way they do. Unlike tutorials (learning-oriented) or how-to
guides (task-oriented), explanations are discussion-oriented and focus on
clarifying and illuminating topics.

## Available Explanations

| Document                            | Description                                                     |
| ----------------------------------- | --------------------------------------------------------------- |
| [Architecture](architecture.md)     | Why extensions use separate processes, JSON-RPC, and stdio      |
| [Events](events.md)                 | Push notifications, subscription model, debouncing              |
| [Lifecycle](lifecycle.md)           | Understanding extension states, transitions, and error handling |
| [Schema Merging](schema-merging.md) | How Hermes combines schemas from multiple sources               |

## How to Use This Section

Read these documents to:

- Understand design trade-offs and alternatives considered
- Build mental models of how components interact
- Learn why certain patterns are recommended
- Gain context for troubleshooting unexpected behaviour

If you're looking for step-by-step instructions, see the
[tutorials](../tutorials/) or [how-to guides](../how-to/). For API details,
see the [reference documentation](../reference/).
