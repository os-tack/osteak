# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `Tea` trait — core TEA contract with `update`, `view`, `init`, and `subscriptions`
- `Action<Msg>` enum — `None`, `Task`, `Batch`, `Quit` with `map()` for composition
- `Cmd<Msg>` struct — update return type with dirty tracking and ergonomic builders
- `Sub<Msg>` struct — subscription descriptor with identity-based lifecycle
- `runner` module — optional event loop powered by crossterm + tokio
- `counter` example — minimal TEA app with runner
- `counter_manual` example — same app with hand-written event loop
- `multi_pane` example — model composition with `Cmd::map`
