# DEPENDENCIES — plato-timing

## Signal Chain Layer

**Cross-cutting (Timing)** — Tensor MIDI timing for agent coordination.

Standalone timing crate. Provides Tensor MIDI-based timing primitives for coordinating PLATO room agents. Used by luciddreamer and other timing-sensitive components.

## Ecosystem Dependencies

| Repo | Relationship | Description |
|------|-------------|-------------|
| [plato-nervous](https://github.com/SuperInstance/plato-nervous) | **Related** | Timing may inform signal chain scheduling |
| [luciddreamer-ai](https://github.com/SuperInstance/luciddreamer-ai) | **Depended on by** | Uses timing primitives for podcast improv coordination |

*Note: plato-timing is largely standalone within the PLATO ecosystem. It has minimal internal dependencies and is consumed primarily by external projects.*

## Data Flow

```
IN:
  - MIDI clock events
  - Tempo and time signature data
  - Scheduling requests

OUT:
  - Tensor timing matrices
  - Synchronized clock signals
  - Scheduled event triggers
```

## Dependency Graph Position

```
plato-timing ← (standalone, no ecosystem deps)
  ↓ used by luciddreamer-ai and other external timing consumers
```
