# osteak

[![Crates.io](https://img.shields.io/crates/v/osteak.svg)](https://crates.io/crates/osteak)
[![docs.rs](https://img.shields.io/docsrs/osteak)](https://docs.rs/osteak)
[![CI](https://github.com/os-tack/osteak/actions/workflows/ci.yml/badge.svg)](https://github.com/os-tack/osteak/actions/workflows/ci.yml)
[![License](https://img.shields.io/crates/l/osteak.svg)](LICENSE-MIT)

**Elm for [ratatui](https://ratatui.rs) — you bring the loop.**

## Why osteak?

Every ratatui app that grows past demo complexity hits the "bag of booleans"
wall — state scattered across fields, implicit transitions, race conditions
when async work outlives UI signals.

| | osteak | ratatui-elm | teatui | tui-realm |
|---|---|---|---|---|
| ratatui 0.30 | **yes** | no (0.29) | no (0.29) | no (0.29) |
| You keep event loop | **yes** | no | no | no |
| `&mut` update (no clone) | **yes** | yes | no | n/a |
| Full `Frame` access | **yes** | yes | no | yes |
| Async task integration | **yes** | no | own runtime | n/a |
| Dirty tracking | **yes** | no | no | no |

## Quick Start

```sh
cargo add osteak
```

```rust
use osteak::{Tea, Cmd};
use ratatui::Frame;
use ratatui::widgets::Paragraph;

struct Counter { count: i32 }

enum Msg { Increment, Decrement, Quit }

impl Tea for Counter {
    type Msg = Msg;

    fn update(&mut self, msg: Msg) -> Cmd<Msg> {
        match msg {
            Msg::Increment => { self.count += 1; Cmd::dirty() }
            Msg::Decrement => { self.count -= 1; Cmd::dirty() }
            Msg::Quit => Cmd::quit(),
        }
    }

    fn view(&mut self, frame: &mut Frame) {
        let text = format!("Count: {}", self.count);
        frame.render_widget(Paragraph::new(text), frame.area());
    }
}
```

### With the built-in runner

```rust,no_run
use crossterm::event::{Event, KeyCode, KeyEventKind};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    osteak::runner::run(Counter { count: 0 }, |ev| {
        let Event::Key(k) = ev else { return None };
        if k.kind != KeyEventKind::Press { return None; }
        match k.code {
            KeyCode::Up => Some(Msg::Increment),
            KeyCode::Down => Some(Msg::Decrement),
            KeyCode::Char('q') => Some(Msg::Quit),
            _ => None,
        }
    }).await
}
```

### Without the runner

You always have the option to write your own event loop — osteak never takes
control away. See [`examples/counter_manual.rs`](examples/counter_manual.rs).

## Architecture

```text
                    ┌──────────────────────┐
                    │    Your Event Loop    │
                    │  (or osteak::runner)  │
                    └──────┬───────────────┘
                           │ Msg
                    ┌──────▼───────────────┐
                    │   Tea::update(&mut)   │
                    │   → Cmd { action,     │
                    │         dirty }       │
                    └──────┬───────────────┘
                           │
              ┌────────────┼────────────┐
              │            │            │
        Action::Task  Action::None  Action::Quit
        (you spawn)   (no-op)       (exit loop)
              │
              │ Msg (on completion)
              └──────────► back to update
```

## Features

| Feature | Default | Description |
|---------|---------|-------------|
| `crossterm-backend` | yes | Crossterm terminal backend + `EventStream` |
| `tokio-runtime` | yes | Tokio integration for the runner module |

To use osteak without the runner (just the traits):

```sh
cargo add osteak --no-default-features
```

## Examples

- [`counter`](examples/counter.rs) — minimal app with the built-in runner
- [`counter_manual`](examples/counter_manual.rs) — same app, hand-written event loop
- [`async_tasks`](examples/async_tasks.rs) — `Cmd::task` with `tokio::spawn` in a manual loop
- [`multi_pane`](examples/multi_pane.rs) — model composition with `Cmd::map`

```sh
cargo run --example counter
cargo run --example counter_manual
cargo run --example multi_pane
```

## MSRV

The minimum supported Rust version is **1.86.0** (same as ratatui 0.30).

## License

[MIT](LICENSE-MIT)

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).
