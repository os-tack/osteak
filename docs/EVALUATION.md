# Crate Evaluation — Why osteak exists

## Evaluated (April 2026)

### ratatui-elm (justDeeevin)
- 29 stars, 1 contributor, pinned to ratatui 0.29
- Owns the event loop — cannot integrate with existing async runtime
- Depends on beta crate (byor)
- Zero community engagement (0 issues ever)
- **Verdict: Architecturally closest, but immature and incompatible**

### teatui (JasterV)
- 47 stars, 1 contributor, "experimental"
- `view()` returns single Widget — can't do multi-area layouts
- Clones entire model on every event
- Creates its own tokio runtime
- Migrated to Codeberg
- **Verdict: Disqualified on engineering grounds**

### tui-realm (veeso)
- 916 stars, 176k downloads, 5 years old
- React-like component model, NOT Elm
- Per-component state, boxed trait objects, mount/unmount lifecycle
- Pinned to ratatui 0.29
- **Verdict: Production-grade but wrong paradigm**

## The gap

No crate provides centralized Elm state management for ratatui that:
- Works with ratatui 0.30
- Lets you keep your event loop
- Supports existing async/tokio architecture
- Gives full Frame access
- Stays under 1k lines

osteak fills this gap.
