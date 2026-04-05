//! Integration test: nested model composition with `Cmd::map`.

use osteak::{Action, Cmd, Tea};

// ── Child model ────────────────────────────────────────────────────

struct Child {
    value: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ChildMsg {
    Add(i32),
}

impl Child {
    fn update(&mut self, msg: ChildMsg) -> Cmd<ChildMsg> {
        match msg {
            ChildMsg::Add(n) => {
                self.value += n;
                Cmd::dirty()
            }
        }
    }
}

// ── Parent model ───────────────────────────────────────────────────

struct Parent {
    left: Child,
    right: Child,
}

enum ParentMsg {
    Left(ChildMsg),
    Right(ChildMsg),
    Quit,
}

impl Tea for Parent {
    type Msg = ParentMsg;

    fn update(&mut self, msg: ParentMsg) -> Cmd<ParentMsg> {
        match msg {
            ParentMsg::Left(m) => self.left.update(m).map(ParentMsg::Left),
            ParentMsg::Right(m) => self.right.update(m).map(ParentMsg::Right),
            ParentMsg::Quit => Cmd::quit(),
        }
    }

    fn view(&mut self, _frame: &mut ratatui::Frame) {
        // View composition is tested in tea_cycle.rs
    }
}

// ── Tests ──────────────────────────────────────────────────────────

#[test]
fn child_update_maps_to_parent() {
    let mut app = Parent {
        left: Child { value: 0 },
        right: Child { value: 10 },
    };

    let cmd = app.update(ParentMsg::Left(ChildMsg::Add(5)));
    assert_eq!(app.left.value, 5);
    assert_eq!(app.right.value, 10);
    assert!(cmd.dirty);

    let cmd = app.update(ParentMsg::Right(ChildMsg::Add(3)));
    assert_eq!(app.left.value, 5);
    assert_eq!(app.right.value, 13);
    assert!(cmd.dirty);
}

#[test]
fn mapped_none_stays_none() {
    let cmd: Cmd<i32> = Cmd::none();
    let mapped: Cmd<String> = cmd.map(|n| n.to_string());
    assert!(!mapped.dirty);
    assert!(matches!(mapped.action, Action::None));
}

#[test]
fn mapped_quit_stays_quit() {
    let cmd: Cmd<i32> = Cmd::quit();
    let mapped: Cmd<String> = cmd.map(|n| n.to_string());
    assert!(matches!(mapped.action, Action::Quit));
}

#[test]
fn mapped_dirty_preserves_dirty() {
    let cmd: Cmd<i32> = Cmd::dirty();
    let mapped: Cmd<String> = cmd.map(|n| n.to_string());
    assert!(mapped.dirty);
}

#[tokio::test]
async fn mapped_task_transforms_message() {
    let cmd: Cmd<i32> = Cmd::task(async { 42 });
    let mapped: Cmd<String> = cmd.map(|n| n.to_string());

    assert!(mapped.dirty);
    match mapped.action {
        Action::Task(fut) => {
            let result = fut.await;
            assert_eq!(result, "42");
        }
        _ => panic!("expected Task"),
    }
}

#[test]
fn parent_quit_works() {
    let mut app = Parent {
        left: Child { value: 0 },
        right: Child { value: 0 },
    };

    let cmd = app.update(ParentMsg::Quit);
    assert!(matches!(cmd.action, Action::Quit));
}
