# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1] - 2026-04-05

### Added

- `Cmd::dirty_with_action` convenience for action + dirty in one call
- `async_tasks` example showing `Cmd::task` with `tokio::spawn` in a manual event loop

### Fixed

- Documented that model must not own the `Terminal` (Tea trait docs)

## [0.1.0] - 2026-04-05

### Added

- `Tea` trait with `update`, `view`, `init`, and `subscriptions`
- `Action<Msg>` enum — `None`, `Task`, `Batch`, `Quit` with `map()` for composition
- `Cmd<Msg>` struct — update return type with dirty tracking and ergonomic builders
- `Sub<Msg>` struct — subscription descriptor with identity-based lifecycle
- `runner` module — optional event loop powered by crossterm + tokio
- `counter` example — minimal TEA app with runner
- `counter_manual` example — same app with hand-written event loop
- `multi_pane` example — model composition with `Cmd::map`
