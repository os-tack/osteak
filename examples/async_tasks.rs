//! Async tasks — demonstrates `Cmd::task` with `tokio::spawn`.
//!
//! A counter that kicks off a simulated async "save" operation on
//! each increment. The save completes after a delay and sends a
//! message back into the update loop. This shows the full async
//! round-trip in a manual event loop.
//!
//! Run with: `cargo run --example async_tasks`

use std::io;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use osteak::{Action, Cmd, Tea};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use tokio::sync::mpsc;

// ── Model ──────────────────────────────────────────────────────────

struct App {
    count: i32,
    status: String,
}

// ── Messages ───────────────────────────────────────────────────────

enum Msg {
    Increment,
    SaveComplete(i32),
    Quit,
}

// ── TEA implementation ─────────────────────────────────────────────

impl Tea for App {
    type Msg = Msg;

    fn update(&mut self, msg: Msg) -> Cmd<Msg> {
        match msg {
            Msg::Increment => {
                self.count += 1;
                self.status = format!("saving {}...", self.count);
                let value = self.count;

                // Return a Task — the caller decides how to spawn it.
                Cmd::task(async move {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    Msg::SaveComplete(value)
                })
            }
            Msg::SaveComplete(value) => {
                self.status = format!("saved {value}");
                Cmd::dirty()
            }
            Msg::Quit => Cmd::quit(),
        }
    }

    fn view(&mut self, frame: &mut Frame) {
        let [counter_area, status_area, help_area] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .areas(frame.area());

        let counter = Paragraph::new(format!("{}", self.count))
            .centered()
            .style(Style::new().fg(Color::Cyan).bold());
        frame.render_widget(counter, counter_area);

        let status = Paragraph::new(self.status.clone())
            .centered()
            .style(Style::new().fg(Color::Yellow));
        frame.render_widget(status, status_area);

        let help = Paragraph::new(Line::from("↑/k increment (triggers async save)  q quit").dim())
            .centered();
        frame.render_widget(help, help_area);
    }
}

// ── Manual event loop with tokio::spawn ────────────────────────────
//
// This shows how Action::Task works in practice: the event loop
// spawns the future with tokio::spawn, and the result feeds back
// through an mpsc channel into the update loop.

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut terminal = ratatui::try_init()?;
    let mut model = App {
        count: 0,
        status: "ready".into(),
    };

    // Channel for task results feeding back into update.
    let (tx, mut rx) = mpsc::unbounded_channel::<Msg>();

    // Initial render.
    terminal.draw(|f| model.view(f))?;

    loop {
        // Poll for terminal events (non-blocking, 50ms timeout)
        // and task completions concurrently.
        let msg = tokio::select! {
            Ok(true) = async { crossterm::event::poll(Duration::from_millis(50)) } => {
                let ev = event::read()?;
                match ev {
                    Event::Key(key) if key.kind == KeyEventKind::Press => {
                        match key.code {
                            KeyCode::Up | KeyCode::Char('k') => Some(Msg::Increment),
                            KeyCode::Char('q') | KeyCode::Esc => Some(Msg::Quit),
                            _ => None,
                        }
                    }
                    _ => None,
                }
            }
            Some(msg) = rx.recv() => Some(msg),
        };

        let Some(msg) = msg else { continue };

        // Update.
        let cmd = model.update(msg);

        // Handle the action.
        match cmd.action {
            Action::Quit => break,
            Action::Task(fut) => {
                // This is the key pattern: YOU decide how to spawn.
                let tx = tx.clone();
                tokio::spawn(async move {
                    let msg = fut.await;
                    let _ = tx.send(msg);
                });
            }
            Action::Batch(actions) => {
                for action in actions {
                    if let Action::Task(fut) = action {
                        let tx = tx.clone();
                        tokio::spawn(async move {
                            let msg = fut.await;
                            let _ = tx.send(msg);
                        });
                    }
                }
            }
            Action::None => {}
        }

        // Re-render if dirty.
        if cmd.dirty {
            terminal.draw(|f| model.view(f))?;
        }
    }

    ratatui::try_restore()?;
    Ok(())
}
