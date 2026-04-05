# osteak

[![Crates.io](https://img.shields.io/crates/v/osteak.svg)](https://crates.io/crates/osteak)
[![docs.rs](https://img.shields.io/docsrs/osteak)](https://docs.rs/osteak)
[![CI](https://github.com/os-tack/osteak/actions/workflows/ci.yml/badge.svg)](https://github.com/os-tack/osteak/actions/workflows/ci.yml)
[![License](https://img.shields.io/crates/l/osteak.svg)](LICENSE-MIT)

Elm Architecture for ratatui. We built this because our TUI kept
accumulating state bugs as it grew — booleans disagreeing, async
tasks outliving the UI state that spawned them, the usual. TEA
fixed it for us. Maybe it helps you too.

You bring the event loop. osteak gives you `update`, `view`, and
a `Cmd` type to describe side effects. There's an optional runner
if you don't have a loop yet.

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
# use osteak::{Tea, Cmd};
# use ratatui::Frame;
# use ratatui::widgets::Paragraph;
# struct Counter { count: i32 }
# enum Msg { Increment, Decrement, Quit }
# impl Tea for Counter {
#     type Msg = Msg;
#     fn update(&mut self, msg: Msg) -> Cmd<Msg> { Cmd::none() }
#     fn view(&mut self, frame: &mut Frame) {}
# }
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

Write your own loop — osteak doesn't take control.
See [`examples/counter_manual.rs`](examples/counter_manual.rs).

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
  ┌────────┼────────┐
  │        │        │
Task     None     Quit
(you spawn)
  │
  │ Msg (on completion)
  └──────► back to update
```

## Features

| Feature | Default | Description |
|---------|---------|-------------|
| `crossterm-backend` | yes | Crossterm backend + `EventStream` |
| `tokio-runtime` | yes | Tokio integration for the runner |

Just the traits, no runtime:

```sh
cargo add osteak --no-default-features
```

## Examples

- [`counter`](examples/counter.rs) — with the runner
- [`counter_manual`](examples/counter_manual.rs) — hand-written event loop
- [`async_tasks`](examples/async_tasks.rs) — `Cmd::task` + `tokio::spawn`
- [`multi_pane`](examples/multi_pane.rs) — composition with `Cmd::map`

```sh
cargo run --example counter
```

## MSRV

1.86.0 (same as ratatui 0.30).

## License

[MIT](LICENSE-MIT)
