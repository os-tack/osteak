# Contributing to osteak

Thank you for your interest in contributing!

## Getting Started

1. Fork and clone the repository
2. Run `cargo test --all-features` to make sure everything works
3. Create a branch for your changes

## Development

```sh
cargo check --all-features      # Quick compilation check
cargo test --all-features       # Run all tests
cargo clippy --all-features     # Lint
cargo fmt                       # Format
cargo doc --no-deps --open      # Build and view docs
```

## Guidelines

- **Keep it small.** osteak is intentionally minimal. If a feature can live in
  user code, it probably should.
- **Tests are required.** Every public API change needs tests — unit tests for
  logic, doc-tests for examples.
- **Document everything public.** `#![warn(missing_docs)]` is enforced.
- **No unsafe code.** `#![forbid(unsafe_code)]` is enforced.

## Pull Requests

- One logical change per PR
- Include a clear description of what and why
- Update CHANGELOG.md under `[Unreleased]`
- Make sure CI passes (`cargo test`, `cargo clippy`, `cargo fmt --check`)

## License

By contributing, you agree that your contributions will be licensed under the
same terms as the project: MIT OR Apache-2.0.
