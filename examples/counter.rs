//! Counter — the simplest osteak app.
//!
//! Demonstrates the full TEA cycle using the built-in runner:
//! Model → Message → Update → View.
//!
//! Run with: `cargo run --example counter`

use crossterm::event::{Event, KeyCode, KeyEventKind};
use osteak::{Cmd, Tea};
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

        let counter = Paragraph::new(format!("{}", self.count))
            .centered()
            .style(Style::new().fg(Color::Cyan).bold());
        frame.render_widget(counter, counter_area);

        let help = Paragraph::new(vec![
            Line::from("↑/k increment  ↓/j decrement  r reset  q quit").dim(),
        ])
        .centered();
        frame.render_widget(help, help_area);
    }
}

// ── Run ────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> std::io::Result<()> {
    osteak::runner::run(Counter { count: 0 }, |ev| {
        let Event::Key(key) = ev else { return None };
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
    })
    .await
}
