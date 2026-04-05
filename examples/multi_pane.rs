//! Multi-pane — composing TEA models without a component registry.
//!
//! Two independent sub-models (a counter and an event log) share the
//! screen via `Layout::horizontal`. Messages are wrapped in a parent
//! enum and routed with `Cmd::map`. Views are plain functions that
//! take `(frame, area)` — no trait objects, no lifecycle hooks.
//!
//! Run with: `cargo run --example multi_pane`

use crossterm::event::{Event, KeyCode, KeyEventKind};
use osteak::{Cmd, Tea};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;

// ── Counter sub-model ──────────────────────────────────────────────

mod counter {
    use osteak::Cmd;
    use ratatui::Frame;
    use ratatui::layout::Rect;
    use ratatui::style::{Color, Style, Stylize};
    use ratatui::widgets::{Block, Paragraph};

    pub struct Model {
        pub count: i32,
    }

    #[derive(Clone, Copy)]
    pub enum Msg {
        Increment,
        Decrement,
    }

    impl Model {
        pub fn update(&mut self, msg: Msg) -> Cmd<Msg> {
            match msg {
                Msg::Increment => {
                    self.count += 1;
                    Cmd::dirty()
                }
                Msg::Decrement => {
                    self.count -= 1;
                    Cmd::dirty()
                }
            }
        }

        pub fn view(&self, frame: &mut Frame, area: Rect, focused: bool) {
            let border_style = if focused {
                Style::new().fg(Color::Cyan)
            } else {
                Style::new().fg(Color::DarkGray)
            };
            let block = Block::bordered()
                .title(" Counter ")
                .border_style(border_style);
            let text = format!("{}", self.count);
            let widget = Paragraph::new(text).centered().bold().block(block);
            frame.render_widget(widget, area);
        }
    }
}

// ── Log sub-model ──────────────────────────────────────────────────

mod log {
    use osteak::Cmd;
    use ratatui::Frame;
    use ratatui::layout::Rect;
    use ratatui::style::{Color, Style};
    use ratatui::widgets::{Block, Paragraph};

    pub struct Model {
        pub entries: Vec<String>,
        capacity: usize,
    }

    pub enum Msg {
        Push(String),
    }

    impl Model {
        pub fn new(capacity: usize) -> Self {
            Model {
                entries: Vec::new(),
                capacity,
            }
        }

        pub fn update(&mut self, msg: Msg) -> Cmd<Msg> {
            match msg {
                Msg::Push(entry) => {
                    self.entries.push(entry);
                    if self.entries.len() > self.capacity {
                        self.entries.remove(0);
                    }
                    Cmd::dirty()
                }
            }
        }

        pub fn view(&self, frame: &mut Frame, area: Rect, focused: bool) {
            let border_style = if focused {
                Style::new().fg(Color::Cyan)
            } else {
                Style::new().fg(Color::DarkGray)
            };
            let block = Block::bordered()
                .title(" Event Log ")
                .border_style(border_style);
            let text = self.entries.join("\n");
            let widget = Paragraph::new(text).block(block);
            frame.render_widget(widget, area);
        }
    }
}

// ── Parent model ───────────────────────────────────────────────────

enum Focus {
    Left,
    Right,
}

struct App {
    counter: counter::Model,
    log: log::Model,
    focus: Focus,
}

enum Msg {
    Counter(counter::Msg),
    Log(log::Msg),
    SwitchFocus,
    Quit,
}

impl Tea for App {
    type Msg = Msg;

    fn update(&mut self, msg: Msg) -> Cmd<Msg> {
        match msg {
            Msg::Counter(m) => {
                let description = match &m {
                    counter::Msg::Increment => format!("count → {}", self.counter.count + 1),
                    counter::Msg::Decrement => format!("count → {}", self.counter.count - 1),
                };
                let counter_cmd = self.counter.update(m).map(Msg::Counter);
                let log_cmd = self.log.update(log::Msg::Push(description)).map(Msg::Log);

                Cmd::batch(vec![counter_cmd.action, log_cmd.action])
            }
            Msg::Log(m) => self.log.update(m).map(Msg::Log),
            Msg::SwitchFocus => {
                self.focus = match self.focus {
                    Focus::Left => Focus::Right,
                    Focus::Right => Focus::Left,
                };
                Cmd::dirty()
            }
            Msg::Quit => Cmd::quit(),
        }
    }

    fn view(&mut self, frame: &mut Frame) {
        let [left, right, help_area] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(frame.area());

        // For narrow terminals, stack vertically. For wide ones, split horizontally.
        let (counter_area, log_area) = if frame.area().width >= 60 {
            let [l, r] = Layout::horizontal([Constraint::Percentage(40), Constraint::Fill(1)])
                .areas(Layout::vertical([Constraint::Fill(1)]).areas::<1>(frame.area())[0]);
            (l, r)
        } else {
            (left, right)
        };

        let left_focused = matches!(self.focus, Focus::Left);
        self.counter.view(frame, counter_area, left_focused);
        self.log.view(frame, log_area, !left_focused);

        let help = Paragraph::new(Line::from("Tab switch pane  ↑/k inc  ↓/j dec  q quit").dim())
            .centered();
        frame.render_widget(help, help_area);
    }
}

// ── Run ────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let app = App {
        counter: counter::Model { count: 0 },
        log: log::Model::new(50),
        focus: Focus::Left,
    };

    osteak::runner::run(app, |ev| {
        let Event::Key(key) = ev else { return None };
        if key.kind != KeyEventKind::Press {
            return None;
        }
        match key.code {
            KeyCode::Tab => Some(Msg::SwitchFocus),
            KeyCode::Up | KeyCode::Char('k') => Some(Msg::Counter(counter::Msg::Increment)),
            KeyCode::Down | KeyCode::Char('j') => Some(Msg::Counter(counter::Msg::Decrement)),
            KeyCode::Char('q') | KeyCode::Esc => Some(Msg::Quit),
            _ => None,
        }
    })
    .await
}
