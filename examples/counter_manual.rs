//! Counter (manual event loop) — shows what osteak gives you.
//!
//! This is the same counter as `counter.rs` but with a hand-written
//! event loop instead of the built-in runner. Use this pattern when
//! you have an existing async runtime or need full control.
//!
//! Run with: `cargo run --example counter_manual`

use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use osteak::{Action, Cmd, Tea};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::Paragraph;

// ── Model ──────────────────────────────────────────────────────────

struct Counter {
    count: i32,
}

// ── Messages ───────────────────────────────────────────────────────

enum Msg {
    Increment,
    Decrement,
    Reset,
    Quit,
}

// ── TEA implementation ─────────────────────────────────────────────

impl Tea for Counter {
    type Msg = Msg;

    fn update(&mut self, msg: Msg) -> Cmd<Msg> {
        match msg {
            Msg::Increment => {
                self.count += 1;
                Cmd::dirty()
            }
            Msg::Decrement => {
                self.count -= 1;
                Cmd::dirty()
            }
            Msg::Reset => {
                self.count = 0;
                Cmd::dirty()
            }
            Msg::Quit => Cmd::quit(),
        }
    }

    fn view(&mut self, frame: &mut Frame) {
        let [counter_area, help_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(3)]).areas(frame.area());

        // Counter display
        let counter_text = format!("{}", self.count);
        let counter = Paragraph::new(counter_text)
            .centered()
            .style(Style::new().fg(Color::Cyan).bold());
        frame.render_widget(counter, counter_area);

        // Key hints
        let help = Paragraph::new(vec![
            Line::from("↑/k increment  ↓/j decrement  r reset  q quit").dim(),
        ])
        .centered();
        frame.render_widget(help, help_area);
    }
}

// ── Event → Message mapping ────────────────────────────────────────

fn event_to_msg(event: &Event) -> Option<Msg> {
    let Event::Key(key) = event else { return None };
    if key.kind != KeyEventKind::Press {
        return None;
    }
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => Some(Msg::Increment),
        KeyCode::Down | KeyCode::Char('j') => Some(Msg::Decrement),
        KeyCode::Char('r') => Some(Msg::Reset),
        KeyCode::Char('q') | KeyCode::Esc => Some(Msg::Quit),
        _ => None,
    }
}

// ── Manual event loop ──────────────────────────────────────────────
//
// This is what the optional `runner` module automates. For simple apps,
// you can use `osteak::runner::run()` instead. But you always have the
// option to write your own loop — osteak never takes control away.

fn main() -> io::Result<()> {
    let mut terminal = ratatui::try_init()?;
    let mut model = Counter { count: 0 };

    // Initial render
    terminal.draw(|f| model.view(f))?;

    loop {
        // Block for next event
        let ev = event::read()?;

        // Map event to message (skip unknown events)
        let Some(msg) = event_to_msg(&ev) else {
            continue;
        };

        // Update
        let cmd = model.update(msg);

        // Handle action
        if matches!(cmd.action, Action::Quit) {
            break;
        }
        // In a real app, you'd match on Action::Task and tokio::spawn it.
        // The counter has no async work.

        // Re-render if dirty
        if cmd.dirty {
            terminal.draw(|f| model.view(f))?;
        }
    }

    ratatui::try_restore()?;
    Ok(())
}
