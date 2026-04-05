//! Integration test: full update → view cycle with `TestBackend`.

use osteak::{Action, Cmd, Tea};
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::widgets::Paragraph;

// ── Test model ─────────────────────────────────────────────────────

struct Counter {
    count: i32,
}

enum Msg {
    Inc,
    Dec,
    Quit,
}

impl Tea for Counter {
    type Msg = Msg;

    fn update(&mut self, msg: Msg) -> Cmd<Msg> {
        match msg {
            Msg::Inc => {
                self.count += 1;
                Cmd::dirty()
            }
            Msg::Dec => {
                self.count -= 1;
                Cmd::dirty()
            }
            Msg::Quit => Cmd::quit(),
        }
    }

    fn view(&mut self, frame: &mut ratatui::Frame) {
        let text = format!("count={}", self.count);
        frame.render_widget(Paragraph::new(text), frame.area());
    }
}

// ── Tests ──────────────────────────────────────────────────────────

#[test]
fn update_mutates_state() {
    let mut app = Counter { count: 0 };

    let cmd = app.update(Msg::Inc);
    assert_eq!(app.count, 1);
    assert!(cmd.dirty);
    assert!(matches!(cmd.action, Action::None));

    let cmd = app.update(Msg::Inc);
    assert_eq!(app.count, 2);
    assert!(cmd.dirty);

    let cmd = app.update(Msg::Dec);
    assert_eq!(app.count, 1);
    assert!(cmd.dirty);
}

#[test]
fn quit_returns_quit_action() {
    let mut app = Counter { count: 0 };
    let cmd = app.update(Msg::Quit);
    assert!(matches!(cmd.action, Action::Quit));
}

#[test]
fn view_renders_to_buffer() {
    let backend = TestBackend::new(40, 5);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = Counter { count: 42 };

    terminal.draw(|f| app.view(f)).unwrap();

    let buf = terminal.backend().buffer().clone();
    let content: String = buf
        .content()
        .iter()
        .map(|cell| cell.symbol().chars().next().unwrap_or(' '))
        .collect();
    assert!(content.contains("count=42"), "buffer: {content:?}");
}

#[test]
fn update_then_view_cycle() {
    let backend = TestBackend::new(40, 5);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = Counter { count: 0 };

    // Initial render
    terminal.draw(|f| app.view(f)).unwrap();

    // Update
    let cmd = app.update(Msg::Inc);
    assert!(cmd.dirty);

    // Re-render
    terminal.draw(|f| app.view(f)).unwrap();

    let buf = terminal.backend().buffer().clone();
    let content: String = buf
        .content()
        .iter()
        .map(|cell| cell.symbol().chars().next().unwrap_or(' '))
        .collect();
    assert!(content.contains("count=1"), "buffer: {content:?}");
}

#[test]
fn dirty_flag_is_false_for_quit() {
    let mut app = Counter { count: 0 };
    let cmd = app.update(Msg::Quit);
    assert!(!cmd.dirty);
}
