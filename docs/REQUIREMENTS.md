# osteak — Requirements

## Problem

Every ratatui app that grows past demo complexity hits the "bag of booleans" wall.
State is scattered across fields, transitions are implicit, and race conditions
emerge when async work outlives UI signals.

Three existing crates were evaluated (ratatui-elm, teatui, tui-realm) and none
met the requirements for a production ratatui app. See docs/EVALUATION.md.

## Requirements (v0.1)

1. **ratatui 0.30+** — track latest ratatui
2. **You own the event loop** — framework provides structure, not control flow
3. **Centralized model with `&mut` update** — not clone-per-event
4. **Full `Frame` access in view** — not single-Widget return
5. **Async runtime integration** — `Action::Task` carries a future; YOU spawn it on YOUR runtime
6. **Composable views without component registry** — nested render fns, not mount/unmount
7. **Dirty tracking** — `update()` returns whether re-render is needed
8. **Subscriptions** — `Stream<Item=Msg>` for external event sources

## Non-goals

- Owning the event loop (that's your job)
- Component lifecycle (mount/unmount/remount)
- Built-in widget library
- Rendering backend abstraction (use ratatui's)
